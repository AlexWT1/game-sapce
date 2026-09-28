use rusqlite::{Connection, OptionalExtension};
use tauri::{AppHandle, Emitter, State};

use crate::error::{AppError, AppResult};
use crate::game::{achievements, theme};
use crate::models::{BulkDeleteResult, Game, GameInput, GameTheme};
use crate::process::monitor::ActiveMap;
use crate::services::placeholder::placeholder_cover;
use crate::AppState;

pub const SELECT_COLUMNS: &str = "id, title, description, executable_path, install_path, cover_path,
    background_path, icon_path, developer, publisher, genre, release_year, platform, source,
    controller_support, is_installed, is_favorite, is_hidden, created_at, updated_at,
    last_played_at, total_playtime_seconds, launch_count";

pub fn map_game(row: &rusqlite::Row<'_>) -> rusqlite::Result<Game> {
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
    })
}

pub fn get_game_internal(state: &AppState, id: i64) -> AppResult<Game> {
    state.db.with(|conn| get_game_conn(conn, id))
}

fn get_game_conn(conn: &Connection, id: i64) -> AppResult<Game> {
    conn.query_row(
        &format!("SELECT {SELECT_COLUMNS} FROM games WHERE id = ?1"),
        [id],
        map_game,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("game with id {id}"))
        }
        other => AppError::Database(other),
    })
}

#[tauri::command]
pub fn get_games(
    state: State<'_, AppState>,
    include_hidden: Option<bool>,
) -> AppResult<Vec<Game>> {
    let include_hidden = include_hidden.unwrap_or(false);
    state.db.with(|conn| {
        let demo_off = super::settings::demo_disabled(conn)?;
        let mut clauses: Vec<String> = Vec::new();
        if !include_hidden {
            clauses.push("is_hidden = 0".into());
        }
        if demo_off {
            clauses.push("source != 'Demo'".into());
        }
        let where_sql = if clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", clauses.join(" AND "))
        };
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM games{where_sql} ORDER BY last_played_at DESC, title ASC"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], map_game)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    })
}

#[tauri::command]
pub fn get_game(state: State<'_, AppState>, id: i64) -> AppResult<Game> {
    get_game_internal(&state, id)
}

#[tauri::command]
pub fn create_game(
    app: AppHandle,
    state: State<'_, AppState>,
    input: GameInput,
) -> AppResult<Game> {
    let title = input.title.clone().unwrap_or_default().trim().to_string();
    if title.is_empty() {
        return Err(AppError::validation("Название игры не может быть пустым"));
    }

    let now = chrono::Utc::now().to_rfc3339();
    let cover = input
        .cover_path
        .clone()
        .filter(|c| !c.trim().is_empty())
        .unwrap_or_else(|| placeholder_cover(&title));
    let exe = input.executable_path.clone().filter(|p| !p.trim().is_empty());
    let installed = input
        .is_installed
        .unwrap_or_else(|| exe.as_deref().map(|p| std::path::Path::new(p).exists()).unwrap_or(false));

    let id = state.db.with(|conn| {
        conn.execute(
            &format!(
                "INSERT INTO games (title, description, executable_path, install_path, cover_path,
                    background_path, icon_path, developer, publisher, genre, release_year, platform,
                    source, controller_support, is_installed, is_favorite, is_hidden,
                    created_at, updated_at, last_played_at, total_playtime_seconds, launch_count)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,NULL,0,0)"
            ),
            rusqlite::params![
                title,
                input.description.clone().unwrap_or_default(),
                exe,
                input.install_path,
                cover,
                input.background_path,
                input.icon_path,
                input.developer.clone().unwrap_or_default(),
                input.publisher.clone().unwrap_or_default(),
                input.genre.clone().unwrap_or_default(),
                input.release_year,
                input.platform.clone().unwrap_or_else(|| "PC".into()),
                input
                    .source
                    .clone()
                    .unwrap_or_else(|| "Manual".into()),
                input
                    .controller_support
                    .clone()
                    .unwrap_or_else(|| "unknown".into()),
                installed as i64,
                input.is_favorite.unwrap_or(false) as i64,
                input.is_hidden.unwrap_or(false) as i64,
                now,
                now,
            ],
        )?;
        let id = conn.last_insert_rowid();
        achievements::ensure_game_achievements(conn, id, &title)?;
        theme::auto_theme(conn, id, &title, &input.genre.clone().unwrap_or_default())?;
        Ok(id)
    })?;

    log::info!("Game created: {title} (#{id})");
    achievements::evaluate(&app);
    let _ = app.emit("statistics-updated", serde_json::json!({}));
    get_game_internal(&state, id)
}

#[tauri::command]
pub fn update_game(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
    input: GameInput,
) -> AppResult<Game> {
    let mut game = get_game_internal(&state, id)?;

    if let Some(t) = input.title {
        let t = t.trim().to_string();
        if t.is_empty() {
            return Err(AppError::validation("Название игры не может быть пустым"));
        }
        game.title = t;
    }
    if let Some(d) = input.description {
        game.description = d;
    }
    if let Some(e) = input.executable_path {
        let e = e.trim().to_string();
        game.executable_path = if e.is_empty() { None } else { Some(e) };
        if let Some(path) = &game.executable_path {
            game.is_installed = std::path::Path::new(path).exists();
        }
    }
    if let Some(p) = input.install_path {
        game.install_path = if p.trim().is_empty() { None } else { Some(p) };
    }
    if let Some(c) = input.cover_path {
        game.cover_path = if c.trim().is_empty() { None } else { Some(c) };
    }
    if let Some(b) = input.background_path {
        game.background_path = if b.trim().is_empty() { None } else { Some(b) };
    }
    if let Some(i) = input.icon_path {
        game.icon_path = if i.trim().is_empty() { None } else { Some(i) };
    }
    if let Some(d) = input.developer {
        game.developer = d;
    }
    if let Some(p) = input.publisher {
        game.publisher = p;
    }
    if let Some(g) = input.genre {
        game.genre = g;
    }
    if input.release_year.is_some() {
        game.release_year = input.release_year;
    }
    if let Some(p) = input.platform {
        game.platform = p;
    }
    if let Some(s) = input.source {
        game.source = s;
    }
    if let Some(c) = input.controller_support {
        game.controller_support = c;
    }
    if let Some(v) = input.is_installed {
        game.is_installed = v;
    }
    if let Some(v) = input.is_favorite {
        game.is_favorite = v;
    }
    if let Some(v) = input.is_hidden {
        game.is_hidden = v;
    }

    let now = chrono::Utc::now().to_rfc3339();
    state.db.with(|conn| {
        conn.execute(
            "UPDATE games SET title=?1, description=?2, executable_path=?3, install_path=?4,
                cover_path=?5, background_path=?6, icon_path=?7, developer=?8, publisher=?9,
                genre=?10, release_year=?11, platform=?12, source=?13, controller_support=?14,
                is_installed=?15, is_favorite=?16, is_hidden=?17, updated_at=?18
             WHERE id=?19",
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
                now,
                id
            ],
        )?;
        Ok(())
    })?;

    log::info!("Game #{} updated", id);
    achievements::evaluate(&app);
    get_game_internal(&state, id)
}

#[tauri::command]
pub fn delete_game(app: AppHandle, state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let game = get_game_internal(&state, id)?;
    if let Some(active) = state.active.lock().ok().and_then(|m| m.get(&id).cloned()) {
        let _ = active;
        return Err(AppError::Conflict(
            "Игра сейчас запущена. Сначала завершите сессию.".into(),
        ));
    }
    state.db.with(|conn| {
        conn.execute("DELETE FROM games WHERE id = ?1", [id])?;
        Ok(())
    })?;
    log::info!("Game deleted: {} (#{} )", game.title, id);
    let _ = app.emit("game-deleted", serde_json::json!({ "gameId": id }));
    let _ = app.emit("statistics-updated", serde_json::json!({}));
    Ok(())
}

/// Removes several library entries at once (running games are skipped).
#[tauri::command]
pub fn delete_games(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<i64>,
) -> AppResult<BulkDeleteResult> {
    let result = state.db.with(|conn| bulk_delete(conn, &state.active, ids))?;
    if !result.deleted_ids.is_empty() {
        let _ = app.emit(
            "games-deleted",
            serde_json::json!({ "ids": result.deleted_ids }),
        );
        let _ = app.emit("statistics-updated", serde_json::json!({}));
    }
    Ok(result)
}

pub(crate) fn bulk_delete(
    conn: &Connection,
    active: &ActiveMap,
    ids: Vec<i64>,
) -> AppResult<BulkDeleteResult> {
    let mut targets: Vec<i64> = ids;
    targets.sort_unstable();
    targets.dedup();

    let mut skipped = 0i64;
    {
        let active = active
            .lock()
            .map_err(|_| AppError::Other("active sessions lock poisoned".into()))?;
        targets.retain(|id| {
            if active.contains_key(id) {
                skipped += 1;
                false
            } else {
                true
            }
        });
    }
    if targets.is_empty() {
        return Ok(BulkDeleteResult {
            deleted_ids: Vec::new(),
            skipped,
        });
    }

    let mut removed: Vec<(i64, String)> = Vec::new();
    conn.execute_batch("BEGIN IMMEDIATE")?;
    let outcome: AppResult<()> = (|| {
        for id in &targets {
            let title: Option<String> = conn
                .query_row("SELECT title FROM games WHERE id = ?1", [*id], |r| r.get(0))
                .optional()?;
            if let Some(title) = title {
                conn.execute("DELETE FROM games WHERE id = ?1", [*id])?;
                removed.push((*id, title));
            }
        }
        Ok(())
    })();
    match outcome {
        Ok(()) => conn.execute_batch("COMMIT")?,
        Err(err) => {
            let _ = conn.execute_batch("ROLLBACK");
            return Err(err);
        }
    }

    let deleted = removed.len();
    let titles: Vec<&str> = removed.iter().map(|(_, title)| title.as_str()).collect();
    log::info!(
        "Bulk delete: removed {deleted} games ({}), skipped {skipped} running",
        titles.join(", ")
    );
    let deleted_ids: Vec<i64> = removed.into_iter().map(|(id, _)| id).collect();
    Ok(BulkDeleteResult { deleted_ids, skipped })
}

#[tauri::command]
pub fn toggle_favorite(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<Game> {
    let mut game = get_game_internal(&state, id)?;
    game.is_favorite = !game.is_favorite;
    state.db.with(|conn| {
        conn.execute(
            "UPDATE games SET is_favorite = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![game.is_favorite as i64, chrono::Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    })?;
    achievements::evaluate(&app);
    Ok(game)
}

#[tauri::command]
pub fn set_game_executable(
    state: State<'_, AppState>,
    id: i64,
    path: String,
) -> AppResult<Game> {
    if path.trim().is_empty() {
        return Err(AppError::validation("Path is empty"));
    }
    let exists = std::path::Path::new(&path).exists();
    state.db.with(|conn| {
        conn.execute(
            "UPDATE games SET executable_path = ?1, is_installed = ?2, updated_at = ?3 WHERE id = ?4",
            rusqlite::params![path, exists as i64, chrono::Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    })?;
    get_game_internal(&state, id)
}

#[tauri::command]
pub fn get_game_theme(state: State<'_, AppState>, game_id: i64) -> AppResult<GameTheme> {
    state.db.with(|conn| theme::get(conn, game_id))
}

#[tauri::command]
pub fn set_game_theme(
    state: State<'_, AppState>,
    theme_input: GameTheme,
) -> AppResult<GameTheme> {
    state.db.with(|conn| theme::upsert(conn, &theme_input))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::monitor::ActiveSession;

    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().expect("memory db");
        conn.pragma_update(None, "foreign_keys", "ON").expect("pragma");
        crate::database::migrations::run(&conn).expect("migrations");
        conn
    }

    fn insert_game(conn: &Connection, title: &str) -> i64 {
        conn.execute(
            "INSERT INTO games (title, created_at, updated_at) VALUES (?1, 't', 't')",
            [title],
        )
        .expect("insert game");
        conn.last_insert_rowid()
    }

    #[test]
    fn bulk_delete_removes_selected_and_skips_running() {
        let conn = memory_db();
        let first = insert_game(&conn, "Alpha");
        let running = insert_game(&conn, "Beta");
        let last = insert_game(&conn, "Gamma");

        let active: ActiveMap = Default::default();
        active.lock().expect("active lock").insert(
            running,
            ActiveSession {
                session_id: 1,
                pid: 4242,
                started_at: "t".into(),
            },
        );

        let result =
            bulk_delete(&conn, &active, vec![first, running, last, running]).expect("bulk delete");
        assert_eq!(result.deleted_ids, vec![first, last]);
        assert_eq!(result.skipped, 1);

        let remaining: String = conn
            .query_row("SELECT title FROM games", [], |r| r.get(0))
            .expect("remaining game");
        assert_eq!(remaining, "Beta");
    }

    #[test]
    fn bulk_delete_without_targets_is_a_noop() {
        let conn = memory_db();
        insert_game(&conn, "Alpha");
        let active: ActiveMap = Default::default();

        let result = bulk_delete(&conn, &active, Vec::new()).expect("bulk delete");
        assert!(result.deleted_ids.is_empty());
        assert_eq!(result.skipped, 0);

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))
            .expect("count");
        assert_eq!(count, 1);
    }
}
