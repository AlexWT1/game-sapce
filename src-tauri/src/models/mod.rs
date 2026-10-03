use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub executable_path: Option<String>,
    pub install_path: Option<String>,
    pub cover_path: Option<String>,
    pub background_path: Option<String>,
    pub icon_path: Option<String>,
    pub developer: String,
    pub publisher: String,
    pub genre: String,
    pub release_year: Option<i64>,
    pub platform: String,
    pub source: String,
    pub controller_support: String,
    pub is_installed: bool,
    pub is_favorite: bool,
    pub is_hidden: bool,
    pub created_at: String,
    pub updated_at: String,
    pub last_played_at: Option<String>,
    pub total_playtime_seconds: i64,
    pub launch_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub executable_path: Option<String>,
    pub install_path: Option<String>,
    pub cover_path: Option<String>,
    pub background_path: Option<String>,
    pub icon_path: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genre: Option<String>,
    pub release_year: Option<i64>,
    pub platform: Option<String>,
    pub source: Option<String>,
    pub controller_support: Option<String>,
    pub is_installed: Option<bool>,
    pub is_favorite: Option<bool>,
    pub is_hidden: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSession {
    pub id: i64,
    pub game_id: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_seconds: i64,
    pub pid: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
    pub id: i64,
    pub game_id: Option<i64>,
    pub game_title: Option<String>,
    pub title: String,
    pub description: String,
    pub icon_path: Option<String>,
    pub rarity: String,
    pub unlocked: bool,
    pub unlocked_at: Option<String>,
    pub progress_current: i64,
    pub progress_target: i64,
    pub scope: String,
    /// Stable rule key, e.g. `first_launch_game`, `playtime_100h`, NULL for demo rows.
    pub key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameTheme {
    pub game_id: i64,
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub background_color: String,
    pub glow_color: String,
    pub font_family: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenreCount {
    pub genre: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaytimeByDay {
    pub date: String,
    pub seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankedPlaytime {
    pub game_id: i64,
    pub title: String,
    pub seconds: i64,
    pub cover_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    pub total_playtime_seconds: i64,
    pub games_count: i64,
    pub installed_count: i64,
    pub sessions_count: i64,
    pub achievements_unlocked: i64,
    pub achievements_total: i64,
    pub favorite_count: i64,
    pub most_played: Vec<RankedPlaytime>,
    pub by_genre: Vec<GenreCount>,
    pub daily: Vec<PlaytimeByDay>,
    pub weekly: Vec<PlaytimeByDay>,
    pub monthly: Vec<PlaytimeByDay>,
    pub avg_session_seconds: i64,
    pub longest_session_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStatistics {
    pub total_playtime_seconds: i64,
    pub launch_count: i64,
    pub sessions_count: i64,
    pub avg_session_seconds: i64,
    pub longest_session_seconds: i64,
    pub last_session_at: Option<String>,
    pub playtime_week_seconds: i64,
    pub playtime_month_seconds: i64,
    pub playtime_year_seconds: i64,
    pub daily: Vec<PlaytimeByDay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub games: Vec<Game>,
    pub achievements: Vec<Achievement>,
    pub genres: Vec<String>,
    pub developers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningGame {
    pub game_id: i64,
    pub title: String,
    pub pid: i64,
    pub started_at: String,
    pub cover_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreAccount {
    pub provider: String,
    pub account_id: String,
    pub profile_url: String,
    pub persona_name: String,
    pub avatar_url: String,
    pub linked_at: String,
    pub last_synced_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamSyncResult {
    pub owned: i64,
    pub installed: i64,
    pub added: i64,
    pub updated: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkDeleteResult {
    pub deleted_ids: Vec<i64>,
    pub skipped: i64,
}

/// Metadata fetched from IGDB for the "Load" button in the add-game form.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IgdbGameMeta {
    pub igdb_id: i64,
    pub name: String,
    pub summary: String,
    pub release_year: Option<i32>,
    pub developer: String,
    pub publisher: String,
    pub genres: Vec<String>,
    pub cover_url: Option<String>,
}
