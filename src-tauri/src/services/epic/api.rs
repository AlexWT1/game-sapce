use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;

use crate::error::AppError;

const AUTH_URL: &str = "https://account-public-service-prod03.ol.epicgames.com/account/api/oauth/token";
const CATALOG_URL: &str = "https://catalog-public-service-prod06.ol.epicgames.com/catalog/api/shared/namespace";
const PLAYTIME_URL: &str = "https://distro-gateway-prod.ol.epicgames.com/library/api/public/playtime/account";
/// Public Epic Games Launcher OAuth2 client credentials (`34a02cf8f4414e29b15921876da36f9a:daafbccc737745039dffe53d94fc76cf`).
const BASIC_AUTH: &str = "Basic MzRhMDJjZjhmNDQxNGUyOWIxNTkyMTg3NmRhMzZmOWE6ZGFhZmJjY2M3Mzc3NDUwMzlkZmZlNTNkOTRmYzc2Y2Y=";

#[derive(Debug, Clone)]
pub struct EpicAuthSession {
    pub access_token: String,
    pub refresh_token: String,
    pub account_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone)]
pub struct EpicCatalogDetails {
    pub title: Option<String>,
    pub description: String,
    pub developer: String,
    pub cover_url: Option<String>,
    pub release_year: Option<i64>,
}

#[derive(Debug, thiserror::Error)]
pub enum EpicError {
    #[error("Укажите код авторизации Epic Games")]
    EmptyCode,
    #[error("Не удалось извлечь authorizationCode из введённых данных")]
    InvalidCodeFormat,
    #[error("Ошибка авторизации Epic Games (возможно, код устарел — получите новый)")]
    Unauthorized,
    #[error("Epic Games API вернул ошибку: {0}")]
    Api(String),
    #[error("Сетевая ошибка при обращении к Epic Games: {0}")]
    Network(#[from] reqwest::Error),
}

impl From<EpicError> for AppError {
    fn from(value: EpicError) -> Self {
        match value {
            EpicError::EmptyCode | EpicError::InvalidCodeFormat => {
                AppError::Validation(value.to_string())
            }
            EpicError::Unauthorized | EpicError::Api(_) | EpicError::Network(_) => {
                AppError::Other(value.to_string())
            }
        }
    }
}

#[derive(Deserialize)]
struct AuthCodeJson {
    #[serde(rename = "authorizationCode")]
    authorization_code: Option<String>,
}

/// Accepts either a raw authorization code or the JSON object shown on Epic's redirect page.
pub fn parse_authorization_code(input: &str) -> Result<String, EpicError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(EpicError::EmptyCode);
    }
    if trimmed.starts_with('{') {
        let parsed: AuthCodeJson =
            serde_json::from_str(trimmed).map_err(|_| EpicError::InvalidCodeFormat)?;
        let code = parsed
            .authorization_code
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
            .ok_or(EpicError::InvalidCodeFormat)?;
        return Ok(code);
    }
    // Strip surrounding quotes if pasted with them.
    let unquoted = trimmed.trim_matches('"').trim();
    if unquoted.is_empty() || unquoted.contains(char::is_whitespace) {
        return Err(EpicError::InvalidCodeFormat);
    }
    Ok(unquoted.to_string())
}

#[derive(Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    refresh_token: String,
    account_id: String,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct OAuthErrorResponse {
    #[serde(rename = "errorMessage")]
    error_message: Option<String>,
}

#[derive(Clone)]
pub struct EpicApiClient {
    http: Client,
}

fn encode_form_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

impl EpicApiClient {
    pub fn new() -> Result<Self, EpicError> {
        let http = Client::builder()
            .user_agent("UELauncher/11.0.1-14907503+++Portal+Release-Live Windows/10.0.19041.1.256.64bit")
            .timeout(Duration::from_secs(15))
            .build()?;
        Ok(Self { http })
    }

    pub async fn exchange_code(&self, code: &str) -> Result<EpicAuthSession, EpicError> {
        let body = format!(
            "grant_type=authorization_code&code={}",
            encode_form_component(code)
        );
        let resp = self
            .http
            .post(AUTH_URL)
            .header("Authorization", BASIC_AUTH)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?;

        self.parse_token_response(resp).await
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<EpicAuthSession, EpicError> {
        let body = format!(
            "grant_type=refresh_token&refresh_token={}",
            encode_form_component(refresh_token)
        );
        let resp = self
            .http
            .post(AUTH_URL)
            .header("Authorization", BASIC_AUTH)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?;

        self.parse_token_response(resp).await
    }

    async fn parse_token_response(
        &self,
        resp: reqwest::Response,
    ) -> Result<EpicAuthSession, EpicError> {
        let status = resp.status();
        let text = resp.text().await?;
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::BAD_REQUEST
        {
            if let Ok(err) = serde_json::from_str::<OAuthErrorResponse>(&text) {
                if let Some(msg) = err.error_message {
                    return Err(EpicError::Api(msg));
                }
            }
            return Err(EpicError::Unauthorized);
        }
        if !status.is_success() {
            return Err(EpicError::Api(format!("HTTP {status}")));
        }

        let token: OAuthTokenResponse = serde_json::from_str(&text)
            .map_err(|e| EpicError::Api(format!("Invalid token JSON: {e}")))?;

        let display_name = token
            .display_name
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| token.account_id.clone());

        Ok(EpicAuthSession {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            account_id: token.account_id,
            display_name,
        })
    }

    pub async fn get_catalog_details(
        &self,
        access_token: &str,
        namespace: &str,
        catalog_item_id: &str,
        locale: &str,
    ) -> Option<EpicCatalogDetails> {
        if namespace.is_empty() || catalog_item_id.is_empty() {
            return None;
        }
        let url = format!("{CATALOG_URL}/{namespace}/bulk/items");
        let resp = self
            .http
            .get(&url)
            .bearer_auth(access_token)
            .query(&[
                ("id", catalog_item_id),
                ("includeDLCDetails", "false"),
                ("includeMainGameDetails", "false"),
                ("country", "US"),
                ("locale", locale),
            ])
            .send()
            .await
            .ok()?;

        if !resp.status().is_success() {
            return None;
        }
        let map: std::collections::HashMap<String, CatalogItem> = resp.json().await.ok()?;
        let item = map.get(catalog_item_id)?;

        let cover_url = pick_cover_image(&item.key_images);
        let release_year = item
            .release_info
            .iter()
            .filter_map(|r| r.date_added.as_deref())
            .chain(item.creation_date.as_deref())
            .find_map(parse_year);

        Some(EpicCatalogDetails {
            title: item.title.clone().filter(|t| !t.trim().is_empty()),
            description: item.description.clone().unwrap_or_default(),
            developer: item.developer.clone().unwrap_or_default(),
            cover_url,
            release_year,
        })
    }

    pub async fn get_all_playtime(
        &self,
        access_token: &str,
        account_id: &str,
    ) -> std::collections::HashMap<String, i64> {
        let mut map = std::collections::HashMap::new();
        if account_id.trim().is_empty() {
            return map;
        }
        let url = format!("{PLAYTIME_URL}/{account_id}/all");
        let Ok(resp) = self.http.get(&url).bearer_auth(access_token).send().await else {
            return map;
        };
        if !resp.status().is_success() {
            return map;
        }
        if let Ok(text) = resp.text().await {
            parse_playtime_array(&text, &mut map);
        }
        map
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlaytimeEntry {
    #[serde(default)]
    pub artifact_id: String,
    #[serde(default)]
    pub total_time: i64,
}

pub(crate) fn parse_playtime_array(
    json: &str,
    out: &mut std::collections::HashMap<String, i64>,
) {
    if let Ok(entries) = serde_json::from_str::<Vec<PlaytimeEntry>>(json) {
        for e in entries {
            if !e.artifact_id.is_empty() && e.total_time > 0 {
                out.insert(e.artifact_id, e.total_time);
            }
        }
    }
}

#[derive(Deserialize)]
struct CatalogItem {
    title: Option<String>,
    description: Option<String>,
    developer: Option<String>,
    #[serde(default, rename = "keyImages")]
    key_images: Vec<KeyImage>,
    #[serde(default, rename = "releaseInfo")]
    release_info: Vec<ReleaseInfo>,
    #[serde(rename = "creationDate")]
    creation_date: Option<String>,
}

#[derive(Deserialize)]
struct KeyImage {
    #[serde(rename = "type")]
    kind: String,
    url: String,
}

#[derive(Deserialize)]
struct ReleaseInfo {
    #[serde(rename = "dateAdded")]
    date_added: Option<String>,
}

fn pick_cover_image(images: &[KeyImage]) -> Option<String> {
    const PRIORITY: &[&str] = &[
        "DieselGameBoxTall",
        "OfferImageTall",
        "DieselStoreFrontTall",
        "DieselGameBox",
        "OfferImageWide",
        "Thumbnail",
    ];
    for kind in PRIORITY {
        if let Some(found) = images.iter().find(|img| img.kind == *kind && !img.url.is_empty()) {
            return Some(found.url.clone());
        }
    }
    images
        .iter()
        .find(|img| !img.url.is_empty())
        .map(|img| img.url.clone())
}

fn parse_year(iso: &str) -> Option<i64> {
    iso.get(0..4)?.parse::<i64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_raw_and_json_authorization_codes() {
        assert_eq!(
            parse_authorization_code("  abc123def456  ").expect("raw"),
            "abc123def456"
        );
        assert_eq!(
            parse_authorization_code(r#"{"redirectUrl":"https://...","authorizationCode":"xyz789","sid":null}"#)
                .expect("json"),
            "xyz789"
        );
        assert!(parse_authorization_code("   ").is_err());
        assert!(parse_authorization_code(r#"{"authorizationCode":""}"#).is_err());
    }

    #[test]
    fn picks_tall_cover_image_first() {
        let images = vec![
            KeyImage {
                kind: "Thumbnail".into(),
                url: "https://example.com/thumb.jpg".into(),
            },
            KeyImage {
                kind: "DieselGameBoxTall".into(),
                url: "https://example.com/tall.jpg".into(),
            },
        ];
        assert_eq!(
            pick_cover_image(&images).as_deref(),
            Some("https://example.com/tall.jpg")
        );
    }

    #[test]
    fn parses_playtime_array_seconds() {
        let mut map = std::collections::HashMap::new();
        let json = r#"[{"accountId":"acc1","artifactId":"9d2d0eb64d5c44529cece33fe2a46482","totalTime":72166},{"accountId":"acc1","artifactId":"Zero","totalTime":0}]"#;
        parse_playtime_array(json, &mut map);
        assert_eq!(map.get("9d2d0eb64d5c44529cece33fe2a46482"), Some(&72166));
        assert!(!map.contains_key("Zero"));
    }
}
