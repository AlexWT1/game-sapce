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
    let exe = game
        .executable_path
        .as_deref()
        .ok_or_else(|| AppError::launch("Executable не указан. Укажите файл игры."))?;

    let path = crate::game::launcher::validate_executable(exe)?;
    let child = crate::game::launcher::spawn_game(&path)?;
    let pid = child.id();
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
                updated_at = ?1, is_installed = 1
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

#[tauri::command]
pub fn stop_game(app: AppHandle, state: State<'_, AppState>, game_id: i64) -> AppResult<()> {
    let pid = {
        let active = state
            .active
            .lock()
            .map_err(|_| AppError::Other("lock poisoned".into()))?;
        active.get(&game_id).map(|s| s.pid)
    };

    if let Some(pid) = pid {
        if monitor::kill_pid(pid) {
            log::info!("Killed pid {pid} for game {game_id}");
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
    Ok(active
        .get(&game_id)
        .map(|s| monitor::pid_alive(s.pid))
        .unwrap_or(false))
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
