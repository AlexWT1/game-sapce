//! Steam Web API client (async HTTP, rustls).

use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;

use crate::error::AppError;

const API_BASE: &str = "https://api.steampowered.com";
const COVER_BASE: &str = "https://cdn.cloudflare.steamstatic.com/steam/apps";
const ICON_BASE: &str = "https://media.steampowered.com/steamcommunity/public/images/apps";

/// User-facing Steam integration errors (friendly RU messages).
#[derive(Debug)]
pub enum SteamError {
    InvalidApiKey,
    BadProfile(String),
    PrivateProfile,
    Network(String),
    Unexpected(String),
}

impl std::fmt::Display for SteamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SteamError::InvalidApiKey => write!(f, "Неверный API-ключ Steam"),
            SteamError::BadProfile(msg) => write!(f, "{msg}"),
            SteamError::PrivateProfile => write!(
                f,
                "Профиль Steam закрыт. Откройте его в настройках приватности, чтобы синхронизировать библиотеку."
            ),
            SteamError::Network(msg) => write!(f, "Ошибка сети при обращении к Steam: {msg}"),
            SteamError::Unexpected(msg) => write!(f, "Steam вернул неожиданный ответ: {msg}"),
        }
    }
}

impl From<SteamError> for AppError {
    fn from(err: SteamError) -> Self {
        AppError::Other(err.to_string())
    }
}

/// Reference to a Steam profile: numeric SteamID64 or vanity name.
#[derive(Debug, Clone, PartialEq)]
pub enum SteamRef {
    Id64(String),
    Vanity(String),
}

fn after_marker<'a>(input: &'a str, marker: &str) -> Option<&'a str> {
    let lower = input.to_lowercase();
    let index = lower.find(marker)?;
    input.get(index + marker.len()..)
}

/// Parses a Steam profile URL, a SteamID64 or a bare vanity name.
pub fn parse_steam_ref(input: &str) -> Result<SteamRef, SteamError> {
    let trimmed = input.trim();
    let cut = trimmed
        .find(['?', '#'])
        .map(|i| &trimmed[..i])
        .unwrap_or(trimmed);
    let value = cut.trim().trim_end_matches('/');
    if value.is_empty() {
        return Err(SteamError::BadProfile(
            "Укажите ссылку на профиль Steam".into(),
        ));
    }

    if let Some(rest) = after_marker(value, "steamcommunity.com/") {
        if let Some(id) = after_marker(rest, "profiles/") {
            let id = id.split('/').next().unwrap_or(id);
            if id.chars().all(|c| c.is_ascii_digit()) && id.len() >= 5 {
                return Ok(SteamRef::Id64(id.to_string()));
            }
            return Err(SteamError::BadProfile(
                "Некорректная ссылка на профиль Steam".into(),
            ));
        }
        if let Some(vanity) = after_marker(rest, "id/") {
            let vanity = vanity.split('/').next().unwrap_or(vanity);
            if vanity.is_empty() {
                return Err(SteamError::BadProfile(
                    "Некорректная ссылка на профиль Steam".into(),
                ));
            }
            return Ok(SteamRef::Vanity(vanity.to_string()));
        }
        return Err(SteamError::BadProfile(
            "Ожидается ссылка вида https://steamcommunity.com/id/имя".into(),
        ));
    }

    if !value.chars().any(|c| c.is_ascii()) {
        return Err(SteamError::BadProfile(
            "Ожидается ссылка на профиль Steam".into(),
        ));
    }
    if value.chars().all(|c| c.is_ascii_digit()) {
        return Ok(SteamRef::Id64(value.to_string()));
    }
    if value.len() <= 64
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        && !value.contains('.')
    {
        return Ok(SteamRef::Vanity(value.to_string()));
    }
    Err(SteamError::BadProfile(
        "Ожидается ссылка на профиль Steam".into(),
    ))
}

/// Public Steam profile data.
#[derive(Debug, Clone)]
pub struct Profile {
    pub steam_id: String,
    pub persona_name: String,
    pub avatar_url: String,
    pub profile_url: String,
    pub is_public: bool,
}

/// A game owned by the linked account.
#[derive(Debug, Clone)]
pub struct OwnedGame {
    pub appid: u32,
    pub name: String,
    /// Total minutes played on Steam (0 when never played).
    pub playtime_forever: u32,
    pub icon_hash: Option<String>,
}

/// A genre label coming from the Steam store.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct GenreEntry {
    #[serde(default)]
    pub description: String,
}

/// Release date as published on the Steam store.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ReleaseDate {
    #[serde(default, rename = "coming_soon")]
    pub coming_soon: bool,
    #[serde(default)]
    pub date: String,
}

/// Store metadata for a single app (description, developer, genres).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppDetails {
    #[serde(default)]
    pub developers: Vec<String>,
    #[serde(default)]
    pub publishers: Vec<String>,
    #[serde(default)]
    pub genres: Vec<GenreEntry>,
    #[serde(default, rename = "short_description")]
    pub short_description: String,
    /// Store header art; available even when the legacy CDN paths are missing.
    #[serde(default, rename = "header_image")]
    pub header_image: String,
    #[serde(default, rename = "release_date")]
    pub release_date: ReleaseDate,
}

/// Steam returns localized plural genre names ("Экшены"); the library stores a single form.
fn normalize_genre(raw: &str) -> &str {
    match raw {
        "Экшены" | "Экшен" => "Экшен",
        "Стратегии" | "Стратегия" => "Стратегия",
        "Ролевые" | "Ролевые игры" => "RPG",
        "Симуляторы" | "Симулятор" => "Симулятор",
        "Шутеры" | "Шутер" => "Шутер",
        "Хорроры" | "Хоррор" => "Хоррор",
        "Спорт" | "Спортивные" => "Спорт",
        other => other,
    }
}

impl AppDetails {
    /// First genre label, used for library grouping.
    pub fn genre(&self) -> String {
        self.genres
            .first()
            .map(|genre| normalize_genre(&genre.description).to_string())
            .unwrap_or_default()
    }

    /// Joins developer names with a comma.
    pub fn developer(&self) -> String {
        self.developers.join(", ")
    }

    /// Joins publisher names with a comma.
    pub fn publisher(&self) -> String {
        self.publishers.join(", ")
    }

    /// Year parsed from a store date ("24 Apr, 2025" or "24 апр. 2025 г.").
    pub fn release_year(&self) -> Option<i64> {
        if self.release_date.coming_soon {
            return None;
        }
        self.release_date
            .date
            .split(|c: char| !c.is_ascii_digit())
            .filter(|token| token.len() == 4)
            .find_map(|token| token.parse().ok())
    }
}

/// Builds the CDN url of a game library cover.
pub fn cover_url(appid: u32) -> String {
    format!("{COVER_BASE}/{appid}/library_600x900.jpg")
}

/// Cover urls ordered by preference; the first reachable one wins.
pub fn cover_candidates(appid: u32) -> Vec<String> {
    vec![
        cover_url(appid),
        format!("{COVER_BASE}/{appid}/header.jpg"),
        format!("{COVER_BASE}/{appid}/capsule_616x353.jpg"),
    ]
}

/// Builds the CDN url of a game icon (hash comes from GetOwnedGames).
pub fn icon_url(appid: u32, hash: &str) -> String {
    format!("{ICON_BASE}/{appid}/{hash}.jpg")
}

/// One `appid` entry of the store `appdetails` response.
#[derive(Debug, Deserialize)]
struct DetailsEntry {
    success: bool,
    data: Option<AppDetails>,
}

#[derive(Clone)]
pub struct SteamApiClient {
    client: reqwest::Client,
    /// Short-lived client for probing cover art (fallback urls are expected to 404).
    probe: reqwest::Client,
    api_key: String,
}

impl SteamApiClient {
    pub fn new(api_key: String) -> Result<Self, SteamError> {
        let client = reqwest::Client::builder()
            .user_agent("GameSpace/1.0")
            .timeout(Duration::from_secs(20))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| SteamError::Unexpected(e.to_string()))?;
        let probe = reqwest::Client::builder()
            .user_agent("GameSpace/1.0")
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(3))
            .build()
            .map_err(|e| SteamError::Unexpected(e.to_string()))?;
        Ok(Self {
            client,
            probe,
            api_key,
        })
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, SteamError> {
        let request = self
            .client
            .get(format!("{API_BASE}{path}"))
            .query(&[("key", self.api_key.as_str())])
            .query(query);
        let response = request.send().await.map_err(map_network_error)?;
        let status = response.status();
        if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(SteamError::InvalidApiKey);
        }
        if !status.is_success() {
            return Err(SteamError::Unexpected(format!("HTTP {}", status.as_u16())));
        }
        response
            .json::<T>()
            .await
            .map_err(|e| SteamError::Unexpected(e.to_string()))
    }

    /// Checks that a url answers with 2xx (used to validate cover art).
    /// Missing art is normal, so failures are logged at debug level.
    /// Transport failures get one retry — direct connections drop
    /// intermittently on some networks and would otherwise miss real art.
    pub async fn url_exists(&self, url: &str) -> bool {
        for attempt in 0..2 {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_millis(600)).await;
            }
            let response = self
                .probe
                .get(url)
                .header(reqwest::header::RANGE, "bytes=0-0")
                .send()
                .await;
            match response {
                Ok(response) if response.status().is_success() => return true,
                Ok(response) => {
                    log::debug!("Steam asset {url}: HTTP {}", response.status());
                    return false;
                }
                Err(err) => log::debug!("Steam asset {url}: {err}"),
            }
        }
        false
    }

    /// Fetches store metadata (genres, developer, description) for one app.
    pub async fn get_app_details(&self, appid: u32, lang: &str) -> Option<AppDetails> {
        let response = match self
            .client
            .get("https://store.steampowered.com/api/appdetails")
            .query(&[("appids", appid.to_string()), ("l", lang.to_string())])
            .send()
            .await
        {
            Ok(response) => response,
            Err(err) => {
                log::warn!("Steam store appdetails {appid}: {err}");
                return None;
            }
        };
        if !response.status().is_success() {
            log::warn!(
                "Steam store appdetails {appid}: HTTP {}",
                response.status()
            );
            return None;
        }
        let body: HashMap<String, DetailsEntry> = match response.json().await {
            Ok(body) => body,
            Err(err) => {
                log::warn!("Steam store appdetails {appid}: {err}");
                return None;
            }
        };
        // The response key is not always the requested appid, but we only ask for one.
        let entry = body.into_iter().next()?.1;
        if !entry.success {
            return None;
        }
        entry.data
    }

    /// Resolves a vanity name to a SteamID64.
    pub async fn resolve_steam_id(&self, vanity: &str) -> Result<String, SteamError> {
        #[derive(Deserialize)]
        struct Envelope {
            response: ResolveBody,
        }
        #[derive(Deserialize)]
        struct ResolveBody {
            success: u8,
            steamid: Option<String>,
        }

        let body: Envelope = self
            .get_json(
                "/ISteamUser/ResolveVanityURL/v1/",
                &[("vanityurl", vanity)],
            )
            .await?;
        if body.response.success == 1 {
            body.response
                .steamid
                .ok_or_else(|| SteamError::Unexpected("пустой steamid".into()))
        } else {
            Err(SteamError::BadProfile("Профиль Steam не найден".into()))
        }
    }

    /// Fetches basic profile data by SteamID64.
    pub async fn get_profile(&self, steam_id: &str) -> Result<Profile, SteamError> {
        #[derive(Deserialize)]
        struct Envelope {
            response: ProfileBody,
        }
        #[derive(Deserialize)]
        struct ProfileBody {
            players: Vec<Player>,
        }
        #[derive(Deserialize)]
        struct Player {
            steamid: String,
            personaname: Option<String>,
            avatarfull: Option<String>,
            profileurl: Option<String>,
            communityvisibilitystate: Option<u8>,
        }

        let body: Envelope = self
            .get_json(
                "/ISteamUser/GetPlayerSummaries/v2/",
                &[("steamids", steam_id)],
            )
            .await?;
        let player = body
            .response
            .players
            .into_iter()
            .next()
            .ok_or_else(|| SteamError::BadProfile("Профиль Steam не найден".into()))?;

        Ok(Profile {
            steam_id: player.steamid,
            persona_name: player.personaname.unwrap_or_default(),
            avatar_url: player.avatarfull.unwrap_or_default(),
            profile_url: player.profileurl.unwrap_or_default(),
            is_public: player.communityvisibilitystate == Some(3),
        })
    }

    /// Lists every game owned by the account (public profiles only).
    pub async fn get_owned_games(&self, steam_id: &str) -> Result<Vec<OwnedGame>, SteamError> {
        #[derive(Deserialize)]
        struct Envelope {
            response: OwnedBody,
        }
        #[derive(Deserialize)]
        struct OwnedBody {
            games: Option<Vec<OwnedRaw>>,
        }
        #[derive(Deserialize)]
        struct OwnedRaw {
            appid: u32,
            name: Option<String>,
            playtime_forever: Option<u32>,
            img_icon_url: Option<String>,
        }

        let body: Envelope = self
            .get_json(
                "/IPlayerService/GetOwnedGames/v1/",
                &[
                    ("steamid", steam_id),
                    ("include_appinfo", "1"),
                    ("include_played_free_games", "1"),
                ],
            )
            .await?;
        let Some(games) = body.response.games else {
            return Err(SteamError::PrivateProfile);
        };
        Ok(games
            .into_iter()
            .map(|g| OwnedGame {
                appid: g.appid,
                name: g.name.unwrap_or_default(),
                playtime_forever: g.playtime_forever.unwrap_or(0),
                icon_hash: g.img_icon_url.filter(|h| !h.is_empty()),
            })
            .collect())
    }
}

fn map_network_error(err: reqwest::Error) -> SteamError {
    if err.is_timeout() {
        SteamError::Network("превышено время ожидания".into())
    } else if err.is_connect() {
        SteamError::Network("нет соединения".into())
    } else if err.is_decode() {
        SteamError::Unexpected(err.to_string())
    } else {
        SteamError::Network(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_profile_urls() {
        assert_eq!(
            parse_steam_ref("https://steamcommunity.com/id/gabelogannewell").unwrap(),
            SteamRef::Vanity("gabelogannewell".into())
        );
        assert_eq!(
            parse_steam_ref("https://steamcommunity.com/profiles/76561197960435530/").unwrap(),
            SteamRef::Id64("76561197960435530".into())
        );
        assert_eq!(
            parse_steam_ref("steamcommunity.com/id/somebody?foo=1").unwrap(),
            SteamRef::Vanity("somebody".into())
        );
        assert_eq!(
            parse_steam_ref("76561197960435530").unwrap(),
            SteamRef::Id64("76561197960435530".into())
        );
        assert_eq!(
            parse_steam_ref("some_user-1").unwrap(),
            SteamRef::Vanity("some_user-1".into())
        );
    }

    #[test]
    fn rejects_bad_profiles() {
        assert!(parse_steam_ref("").is_err());
        assert!(parse_steam_ref("   ").is_err());
        assert!(parse_steam_ref("https://example.com/id/whatever").is_err());
        assert!(parse_steam_ref("https://steamcommunity.com/").is_err());
    }

    #[test]
    fn builds_cdn_urls() {
        assert_eq!(
            cover_url(730),
            "https://cdn.cloudflare.steamstatic.com/steam/apps/730/library_600x900.jpg"
        );
        assert_eq!(
            icon_url(730, "abc123"),
            "https://media.steampowered.com/steamcommunity/public/images/apps/730/abc123.jpg"
        );
    }

    #[test]
    fn cover_candidates_prefer_library_art() {
        let candidates = cover_candidates(730);
        assert_eq!(candidates.len(), 3);
        assert_eq!(candidates[0], cover_url(730));
        assert!(candidates[1].ends_with("/730/header.jpg"));
        assert!(candidates[2].ends_with("/730/capsule_616x353.jpg"));
    }

    #[test]
    fn parses_store_details() {
        // Real responses may be keyed by an app id different from the requested one.
        let body = r#"{
            "2678630": {
                "success": true,
                "data": {
                    "developers": ["Valve"],
                    "publishers": ["Valve"],
                    "genres": [{"id": "1", "description": "Action"}],
                    "short_description": "Counter-Strike 2 is the largest technical leap forward.",
                    "header_image": "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/730/header.jpg",
                    "release_date": {"coming_soon": false, "date": "21 Aug, 2012"}
                }
            }
        }"#;
        let body: HashMap<String, DetailsEntry> = serde_json::from_str(body).unwrap();
        let details = body.into_iter().next().unwrap().1.data.unwrap();
        assert_eq!(details.developer(), "Valve");
        assert_eq!(details.publisher(), "Valve");
        assert_eq!(details.genre(), "Action");
        assert_eq!(details.release_year(), Some(2012));
        assert!(details.short_description.starts_with("Counter-Strike"));
        assert!(details.header_image.ends_with("/730/header.jpg"));
    }

    #[test]
    fn normalizes_steam_genres() {
        assert_eq!(normalize_genre("Экшены"), "Экшен");
        assert_eq!(normalize_genre("Стратегии"), "Стратегия");
        assert_eq!(normalize_genre("Ролевые"), "RPG");
        assert_eq!(normalize_genre("Приключения"), "Приключения");
        assert_eq!(normalize_genre("Action"), "Action");
    }

    #[test]
    fn ignores_unreleased_store_dates() {
        let details = AppDetails {
            release_date: ReleaseDate {
                coming_soon: true,
                date: "Coming soon".into(),
            },
            ..AppDetails::default()
        };
        assert_eq!(details.release_year(), None);
    }

    #[test]
    fn parses_localized_release_dates() {
        for (date, expected) in [
            ("21 Aug, 2012", Some(2012)),
            ("24 апр. 2015 г.", Some(2015)),
            ("Q3 2026", Some(2026)),
            ("Скоро", None),
        ] {
            let details = AppDetails {
                release_date: ReleaseDate {
                    coming_soon: false,
                    date: date.into(),
                },
                ..AppDetails::default()
            };
            assert_eq!(details.release_year(), expected, "{date}");
        }
    }

    #[test]
    fn formats_error_messages() {
        assert_eq!(
            SteamError::InvalidApiKey.to_string(),
            "Неверный API-ключ Steam"
        );
        assert!(SteamError::PrivateProfile.to_string().contains("закрыт"));
    }
}
