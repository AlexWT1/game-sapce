use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension};
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::AppState;

pub type SettingsMap = HashMap<String, serde_json::Value>;

/// Key under which the player nickname (how the app addresses the user)
/// is stored in the `settings` table.
const PLAYER_NAME_KEY: &str = "player_name";

/// Hard cap for the nickname so it fits UI slots (topbar, greetings).
const PLAYER_NAME_MAX_LEN: usize = 32;

pub fn demo_disabled(conn: &Connection) -> Result<bool, rusqlite::Error> {
    let mode: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'demo_mode'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    Ok(mode.as_deref() == Some("off"))
}

pub fn read_setting(conn: &Connection, key: &str) -> Result<Option<String>, rusqlite::Error> {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
        .optional()
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppResult<SettingsMap> {
    state.db.with(|conn| {
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;
        let mut map = SettingsMap::new();
        for row in rows {
            let (k, v) = row?;
            let value = serde_json::from_str::<serde_json::Value>(&v)
                .unwrap_or(serde_json::Value::String(v));
            map.insert(k, value);
        }
        Ok(map)
    })
}

fn clear_demo_data(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM achievements WHERE game_id IN (SELECT id FROM games WHERE source = 'Demo')",
        [],
    )?;
    conn.execute(
        "DELETE FROM game_sessions WHERE game_id IN (SELECT id FROM games WHERE source = 'Demo')",
        [],
    )?;
    conn.execute(
        "DELETE FROM game_themes WHERE game_id IN (SELECT id FROM games WHERE source = 'Demo')",
        [],
    )?;
    conn.execute("DELETE FROM games WHERE source = 'Demo'", [])?;
    conn.execute("DELETE FROM settings WHERE key = 'demo_seeded'", [])?;
    Ok(())
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    settings: SettingsMap,
) -> AppResult<SettingsMap> {
    state.db.with(|conn| {
        for (key, value) in &settings {
            let raw = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                rusqlite::params![key, raw],
            )?;
        }
        if settings.contains_key("demo_mode") {
            let new_mode = settings.get("demo_mode").and_then(|v| v.as_str()).unwrap_or("on");
            if new_mode == "off" {
                if let Err(e) = clear_demo_data(conn) {
                    log::warn!("clear demo data failed: {e}");
                }
            }
            if let Err(e) = crate::game::seed::seed_if_empty(conn) {
                log::warn!("seed after demo_mode update failed: {e}");
            }
        }
        Ok(())
    })?;
    get_settings(state)
}

#[tauri::command]
pub fn get_setting(state: State<'_, AppState>, key: String) -> AppResult<Option<String>> {
    state.db.with(|conn| read_setting(conn, &key).map_err(Into::into))
}

/// Nickname saved in the database, or `null` when the user has not picked
/// one yet (first launch / fresh database).
#[tauri::command]
pub fn get_player_name(state: State<'_, AppState>) -> AppResult<Option<String>> {
    state
        .db
        .with(|conn| read_setting(conn, PLAYER_NAME_KEY).map_err(Into::into))
}

/// Persists the player nickname in the database (upsert into `settings`).
/// Returns the stored (trimmed and length-checked) value.
///
/// The client displays it on the spot — no extra `get` round-trip is needed.
#[tauri::command]
pub fn set_player_name(state: State<'_, AppState>, name: String) -> AppResult<String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Other("player name must not be empty".into()));
    }
    if name.chars().count() > PLAYER_NAME_MAX_LEN {
        return Err(AppError::Other(format!(
            "player name is too long (max {PLAYER_NAME_MAX_LEN} characters)"
        )));
    }

    let stored = name.clone();
    state.db.with(|conn| {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![PLAYER_NAME_KEY, stored],
        )?;
        Ok(())
    })?;
    log::info!("Player name saved: \"{name}\"");
    Ok(name)
}
