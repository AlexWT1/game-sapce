use rusqlite::OptionalExtension;
use tauri::{AppHandle, Emitter, State};

use crate::error::{AppError, AppResult};
use crate::models::GameSession;
use crate::process::monitor;
use crate::AppState;

#[tauri::command]
pub fn launch_game(
    app: AppHandle,
    state: State<'_, AppState>,
    game_id: i64,
) -> AppResult<crate::models::RunningGame> {
    // Already running?
    {
        let active = state
            .active
            .lock()
            .map_err(|_| AppError::Other("lock poisoned".into()))?;
        if let Some(existing) = active.get(&game_id) {
            if monitor::pid_alive(existing.pid) {
                let game = crate::commands::games::get_game_internal(&state, game_id)?;
                return Ok(crate::models::RunningGame {
                    game_id,
                    title: game.title,
                    pid: existing.pid as i64,
                    started_at: existing.started_at.clone(),
                    cover_path: game.cover_path,
                });
            }
        }
    }

    let game = crate::commands::games::get_game_internal(&state, game_id)?;

    // Steam games are started through the steam:// protocol and have no local executable.
    // A working local executable always wins over the Steam link.
    let local_exe_ready = game
        .executable_path
        .as_deref()
        .map(|path| !path.trim().is_empty() && std::path::Path::new(path).exists())
        .unwrap_or(false);

    // Launcher-based games spawn their own launcher; the game is started from it.
    let launcher_ready = game.use_launcher
        && game
            .launcher_path
            .as_deref()
            .map(|p| std::path::Path::new(p).exists())
            .unwrap_or(false);

    // Never start a session (or open Steam's download prompt) for a game that
    // is not installed — it would count playtime for a game nobody is playing.
    if !game.is_installed && !local_exe_ready && !launcher_ready {
        return Err(AppError::validation("Игра не установлена"));
    }

    let pid = if launcher_ready {
        // Spawn the launcher and track the session by the game folder (pid = 0):
        // the launcher may close itself once the game process is up.
        let launcher = game
            .launcher_path
            .as_deref()
            .ok_or_else(|| AppError::launch("Путь к лаунчеру не указан. Укажите файл лаунчера."))?;
        let path = crate::game::launcher::validate_executable(launcher)?;
        let args: Vec<String> = game
            .launcher_args
            .as_deref()
            .map(crate::game::launcher::split_args)
            .unwrap_or_default();
        crate::game::launcher::spawn_game(&path, &args)?;
        log::info!(
            "Launched {} via its own launcher (args: {:?})",
            game.title,
            args
        );
        0u32
    } else if local_exe_ready {
        let exe = game
            .executable_path
            .as_deref()
            .ok_or_else(|| AppError::launch("Executable не указан. Укажите файл игры."))?;
        let path = crate::game::launcher::validate_executable(exe)?;
        crate::game::launcher::spawn_game(&path, &[])?.id()
    } else {
        // Check Epic Games protocol first if linked in `epic_games`, otherwise Steam.
        let epic_entry: Option<(String, String, String)> = state.db.with(|conn| {
            conn.query_row(
                "SELECT app_name, catalog_item_id, namespace FROM epic_games WHERE game_id = ?1",
                [game_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(AppError::Database)
        })?;

        if let Some((app_name, catalog_item_id, namespace)) = epic_entry {
            let launch_id = if !namespace.is_empty() && !catalog_item_id.is_empty() {
                format!("{namespace}%3A{catalog_item_id}%3A{app_name}")
            } else {
                app_name
            };
            crate::services::opener::open_url(&format!(
                "com.epicgames.launcher://apps/{launch_id}?action=launch&silent=true"
            ))?;
            0u32
        } else {
            // Steam launch via steam:// protocol.
            let appid: i64 = state.db.with(|conn| {
                conn.query_row(
                    "SELECT appid FROM steam_games WHERE game_id = ?1",
                    [game_id],
                    |r| r.get::<_, i64>(0),
                )
                .optional()
                .map_err(AppError::Database)
            })?.ok_or_else(|| AppError::NotFound("Steam appid not found for this game".into()))?;
            crate::services::opener::open_url(&format!("steam://rungameid/{appid}"))?;
            0u32
        }
    };
    let now = chrono::Utc::now().to_rfc3339();

    let session_id = state.db.with(|conn| {
        conn.execute(
            "INSERT INTO game_sessions (game_id, started_at, ended_at, duration_seconds, pid)
             VALUES (?1, ?2, NULL, 0, ?3)",
            rusqlite::params![game_id, now, pid as i64],
        )?;
        let id = conn.last_insert_rowid();
        conn.execute(
            "UPDATE games SET launch_count = launch_count + 1, last_played_at = ?1,
                updated_at = ?1
             WHERE id = ?2",
            rusqlite::params![now, game_id],
        )?;
        Ok(id)
    })?;

    {
        let mut active = state
            .active
            .lock()
            .map_err(|_| AppError::Other("lock poisoned".into()))?;
        active.insert(
            game_id,
            monitor::ActiveSession {
                session_id,
                pid,
                started_at: now.clone(),
            },
        );
    }

    log::info!("Launched {} (game #{}, pid {pid})", game.title, game_id);
    let _ = app.emit(
        "game-started",
        serde_json::json!({ "gameId": game_id, "pid": pid, "title": game.title }),
    );
    crate::game::achievements::evaluate(&app);
    let _ = app.emit("statistics-updated", serde_json::json!({}));

    Ok(crate::models::RunningGame {
        game_id,
        title: game.title,
        pid: pid as i64,
        started_at: now,
        cover_path: game.cover_path,
    })
}

/// Opens the Steam install wizard for a game that is not installed locally.
#[tauri::command]
pub fn download_game(state: State<'_, AppState>, game_id: i64) -> AppResult<()> {
    let appid: Option<i64> = state.db.with(|conn| {
        conn.query_row(
            "SELECT appid FROM steam_games WHERE game_id = ?1",
            [game_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(AppError::Database)
    })?;
    let Some(appid) = appid else {
        return Err(AppError::validation("Игра не привязана к Steam"));
    };
    crate::services::opener::open_url(&format!("steam://install/{appid}"))?;
    log::info!("Opened Steam install for game #{game_id} (appid {appid})");
    Ok(())
}

#[tauri::command]
pub fn stop_game(app: AppHandle, state: State<'_, AppState>, game_id: i64) -> AppResult<()> {
    let session = {
        let active = state
            .active
            .lock()
            .map_err(|_| AppError::Other("lock poisoned".into()))?;
        active.get(&game_id).cloned()
    };

    if let Some(session) = session {
        if session.pid == 0 {
            // Started via steam:// or a game launcher — pid is unknown, stop
            // processes from the game folder.
            if let Ok(game) = crate::commands::games::get_game_internal(&state, game_id) {
                let mut killed = false;
                if let Some(path) = game.install_path.as_deref().filter(|p| !p.trim().is_empty()) {
                    if monitor::kill_by_path(&path) {
                        log::info!("Killed processes of game {game_id} under {path}");
                        killed = true;
                    }
                }
                if !killed {
                    if let Some(exe) = game.executable_path.as_deref().filter(|p| !p.trim().is_empty()) {
                        if let Some(dir) = std::path::Path::new(exe).parent() {
                            if !dir.as_os_str().is_empty() {
                                let dir_str = dir.to_string_lossy().to_string();
                                if monitor::kill_by_path(&dir_str) {
                                    log::info!("Killed processes of game {game_id} under {dir_str}");
                                }
                            }
                        }
                    }
                }
            }
        } else if monitor::kill_pid(session.pid) {
            log::info!("Killed pid {} for game {game_id}", session.pid);
        }
        // Finalize immediately; monitor would do it otherwise.
        monitor::finish_session(&app, game_id)?;
    } else {
        monitor::finish_session(&app, game_id)?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_running_games(app: AppHandle) -> AppResult<Vec<crate::models::RunningGame>> {
    Ok(monitor::running_games(&app))
}

#[tauri::command]
pub fn is_game_running(state: State<'_, AppState>, game_id: i64) -> AppResult<bool> {
    let active = state
        .active
        .lock()
        .map_err(|_| AppError::Other("lock poisoned".into()))?;
    Ok(match active.get(&game_id) {
        Some(session) => monitor::session_alive(&state, game_id, session),
        None => false,
    })
}

#[tauri::command]
pub fn start_game_session(state: State<'_, AppState>, game_id: i64) -> AppResult<GameSession> {
    let now = chrono::Utc::now().to_rfc3339();
    let id = state.db.with(|conn| {
        conn.execute(
            "INSERT INTO game_sessions (game_id, started_at, ended_at, duration_seconds, pid)
             VALUES (?1, ?2, NULL, 0, NULL)",
            rusqlite::params![game_id, now],
        )?;
        Ok(conn.last_insert_rowid())
    })?;
    get_session(&state, id)
}

#[tauri::command]
pub fn finish_game_session(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: i64,
) -> AppResult<GameSession> {
    let game_id: i64 = state.db.with(|conn| {
        conn.query_row(
            "SELECT game_id FROM game_sessions WHERE id = ?1",
            [session_id],
            |r| r.get(0),
        )
        .map_err(|_| AppError::NotFound(format!("session {session_id}")))
    })?;
    monitor::finish_session(&app, game_id)?;
    get_session(&state, session_id)
}

#[tauri::command]
pub fn get_game_sessions(
    state: State<'_, AppState>,
    game_id: i64,
) -> AppResult<Vec<GameSession>> {
    state.db.with(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, game_id, started_at, ended_at, duration_seconds, pid
             FROM game_sessions WHERE game_id = ?1 ORDER BY started_at DESC LIMIT 500",
        )?;
        let rows = stmt.query_map([game_id], map_session)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    })
}

fn get_session(state: &AppState, id: i64) -> AppResult<GameSession> {
    state.db.with(|conn| {
        conn.query_row(
            "SELECT id, game_id, started_at, ended_at, duration_seconds, pid
             FROM game_sessions WHERE id = ?1",
            [id],
            map_session,
        )
        .map_err(|_| AppError::NotFound(format!("session {id}")))
    })
}

fn map_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<GameSession> {
    Ok(GameSession {
        id: row.get(0)?,
        game_id: row.get(1)?,
        started_at: row.get(2)?,
        ended_at: row.get(3)?,
        duration_seconds: row.get(4)?,
        pid: row.get(5)?,
    })
}
