use std::time::Duration;

use serde::Deserialize;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::AppState;

pub const TOKEN_KEY: &str = "igdb_access_token";
pub const TOKEN_EXPIRY_KEY: &str = "igdb_token_expiry";
pub const CLIENT_ID_KEY: &str = "igdb_client_id";
pub const CLIENT_SECRET_KEY: &str = "igdb_client_secret";

const TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";
const API_URL: &str = "https://api.igdb.com/v4/games";

#[derive(Debug, Deserialize)]
pub struct InvolvedCompany {
    #[serde(default)]
    pub developer: bool,
    #[serde(default)]
    pub publisher: bool,
    #[serde(default)]
    pub company: Option<Company>,
}

#[derive(Debug, Deserialize)]
pub struct Company {
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Genre {
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Cover {
    #[serde(default)]
    pub image_id: String,
}

/// IGDB game record — only the fields we request via Apicalypse.
#[derive(Debug, Deserialize)]
pub struct IgdbGame {
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub first_release_date: Option<i64>,
    #[serde(default)]
    pub cover: Option<Cover>,
    #[serde(default)]
    pub involved_companies: Vec<InvolvedCompany>,
    #[serde(default)]
    pub genres: Vec<Genre>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
}

pub struct IgdbClient {
    client: reqwest::Client,
    pub client_id: String,
    pub client_secret: String,
}

impl IgdbClient {
    pub fn new(client_id: String, client_secret: String) -> AppResult<Self> {
        let client = reqwest::Client::builder()
            .user_agent("GameSpace/1.0")
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| AppError::Other(format!("IGDB client init failed: {e}")))?;
        Ok(Self {
            client,
            client_id,
            client_secret,
        })
    }

    /// Reads credentials from the settings table.
    pub fn from_settings(state: &State<'_, AppState>) -> AppResult<Self> {
        let (client_id, client_secret) = state.db.with(|conn| {
            let read = |key: &str| -> AppResult<String> {
                Ok(crate::commands::settings::read_setting(conn, key)?
                    .unwrap_or_default()
                    .trim_matches('"')
                    .trim()
                    .to_string())
            };
            Ok((read(CLIENT_ID_KEY)?, read(CLIENT_SECRET_KEY)?))
        })?;
        if client_id.is_empty() || client_secret.is_empty() {
            return Err(AppError::validation(
                "Укажите Client-ID и Client-Secret IGDB в настройках (Настройки → Игры)",
            ));
        }
        Self::new(client_id, client_secret)
    }

    /// Returns a valid access token, using the cached one when possible.
    pub async fn access_token(&self, state: &State<'_, AppState>) -> AppResult<String> {
        let now = chrono::Utc::now().timestamp();
        let cached: (String, i64) = state.db.with(|conn| {
            let token =
                crate::commands::settings::read_setting(conn, TOKEN_KEY)?.unwrap_or_default();
            let expiry = crate::commands::settings::read_setting(conn, TOKEN_EXPIRY_KEY)?
                .and_then(|v| v.trim_matches('"').parse::<i64>().ok());
            Ok((token.trim_matches('"').to_string(), expiry.unwrap_or(0)))
        })?;
        if !cached.0.is_empty() && cached.1 > now + 60 {
            return Ok(cached.0);
        }

        let response = self
            .client
            .post(TOKEN_URL)
            .query(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("grant_type", "client_credentials"),
            ])
            .send()
            .await
            .map_err(|e| AppError::Other(format!("Twitch OAuth failed: {e}")))?;
        if !response.status().is_success() {
            return Err(AppError::Other(format!(
                "Twitch OAuth failed: HTTP {}",
                response.status()
            )));
        }
        let token: TokenResponse = response
            .json()
            .await
            .map_err(|e| AppError::Other(format!("Twitch OAuth bad response: {e}")))?;

        let expires_at = chrono::Utc::now().timestamp() + token.expires_in;
        state.db.with(|conn| {
            for (key, value) in [
                (TOKEN_KEY, token.access_token.clone()),
                (TOKEN_EXPIRY_KEY, expires_at.to_string()),
            ] {
                conn.execute(
                    "INSERT INTO settings (key, value) VALUES (?1, ?2)
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    rusqlite::params![key, value],
                )?;
            }
            Ok(())
        })?;
        Ok(token.access_token)
    }

    /// Runs an Apicalypse query against the IGDB games endpoint.
    pub async fn query(
        &self,
        state: &State<'_, AppState>,
        body: String,
    ) -> AppResult<Vec<IgdbGame>> {
        let token = self.access_token(state).await?;
        let response = self
            .client
            .post(API_URL)
            .header("Client-ID", &self.client_id)
            .bearer_auth(token)
            .header("Accept", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| AppError::Other(format!("IGDB request failed: {e}")))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| AppError::Other(format!("IGDB response read failed: {e}")))?;
        if !status.is_success() {
            return Err(AppError::Other(format!(
                "IGDB request failed: HTTP {status} — {text}"
            )));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::Other(format!("IGDB bad response: {e}")))
    }
}

/// cover.image_id → direct https cover-big URL.
pub fn cover_big_url(image_id: &str) -> Option<String> {
    if image_id.is_empty() {
        return None;
    }
    Some(format!(
        "https://images.igdb.com/igdb/image/upload/t_cover_big/{image_id}.jpg"
    ))
}

pub const SEARCH_FIELDS: &str = "fields name, summary, first_release_date, \
     cover.image_id, involved_companies.company.name, involved_companies.developer, \
     involved_companies.publisher, genres.name;";

/// Builds an Apicalypse search query.
pub fn search_body(query: &str, limit: usize) -> String {
    format!(
        "{SEARCH_FIELDS} search \"{}\"; limit {limit};",
        query.replace('"', "")
    )
}

/// Builds an Apicalypse query for a concrete game id.
pub fn id_body(game_id: i64) -> String {
    format!("{SEARCH_FIELDS} where id = {game_id}; limit 1;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cover_big_url_builds_https_link() {
        assert_eq!(
            cover_big_url("co1l1f").unwrap(),
            "https://images.igdb.com/igdb/image/upload/t_cover_big/co1l1f.jpg"
        );
        assert!(cover_big_url("").is_none());
    }

    #[test]
    fn search_body_escapes_quotes() {
        let body = search_body("witcher 3", 5);
        assert!(body.starts_with("fields name"));
        assert!(body.contains("search \"witcher 3\";"));
        assert!(body.ends_with("limit 5;"));
    }

    #[test]
    fn id_body_targets_id() {
        assert!(id_body(42).ends_with("where id = 42; limit 1;"));
    }
}
