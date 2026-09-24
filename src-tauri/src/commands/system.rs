use tauri::{AppHandle, Emitter, Manager, State};

use crate::error::{AppError, AppResult};
use crate::models::{DiscoveredGame, SearchResults};
use crate::services::{discovery, export};
use crate::AppState;

#[tauri::command]
pub fn window_minimize(app: AppHandle) -> AppResult<()> {
    let win = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::Other("window not found".into()))?;
    win.minimize().map_err(|e| AppError::Other(e.to_string()))
}

#[tauri::command]
pub fn window_toggle_maximize(app: AppHandle) -> AppResult<()> {
    let win = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::Other("window not found".into()))?;
    if win.is_maximized().unwrap_or(false) {
        win.unmaximize()
    } else {
        win.maximize()
    }
    .map_err(|e| AppError::Other(e.to_string()))
}

#[tauri::command]
pub fn window_close(app: AppHandle) -> AppResult<()> {
    let win = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::Other("window not found".into()))?;
    win.hide().map_err(|e| AppError::Other(e.to_string()))
}

#[tauri::command]
pub fn get_app_info(state: State<'_, AppState>) -> AppResult<serde_json::Value> {
    Ok(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "databasePath": state.db.path().display().to_string(),
        "platform": std::env::consts::OS,
    }))
}

#[tauri::command]
pub fn select_game_executable(app: AppHandle) -> AppResult<Option<String>> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app.dialog().file().blocking_pick_file();
    Ok(picked.and_then(|f| f.into_path().ok()).map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn select_game_directory(app: AppHandle) -> AppResult<Option<String>> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app.dialog().file().blocking_pick_folder();
    Ok(picked.and_then(|f| f.into_path().ok()).map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn select_any_file(app: AppHandle) -> AppResult<Option<String>> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app.dialog().file().blocking_pick_file();
    Ok(picked.and_then(|f| f.into_path().ok()).map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn discover_games(dirs: Vec<String>) -> AppResult<Vec<DiscoveredGame>> {
    let dirs = if dirs.is_empty() {
        crate::game::launcher::default_game_dirs()
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect()
    } else {
        dirs
    };
    log::info!("Discovering games in {} dirs", dirs.len());
    Ok(discovery::discover(&dirs))
}

#[tauri::command]
pub fn export_data(app: AppHandle, state: State<'_, AppState>, path: Option<String>) -> AppResult<String> {
    let json = state.db.with(export::export)?;
    let target = match path {
        Some(p) => p,
        None => {
            use tauri_plugin_dialog::DialogExt;
            let picked = app
                .dialog()
                .file()
                .set_file_name("game-space-export.json")
                .blocking_save_file();
            match picked.and_then(|f| f.into_path().ok()) {
                Some(p) => p.to_string_lossy().to_string(),
                None => return Err(AppError::validation("Операция отменена")),
            }
        }
    };
    std::fs::write(&target, json)?;
    log::info!("Exported data to {target}");
    Ok(target)
}

#[tauri::command]
pub fn import_data(app: AppHandle, state: State<'_, AppState>, path: Option<String>) -> AppResult<(usize, usize)> {
    let source = match path {
        Some(p) => p,
        None => {
            use tauri_plugin_dialog::DialogExt;
            let picked = app.dialog().file().add_filter("JSON", &["json"]).blocking_pick_file();
            match picked.and_then(|f| f.into_path().ok()) {
                Some(p) => p.to_string_lossy().to_string(),
                None => return Err(AppError::validation("Операция отменена")),
            }
        }
    };
    let json = std::fs::read_to_string(&source).map_err(|e| {
        AppError::Other(format!("Не удалось прочитать файл: {e}"))
    })?;
    let result = state.db.with(|conn| export::import(conn, &json))?;
    let _ = app.emit("statistics-updated", serde_json::json!({}));
    Ok(result)
}

#[tauri::command]
pub fn reset_data(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    state.db.with(|conn| {
        conn.execute("DELETE FROM game_sessions", [])?;
        conn.execute("DELETE FROM achievements", [])?;
        conn.execute("DELETE FROM game_themes", [])?;
        conn.execute("DELETE FROM game_tags", [])?;
        conn.execute("DELETE FROM games", [])?;
        conn.execute("DELETE FROM settings WHERE key = 'demo_seeded'", [])?;
        if let Err(e) = crate::game::achievements::ensure_app_achievements(conn) {
            log::warn!("re-ensure app achievements failed: {e}");
        }
        if let Err(e) = crate::game::seed::seed_if_empty(conn) {
            log::warn!("re-seed after reset failed: {e}");
        }
        Ok(())
    })?;
    log::warn!("User data reset");
    let _ = app.emit("statistics-updated", serde_json::json!({}));
    Ok(())
}

#[tauri::command]
pub fn search_all(state: State<'_, AppState>, query: String) -> AppResult<SearchResults> {
    let q = query.trim().to_string();
    if q.is_empty() {
        return Ok(SearchResults {
            games: Vec::new(),
            achievements: Vec::new(),
            genres: Vec::new(),
            developers: Vec::new(),
        });
    }
    let like = format!("%{q}%");
    state.db.with(|conn| {
        let demo_off = super::settings::demo_disabled(conn)?;
        let demo_clause = if demo_off { " AND source != 'Demo'" } else { "" };
        let games = {
            let mut stmt = conn.prepare(&format!(
                "SELECT {} FROM games WHERE is_hidden = 0{demo_clause} AND (
                    title LIKE ?1 OR genre LIKE ?1 OR developer LIKE ?1 OR publisher LIKE ?1
                 ) ORDER BY title LIMIT 20",
                super::games::SELECT_COLUMNS
            ))?;
            let rows = stmt.query_map([&like], super::games::map_game)?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let achievements = {
            let mut stmt = conn.prepare(&format!(
                "{} WHERE a.title LIKE ?1 OR a.description LIKE ?1 LIMIT 15",
                super::achievements::SELECT
            ))?;
            let rows = stmt.query_map([&like], crate::game::achievements::map_achievement)?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let genres = {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT genre FROM games WHERE genre LIKE ?1 AND genre != '' LIMIT 8",
            )?;
            let rows = stmt.query_map([&like], |r| r.get::<_, String>(0))?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let developers = {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT developer FROM games WHERE developer LIKE ?1 AND developer != '' LIMIT 8",
            )?;
            let rows = stmt.query_map([&like], |r| r.get::<_, String>(0))?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        Ok(SearchResults {
            games,
            achievements,
            genres,
            developers,
        })
    })
}

#[tauri::command]
pub fn open_path(app: AppHandle, path: String) -> AppResult<()> {
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| AppError::Other(e.to_string()))?;
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| AppError::Other(e.to_string()))?;
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| AppError::Other(e.to_string()))?;
    }
    let _ = app;
    Ok(())
}

#[tauri::command]
pub fn get_default_game_dirs() -> AppResult<Vec<String>> {
    Ok(crate::game::launcher::default_game_dirs()
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}
