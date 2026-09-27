use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension};
use tauri::{AppHandle, Emitter, State};

use crate::commands::settings;
use crate::error::{AppError, AppResult};
use crate::game::{achievements, theme};
use crate::models::{SteamSyncResult, StoreAccount};
use crate::services::placeholder::placeholder_cover;
use crate::services::steam::{api, manifest};
use crate::AppState;

const PROVIDER: &str = "steam";

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

    fn meta_for(appid: u32, cover: CoverChoice) -> HashMap<u32, SteamMeta> {
        let mut meta = HashMap::new();
        meta.insert(
            appid,
            SteamMeta {
                cover,
                description: "Описание из Steam".into(),
                developer: "Valve".into(),
                publisher: "Valve".into(),
                genre: "Экшен".into(),
                release_year: Some(2012),
                playtime_seconds: 3600,
            },
        );
        meta
    }

    #[test]
    fn sync_fills_missing_fields_and_repairs_covers() {
        let conn = memory_db();
        conn.execute(
            "INSERT INTO games (title, description, cover_path, developer, publisher, genre,
                    created_at, updated_at, source, is_installed, total_playtime_seconds)
             VALUES ('Counter-Strike 2', '', 'data:image/svg+xml,x', '', '', '', 't', 't',
                'Manual', 0, 0)",
            [],
        )
        .expect("insert game");
        let id = conn.last_insert_rowid();

        let meta = meta_for(730, CoverChoice::Url("https://cdn.example/730/header.jpg".into()));
        let updates = vec![(id, 730u32, "C:/Steam/steamapps/common/csgo".to_string())];
        let (added, updated) =
            apply_sync(&conn, &updates, &[], &meta, "2026-01-01T00:00:00+00:00").expect("apply");
        assert_eq!((added, updated), (0, 1));

        let (cover, description, genre, year, play, installed, source): (
            String,
            String,
            String,
            Option<i64>,
            i64,
            i64,
            String,
        ) = conn
            .query_row(
                "SELECT cover_path, description, genre, release_year, total_playtime_seconds,
                        is_installed, source
                 FROM games WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)),
            )
            .expect("read");
        assert_eq!(cover, "https://cdn.example/730/header.jpg");
        assert_eq!(description, "Описание из Steam");
        assert_eq!(genre, "Экшен");
        assert_eq!(year, Some(2012));
        assert_eq!(play, 3600);
        assert_eq!(installed, 1);
        assert_eq!(source, "Steam");

        let links: i64 = conn
            .query_row("SELECT COUNT(*) FROM steam_games WHERE game_id = ?1", [id], |r| r.get(0))
            .expect("links");
        assert_eq!(links, 1);
    }

    #[test]
    fn sync_keeps_manual_fields_and_higher_playtime() {
        let conn = memory_db();
        conn.execute(
            "INSERT INTO games (title, description, cover_path, developer, publisher, genre,
                    release_year, created_at, updated_at, source, total_playtime_seconds)
             VALUES ('Dota 2', 'Своя заметка', 'C:/covers/dota.jpg', 'Свой девелопер', 'Valve',
                'MOBA', 2013, 't', 't', 'Manual', 9999)",
            [],
        )
        .expect("insert game");
        let id = conn.last_insert_rowid();

        let meta = meta_for(570, CoverChoice::Url("https://cdn.example/570/header.jpg".into()));
        let updates = vec![(id, 570u32, "C:/Steam/steamapps/common/dota 2 beta".to_string())];
        let (added, updated) = apply_sync(&conn, &updates, &[], &meta, "now").expect("apply");
        assert_eq!((added, updated), (0, 1));

        let (description, developer, genre, year, play, cover): (
            String,
            String,
            String,
            Option<i64>,
            i64,
            String,
        ) = conn
            .query_row(
                "SELECT description, developer, genre, release_year, total_playtime_seconds,
                        cover_path
                 FROM games WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
            )
            .expect("read");
        assert_eq!(description, "Своя заметка");
        assert_eq!(developer, "Свой девелопер");
        assert_eq!(genre, "MOBA");
        assert_eq!(year, Some(2013));
        assert_eq!(play, 9999);
        assert_eq!(cover, "C:/covers/dota.jpg");
    }

    #[test]
    fn sync_inserts_new_steam_game_with_store_data() {
        let conn = memory_db();
        let meta = meta_for(570, CoverChoice::Placeholder);
        let new_rows = vec![(570u32, "Dota 2".to_string(), "C:/Steam/dota2".to_string(), None)];
        let (added, updated) = apply_sync(&conn, &[], &new_rows, &meta, "now").expect("apply");
        assert_eq!((added, updated), (1, 0));

        let (description, developer, genre, year, play, cover, installed): (
            String,
            String,
            String,
            Option<i64>,
            i64,
            String,
            i64,
        ) = conn
            .query_row(
                "SELECT description, developer, genre, release_year, total_playtime_seconds,
                        cover_path, is_installed
                 FROM games WHERE title = 'Dota 2'",
                [],
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
            )
            .expect("read");
        assert_eq!(description, "Описание из Steam");
        assert_eq!(developer, "Valve");
        assert_eq!(genre, "Экшен");
        assert_eq!(year, Some(2012));
        assert_eq!(play, 3600);
        assert!(cover.starts_with("data:image/svg+xml"));
        assert_eq!(installed, 1);

        let achievements: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM achievements a JOIN games g ON g.id = a.game_id
                 WHERE g.title = 'Dota 2'",
                [],
                |r| r.get(0),
            )
            .expect("achievements");
        assert_eq!(achievements, 1);
    }

    #[test]
    fn apply_sync_never_duplicates_an_existing_title() {
        let conn = memory_db();
        let meta = meta_for(570, CoverChoice::Url("https://cdn.example/570/header.jpg".into()));
        let new_rows = vec![(
            570u32,
            "Dota 2".to_string(),
            "C:/Steam/dota2".to_string(),
            None,
        )];

        let first = apply_sync(&conn, &[], &new_rows, &meta, "t1").expect("first");
        assert_eq!(first, (1, 0));

        // A second run (for example an overlapping sync) must not add a copy.
        let second = apply_sync(&conn, &[], &new_rows, &meta, "t2").expect("second");
        assert_eq!(second, (0, 1));

        let games: i64 = conn
            .query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))
            .expect("games");
        assert_eq!(games, 1);
        let links: i64 = conn
            .query_row("SELECT COUNT(*) FROM steam_games", [], |r| r.get(0))
            .expect("links");
        assert_eq!(links, 1);
    }

    #[test]
    fn merge_collapses_duplicate_steam_rows() {
        let conn = memory_db();
        conn.execute(
            "INSERT INTO games (title, created_at, updated_at, source, total_playtime_seconds,
                    launch_count, developer)
             VALUES ('Rust', 't', 't', 'Steam', 100, 1, '')",
            [],
        )
        .expect("keeper");
        let keeper = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO steam_games (game_id, appid) VALUES (?1, 252490)",
            [keeper],
        )
        .expect("link");

        conn.execute(
            "INSERT INTO games (title, created_at, updated_at, source, total_playtime_seconds,
                    launch_count, developer, genre)
             VALUES ('rust', 't', 't', 'Steam', 5000, 9, 'Facepunch Studios', 'Экшен')",
            [],
        )
        .expect("loser");
        let loser = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO game_sessions (game_id, started_at, duration_seconds)
             VALUES (?1, 't', 4200)",
            [loser],
        )
        .expect("session");

        let merged = merge_duplicate_steam_games(&conn, "now").expect("merge");
        assert_eq!(merged, 1);

        let games: i64 = conn
            .query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))
            .expect("games");
        assert_eq!(games, 1);

        let (id, play, launches, developer, genre): (i64, i64, i64, String, String) = conn
            .query_row(
                "SELECT id, total_playtime_seconds, launch_count, developer, genre FROM games",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .expect("row");
        assert_eq!(id, keeper);
        assert_eq!(play, 5000);
        assert_eq!(launches, 9);
        assert_eq!(developer, "Facepunch Studios");
        assert_eq!(genre, "Экшен");

        let session_game: i64 = conn
            .query_row("SELECT game_id FROM game_sessions", [], |r| r.get(0))
            .expect("session");
        assert_eq!(session_game, keeper);

        let link_game: i64 = conn
            .query_row("SELECT game_id FROM steam_games WHERE appid = 252490", [], |r| r.get(0))
            .expect("link");
        assert_eq!(link_game, keeper);
    }
}

struct LinkedAccount {
    account: StoreAccount,
    api_key: String,
}

/// Metadata stored in `games` for an already known game.
struct KnownFields {
    genre: String,
    developer: String,
    publisher: String,
    description: String,
    release_year: Option<i64>,
    cover: String,
}

/// Artwork decision made during a sync.
enum CoverChoice {
    /// Existing artwork is fine and must not be touched.
    Keep,
    /// A CDN asset that answered with 2xx.
    Url(String),
    /// Nothing reachable on the CDN — fall back to a local placeholder.
    Placeholder,
}

/// Store data collected for one app during a sync.
struct SteamMeta {
    cover: CoverChoice,
    description: String,
    developer: String,
    publisher: String,
    genre: String,
    release_year: Option<i64>,
    playtime_seconds: i64,
}

fn load_account(conn: &Connection) -> AppResult<Option<LinkedAccount>> {
    let row = conn
        .query_row(
            "SELECT provider, account_id, profile_url, persona_name, avatar_url, api_key,
                    linked_at, last_synced_at
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
                        linked_at: r.get(6)?,
                        last_synced_at: r.get(7)?,
                    },
                    r.get::<_, String>(5)?,
                ))
            },
        )
        .optional()?;
    Ok(row.map(|(account, api_key)| LinkedAccount { account, api_key }))
}

fn current_account(state: &AppState) -> AppResult<StoreAccount> {
    state
        .db
        .with(|conn| Ok(load_account(conn)?.map(|linked| linked.account)))?
        .ok_or_else(|| AppError::Other("Не удалось сохранить аккаунт Steam".into()))
}

#[tauri::command]
pub fn get_store_account(state: State<'_, AppState>) -> AppResult<Option<StoreAccount>> {
    state.db.with(|conn| Ok(load_account(conn)?.map(|linked| linked.account)))
}

#[tauri::command]
pub async fn link_steam_account(
    state: State<'_, AppState>,
    api_key: String,
    profile_url: String,
) -> AppResult<StoreAccount> {
    match run_link(state, api_key, profile_url).await {
        Ok(account) => Ok(account),
        Err(err) => {
            log::error!("Steam link failed: {err}");
            Err(err)
        }
    }
}

async fn run_link(
    state: State<'_, AppState>,
    api_key: String,
    profile_url: String,
) -> AppResult<StoreAccount> {
    let api_key = api_key.trim().to_string();
    if api_key.is_empty() {
        return Err(AppError::validation("Укажите API-ключ Steam"));
    }
    let profile_url = profile_url.trim().to_string();
    if profile_url.is_empty() {
        return Err(AppError::validation("Укажите ссылку на профиль Steam"));
    }

    let client = api::SteamApiClient::new(api_key.clone())?;
    let steam_id = match api::parse_steam_ref(&profile_url)? {
        api::SteamRef::Id64(id) => id,
        api::SteamRef::Vanity(name) => client.resolve_steam_id(&name).await?,
    };
    let profile = client.get_profile(&steam_id).await?;
    if !profile.is_public {
        return Err(api::SteamError::PrivateProfile.into());
    }

    let now = chrono::Utc::now().to_rfc3339();
    state.db.with(|conn| {
        conn.execute(
            "INSERT INTO store_accounts (provider, account_id, profile_url, persona_name,
                    avatar_url, api_key, linked_at, last_synced_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)
             ON CONFLICT(provider) DO UPDATE SET
                account_id = excluded.account_id,
                profile_url = excluded.profile_url,
                persona_name = excluded.persona_name,
                avatar_url = excluded.avatar_url,
                api_key = excluded.api_key,
                linked_at = excluded.linked_at,
                last_synced_at = NULL",
            rusqlite::params![
                PROVIDER,
                profile.steam_id,
                profile.profile_url,
                profile.persona_name,
                profile.avatar_url,
                api_key,
                now
            ],
        )?;
        Ok(())
    })?;

    log::info!(
        "Steam account linked: {} ({})",
        profile.persona_name,
        profile.steam_id
    );
    current_account(&state)
}

#[tauri::command]
pub fn unlink_steam_account(state: State<'_, AppState>) -> AppResult<()> {
    state.db.with(|conn| {
        conn.execute("DELETE FROM store_accounts WHERE provider = ?1", [PROVIDER])?;
        Ok(())
    })?;
    log::info!("Steam account unlinked");
    Ok(())
}

#[tauri::command]
pub async fn sync_steam_library(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<SteamSyncResult> {
    let flag = state.steam_syncing.clone();
    if flag.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Err(AppError::Conflict(
            "Синхронизация уже выполняется".into(),
        ));
    }
    let outcome = run_sync(app, state).await;
    flag.store(false, std::sync::atomic::Ordering::SeqCst);
    match outcome {
        Ok(result) => Ok(result),
        Err(err) => {
            log::error!("Steam sync failed: {err}");
            Err(err)
        }
    }
}

/// Collects store metadata and cover art for a single app in one task so
/// several games can be fetched in parallel.
async fn fetch_steam_meta(
    client: api::SteamApiClient,
    appid: u32,
    playtime_seconds: i64,
    needs_details: bool,
    needs_cover: bool,
    lang: &'static str,
) -> SteamMeta {
    let details = if needs_details {
        client.get_app_details(appid, lang).await
    } else {
        None
    };

    let cover = if needs_cover {
        if let Some(details) = &details {
            if !details.header_image.is_empty() {
                CoverChoice::Url(details.header_image.clone())
            } else {
                let urls = api::cover_candidates(appid);
                let mut resolved = None;
                for url in urls {
                    if client.url_exists(&url).await {
                        resolved = Some(url);
                        break;
                    }
                }
                resolved
                    .map(CoverChoice::Url)
                    .unwrap_or(CoverChoice::Placeholder)
            }
        } else {
            let urls = api::cover_candidates(appid);
            let mut resolved = None;
            for url in urls {
                if client.url_exists(&url).await {
                    resolved = Some(url);
                    break;
                }
            }
            resolved
                .map(CoverChoice::Url)
                .unwrap_or(CoverChoice::Placeholder)
        }
    } else {
        CoverChoice::Keep
    };

    let (description, developer, publisher, genre, release_year) = match &details {
        Some(details) => (
            details.short_description.clone(),
            details.developer(),
            details.publisher(),
            details.genre(),
            details.release_year(),
        ),
        None => (String::new(), String::new(), String::new(), String::new(), None),
    };

    SteamMeta {
        cover,
        description,
        developer,
        publisher,
        genre,
        release_year,
        playtime_seconds,
    }
}

async fn run_sync(app: AppHandle, state: State<'_, AppState>) -> AppResult<SteamSyncResult> {
    let linked = state
        .db
        .with(load_account)?
        .ok_or_else(|| AppError::validation("Сначала подключите Steam-аккаунт"))?;

    let now = chrono::Utc::now().to_rfc3339();

    // Collapse leftovers of a duplicated sync before titles are matched.
    let merged = state.db.with(|conn| merge_duplicate_steam_games(conn, &now))?;
    if merged > 0 {
        log::info!("Steam sync: merged {merged} duplicate library entries");
    }

    let client = api::SteamApiClient::new(linked.api_key)?;
    let owned = client.get_owned_games(&linked.account.account_id).await?;

    let steam_root = manifest::find_steam_root().ok_or_else(|| {
        AppError::Other(
            "Steam не найден на этом компьютере. Установите Steam или добавьте игру вручную."
                .into(),
        )
    })?;
    let installed: HashMap<u32, manifest::InstalledApp> = manifest::installed_apps(&steam_root)
        .into_iter()
        .map(|app| (app.appid, app))
        .collect();

    let candidates: Vec<&api::OwnedGame> = owned
        .iter()
        .filter(|game| installed.contains_key(&game.appid))
        .collect();
    let installed_count = candidates.len() as i64;

    let (links, titles, fields) = state.db.with(|conn| {
        let mut links: HashMap<u32, i64> = HashMap::new();
        {
            let mut stmt = conn.prepare("SELECT appid, game_id FROM steam_games")?;
            let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?;
            for row in rows {
                let (appid, game_id) = row?;
                links.insert(appid as u32, game_id);
            }
        }

        let mut titles: HashMap<String, i64> = HashMap::new();
        {
            let mut stmt = conn.prepare("SELECT id, title FROM games")?;
            let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
            for row in rows {
                let (id, title) = row?;
                titles.insert(title.to_lowercase(), id);
            }
        }

        let mut fields: HashMap<i64, KnownFields> = HashMap::new();
        {
            let mut stmt = conn.prepare(
                "SELECT id, genre, developer, publisher, description, release_year,
                        COALESCE(cover_path, '')
                 FROM games",
            )?;
            let rows = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    KnownFields {
                        genre: r.get(1)?,
                        developer: r.get(2)?,
                        publisher: r.get(3)?,
                        description: r.get(4)?,
                        release_year: r.get(5)?,
                        cover: r.get(6)?,
                    },
                ))
            })?;
            for row in rows {
                let (id, known) = row?;
                fields.insert(id, known);
            }
        }
        Ok((links, titles, fields))
    })?;

    let mut updates: Vec<(i64, u32, String)> = Vec::new();
    let mut additions: Vec<(&api::OwnedGame, &manifest::InstalledApp)> = Vec::new();
    for game in &candidates {
        let Some(install) = installed.get(&game.appid) else {
            continue;
        };
        let install_path = install.install_path.to_string_lossy().to_string();
        if let Some(&game_id) = links.get(&game.appid) {
            updates.push((game_id, game.appid, install_path));
            continue;
        }
        if let Some(&game_id) = titles.get(&game.name.to_lowercase()) {
            updates.push((game_id, game.appid, install_path));
            continue;
        }
        additions.push((game, install));
    }

    let lang = state.db.with(|conn| {
        Ok(match settings::read_setting(conn, "language") {
            Ok(Some(value)) if value.eq_ignore_ascii_case("en") => "english",
            _ => "russian",
        })
    })?;

    // Store metadata and cover art; every game that misses fields is fetched
    // in its own task so the whole sync stays within a few seconds.
    let mut meta: HashMap<u32, SteamMeta> = HashMap::new();
    let mut pending = Vec::new();
    for game in &candidates {
        let known = links
            .get(&game.appid)
            .or_else(|| titles.get(&game.name.to_lowercase()))
            .and_then(|id| fields.get(id));

        let needs_cover = known
            .map(|f| f.cover.is_empty() || f.cover.starts_with("data:"))
            .unwrap_or(true);
        let needs_details = needs_cover
            || known
                .map(|f| {
                    f.genre.is_empty()
                        || f.developer.is_empty()
                        || f.publisher.is_empty()
                        || f.description.is_empty()
                        || f.release_year.is_none()
                })
                .unwrap_or(true);
        let playtime_seconds = i64::from(game.playtime_forever) * 60;

        if !needs_cover && !needs_details {
            meta.insert(game.appid, SteamMeta {
                cover: CoverChoice::Keep,
                description: String::new(),
                developer: String::new(),
                publisher: String::new(),
                genre: String::new(),
                release_year: None,
                playtime_seconds,
            });
            continue;
        }

        let handle = tauri::async_runtime::spawn(fetch_steam_meta(
            client.clone(),
            game.appid,
            playtime_seconds,
            needs_details,
            needs_cover,
            lang,
        ));
        pending.push((game.appid, playtime_seconds, needs_cover, handle));
    }

    for (appid, playtime_seconds, needs_cover, handle) in pending {
        match handle.await {
            Ok(stored) => {
                meta.insert(appid, stored);
            }
            Err(err) => {
                log::error!("Steam sync: metadata for app {appid} failed: {err}");
                meta.insert(appid, SteamMeta {
                    cover: if needs_cover {
                        CoverChoice::Placeholder
                    } else {
                        CoverChoice::Keep
                    },
                    description: String::new(),
                    developer: String::new(),
                    publisher: String::new(),
                    genre: String::new(),
                    release_year: None,
                    playtime_seconds,
                });
            }
        }
    }

    let mut new_rows: Vec<(u32, String, String, Option<String>)> = Vec::new();
    for (game, install) in &additions {
        let title = if game.name.trim().is_empty() {
            install.name.clone()
        } else {
            game.name.trim().to_string()
        };
        let install_path = install.install_path.to_string_lossy().to_string();
        let icon = game
            .icon_hash
            .as_deref()
            .map(|hash| api::icon_url(game.appid, hash));
        new_rows.push((game.appid, title, install_path, icon));
    }

    let result = state.db.with(|conn| {
        let (added, updated) = apply_sync(conn, &updates, &new_rows, &meta, &now)?;

        conn.execute(
            "UPDATE store_accounts SET last_synced_at = ?1 WHERE provider = ?2",
            rusqlite::params![now, PROVIDER],
        )?;

        Ok(SteamSyncResult {
            owned: owned.len() as i64,
            installed: installed_count,
            added,
            updated,
        })
    })?;

    log::info!(
        "Steam sync finished: {} owned, {} installed, {} added, {} updated",
        result.owned,
        result.installed,
        result.added,
        result.updated
    );
    achievements::evaluate(&app);
    let _ = app.emit("statistics-updated", serde_json::json!({}));
    Ok(result)
}

/// Writes synced games: fills missing store fields, repairs placeholder covers
/// and keeps the highest known playtime.
fn apply_sync(
    conn: &Connection,
    updates: &[(i64, u32, String)],
    new_rows: &[(u32, String, String, Option<String>)],
    meta: &HashMap<u32, SteamMeta>,
    now: &str,
) -> AppResult<(i64, i64)> {
    let mut added = 0i64;
    let mut updated = 0i64;

    for (game_id, appid, install_path) in updates {
        if apply_update(conn, *game_id, *appid, install_path, meta.get(appid), now)? {
            updated += 1;
        }
    }

    for (appid, title, install_path, icon) in new_rows {
        let info = meta.get(appid);

        // A concurrent run may have inserted this game already — update it instead.
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM games WHERE lower(title) = lower(?1)",
                [title],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(existing) = existing {
            if apply_update(conn, existing, *appid, install_path, info, now)? {
                updated += 1;
            }
            continue;
        }

        let cover = match info.map(|m| &m.cover) {
            Some(CoverChoice::Url(url)) => url.clone(),
            _ => placeholder_cover(title),
        };
        let description = info.map(|m| m.description.as_str()).unwrap_or("");
        let developer = info.map(|m| m.developer.as_str()).unwrap_or("");
        let publisher = info.map(|m| m.publisher.as_str()).unwrap_or("");
        let genre = info.map(|m| m.genre.as_str()).unwrap_or("");
        let release_year = info.and_then(|m| m.release_year);
        let playtime = info.map(|m| m.playtime_seconds).unwrap_or(0);

        conn.execute(
            "INSERT INTO games (title, description, executable_path, install_path, cover_path,
                    background_path, icon_path, developer, publisher, genre, release_year, platform,
                    source, controller_support, is_installed, is_favorite, is_hidden,
                    created_at, updated_at, last_played_at, total_playtime_seconds, launch_count)
             VALUES (?1, ?2, NULL, ?3, ?4, NULL, ?5, ?6, ?7, ?8, ?9, 'PC', 'Steam',
                'unknown', 1, 0, 0, ?10, ?10, NULL, ?11, 0)",
            rusqlite::params![
                title,
                description,
                install_path,
                cover,
                icon,
                developer,
                publisher,
                genre,
                release_year,
                now,
                playtime
            ],
        )?;
        let id = conn.last_insert_rowid();
        conn.execute(
            "INSERT OR IGNORE INTO steam_games (game_id, appid) VALUES (?1, ?2)",
            rusqlite::params![id, appid],
        )?;
        if let Err(err) = achievements::ensure_game_achievements(conn, id, title) {
            log::error!("Steam sync: achievements for \"{title}\" failed: {err}");
        }
        if let Err(err) = theme::auto_theme(conn, id, title, genre) {
            log::error!("Steam sync: theme for \"{title}\" failed: {err}");
        }
        added += 1;
    }

    Ok((added, updated))
}

/// Fills one Steam-linked game with store data, repairs its cover art and
/// keeps the highest known playtime. Returns true when a row changed.
fn apply_update(
    conn: &Connection,
    game_id: i64,
    appid: u32,
    install_path: &str,
    info: Option<&SteamMeta>,
    now: &str,
) -> AppResult<bool> {
    let cover = info.and_then(|m| match &m.cover {
        CoverChoice::Url(url) => Some(url.as_str()),
        _ => None,
    });
    let description = info.map(|m| m.description.as_str()).unwrap_or("");
    let developer = info.map(|m| m.developer.as_str()).unwrap_or("");
    let publisher = info.map(|m| m.publisher.as_str()).unwrap_or("");
    let genre = info.map(|m| m.genre.as_str()).unwrap_or("");
    let release_year = info.and_then(|m| m.release_year);
    let playtime = info.map(|m| m.playtime_seconds).unwrap_or(0);

    // Empty fields are filled from the store; playtime only ever grows.
    let changed = conn.execute(
        "UPDATE games SET install_path = ?1, is_installed = 1,
                source = CASE WHEN executable_path IS NULL OR executable_path = ''
                        THEN 'Steam' ELSE source END,
                cover_path = CASE WHEN ?2 IS NOT NULL
                            AND (cover_path IS NULL OR cover_path = ''
                                OR cover_path LIKE 'data:%')
                        THEN ?2 ELSE cover_path END,
                description = CASE WHEN description = '' THEN ?3 ELSE description END,
                developer = CASE WHEN developer = '' THEN ?4 ELSE developer END,
                publisher = CASE WHEN publisher = '' THEN ?5 ELSE publisher END,
                genre = CASE WHEN genre = '' THEN ?6 ELSE genre END,
                release_year = CASE WHEN release_year IS NULL THEN ?7 ELSE release_year END,
                total_playtime_seconds = MAX(total_playtime_seconds, ?8),
                updated_at = ?9
         WHERE id = ?10",
        rusqlite::params![
            install_path,
            cover,
            description,
            developer,
            publisher,
            genre,
            release_year,
            playtime,
            now,
            game_id
        ],
    )? > 0;

    conn.execute(
        "INSERT OR IGNORE INTO steam_games (game_id, appid) VALUES (?1, ?2)",
        rusqlite::params![game_id, appid],
    )?;

    // Heals games that were imported by an interrupted sync.
    let title: Option<String> = conn
        .query_row("SELECT title FROM games WHERE id = ?1", [game_id], |r| r.get(0))
        .optional()?;
    if let Some(title) = title {
        if let Err(err) = achievements::ensure_game_achievements(conn, game_id, &title) {
            log::error!("Steam sync: achievements for \"{title}\" failed: {err}");
        }
    }
    Ok(changed)
}

/// Collapses library rows that duplicate a Steam game (same lowercased title).
/// Keeps the row linked in `steam_games`, carries over sessions and playtime.
fn merge_duplicate_steam_games(conn: &Connection, now: &str) -> AppResult<usize> {
    let mut keys: Vec<String> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT lower(title) FROM games
             WHERE source = 'Steam'
             GROUP BY 1
             HAVING COUNT(*) > 1",
        )?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        for row in rows {
            keys.push(row?);
        }
    }

    let mut merged = 0usize;
    for key in keys {
        let mut rows: Vec<(i64, bool)> = Vec::new();
        {
            let mut stmt = conn.prepare(
                "SELECT g.id, EXISTS(SELECT 1 FROM steam_games s WHERE s.game_id = g.id)
                 FROM games g
                 WHERE lower(g.title) = ?1 AND g.source = 'Steam'
                 ORDER BY g.id",
            )?;
            let mapped = stmt.query_map([&key], |r| Ok((r.get(0)?, r.get(1)?)))?;
            for row in mapped {
                rows.push(row?);
            }
        }
        if rows.len() < 2 {
            continue;
        }
        let Some(keeper) = rows
            .iter()
            .find(|(_, linked)| *linked)
            .or_else(|| rows.first())
            .map(|(id, _)| *id)
        else {
            continue;
        };
        for (id, _) in &rows {
            if *id == keeper {
                continue;
            }
            merge_game_rows(conn, keeper, *id, now)?;
            merged += 1;
            log::info!("Merged duplicate \"{key}\": #{keeper} kept, #{id} removed");
        }
    }
    Ok(merged)
}

/// Moves sessions and playtime from `loser` to `keeper`, then drops `loser`.
fn merge_game_rows(conn: &Connection, keeper: i64, loser: i64, now: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE games SET
            total_playtime_seconds = MAX(total_playtime_seconds,
                COALESCE((SELECT total_playtime_seconds FROM games WHERE id = ?2), 0)),
            launch_count = MAX(launch_count,
                COALESCE((SELECT launch_count FROM games WHERE id = ?2), 0)),
            last_played_at = NULLIF(MAX(COALESCE(last_played_at, ''),
                COALESCE((SELECT last_played_at FROM games WHERE id = ?2), '')), ''),
            description = CASE WHEN description = ''
                THEN COALESCE((SELECT description FROM games WHERE id = ?2), '')
                ELSE description END,
            developer = CASE WHEN developer = ''
                THEN COALESCE((SELECT developer FROM games WHERE id = ?2), '')
                ELSE developer END,
            publisher = CASE WHEN publisher = ''
                THEN COALESCE((SELECT publisher FROM games WHERE id = ?2), '')
                ELSE publisher END,
            genre = CASE WHEN genre = ''
                THEN COALESCE((SELECT genre FROM games WHERE id = ?2), '')
                ELSE genre END,
            release_year = COALESCE(release_year, (SELECT release_year FROM games WHERE id = ?2)),
            updated_at = ?3
         WHERE id = ?1",
        rusqlite::params![keeper, loser, now],
    )?;
    conn.execute(
        "UPDATE game_sessions SET game_id = ?1 WHERE game_id = ?2",
        rusqlite::params![keeper, loser],
    )?;
    conn.execute(
        "UPDATE OR IGNORE steam_games SET game_id = ?1 WHERE game_id = ?2",
        rusqlite::params![keeper, loser],
    )?;
    conn.execute("DELETE FROM achievements WHERE game_id = ?1", [loser])?;
    conn.execute("DELETE FROM games WHERE id = ?1", [loser])?;

    let title: String =
        conn.query_row("SELECT title FROM games WHERE id = ?1", [keeper], |r| r.get(0))?;
    if let Err(err) = achievements::ensure_game_achievements(conn, keeper, &title) {
        log::error!("Steam sync: achievements for \"{title}\" failed: {err}");
    }
    Ok(())
}
