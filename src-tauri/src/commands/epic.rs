use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension};
use tauri::{AppHandle, Emitter, State};

use crate::commands::settings;
use crate::error::{AppError, AppResult};
use crate::game::{achievements, theme};
use crate::models::{EpicSyncResult, StoreAccount};
use crate::services::epic::{api, manifest};
use crate::services::placeholder::placeholder_cover;
use crate::AppState;

const PROVIDER: &str = "epic";

#[derive(Debug, Clone)]
struct EpicMeta {
    title: Option<String>,
    cover_url: Option<String>,
    description: String,
    developer: String,
    publisher: String,
    genre: String,
    release_year: Option<i64>,
    playtime_seconds: i64,
}

fn load_account(conn: &Connection) -> Result<Option<(StoreAccount, String)>, rusqlite::Error> {
    conn.query_row(
        "SELECT provider, account_id, profile_url, persona_name, avatar_url,
                linked_at, last_synced_at, api_key
         FROM store_accounts WHERE provider = ?1",
        [PROVIDER],
        |r| {
            Ok((
                StoreAccount {
                    provider: r.get(0)?,
                    account_id: r.get(1)?,
                    profile_url: r.get(2)?,
                    persona_name: r.get(3)?,
                    avatar_url: r.get(4)?,
                    linked_at: r.get(5)?,
                    last_synced_at: r.get(6)?,
                },
                r.get::<_, String>(7)?,
            ))
        },
    )
    .optional()
}

fn current_account(state: &State<'_, AppState>) -> AppResult<StoreAccount> {
    state
        .db
        .with(|conn| Ok(load_account(conn)?.map(|(acc, _)| acc)))?
        .ok_or_else(|| AppError::NotFound("Epic Games account is not linked".into()))
}

#[tauri::command]
pub fn get_epic_account(state: State<'_, AppState>) -> AppResult<Option<StoreAccount>> {
    state
        .db
        .with(|conn| Ok(load_account(conn)?.map(|(acc, _)| acc)))
}

#[tauri::command]
pub async fn link_epic_account(
    state: State<'_, AppState>,
    auth_code: String,
) -> AppResult<StoreAccount> {
    match run_link(&state, auth_code).await {
        Ok(account) => Ok(account),
        Err(err) => {
            log::error!("Epic link failed: {err}");
            Err(err)
        }
    }
}

async fn run_link(state: &State<'_, AppState>, auth_code: String) -> AppResult<StoreAccount> {
    let code = api::parse_authorization_code(&auth_code)?;
    let client = api::EpicApiClient::new()?;
    let session = client.exchange_code(&code).await?;

    let now = chrono::Utc::now().to_rfc3339();
    state.db.with(|conn| {
        conn.execute(
            "INSERT INTO store_accounts (provider, account_id, profile_url, persona_name,
                    avatar_url, api_key, linked_at, last_synced_at)
             VALUES (?1, ?2, '', ?3, '', ?4, ?5, NULL)
             ON CONFLICT(provider) DO UPDATE SET
                account_id = excluded.account_id,
                persona_name = excluded.persona_name,
                api_key = excluded.api_key,
                linked_at = excluded.linked_at,
                last_synced_at = NULL",
            rusqlite::params![
                PROVIDER,
                session.account_id,
                session.display_name,
                session.refresh_token,
                now
            ],
        )?;
        Ok(())
    })?;

    log::info!(
        "Epic Games account linked: {} ({})",
        session.display_name,
        session.account_id
    );
    current_account(state)
}

#[tauri::command]
pub fn unlink_epic_account(state: State<'_, AppState>) -> AppResult<()> {
    state.db.with(|conn| {
        conn.execute("DELETE FROM store_accounts WHERE provider = ?1", [PROVIDER])?;
        Ok(())
    })?;
    log::info!("Epic Games account unlinked");
    Ok(())
}

#[tauri::command]
pub async fn sync_epic_library(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<EpicSyncResult> {
    let flag = state.epic_syncing.clone();
    if flag.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Err(AppError::Conflict(
            "Синхронизация Epic Games уже выполняется".into(),
        ));
    }
    let outcome = run_sync(app, &state).await;
    flag.store(false, std::sync::atomic::Ordering::SeqCst);
    match outcome {
        Ok(result) => Ok(result),
        Err(err) => {
            log::error!("Epic sync failed: {err}");
            Err(err)
        }
    }
}

async fn run_sync(app: AppHandle, state: &State<'_, AppState>) -> AppResult<EpicSyncResult> {
    let (account, refresh_token) = state
        .db
        .with(|conn| Ok(load_account(conn)?))?
        .ok_or_else(|| AppError::validation("Сначала подключите аккаунт Epic Games"))?;

    let client = api::EpicApiClient::new()?;
    let access_token = if !refresh_token.trim().is_empty() {
        match client.refresh_token(&refresh_token).await {
            Ok(session) => {
                let _ = state.db.with(|conn| {
                    conn.execute(
                        "UPDATE store_accounts SET api_key = ?1, persona_name = ?2 WHERE provider = ?3",
                        rusqlite::params![session.refresh_token, session.display_name, PROVIDER],
                    )?;
                    Ok(())
                });
                Some(session.access_token)
            }
            Err(e) => {
                log::warn!("Epic token refresh failed, syncing local manifests only: {e}");
                None
            }
        }
    } else {
        None
    };

    // Only installed Epic Games are scanned from Epic's local manifests.
    let installed_map = manifest::scan_installed_games().unwrap_or_default();
    let installed_count = installed_map.len() as i64;

    let mut playtime_map = if let Some(token) = &access_token {
        client.get_all_playtime(token, &account.account_id).await
    } else {
        HashMap::new()
    };
    if playtime_map.is_empty() {
        playtime_map = manifest::read_v2_playtime_cache(&account.account_id);
    }

    let locale = state.db.with(|conn| {
        let lang = settings::read_setting(conn, "locale")?
            .or(settings::read_setting(conn, "language")?)
            .unwrap_or_else(|| "ru".into());
        Ok(if lang == "en" { "en" } else { "ru" })
    })?;

    // Determine which games need metadata/cover fetching.
    let existing_by_app: HashMap<String, (i64, String, String, String)> = state.db.with(|conn| {
        let mut stmt = conn.prepare(
            "SELECT eg.app_name, g.id, g.cover_path, g.description, g.genre
             FROM epic_games eg
             JOIN games g ON g.id = eg.game_id",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                (
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                ),
            ))
        })?;
        let mut map = HashMap::new();
        for row in rows {
            let (app_name, info) = row?;
            map.insert(app_name, info);
        }
        Ok(map)
    })?;

    let mut meta_map: HashMap<String, EpicMeta> = HashMap::new();
    for (app_name, game) in &installed_map {
        let playtime_seconds = playtime_map.get(app_name).copied().unwrap_or(0);
        let needs_fetch = match existing_by_app.get(app_name) {
            Some((_, cover, desc, genre)) => {
                cover.is_empty()
                    || cover.starts_with("data:")
                    || desc.trim().is_empty()
                    || genre.trim().is_empty()
            }
            None => true,
        };

        let mut meta = EpicMeta {
            title: None,
            cover_url: game.cover_url.clone(),
            description: game.description.clone().unwrap_or_default(),
            developer: game.developer.clone().unwrap_or_default(),
            publisher: game.developer.clone().unwrap_or_default(),
            genre: String::new(),
            release_year: None,
            playtime_seconds,
        };

        if needs_fetch {
            if let Some(token) = &access_token {
                if let Some(details) = client
                    .get_catalog_details(token, &game.namespace, &game.catalog_item_id, locale)
                    .await
                {
                    if details.title.is_some() {
                        meta.title = details.title;
                    }
                    if meta.cover_url.is_none() {
                        meta.cover_url = details.cover_url;
                    }
                    if meta.description.trim().is_empty() {
                        meta.description = details.description;
                    }
                    if meta.developer.trim().is_empty() {
                        meta.developer = details.developer.clone();
                        meta.publisher = details.developer;
                    }
                    meta.release_year = details.release_year;
                }
            }

            // Enrich genre / release_year / missing fields from IGDB if configured.
            let lookup_title = meta.title.as_deref().unwrap_or(&game.display_name);
            if meta.genre.trim().is_empty() || meta.release_year.is_none() || meta.cover_url.is_none() {
                if let Some(igdb) = crate::commands::igdb::search_first(state, lookup_title).await {
                    if meta.genre.trim().is_empty() {
                        if let Some(first_genre) = igdb.genres.first() {
                            meta.genre = first_genre.clone();
                        }
                    }
                    if meta.release_year.is_none() {
                        meta.release_year = igdb.release_year.map(i64::from);
                    }
                    if meta.description.trim().is_empty() {
                        meta.description = igdb.summary;
                    }
                    if meta.developer.trim().is_empty() {
                        meta.developer = igdb.developer;
                    }
                    if meta.publisher.trim().is_empty() {
                        meta.publisher = igdb.publisher;
                    }
                    if meta.cover_url.is_none() {
                        meta.cover_url = igdb.cover_url;
                    }
                }
            }
        }

        meta_map.insert(app_name.clone(), meta);
    }

    let now = chrono::Utc::now().to_rfc3339();
    let (added, updated) = state
        .db
        .with(|conn| Ok(apply_epic_sync(conn, &installed_map, &meta_map, &now)?))?;

    let _ = state.db.with(|conn| {
        conn.execute(
            "UPDATE store_accounts SET last_synced_at = ?1 WHERE provider = ?2",
            rusqlite::params![now, PROVIDER],
        )?;
        let _ = achievements::ensure_app_achievements(conn);
        Ok(())
    });

    let _ = app.emit("games:updated", ());

    log::info!(
        "Epic sync complete: installed={}, added={}, updated={}",
        installed_count,
        added,
        updated
    );

    Ok(EpicSyncResult {
        owned: installed_count,
        installed: installed_count,
        added,
        updated,
    })
}

fn apply_epic_sync(
    conn: &Connection,
    installed_map: &HashMap<String, manifest::InstalledEpicGame>,
    meta_map: &HashMap<String, EpicMeta>,
    now: &str,
) -> Result<(i64, i64), rusqlite::Error> {
    let mut added = 0i64;
    let mut updated = 0i64;

    // Mark previously synced Epic games that are no longer installed as not installed.
    {
        let mut stmt = conn.prepare("SELECT game_id, app_name FROM epic_games")?;
        let existing: Vec<(i64, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        for (game_id, app_name) in existing {
            if !installed_map.contains_key(&app_name) {
                conn.execute(
                    "UPDATE games SET is_installed = 0, updated_at = ?1 WHERE id = ?2",
                    rusqlite::params![now, game_id],
                )?;
            }
        }
    }

    for (app_name, game) in installed_map {
        let meta = meta_map.get(app_name);

        let existing_id: Option<i64> = conn
            .query_row(
                "SELECT game_id FROM epic_games WHERE app_name = ?1",
                [app_name],
                |r| r.get(0),
            )
            .optional()?;

        // Only match by title if it is NOT a Steam game (so Steam games are never mixed into Epic).
        let matched_id = match existing_id {
            Some(id) => Some(id),
            None => conn
                .query_row(
                    "SELECT id FROM games WHERE LOWER(TRIM(title)) = LOWER(TRIM(?1)) AND source != 'Steam' LIMIT 1",
                    [&game.display_name],
                    |r| r.get(0),
                )
                .optional()?,
        };

        if let Some(game_id) = matched_id {
            let (cur_desc, cur_cover, cur_dev, cur_pub, cur_genre, cur_year, cur_exe): (
                String,
                String,
                String,
                String,
                String,
                Option<i64>,
                Option<String>,
            ) = conn.query_row(
                "SELECT description, cover_path, developer, publisher, genre, release_year, executable_path
                 FROM games WHERE id = ?1",
                [game_id],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                        r.get(6)?,
                    ))
                },
            )?;

            let new_desc = if cur_desc.trim().is_empty() {
                meta.map(|m| m.description.clone())
                    .filter(|d| !d.trim().is_empty())
                    .or_else(|| game.description.clone())
                    .unwrap_or(cur_desc)
            } else {
                cur_desc
            };
            let new_cover = if cur_cover.is_empty() || cur_cover.starts_with("data:") {
                meta.and_then(|m| m.cover_url.clone())
                    .or_else(|| game.cover_url.clone())
                    .unwrap_or_else(|| {
                        if cur_cover.is_empty() {
                            placeholder_cover(&game.display_name)
                        } else {
                            cur_cover
                        }
                    })
            } else {
                cur_cover
            };
            let new_dev = if cur_dev.trim().is_empty() {
                meta.map(|m| m.developer.clone())
                    .filter(|d| !d.trim().is_empty())
                    .or_else(|| game.developer.clone())
                    .unwrap_or(cur_dev)
            } else {
                cur_dev
            };
            let new_pub = if cur_pub.trim().is_empty() {
                meta.map(|m| m.publisher.clone())
                    .filter(|d| !d.trim().is_empty())
                    .or_else(|| game.developer.clone())
                    .unwrap_or(cur_pub)
            } else {
                cur_pub
            };
            let new_genre = if cur_genre.trim().is_empty() {
                meta.map(|m| m.genre.clone())
                    .filter(|g| !g.trim().is_empty())
                    .unwrap_or(cur_genre)
            } else {
                cur_genre
            };
            let new_year = cur_year.or_else(|| meta.and_then(|m| m.release_year));
            // Do not force `launch_executable` (often a bootstrapper like PlayGTAV.exe that exits
            // immediately); leave `executable_path` unset so Epic URI + `install_path` monitoring is used.
            let new_exe = cur_exe.filter(|p| {
                !p.trim().is_empty()
                    && std::path::Path::new(p).exists()
                    && game.executable_path.as_deref() != Some(p.as_str())
            });
            let playtime = meta.map(|m| m.playtime_seconds).unwrap_or(0);

            conn.execute(
                "UPDATE games SET
                    description = ?1, cover_path = ?2, developer = ?3, publisher = ?4,
                    genre = ?5, release_year = ?6, executable_path = ?7, install_path = ?8,
                    total_playtime_seconds = MAX(total_playtime_seconds, ?9),
                    is_installed = 1, source = 'Epic', updated_at = ?10
                 WHERE id = ?11",
                rusqlite::params![
                    new_desc,
                    new_cover,
                    new_dev,
                    new_pub,
                    new_genre,
                    new_year,
                    new_exe,
                    game.install_location,
                    playtime,
                    now,
                    game_id
                ],
            )?;

            conn.execute(
                "INSERT INTO epic_games (game_id, app_name, catalog_item_id, namespace)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(game_id) DO UPDATE SET
                    app_name = excluded.app_name,
                    catalog_item_id = excluded.catalog_item_id,
                    namespace = excluded.namespace",
                rusqlite::params![game_id, app_name, game.catalog_item_id, game.namespace],
            )?;
            let _ = achievements::ensure_game_achievements(conn, game_id, &game.display_name);
            updated += 1;
        } else {
            let title = meta
                .and_then(|m| m.title.as_deref())
                .unwrap_or(&game.display_name);
            let cover = meta
                .and_then(|m| m.cover_url.clone())
                .or_else(|| game.cover_url.clone())
                .unwrap_or_else(|| placeholder_cover(title));
            let desc = meta
                .map(|m| m.description.as_str())
                .filter(|d| !d.trim().is_empty())
                .or(game.description.as_deref())
                .unwrap_or("");
            let dev = meta
                .map(|m| m.developer.as_str())
                .filter(|d| !d.trim().is_empty())
                .or(game.developer.as_deref())
                .unwrap_or("");
            let publisher = meta
                .map(|m| m.publisher.as_str())
                .filter(|p| !p.trim().is_empty())
                .unwrap_or(dev);
            let genre = meta.map(|m| m.genre.as_str()).unwrap_or("");
            let year = meta.and_then(|m| m.release_year);
            let playtime = meta.map(|m| m.playtime_seconds).unwrap_or(0);

            conn.execute(
                "INSERT INTO games (title, description, executable_path, install_path, cover_path,
                        developer, publisher, genre, release_year, platform, source,
                        is_installed, total_playtime_seconds, created_at, updated_at)
                 VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8, 'PC', 'Epic', 1, ?9, ?10, ?10)",
                rusqlite::params![
                    title,
                    desc,
                    game.install_location,
                    cover,
                    dev,
                    publisher,
                    genre,
                    year,
                    playtime,
                    now
                ],
            )?;
            let game_id = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO epic_games (game_id, app_name, catalog_item_id, namespace)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![game_id, app_name, game.catalog_item_id, game.namespace],
            )?;
            let _ = theme::auto_theme(conn, game_id, title, genre);
            let _ = achievements::ensure_game_achievements(conn, game_id, title);
            added += 1;
        }
    }

    Ok((added, updated))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().expect("memory db");
        conn.pragma_update(None, "foreign_keys", "ON")
            .expect("pragma");
        crate::database::migrations::run(&conn).expect("migrations");
        conn
    }

    #[test]
    fn epic_sync_inserts_only_installed_epic_and_never_touches_steam() {
        let conn = memory_db();
        // Existing Steam game with the same title should NOT be overwritten by Epic sync.
        conn.execute(
            "INSERT INTO games (title, description, cover_path, developer, publisher, genre,
                    created_at, updated_at, source, is_installed)
             VALUES ('Rocket League', 'Steam desc', 'steam.jpg', 'Psyonix', 'Psyonix', 'Sports',
                't', 't', 'Steam', 1)",
            [],
        )
        .expect("insert steam game");
        let steam_id = conn.last_insert_rowid();

        let mut installed = HashMap::new();
        installed.insert(
            "Sugar".to_string(),
            manifest::InstalledEpicGame {
                app_name: "Sugar".into(),
                display_name: "Rocket League".into(),
                catalog_item_id: "cat1".into(),
                namespace: "ns1".into(),
                install_location: "C:/Epic/RocketLeague".into(),
                executable_path: None,
                description: None,
                developer: None,
                cover_url: None,
            },
        );

        let (added, updated) =
            apply_epic_sync(&conn, &installed, &HashMap::new(), "now").expect("sync");
        assert_eq!((added, updated), (1, 0));

        // Steam row untouched.
        let steam_source: String = conn
            .query_row("SELECT source FROM games WHERE id = ?1", [steam_id], |r| r.get(0))
            .expect("read steam");
        assert_eq!(steam_source, "Steam");

        // Epic row created separately.
        let epic_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM games WHERE source = 'Epic'", [], |r| r.get(0))
            .expect("count epic");
        assert_eq!(epic_count, 1);

        // If uninstalled later, marks is_installed = 0.
        let (added2, updated2) =
            apply_epic_sync(&conn, &HashMap::new(), &HashMap::new(), "later").expect("sync2");
        assert_eq!((added2, updated2), (0, 0));
        let epic_installed: i64 = conn
            .query_row(
                "SELECT is_installed FROM games WHERE source = 'Epic'",
                [],
                |r| r.get(0),
            )
            .expect("read installed");
        assert_eq!(epic_installed, 0);
    }
}
