use rusqlite::{Connection, OptionalExtension};
use tauri::{AppHandle, Emitter, Manager};

use crate::models::Achievement;
use crate::AppState;

/// Built-in extensible Game Space achievement definitions.
struct AppRule {
    key: &'static str,
    title: &'static str,
    description: &'static str,
    rarity: &'static str,
    /// Returns current progress (can exceed target).
    progress: fn(&Connection) -> Result<(i64, i64), rusqlite::Error>,
}

const APP_RULES: &[AppRule] = &[
    AppRule {
        key: "first_launch",
        title: "First Launch",
        description: "Запусти первую игру",
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
        title: "Collector",
        description: "Добавь 10 игр в библиотеку",
        rarity: "Rare",
        progress: |conn| {
            conn.query_row("SELECT COUNT(*) FROM games", [], |r| r.get::<_, i64>(0))
                .map(|c| (c, 10))
        },
    },
    AppRule {
        key: "dedicated_player",
        title: "Dedicated Player",
        description: "Проведи 50 часов в играх",
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
        title: "Night Owl",
        description: "Поиграй после полуночи",
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
        title: "Marathon",
        description: "Проведи в игре больше 5 часов за сессию",
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
        title: "Explorer",
        description: "Добавь игры 5 разных жанров",
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
        title: "Centurion",
        description: "Заверши 100 игровых сессий",
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
        title: "Curator",
        description: "Отметь 5 игр избранными",
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
    for rule in APP_RULES {
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
                rusqlite::params![rule.title, rule.description, rule.rarity, rule.key],
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
        conn.execute(
            "UPDATE achievements SET unlocked = 1, unlocked_at = COALESCE(unlocked_at, ?1)
             WHERE game_id IS NOT NULL AND unlocked = 0 AND progress_current >= progress_target",
            [chrono::Utc::now().to_rfc3339()],
        )?;

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

fn get_by_id(state: &AppState, id: i64) -> Result<Achievement, crate::error::AppError> {
    state.db.with(|conn| {
        conn.query_row(
            "SELECT a.id, a.game_id, g.title, a.title, a.description, a.icon_path, a.rarity,
                    a.unlocked, a.unlocked_at, a.progress_current, a.progress_target, a.scope
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
    })
}

/// Creates a launch achievement for a newly added game.
pub fn ensure_game_achievements(
    conn: &Connection,
    game_id: i64,
    title: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO achievements (game_id, title, description, rarity, scope, key, progress_target)
         SELECT ?1, 'Первый запуск', 'Запусти «' || ?2 || '»', 'Common', 'game', 'first_launch_game', 1
         WHERE NOT EXISTS (SELECT 1 FROM achievements WHERE game_id = ?1 AND key = 'first_launch_game')",
        rusqlite::params![game_id, title],
    )?;
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
                a.unlocked, a.unlocked_at, a.progress_current, a.progress_target, a.scope
         FROM achievements a LEFT JOIN games g ON g.id = a.game_id WHERE a.id = ?1",
        [id],
        map_achievement,
    )
    .map_err(|_| crate::error::AppError::NotFound(format!("achievement {id}")))
}
