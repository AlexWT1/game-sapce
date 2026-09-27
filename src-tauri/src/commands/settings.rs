use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension};
use tauri::State;

use crate::error::AppResult;
use crate::AppState;

pub type SettingsMap = HashMap<String, serde_json::Value>;

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
