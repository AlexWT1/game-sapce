use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::models::{Achievement, Game, GameSession, GameTheme};

pub const EXPORT_VERSION: i64 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportPayload {
    pub version: i64,
    pub exported_at: String,
    pub games: Vec<Game>,
    pub sessions: Vec<GameSession>,
    pub achievements: Vec<Achievement>,
    pub themes: Vec<GameTheme>,
    pub settings: Vec<SettingEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingEntry {
    pub key: String,
    pub value: String,
}

fn map_game(row: &rusqlite::Row<'_>) -> rusqlite::Result<Game> {
    Ok(Game {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        executable_path: row.get(3)?,
        install_path: row.get(4)?,
        cover_path: row.get(5)?,
        background_path: row.get(6)?,
        icon_path: row.get(7)?,
        developer: row.get(8)?,
        publisher: row.get(9)?,
        genre: row.get(10)?,
        release_year: row.get(11)?,
        platform: row.get(12)?,
        source: row.get(13)?,
        controller_support: row.get(14)?,
        is_installed: row.get::<_, i64>(15)? != 0,
        is_favorite: row.get::<_, i64>(16)? != 0,
        is_hidden: row.get::<_, i64>(17)? != 0,
        created_at: row.get(18)?,
        updated_at: row.get(19)?,
        last_played_at: row.get(20)?,
        total_playtime_seconds: row.get(21)?,
        launch_count: row.get(22)?,
        use_launcher: row.get::<_, i64>(23)? != 0,
        launcher_path: row.get(24)?,
        launcher_args: row.get(25).ok().flatten(),
    })
}

pub fn export(conn: &Connection) -> AppResult<String> {
    let games = {
        let mut stmt = conn.prepare(
            "SELECT id, title, description, executable_path, install_path, cover_path, background_path,
                    icon_path, developer, publisher, genre, release_year, platform, source, controller_support,
                    is_installed, is_favorite, is_hidden, created_at, updated_at, last_played_at,
                    total_playtime_seconds, launch_count, use_launcher, launcher_path, launcher_args
             FROM games ORDER BY id",
        )?;
        let rows = stmt.query_map([], map_game)?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let sessions = {
        let mut stmt = conn.prepare(
            "SELECT id, game_id, started_at, ended_at, duration_seconds, pid FROM game_sessions ORDER BY id",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(GameSession {
                id: r.get(0)?,
                game_id: r.get(1)?,
                started_at: r.get(2)?,
                ended_at: r.get(3)?,
                duration_seconds: r.get(4)?,
                pid: r.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let achievements = {
        let mut stmt = conn.prepare(
            "SELECT a.id, a.game_id, g.title, a.title, a.description, a.icon_path, a.rarity,
                    a.unlocked, a.unlocked_at, a.progress_current, a.progress_target, a.scope
             FROM achievements a LEFT JOIN games g ON g.id = a.game_id ORDER BY a.id",
        )?;
        let rows = stmt.query_map([], crate::game::achievements::map_achievement)?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let themes = {
        let mut stmt = conn.prepare(
            "SELECT game_id, primary_color, secondary_color, accent_color, background_color, glow_color, font_family
             FROM game_themes",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(GameTheme {
                game_id: r.get(0)?,
                primary_color: r.get(1)?,
                secondary_color: r.get(2)?,
                accent_color: r.get(3)?,
                background_color: r.get(4)?,
                glow_color: r.get(5)?,
                font_family: r.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let settings = {
        let mut stmt = conn.prepare("SELECT key, value FROM settings ORDER BY key")?;
        let rows = stmt.query_map([], |r| {
            Ok(SettingEntry {
                key: r.get(0)?,
                value: r.get(1)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let payload = ExportPayload {
        version: EXPORT_VERSION,
        exported_at: chrono::Utc::now().to_rfc3339(),
        games,
        sessions,
        achievements,
        themes,
        settings,
    };
    serde_json::to_string_pretty(&payload).map_err(AppError::from)
}

/// Imports a JSON export. Merges by title (skips existing).
pub fn import(conn: &Connection, json: &str) -> AppResult<(usize, usize)> {
    let payload: ExportPayload =
        serde_json::from_str(json).map_err(|e| AppError::validation(format!("Invalid file: {e}")))?;
    if payload.version > EXPORT_VERSION {
        return Err(AppError::validation(
            "Файл создан в более новой версии приложения",
        ));
    }

    let mut imported_games = 0usize;
    let mut imported_sessions = 0usize;

    for game in &payload.games {
        let exists: Option<i64> = conn
            .query_row("SELECT id FROM games WHERE title = ?1", [game.title.as_str()], |r| {
                r.get(0)
            })
            .optional()
            .map_err(AppError::Database)?;
        if exists.is_some() {
            continue;
        }
        conn.execute(
            "INSERT INTO games (title, description, executable_path, install_path, cover_path, background_path,
                icon_path, developer, publisher, genre, release_year, platform, source, controller_support,
                is_installed, is_favorite, is_hidden, created_at, updated_at, last_played_at,
                total_playtime_seconds, launch_count, use_launcher, launcher_path, launcher_args)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25)",
            rusqlite::params![
                game.title,
                game.description,
                game.executable_path,
                game.install_path,
                game.cover_path,
                game.background_path,
                game.icon_path,
                game.developer,
                game.publisher,
                game.genre,
                game.release_year,
                game.platform,
                game.source,
                game.controller_support,
                game.is_installed as i64,
                game.is_favorite as i64,
                game.is_hidden as i64,
                game.created_at,
                game.updated_at,
                game.last_played_at,
                game.total_playtime_seconds,
                game.launch_count,
                game.use_launcher as i64,
                game.launcher_path,
                game.launcher_args
            ],
        )
        .map_err(AppError::Database)?;
        imported_games += 1;
    }

    for session in &payload.sessions {
        let title: Option<String> = conn
            .query_row(
                "SELECT title FROM games WHERE id = ?1",
                [session.game_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(AppError::Database)?;
        let Some(title) = title else { continue };
        let new_id: Option<i64> = conn
            .query_row("SELECT id FROM games WHERE title = ?1", [&title], |r| r.get(0))
            .optional()
            .map_err(AppError::Database)?;
        let Some(new_game_id) = new_id else { continue };
        let dup: Option<i64> = conn
            .query_row(
                "SELECT id FROM game_sessions WHERE game_id = ?1 AND started_at = ?2",
                rusqlite::params![new_game_id, session.started_at],
                |r| r.get(0),
            )
            .optional()
            .map_err(AppError::Database)?;
        if dup.is_some() {
            continue;
        }
        conn.execute(
            "INSERT INTO game_sessions (game_id, started_at, ended_at, duration_seconds, pid)
             VALUES (?1,?2,?3,?4,NULL)",
            rusqlite::params![
                new_game_id,
                session.started_at,
                session.ended_at,
                session.duration_seconds
            ],
        )
        .map_err(AppError::Database)?;
        imported_sessions += 1;
    }

    for setting in &payload.settings {
        if setting.key == "onboarding_done" || setting.key.starts_with('_') {
            continue;
        }
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![setting.key, setting.value],
        )
        .map_err(AppError::Database)?;
    }

    for theme in &payload.themes {
        let title: Option<String> = conn
            .query_row("SELECT title FROM games WHERE id = ?1", [theme.game_id], |r| r.get(0))
            .optional()
            .map_err(AppError::Database)?;
        let Some(title) = title else { continue };
        let new_id: Option<i64> = conn
            .query_row("SELECT id FROM games WHERE title = ?1", [&title], |r| r.get(0))
            .optional()
            .map_err(AppError::Database)?;
        let Some(new_game_id) = new_id else { continue };
        conn.execute(
            "INSERT INTO game_themes (game_id, primary_color, secondary_color, accent_color, background_color, glow_color, font_family)
             VALUES (?1,?2,?3,?4,?5,?6,?7)
             ON CONFLICT(game_id) DO UPDATE SET
               primary_color = excluded.primary_color,
               secondary_color = excluded.secondary_color,
               accent_color = excluded.accent_color,
               background_color = excluded.background_color,
               glow_color = excluded.glow_color,
               font_family = excluded.font_family",
            rusqlite::params![
                new_game_id,
                theme.primary_color,
                theme.secondary_color,
                theme.accent_color,
                theme.background_color,
                theme.glow_color,
                theme.font_family
            ],
        )
        .map_err(AppError::Database)?;
    }

    log::info!(
        "Import finished: {imported_games} games, {imported_sessions} sessions"
    );
    Ok((imported_games, imported_sessions))
}

use rusqlite::OptionalExtension;
