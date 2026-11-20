use rusqlite::{Connection, OptionalExtension};
use tauri::{AppHandle, Emitter, Manager};

use crate::models::Achievement;
use crate::AppState;

/// Built-in extensible Game Space achievement definitions.
struct AppRule {
    key: &'static str,
    title: &'static str,
    title_en: &'static str,
    description: &'static str,
    description_en: &'static str,
    rarity: &'static str,
    /// Returns current progress (can exceed target).
    progress: fn(&Connection) -> Result<(i64, i64), rusqlite::Error>,
}

const APP_RULES: &[AppRule] = &[
    AppRule {
        key: "first_launch",
        title: "Первый запуск",
        title_en: "First Launch",
        description: "Запусти первую игру",
        description_en: "Launch your first game",
        rarity: "Common",
        progress: |conn| {
            conn.query_row("SELECT COUNT(*) FROM games WHERE launch_count > 0", [], |r| {
                r.get::<_, i64>(0)
            })
            .map(|c| (c.min(1), 1))
        },
    },
    AppRule {
        key: "collector",
        title: "Коллекционер",
        title_en: "Collector",
        description: "Добавь 10 игр в библиотеку",
        description_en: "Add 10 games to library",
        rarity: "Rare",
        progress: |conn| {
            conn.query_row("SELECT COUNT(*) FROM games", [], |r| r.get::<_, i64>(0))
                .map(|c| (c, 10))
        },
    },
    AppRule {
        key: "dedicated_player",
        title: "Преданный игрок",
        title_en: "Dedicated Player",
        description: "Проведи 50 часов в играх",
        description_en: "Spend 50 hours in games",
        rarity: "Epic",
        progress: |conn| {
            conn.query_row(
                "SELECT COALESCE(SUM(total_playtime_seconds), 0) FROM games",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|s| (s, 50 * 3600))
        },
    },
    AppRule {
        key: "night_owl",
        title: "Полуночник",
        title_en: "Night Owl",
        description: "Поиграй после полуночи",
        description_en: "Play after midnight",
        rarity: "Rare",
        progress: |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM game_sessions WHERE CAST(strftime('%H', started_at) AS INTEGER) < 5",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|c| (c.min(1), 1))
        },
    },
    AppRule {
        key: "marathon",
        title: "Марафон",
        title_en: "Marathon",
        description: "Проведи в игре больше 5 часов за сессию",
        description_en: "Play for more than 5 hours in a single session",
        rarity: "Epic",
        progress: |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM game_sessions WHERE duration_seconds > 18000",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|c| (c.min(1), 1))
        },
    },
    AppRule {
        key: "explorer",
        title: "Исследователь",
        title_en: "Explorer",
        description: "Добавь игры 5 разных жанров",
        description_en: "Add games of 5 different genres",
        rarity: "Rare",
        progress: |conn| {
            conn.query_row(
                "SELECT COUNT(DISTINCT genre) FROM games WHERE genre != ''",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|c| (c, 5))
        },
    },
    AppRule {
        key: "century",
        title: "Центурион",
        title_en: "Centurion",
        description: "Заверши 100 игровых сессий",
        description_en: "Complete 100 gaming sessions",
        rarity: "Legendary",
        progress: |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM game_sessions WHERE ended_at IS NOT NULL",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|c| (c, 100))
        },
    },
    AppRule {
        key: "curator",
        title: "Куратор",
        title_en: "Curator",
        description: "Отметь 5 игр избранными",
        description_en: "Mark 5 games as favorites",
        rarity: "Common",
        progress: |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM games WHERE is_favorite = 1",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|c| (c, 5))
        },
    },
];

/// Inserts any missing built-in app achievements.
pub fn ensure_app_achievements(conn: &Connection) -> Result<(), rusqlite::Error> {
    let locale_en = crate::commands::settings::read_setting(conn, "locale")
        .unwrap_or(None)
        .as_deref() == Some("en")
        || crate::commands::settings::read_setting(conn, "language")
            .unwrap_or(None)
            .as_deref() == Some("en");
    for rule in APP_RULES {
        let title = if locale_en { rule.title_en } else { rule.title };
        let desc = if locale_en { rule.description_en } else { rule.description };
        let exists = conn
            .query_row(
                "SELECT 1 FROM achievements WHERE key = ?1",
                [rule.key],
                |r| r.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if !exists {
            conn.execute(
                "INSERT INTO achievements (game_id, title, description, rarity, scope, key, progress_target)
                 VALUES (NULL, ?1, ?2, ?3, 'app', ?4, 1)",
                rusqlite::params![title, desc, rule.rarity, rule.key],
            )?;
        } else {
            conn.execute(
                "UPDATE achievements SET title = ?1, description = ?2 WHERE key = ?3 AND scope = 'app'",
                rusqlite::params![title, desc, rule.key],
            )?;
        }
    }
    Ok(())
}

/// Evaluates all achievement rules; unlocks and returns newly unlocked ones.
pub fn evaluate(app: &AppHandle) -> Vec<Achievement> {
    let state = app.state::<AppState>();
    let mut unlocked = Vec::new();

    let result = state.db.with(|conn| {
        ensure_app_achievements(conn).map_err(crate::error::AppError::Database)?;
        // Heal libraries that predate the per-game achievement set.
        ensure_all_game_achievements(conn).map_err(crate::error::AppError::Database)?;
        let mut newly = Vec::new();

        for rule in APP_RULES {
            let (current, target) = (rule.progress)(conn)?;
            let row_id: Option<i64> = conn
                .query_row(
                    "SELECT id FROM achievements WHERE key = ?1",
                    [rule.key],
                    |r| r.get(0),
                )
                .optional()?;
            let Some(id) = row_id else { continue };

            conn.execute(
                "UPDATE achievements SET progress_current = ?1, progress_target = ?2 WHERE id = ?3",
                rusqlite::params![current, target, id],
            )?;

            if current >= target {
                let changed = conn.execute(
                    "UPDATE achievements SET unlocked = 1, unlocked_at = COALESCE(unlocked_at, ?1) WHERE id = ?2 AND unlocked = 0",
                    rusqlite::params![chrono::Utc::now().to_rfc3339(), id],
                )?;
                if changed > 0 {
                    log::info!("Achievement unlocked: {}", rule.title);
                    newly.push(id);
                }
            }
        }

        // Refresh per-game achievement progress (launch-based).
        conn.execute(
            "UPDATE achievements SET progress_current = (
                 SELECT launch_count FROM games WHERE games.id = achievements.game_id
               ) WHERE game_id IS NOT NULL AND progress_target <= 1",
            [],
        )?;
        // Hour-based achievements track played hours for progress display.
        newly.extend(refresh_and_unlock(conn)?);

        Ok(newly)
    });

    match result {
        Ok(ids) => {
            for id in ids {
                if let Ok(ach) = get_by_id(&state, id) {
                    let _ = app.emit("achievement-unlocked", &ach);
                    unlocked.push(ach);
                }
            }
            if !unlocked.is_empty() {
                let _ = app.emit("statistics-updated", serde_json::json!({}));
            }
            unlocked
        }
        Err(e) => {
            log::error!("achievement evaluation failed: {e}");
            Vec::new()
        }
    }
}

/// Refreshes per-game achievement progress and unlocks everything whose
/// target is reached (launch count and played hours). Returns unlocked ids.
pub fn refresh_and_unlock(conn: &Connection) -> Result<Vec<i64>, rusqlite::Error> {
    // Launch-based progress (targets of 1: "Первый запуск").
    conn.execute(
        "UPDATE achievements SET progress_current = (
             SELECT launch_count FROM games WHERE games.id = achievements.game_id
           ) WHERE game_id IS NOT NULL AND progress_target <= 1",
        [],
    )?;
    // Hour-based achievements track played hours for progress display.
    conn.execute(
        "UPDATE achievements SET progress_current = (
             SELECT total_playtime_seconds / 3600 FROM games
              WHERE games.id = achievements.game_id
           ) WHERE game_id IS NOT NULL AND key LIKE 'playtime_%' AND unlocked = 0",
        [],
    )?;
    let now = chrono::Utc::now().to_rfc3339();
    let mut stmt = conn.prepare(
        "UPDATE achievements SET unlocked = 1, unlocked_at = COALESCE(unlocked_at, ?1)
         WHERE game_id IS NOT NULL AND unlocked = 0
           AND progress_current >= progress_target
         RETURNING id",
    )?;
    let ids = stmt
        .query_map([now], |r| r.get::<_, i64>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);
    Ok(ids)
}

fn get_by_id(state: &AppState, id: i64) -> Result<Achievement, crate::error::AppError> {
    state.db.with(|conn| {
        conn.query_row(
            "SELECT a.id, a.game_id, g.title, a.title, a.description, a.icon_path, a.rarity,
                    a.unlocked, a.unlocked_at, a.progress_current, a.progress_target, a.scope, a.key
             FROM achievements a LEFT JOIN games g ON g.id = a.game_id
             WHERE a.id = ?1",
            [id],
            map_achievement,
        )
        .map_err(|_| crate::error::AppError::NotFound(format!("achievement {id}")))
    })
}

pub fn map_achievement(row: &rusqlite::Row<'_>) -> rusqlite::Result<Achievement> {
    Ok(Achievement {
        id: row.get(0)?,
        game_id: row.get(1)?,
        game_title: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        icon_path: row.get(5)?,
        rarity: row.get(6)?,
        unlocked: row.get::<_, i64>(7)? != 0,
        unlocked_at: row.get(8)?,
        progress_current: row.get(9)?,
        progress_target: row.get(10)?,
        scope: row.get(11)?,
        key: row.get(12)?,
    })
}

struct HourAchievement {
    key: &'static str,
    title_ru: &'static str,
    title_en: &'static str,
    rarity: &'static str,
    hours: i64,
}

/// Per-game playtime achievement ladder (hours, in-game title, rarity).
const GAME_HOUR_ACHIEVEMENTS: &[HourAchievement] = &[
    HourAchievement {
        key: "playtime_10h",
        title_ru: "Разогрев",
        title_en: "Warming Up",
        rarity: "Common",
        hours: 10,
    },
    HourAchievement {
        key: "playtime_100h",
        title_ru: "Втянулся",
        title_en: "Hooked",
        rarity: "Rare",
        hours: 100,
    },
    HourAchievement {
        key: "playtime_1000h",
        title_ru: "Живая легенда",
        title_en: "Living Legend",
        rarity: "Legendary",
        hours: 1000,
    },
];

/// Creates the standard per-game achievement set for a newly added game:
/// launch + 10/100/1000 hours. Idempotent.
pub fn ensure_game_achievements(
    conn: &Connection,
    game_id: i64,
    title: &str,
) -> Result<(), rusqlite::Error> {
    let locale_en = crate::commands::settings::read_setting(conn, "locale")
        .unwrap_or(None)
        .as_deref() == Some("en")
        || crate::commands::settings::read_setting(conn, "language")
            .unwrap_or(None)
            .as_deref() == Some("en");
    let first_launch_title = if locale_en { "First Launch" } else { "Первый запуск" };
    let first_launch_desc = if locale_en {
        format!("Launch \"{}\"", title)
    } else {
        format!("Запусти «{}»", title)
    };
    conn.execute(
        "INSERT INTO achievements (game_id, title, description, rarity, scope, key, progress_target)
         SELECT ?1, ?2, ?3, 'Common', 'game', 'first_launch_game', 1
         WHERE NOT EXISTS (SELECT 1 FROM achievements WHERE game_id = ?1 AND key = 'first_launch_game')",
        rusqlite::params![game_id, first_launch_title, first_launch_desc],
    )?;
    conn.execute(
        "UPDATE achievements SET title = ?2, description = ?3 WHERE game_id = ?1 AND key = 'first_launch_game'",
        rusqlite::params![game_id, first_launch_title, first_launch_desc],
    )?;
    for ha in GAME_HOUR_ACHIEVEMENTS {
        let ach_title = if locale_en { ha.title_en } else { ha.title_ru };
        let ach_desc = if locale_en {
            format!("Spend {} hours playing \"{}\"", ha.hours, title)
        } else {
            format!("Проведи {} часов в игре «{}»", ha.hours, title)
        };
        conn.execute(
            "INSERT INTO achievements (game_id, title, description, rarity, scope, key, progress_target)
             SELECT ?1, ?2, ?3, ?4, 'game', ?5, ?6
             WHERE NOT EXISTS (SELECT 1 FROM achievements WHERE game_id = ?1 AND key = ?5)",
            rusqlite::params![game_id, ach_title, ach_desc, ha.rarity, ha.key, ha.hours],
        )?;
        conn.execute(
            "UPDATE achievements SET title = ?2, description = ?3 WHERE game_id = ?1 AND key = ?4",
            rusqlite::params![game_id, ach_title, ach_desc, ha.key],
        )?;
    }
    Ok(())
}

/// Backfills the standard achievement set for every game that lacks it
/// (e.g. libraries created before the feature existed).
pub fn ensure_all_game_achievements(conn: &Connection) -> Result<(), rusqlite::Error> {
    let rows: Vec<(i64, String)> = {
        let mut stmt = conn.prepare("SELECT id, title FROM games")?;
        let mapped = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        mapped.collect::<Result<Vec<_>, _>>()?
    };
    for (id, title) in rows {
        ensure_game_achievements(conn, id, &title)?;
    }
    Ok(())
}

/// Unlocks an achievement manually (used by tests / future integrations).
pub fn unlock(conn: &Connection, id: i64) -> Result<Achievement, crate::error::AppError> {
    conn.execute(
        "UPDATE achievements SET unlocked = 1, unlocked_at = COALESCE(unlocked_at, ?1),
         progress_current = progress_target WHERE id = ?2",
        rusqlite::params![chrono::Utc::now().to_rfc3339(), id],
    )
    .map_err(crate::error::AppError::Database)?;
    conn.query_row(
        "SELECT a.id, a.game_id, g.title, a.title, a.description, a.icon_path, a.rarity,
                a.unlocked, a.unlocked_at, a.progress_current, a.progress_target, a.scope, a.key
         FROM achievements a LEFT JOIN games g ON g.id = a.game_id WHERE a.id = ?1",
        [id],
        map_achievement,
    )
    .map_err(|_| crate::error::AppError::NotFound(format!("achievement {id}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::migrations;

    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().expect("memory db");
        conn.pragma_update(None, "foreign_keys", "ON")
            .expect("pragma");
        migrations::run(&conn).expect("migrations");
        conn
    }

    fn insert_game(conn: &Connection, title: &str) -> i64 {
        conn.execute(
            "INSERT INTO games (title, created_at, updated_at) VALUES (?1, ?2, ?2)",
            rusqlite::params![title, "2026-01-01T00:00:00Z"],
        )
        .expect("insert game");
        conn.last_insert_rowid()
    }

    #[test]
    fn launch_achievement_is_created_for_every_game() {
        let conn = memory_db();
        for title in ["Alpha", "Beta"] {
            let id = insert_game(&conn, title);
            ensure_game_achievements(&conn, id, title).expect("ensure achievements");
        }
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM achievements WHERE key = 'first_launch_game'",
                [],
                |r| r.get(0),
            )
            .expect("count");
        assert_eq!(count, 2);
    }

    #[test]
    fn app_achievement_keys_stay_unique() {
        let conn = memory_db();
        conn.execute(
            "INSERT INTO achievements (game_id, title, key, scope)
             VALUES (NULL, 'Century', 'century', 'app')",
            [],
        )
        .expect("first app achievement");
        let duplicate = conn.execute(
            "INSERT INTO achievements (game_id, title, key, scope)
             VALUES (NULL, 'Century again', 'century', 'app')",
            [],
        );
        assert!(duplicate.is_err());
    }

    #[test]
    fn standard_set_has_four_achievements_and_is_idempotent() {
        let conn = memory_db();
        let id = insert_game(&conn, "Alpha");
        ensure_game_achievements(&conn, id, "Alpha").expect("first ensure");
        ensure_game_achievements(&conn, id, "Alpha").expect("second ensure");

        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM achievements WHERE game_id = ?1",
                [id],
                |r| r.get(0),
            )
            .expect("count");
        assert_eq!(total, 4);

        for key in ["first_launch_game", "playtime_10h", "playtime_100h", "playtime_1000h"] {
            let n: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM achievements WHERE game_id = ?1 AND key = ?2",
                    rusqlite::params![id, key],
                    |r| r.get(0),
                )
                .expect("key count");
            assert_eq!(n, 1, "key {key} must exist exactly once");
        }
    }

    #[test]
    fn backfill_covers_games_without_achievements() {
        let conn = memory_db();
        insert_game(&conn, "Legacy");
        ensure_all_game_achievements(&conn).expect("backfill");
        ensure_all_game_achievements(&conn).expect("second backfill");

        let per_game: i64 = conn
            .query_row("SELECT COUNT(*) FROM achievements", [], |r| r.get(0))
            .expect("count");
        assert_eq!(per_game, 4);
    }

    #[test]
    fn imported_playtime_unlocks_hour_achievements() {
        let conn = memory_db();
        let id = insert_game(&conn, "Steam Game");
        // Imported Steam stats: 40 hours played, never launched locally.
        conn.execute(
            "UPDATE games SET total_playtime_seconds = 40 * 3600 WHERE id = ?1",
            [id],
        )
        .expect("playtime");
        ensure_game_achievements(&conn, id, "Steam Game").expect("ensure");

        let unlocked = refresh_and_unlock(&conn).expect("unlock");
        assert_eq!(unlocked.len(), 1, "10h must unlock for 40h played");

        let locked: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM achievements WHERE game_id = ?1 AND unlocked = 0",
                [id],
                |r| r.get(0),
            )
            .expect("locked count");
        assert_eq!(locked, 3, "launch, 100h and 1000h stay locked");

        let hours: i64 = conn
            .query_row(
                "SELECT progress_current FROM achievements WHERE game_id = ?1 AND key = 'playtime_100h'",
                [id],
                |r| r.get(0),
            )
            .expect("progress");
        assert_eq!(hours, 40);
    }

    #[test]
    fn fresh_game_keeps_hour_achievements_locked() {
        let conn = memory_db();
        let id = insert_game(&conn, "New Game");
        ensure_game_achievements(&conn, id, "New Game").expect("ensure");

        let unlocked = refresh_and_unlock(&conn).expect("unlock");
        assert!(unlocked.is_empty(), "nothing to unlock without stats");
    }

    #[test]
    fn achievements_respect_locale_and_update_on_change() {
        let conn = memory_db();
        let id = insert_game(&conn, "Doom");

        // 1. Russian default
        ensure_app_achievements(&conn).expect("ensure app");
        ensure_game_achievements(&conn, id, "Doom").expect("ensure game");

        let app_title: String = conn
            .query_row("SELECT title FROM achievements WHERE key = 'first_launch'", [], |r| r.get(0))
            .expect("title");
        assert_eq!(app_title, "Первый запуск");

        let game_desc: String = conn
            .query_row("SELECT description FROM achievements WHERE game_id = ?1 AND key = 'first_launch_game'", [id], |r| r.get(0))
            .expect("desc");
        assert_eq!(game_desc, "Запусти «Doom»");

        // 2. Change locale to English
        conn.execute("INSERT OR REPLACE INTO settings (key, value) VALUES ('locale', 'en')", [])
            .expect("set locale");
        ensure_app_achievements(&conn).expect("ensure app en");
        ensure_game_achievements(&conn, id, "Doom").expect("ensure game en");

        let app_title_en: String = conn
            .query_row("SELECT title FROM achievements WHERE key = 'first_launch'", [], |r| r.get(0))
            .expect("title en");
        assert_eq!(app_title_en, "First Launch");

        let game_desc_en: String = conn
            .query_row("SELECT description FROM achievements WHERE game_id = ?1 AND key = 'first_launch_game'", [id], |r| r.get(0))
            .expect("desc en");
        assert_eq!(game_desc_en, "Launch \"Doom\"");
    }
}
