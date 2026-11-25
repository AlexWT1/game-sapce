use std::collections::{HashMap, HashSet};
use std::sync::atomic::Ordering;

use rusqlite::{Connection, OptionalExtension};
use tauri::{AppHandle, Emitter, State};

use crate::error::{AppError, AppResult};
use crate::game::{achievements, theme};
use crate::models::{BulkDeleteResult, Game, GameInput, GameTheme, InstallStatusChange};
use crate::process::monitor::ActiveMap;
use crate::services::placeholder::placeholder_cover;
use crate::services::steam::manifest;
use crate::AppState;

pub const SELECT_COLUMNS: &str = "id, title, description, executable_path, install_path, cover_path,
    background_path, icon_path, developer, publisher, genre, release_year, platform, source,
    controller_support, is_installed, is_favorite, is_hidden, created_at, updated_at,
    last_played_at, total_playtime_seconds, launch_count, use_launcher, launcher_path, launcher_args";

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
        use_launcher: row.get::<_, i64>(23)? != 0,
        launcher_path: row.get(24)?,
        // `launcher_args` is a NOT NULL column; an empty value means "no args".
        launcher_args: row
            .get::<_, Option<String>>(25)?
            .filter(|a| !a.is_empty()),
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
    let launcher = input.launcher_path.clone().filter(|p| !p.trim().is_empty());
    let launcher_args = input
        .launcher_args
        .clone()
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty());
    let use_launcher = input.use_launcher.unwrap_or(false) && launcher.is_some();
    let installed = input.is_installed.unwrap_or_else(|| {
        if use_launcher {
            launcher
                .as_deref()
                .map(|p| std::path::Path::new(p).exists())
                .unwrap_or(false)
        } else {
            exe.as_deref()
                .map(|p| std::path::Path::new(p).exists())
                .unwrap_or(false)
        }
    });

    let id = state.db.with(|conn| {
        conn.execute(
            &format!(
                "INSERT INTO games (title, description, executable_path, install_path, cover_path,
                    background_path, icon_path, developer, publisher, genre, release_year, platform,
                    source, controller_support, is_installed, is_favorite, is_hidden,
                    created_at, updated_at, last_played_at, total_playtime_seconds, launch_count,
                    use_launcher, launcher_path, launcher_args)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,NULL,0,0,?20,?21,?22)"
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
                use_launcher as i64,
                launcher,
                // The column is NOT NULL — an empty string means "no args".
                launcher_args.as_deref().unwrap_or(""),
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
        // Launcher-based games verify installation against the launcher executable.
        if !game.use_launcher {
            if let Some(path) = &game.executable_path {
                game.is_installed = std::path::Path::new(path).exists();
            }
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
    // Launcher path first so `use_launcher` sees the fresh value.
    let mut launcher_touched = false;
    if let Some(l) = input.launcher_path {
        let l = l.trim().to_string();
        game.launcher_path = if l.is_empty() { None } else { Some(l) };
        launcher_touched = true;
    }
    if let Some(a) = input.launcher_args {
        let a = a.trim().to_string();
        game.launcher_args = if a.is_empty() { None } else { Some(a) };
    }
    if let Some(v) = input.use_launcher {
        game.use_launcher = v && game.launcher_path.is_some();
        launcher_touched = true;
    }
    if launcher_touched && game.use_launcher {
        game.is_installed = game
            .launcher_path
            .as_deref()
            .map(|p| std::path::Path::new(p).exists())
            .unwrap_or(false);
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
                is_installed=?15, is_favorite=?16, is_hidden=?17, updated_at=?18,
                use_launcher=?19, launcher_path=?20, launcher_args=?21
             WHERE id=?22",
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
                game.use_launcher as i64,
                game.launcher_path,
                // The column is NOT NULL — an empty string means "no args".
                game.launcher_args.as_deref().unwrap_or(""),
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
    let game = get_game_internal(&state, id)?;
    let exists = std::path::Path::new(&path).exists();
    state.db.with(|conn| {
        if game.use_launcher {
            // For launcher-based games the picked file is the launcher executable.
            conn.execute(
                "UPDATE games SET launcher_path = ?1, is_installed = ?2, updated_at = ?3 WHERE id = ?4",
                rusqlite::params![path, exists as i64, chrono::Utc::now().to_rfc3339(), id],
            )?;
        } else {
            conn.execute(
                "UPDATE games SET executable_path = ?1, is_installed = ?2, updated_at = ?3 WHERE id = ?4",
                rusqlite::params![path, exists as i64, chrono::Utc::now().to_rfc3339(), id],
            )?;
        }
        Ok(())
    })?;
    get_game_internal(&state, id)
}

/// Resolves the real install state of one game.
///
/// Returns `None` when there is nothing to verify against (no paths, no Steam
/// link, or Steam itself is not found) so the stored flag is left untouched.
/// The second value is a refreshed `install_path` when Steam reports a new one.
fn verify_target(
    use_launcher: bool,
    launcher_path: Option<&str>,
    executable_path: Option<&str>,
    install_path: Option<&str>,
    appid: Option<u32>,
    steam_apps: Option<&HashMap<u32, String>>,
) -> Option<(bool, Option<String>)> {
    if let Some(appid) = appid {
        let apps = steam_apps?;
        let installed = apps.contains_key(&appid);
        let fresh_path = apps
            .get(&appid)
            .cloned()
            .filter(|p| Some(p.as_str()) != install_path);
        return Some((installed, fresh_path));
    }
    // Launcher-based games are installed as long as the launcher itself exists.
    if use_launcher {
        if let Some(l) = launcher_path.filter(|p| !p.trim().is_empty()) {
            return Some((std::path::Path::new(l).exists(), None));
        }
        return Some((false, None));
    }
    if let Some(exe) = executable_path.filter(|p| !p.trim().is_empty()) {
        return Some((std::path::Path::new(exe).exists(), None));
    }
    if let Some(dir) = install_path.filter(|p| !p.trim().is_empty()) {
        return Some((std::path::Path::new(dir).is_dir(), None));
    }
    None
}

/// Re-checks every library entry against the disk (and Steam manifests) and
/// flips `is_installed` when a game appeared or disappeared. Rows are never
/// removed — uninstalled games stay in the library as "not installed".
pub fn verify_installations_impl(
    app: &AppHandle,
    state: &AppState,
) -> AppResult<Vec<InstallStatusChange>> {
    if state.install_verifying.swap(true, Ordering::SeqCst) {
        return Ok(Vec::new());
    }
    let outcome = run_install_verification(state);
    state.install_verifying.store(false, Ordering::SeqCst);
    let changes = outcome?;

    if !changes.is_empty() {
        log::info!("Install verification: {} status change(s)", changes.len());
        let _ = app.emit("statistics-updated", serde_json::json!({}));
    }
    Ok(changes)
}

fn run_install_verification(state: &AppState) -> AppResult<Vec<InstallStatusChange>> {
    if state.steam_syncing.load(Ordering::SeqCst) || state.epic_syncing.load(Ordering::SeqCst) {
        return Ok(Vec::new());
    }

    let steam_apps: Option<HashMap<u32, String>> = manifest::find_steam_root().map(|root| {
        manifest::installed_apps(&root)
            .into_iter()
            .map(|app| (app.appid, app.install_path.to_string_lossy().into_owned()))
            .collect()
    });
    if steam_apps.is_none() {
        log::debug!("Install verification: Steam not found, Steam-linked games skipped");
    }

    let epic_apps: Option<HashMap<String, String>> =
        crate::services::epic::manifest::scan_installed_games().map(|apps| {
            apps.into_iter()
                .map(|(k, v)| (k, v.install_location))
                .collect()
        });

    let running: HashSet<i64> = state
        .active
        .lock()
        .map(|m| m.keys().copied().collect())
        .unwrap_or_default();

    type VerifyRow = (
        i64,
        Option<String>,
        Option<String>,
        bool,
        Option<u32>,
        Option<String>,
        bool,
        Option<String>,
    );
    let rows: Vec<VerifyRow> = state.db.with(|conn| {
        let mut stmt = conn.prepare(
            "SELECT g.id, g.executable_path, g.install_path, g.is_installed, sg.appid,
                    eg.app_name, g.use_launcher, g.launcher_path
             FROM games g
             LEFT JOIN steam_games sg ON sg.game_id = g.id
             LEFT JOIN epic_games eg ON eg.game_id = g.id",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, i64>(3)? != 0,
                r.get::<_, Option<i64>>(4)?.map(|v| v as u32),
                r.get::<_, Option<String>>(5)?,
                r.get::<_, i64>(6)? != 0,
                r.get::<_, Option<String>>(7)?,
            ))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    })?;

    // Pure filesystem reads — computed before touching the database.
    let mut plan: Vec<(i64, bool, bool, Option<String>)> = Vec::new();
    for (id, exe, install_path, is_installed, appid, epic_app, use_launcher, launcher_path) in rows
    {
        if running.contains(&id) {
            continue;
        }
        let resolved = if let Some(app_name) = epic_app.as_deref() {
            epic_apps.as_ref().map(|apps| {
                let installed = apps.contains_key(app_name);
                let fresh_path = apps
                    .get(app_name)
                    .cloned()
                    .filter(|p| Some(p.as_str()) != install_path.as_deref());
                (installed, fresh_path)
            })
        } else {
            verify_target(
                use_launcher,
                launcher_path.as_deref(),
                exe.as_deref(),
                install_path.as_deref(),
                appid,
                steam_apps.as_ref(),
            )
        };
        let Some((verified, fresh_path)) = resolved else {
            continue;
        };
        if verified != is_installed || fresh_path.is_some() {
            plan.push((id, verified, verified != is_installed, fresh_path));
        }
    }

    if plan.is_empty() {
        return Ok(Vec::new());
    }

    let changes = state.db.with(|conn| {
        conn.execute_batch("BEGIN IMMEDIATE")?;
        let outcome: AppResult<Vec<InstallStatusChange>> = (|| {
            let mut changes = Vec::new();
            for (id, verified, status_changed, fresh_path) in &plan {
                if let Some(path) = fresh_path {
                    conn.execute(
                        "UPDATE games SET install_path = ?1, is_installed = ?2 WHERE id = ?3",
                        rusqlite::params![path, *verified as i64, id],
                    )?;
                } else {
                    conn.execute(
                        "UPDATE games SET is_installed = ?1 WHERE id = ?2",
                        rusqlite::params![*verified as i64, id],
                    )?;
                }
                if *status_changed {
                    changes.push(InstallStatusChange {
                        id: *id,
                        is_installed: *verified,
                    });
                }
            }
            Ok(changes)
        })();
        match outcome {
            Ok(changes) => {
                conn.execute_batch("COMMIT")?;
                Ok(changes)
            }
            Err(err) => {
                let _ = conn.execute_batch("ROLLBACK");
                Err(err)
            }
        }
    })?;
    Ok(changes)
}

/// Verifies that every game in the library still exists on disk.
#[tauri::command]
pub fn verify_installations(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<Vec<InstallStatusChange>> {
    verify_installations_impl(&app, &state)
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

    fn steam_apps() -> HashMap<u32, String> {
        HashMap::from([(730u32, r"D:\Steam\steamapps\common\CS2".to_string())])
    }

    #[test]
    fn verify_target_tracks_steam_installs() {
        let apps = steam_apps();

        // Installed — path refreshed when Steam reports a different one.
        let (installed, fresh) = verify_target(
            false,
            None,
            None,
            Some(r"E:\old\CS2"),
            Some(730),
            Some(&apps),
        )
        .expect("verifiable");
        assert!(installed);
        assert_eq!(fresh.as_deref(), Some(r"D:\Steam\steamapps\common\CS2"));

        // Installed with an up-to-date path — no refresh needed.
        let (installed, fresh) = verify_target(
            false,
            None,
            None,
            Some(r"D:\Steam\steamapps\common\CS2"),
            Some(730),
            Some(&apps),
        )
        .expect("verifiable");
        assert!(installed);
        assert!(fresh.is_none());

        // Uninstalled — flagged but the row and its path stay untouched.
        let (installed, fresh) =
            verify_target(false, None, None, Some(r"E:\old\CS2"), Some(440), Some(&apps))
                .expect("verifiable");
        assert!(!installed);
        assert!(fresh.is_none());

        // Steam itself is missing — nothing to verify against, keep the flag.
        assert!(verify_target(false, None, None, Some(r"E:\old\CS2"), Some(730), None).is_none());
    }

    #[test]
    fn verify_target_checks_local_paths() {
        let dir = std::env::temp_dir().join(format!(
            "gs-verify-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let exe = dir.join("game.exe");
        std::fs::write(&exe, b"x").expect("temp exe");
        let exe_str = exe.to_string_lossy().to_string();
        let dir_str = dir.to_string_lossy().to_string();
        let missing = dir.join("gone.exe").to_string_lossy().to_string();

        let (installed, fresh) =
            verify_target(false, None, Some(&exe_str), Some(&dir_str), None, None)
                .expect("verifiable");
        assert!(installed);
        assert!(fresh.is_none());

        let (installed, _) =
            verify_target(false, None, Some(&missing), Some(&dir_str), None, None)
                .expect("verifiable");
        assert!(!installed);

        // No executable — the install directory is the fallback check.
        let (installed, _) =
            verify_target(false, None, None, Some(&dir_str), None, None).expect("verifiable");
        assert!(installed);
        let (installed, _) =
            verify_target(false, None, None, Some(&missing), None, None).expect("verifiable");
        assert!(!installed);

        // Nothing to check against.
        assert!(verify_target(false, None, None, None, None, None).is_none());
        assert!(verify_target(false, None, Some(""), Some("  "), None, None).is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn launcher_args_empty_stays_within_not_null_column() {
        let conn = memory_db();

        // Same binds the create/update commands use for "no launcher args":
        // an empty string must be stored instead of an explicit NULL.
        let no_args: Option<String> = None;
        conn.execute(
            "INSERT INTO games (title, created_at, updated_at, launcher_args)
             VALUES (?1, 't', 't', ?2)",
            rusqlite::params!["Soulstone", no_args.as_deref().unwrap_or("")],
        )
        .expect("empty launcher_args must not violate NOT NULL");

        let id: i64 = conn.last_insert_rowid();
        let stored: String = conn
            .query_row("SELECT launcher_args FROM games WHERE id = ?1", [id], |r| r.get(0))
            .expect("stored args");
        assert_eq!(stored, "");

        // The model keeps the "no args" semantics as None.
        let game = get_game_conn(&conn, id).expect("game");
        assert_eq!(game.launcher_args, None);
    }
}
