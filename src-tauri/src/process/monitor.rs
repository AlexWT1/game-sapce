use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use rusqlite::OptionalExtension;
use tauri::{AppHandle, Emitter, Manager};
use sysinfo::{Pid, ProcessesToUpdate, System};

use crate::game::achievements;
use crate::models::RunningGame;
use crate::AppState;

#[derive(Debug, Clone)]
pub struct ActiveSession {
    pub session_id: i64,
    pub pid: u32,
    pub started_at: String,
}

pub type ActiveMap = Arc<Mutex<HashMap<i64, ActiveSession>>>;

pub fn pid_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    let mut sys = System::new();
    let target = Pid::from_u32(pid);
    sys.refresh_processes(ProcessesToUpdate::Some(&[target]), true);
    sys.process(target).is_some()
}

pub fn kill_pid(pid: u32) -> bool {
    let mut sys = System::new();
    let target = Pid::from_u32(pid);
    sys.refresh_processes(ProcessesToUpdate::Some(&[target]), true);
    match sys.process(target) {
        Some(p) => p.kill(),
        None => false,
    }
}

/// Reconciles sessions that were left open when the app was closed.
pub fn recover_sessions(app: &AppHandle) {
    let state = app.state::<AppState>();
    let open: Vec<(i64, i64, String, Option<i64>)> = match state.db.with(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, game_id, started_at, pid FROM game_sessions WHERE ended_at IS NULL",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }) {
        Ok(v) => v,
        Err(e) => {
            log::error!("recover_sessions: {e}");
            return;
        }
    };

    for (session_id, game_id, started_at, pid) in open {
        let alive = pid.map(|p| pid_alive(p as u32)).unwrap_or(false);
        if alive {
            if let Ok(mut active) = state.active.lock() {
                active.insert(
                    game_id,
                    ActiveSession {
                        session_id,
                        pid: pid.unwrap_or(0) as u32,
                        started_at: started_at.clone(),
                    },
                );
            }
            log::info!("Recovered running session for game {game_id} (pid {pid:?})");
            let _ = app.emit(
                "game-started",
                serde_json::json!({ "gameId": game_id, "recovered": true }),
            );
        } else {
            // Process died while we were not running — close the session.
            let started = DateTime::parse_from_rfc3339(&started_at)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let duration = (Utc::now() - started).num_seconds().max(0);
            let res = state.db.with(|conn| {
                conn.execute(
                    "UPDATE game_sessions SET ended_at = ?1, duration_seconds = ?2 WHERE id = ?3 AND ended_at IS NULL",
                    rusqlite::params![Utc::now().to_rfc3339(), duration, session_id],
                )?;
                conn.execute(
                    "UPDATE games SET total_playtime_seconds = total_playtime_seconds + ?1 WHERE id = ?2",
                    rusqlite::params![duration, game_id],
                )?;
                Ok(())
            });
            if let Err(e) = res {
                log::error!("Failed to close orphan session {session_id}: {e}");
            } else {
                log::info!("Closed orphan session {session_id} ({duration}s)");
                let _ = app.emit("game-stopped", serde_json::json!({ "gameId": game_id }));
                achievements::evaluate(app);
            }
        }
    }
}

/// Finalizes an active session: writes duration, updates playtime, emits events.
pub fn finish_session(app: &AppHandle, game_id: i64) -> Result<i64, crate::error::AppError> {
    let state = app.state::<AppState>();
    let active_session = state
        .active
        .lock()
        .ok()
        .and_then(|mut map| map.remove(&game_id));

    let (session_id, started_at) = match active_session {
        Some(a) => (a.session_id, a.started_at),
        None => {
            // No in-memory session; try to close the newest open row.
            let row = state.db.with(|conn| {
                conn.query_row(
                    "SELECT id, started_at FROM game_sessions WHERE game_id = ?1 AND ended_at IS NULL ORDER BY id DESC LIMIT 1",
                    [game_id],
                    |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
                )
                .map_err(|_| crate::error::AppError::NotFound("active session".into()))
            });
            match row {
                Ok(v) => v,
                Err(_) => return Ok(0),
            }
        }
    };

    let started = DateTime::parse_from_rfc3339(&started_at)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    let duration = (Utc::now() - started).num_seconds().max(1);
    let now = Utc::now().to_rfc3339();

    state.db.with(|conn| {
        let updated = conn.execute(
            "UPDATE game_sessions SET ended_at = ?1, duration_seconds = ?2 WHERE id = ?3 AND ended_at IS NULL",
            rusqlite::params![now, duration, session_id],
        )?;
        if updated == 0 {
            // Another path already closed this session — do not double-count playtime.
            return Ok(());
        }
        conn.execute(
            "UPDATE games SET total_playtime_seconds = total_playtime_seconds + ?1, last_played_at = ?2 WHERE id = ?3",
            rusqlite::params![duration, now, game_id],
        )?;
        Ok(())
    })?;

    log::info!(
        "Session {} finished for game {}: {}s",
        session_id,
        game_id,
        duration
    );

    let _ = app.emit(
        "game-stopped",
        serde_json::json!({ "gameId": game_id, "durationSeconds": duration }),
    );
    let _ = app.emit("statistics-updated", serde_json::json!({}));
    achievements::evaluate(app);

    Ok(duration)
}

/// Background monitoring loop: checks active pids every `interval_secs`.
pub fn start_monitor(app: AppHandle) {
    std::thread::spawn(move || loop {
        let interval = monitoring_interval(&app);
        std::thread::sleep(Duration::from_secs(interval));

        let active_map = app.state::<AppState>().active.clone();
        let snapshot: Vec<(i64, u32)> = match active_map.lock() {
            Ok(map) => map.iter().map(|(k, v)| (*k, v.pid)).collect(),
            Err(_) => Vec::new(),
        };

        for (game_id, pid) in snapshot {
            if !pid_alive(pid) {
                log::info!("Process {pid} for game {game_id} exited");
                if let Err(e) = finish_session(&app, game_id) {
                    log::error!("Failed to finish session for game {game_id}: {e}");
                }
            }
        }

        update_tray_session_text(&app);
    });
}

fn monitoring_interval(app: &AppHandle) -> u64 {
    let state = app.state::<AppState>();
    let raw: Option<String> = state
        .db
        .with(|conn| {
            conn.query_row(
                "SELECT value FROM settings WHERE key = 'monitoring_interval'",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(crate::error::AppError::Database)
        })
        .unwrap_or(None);
    raw.and_then(|v| v.parse::<u64>().ok()).unwrap_or(2).clamp(1, 10)
}

fn update_tray_session_text(app: &AppHandle) {
    let state = app.state::<AppState>();
    let running = running_games(app);
    let text = running
        .first()
        .map(|g| {
            let active = state.active.lock().ok().and_then(|m| m.get(&g.game_id).cloned());
            let elapsed = active
                .map(|a| {
                    DateTime::parse_from_rfc3339(&a.started_at)
                        .map(|s| (Utc::now() - s.with_timezone(&Utc)).num_seconds().max(0))
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            format!("{} — {}", g.title, crate::game::format_hms(elapsed))
        })
        .unwrap_or_else(|| "Нет активной сессии".into());

    if let Ok(guard) = state.tray_session_item.lock() {
        if let Some(item) = guard.as_ref() {
            let _ = item.set_text(text);
        }
    }

    let tooltip = running
        .first()
        .map(|g| format!("Game Space — В игре: {}", g.title))
        .unwrap_or_else(|| "Game Space".into());
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(tooltip));
    }
}

pub fn running_games(app: &AppHandle) -> Vec<RunningGame> {
    let state = app.state::<AppState>();
    let active: Vec<(i64, ActiveSession)> = match state.active.lock() {
        Ok(map) => map.iter().map(|(k, v)| (*k, v.clone())).collect(),
        Err(_) => return Vec::new(),
    };

    let mut out = Vec::new();
    for (game_id, session) in active {
        if !pid_alive(session.pid) {
            continue;
        }
        if let Ok(game) = crate::commands::games::get_game_internal(&state, game_id) {
            out.push(RunningGame {
                game_id,
                title: game.title,
                pid: session.pid as i64,
                started_at: session.started_at,
                cover_path: game.cover_path,
            });
        }
    }
    out
}
