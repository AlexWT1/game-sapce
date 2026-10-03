use chrono::Datelike;
use rusqlite::Connection;
use tauri::State;

use crate::error::AppResult;
use crate::models::{GameStatistics, PlaytimeByDay, RankedPlaytime, Statistics};
use crate::AppState;

#[tauri::command]
pub fn get_statistics(state: State<'_, AppState>) -> AppResult<Statistics> {
    state.db.with(|conn| build_statistics(conn))
}

fn scalar_i64(conn: &Connection, sql: &str) -> Result<i64, rusqlite::Error> {
    conn.query_row(sql, [], |r| r.get(0)).map(|v: Option<i64>| v.unwrap_or(0))
}

pub fn build_statistics(conn: &Connection) -> AppResult<Statistics> {
    let demo_off = crate::commands::settings::demo_disabled(conn)?;
    let gwhere = if demo_off {
        " WHERE source != 'Demo'"
    } else {
        ""
    };
    let gand = if demo_off {
        " AND source != 'Demo'"
    } else {
        ""
    };

    let total_playtime_seconds = scalar_i64(
        conn,
        &format!("SELECT COALESCE(SUM(total_playtime_seconds), 0) FROM games{gwhere}"),
    )?;
    let games_count = scalar_i64(conn, &format!("SELECT COUNT(*) FROM games{gwhere}"))?;
    let installed_count = scalar_i64(
        conn,
        &format!("SELECT COUNT(*) FROM games WHERE is_installed = 1{gand}"),
    )?;
    let favorite_count = scalar_i64(
        conn,
        &format!("SELECT COUNT(*) FROM games WHERE is_favorite = 1{gand}"),
    )?;

    let (sessions_count, avg_session_seconds, longest_session_seconds) = if demo_off {
        (
            scalar_i64(
                conn,
                "SELECT COUNT(*) FROM game_sessions gs
                 JOIN games g ON g.id = gs.game_id
                 WHERE g.source != 'Demo' AND gs.ended_at IS NOT NULL",
            )?,
            scalar_i64(
                conn,
                "SELECT CAST(AVG(gs.duration_seconds) AS INTEGER) FROM game_sessions gs
                 JOIN games g ON g.id = gs.game_id
                 WHERE g.source != 'Demo' AND gs.ended_at IS NOT NULL AND gs.duration_seconds > 0",
            )?,
            scalar_i64(
                conn,
                "SELECT COALESCE(MAX(gs.duration_seconds), 0) FROM game_sessions gs
                 JOIN games g ON g.id = gs.game_id
                 WHERE g.source != 'Demo'",
            )?,
        )
    } else {
        (
            scalar_i64(
                conn,
                "SELECT COUNT(*) FROM game_sessions WHERE ended_at IS NOT NULL",
            )?,
            scalar_i64(
                conn,
                "SELECT CAST(AVG(duration_seconds) AS INTEGER) FROM game_sessions
                 WHERE ended_at IS NOT NULL AND duration_seconds > 0",
            )?,
            scalar_i64(
                conn,
                "SELECT COALESCE(MAX(duration_seconds), 0) FROM game_sessions",
            )?,
        )
    };

    let ach_where = if demo_off {
        "LEFT JOIN games g ON g.id = a.game_id
         WHERE (a.game_id IS NULL OR g.source != 'Demo')"
    } else {
        ""
    };
    let achievements_total =
        scalar_i64(conn, &format!("SELECT COUNT(*) FROM achievements a {ach_where}"))?;
    let achievements_unlocked = scalar_i64(
        conn,
        &format!(
            "SELECT COUNT(*) FROM achievements a {ach_where}{}",
            if demo_off {
                " AND a.unlocked = 1"
            } else {
                " WHERE a.unlocked = 1"
            }
        ),
    )?;

    let most_played = {
        let mut stmt = conn.prepare(&format!(
            "SELECT id, title, total_playtime_seconds, cover_path FROM games{gwhere}
             ORDER BY total_playtime_seconds DESC LIMIT 5"
        ))?;
        let rows = stmt.query_map([], |r| {
            Ok(RankedPlaytime {
                game_id: r.get(0)?,
                title: r.get(1)?,
                seconds: r.get(2)?,
                cover_path: r.get(3)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let by_genre = {
        let mut stmt = conn.prepare(&format!(
            "SELECT CASE WHEN genre = '' THEN 'Другое' ELSE genre END AS g, COUNT(*) as c
             FROM games{gwhere} GROUP BY g ORDER BY c DESC"
        ))?;
        let rows = stmt.query_map([], |r| {
            Ok(crate::models::GenreCount {
                genre: r.get(0)?,
                count: r.get(1)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let daily = duration_by_period(conn, 14, "days", demo_off)?;
    let weekly = duration_by_period(conn, 12, "weeks", demo_off)?;
    let monthly = duration_by_period(conn, 12, "months", demo_off)?;

    Ok(Statistics {
        total_playtime_seconds,
        games_count,
        installed_count,
        sessions_count,
        achievements_unlocked,
        achievements_total,
        favorite_count,
        most_played,
        by_genre,
        daily,
        weekly,
        monthly,
        avg_session_seconds,
        longest_session_seconds,
    })
}

fn duration_by_period(
    conn: &Connection,
    n: i64,
    unit: &str,
    demo_off: bool,
) -> AppResult<Vec<PlaytimeByDay>> {
    // Group by day for "days", by ISO week start for "weeks", by month for "months".
    // Note: SQLite's datetime() has no "weeks" modifier unit at all, and "months"/"years"
    // only work with floating-point values ("12.0 months"), so use day-based offsets.
    let (bucket, back): (&str, &str) = match unit {
        "weeks" => ("date(started_at, '-' || ((strftime('%w', started_at) + 6) % 7) || ' days')", "-84.0 days"),
        "months" => ("date(started_at, 'start of month')", "-365.0 days"),
        _ => ("date(started_at)", "-14.0 days"),
    };
    let sql = if demo_off {
        format!(
            "SELECT {bucket} AS bucket, COALESCE(SUM(gs.duration_seconds), 0) AS secs
             FROM game_sessions gs
             JOIN games g ON g.id = gs.game_id
             WHERE g.source != 'Demo' AND gs.started_at >= datetime('now', ?1)
             GROUP BY bucket ORDER BY bucket"
        )
    } else {
        format!(
            "SELECT {bucket} AS bucket, COALESCE(SUM(duration_seconds), 0) AS secs
             FROM game_sessions
             WHERE started_at >= datetime('now', ?1)
             GROUP BY bucket ORDER BY bucket"
        )
    };
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([back], |r| {
        Ok(PlaytimeByDay {
            date: r.get(0)?,
            seconds: r.get(1)?,
        })
    })?;
    let mut out = rows.collect::<Result<Vec<_>, _>>()?;

    // Fill gaps so charts look continuous.
    if !out.is_empty() {
        let step_days = match unit {
            "weeks" => 7,
            "months" => 30,
            _ => 1,
        };
        let mut filled = Vec::with_capacity(n as usize);
        let today = chrono::Utc::now().date_naive();
        for i in (0..n).rev() {
            let date = today - chrono::Duration::days(i * step_days);
            let key = match unit {
                "weeks" => {
                    let dow = date.weekday().num_days_from_monday() as i64;
                    (date - chrono::Duration::days(dow)).format("%Y-%m-%d").to_string()
                }
                "months" => date.format("%Y-%m-01").to_string(),
                _ => date.format("%Y-%m-%d").to_string(),
            };
            let secs = out
                .iter()
                .find(|p| p.date.starts_with(&key))
                .map(|p| p.seconds)
                .unwrap_or(0);
            filled.push(PlaytimeByDay {
                date: key,
                seconds: secs,
            });
        }
        // Deduplicate identical keys produced by month/week joins.
        let mut seen = std::collections::HashSet::new();
        out = filled
            .into_iter()
            .filter(|p| seen.insert(p.date.clone()))
            .collect();
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().expect("memory db");
        conn.pragma_update(None, "foreign_keys", "ON").expect("pragma");
        crate::database::migrations::run(&conn).expect("migrations");
        crate::commands::settings::demo_disabled(&conn).expect("demo flag");
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('demo_mode', 'off')",
            [],
        )
        .expect("demo off");
        conn
    }

    fn insert_session(conn: &Connection, game_id: i64, days_ago: i64, secs: i64) {
        let started = (Utc::now() - chrono::Duration::days(days_ago)).to_rfc3339();
        conn.execute(
            "INSERT INTO game_sessions (game_id, started_at, ended_at, duration_seconds, pid)
             VALUES (?1, ?2, ?3, ?4, NULL)",
            rusqlite::params![game_id, started, started, secs],
        )
        .expect("insert session");
    }

    #[test]
    fn weekly_buckets_use_rfc3339_timestamps() {
        let conn = memory_db();
        conn.execute(
            "INSERT INTO games (title, source, created_at, updated_at)
             VALUES ('Witcher 3', 'Manual', 't', 't')",
            [],
        )
        .expect("insert game");
        let game_id = conn.last_insert_rowid();

        insert_session(&conn, game_id, 0, 3600);
        insert_session(&conn, game_id, 3, 1800);
        insert_session(&conn, game_id, 20, 600);
        insert_session(&conn, game_id, 50, 7200);

        let weekly = duration_by_period(&conn, 12, "weeks", true).expect("weekly");
        assert_eq!(weekly.len(), 12, "weekly must be gap-filled to 12 buckets");
        let total: i64 = weekly.iter().map(|p| p.seconds).sum();
        assert_eq!(total, 3600 + 1800 + 600 + 7200, "all sessions must land in weekly buckets");
        // Newest session must land in the current ISO-week bucket (its Monday).
        let today = Utc::now().date_naive();
        let dow = today.weekday().num_days_from_monday() as i64;
        let monday = (today - chrono::Duration::days(dow))
            .format("%Y-%m-%d")
            .to_string();
        assert_eq!(weekly.last().expect("bucket").date, monday);
    }

    #[test]
    fn daily_buckets_use_rfc3339_timestamps() {
        let conn = memory_db();
        conn.execute(
            "INSERT INTO games (title, source, created_at, updated_at)
             VALUES ('Hades', 'Manual', 't', 't')",
            [],
        )
        .expect("insert game");
        let game_id = conn.last_insert_rowid();
        insert_session(&conn, game_id, 0, 900);
        insert_session(&conn, game_id, 5, 300);

        let daily = duration_by_period(&conn, 14, "days", true).expect("daily");
        assert_eq!(daily.len(), 14);
        let total: i64 = daily.iter().map(|p| p.seconds).sum();
        assert_eq!(total, 1200);
    }

    #[test]
    fn monthly_buckets_use_rfc3339_timestamps() {
        let conn = memory_db();
        conn.execute(
            "INSERT INTO games (title, source, created_at, updated_at)
             VALUES ('Disco Elysium', 'Manual', 't', 't')",
            [],
        )
        .expect("insert game");
        let game_id = conn.last_insert_rowid();
        insert_session(&conn, game_id, 0, 3600);
        insert_session(&conn, game_id, 50, 600);

        let monthly = duration_by_period(&conn, 12, "months", true).expect("monthly");
        assert!(!monthly.is_empty(), "monthly must not be empty");
        let total: i64 = monthly.iter().map(|p| p.seconds).sum();
        assert_eq!(total, 4200, "all sessions must land in monthly buckets");
    }
}

#[tauri::command]
pub fn get_game_statistics(
    state: State<'_, AppState>,
    game_id: i64,
) -> AppResult<GameStatistics> {
    state.db.with(|conn| {
        let total_playtime_seconds: i64 = conn
            .query_row(
                "SELECT total_playtime_seconds FROM games WHERE id = ?1",
                [game_id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let launch_count: i64 = conn
            .query_row(
                "SELECT launch_count FROM games WHERE id = ?1",
                [game_id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let sessions_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM game_sessions WHERE game_id = ?1 AND ended_at IS NOT NULL",
                [game_id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let avg_session_seconds: i64 = conn
            .query_row(
                "SELECT CAST(AVG(duration_seconds) AS INTEGER) FROM game_sessions
                 WHERE game_id = ?1 AND ended_at IS NOT NULL AND duration_seconds > 0",
                [game_id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let longest_session_seconds: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(duration_seconds), 0) FROM game_sessions WHERE game_id = ?1",
                [game_id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let last_session_at: Option<String> = conn
            .query_row(
                "SELECT MAX(ended_at) FROM game_sessions WHERE game_id = ?1",
                [game_id],
                |r| r.get(0),
            )
            .unwrap_or(None);
        let playtime_week_seconds: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(duration_seconds), 0) FROM game_sessions
                 WHERE game_id = ?1 AND started_at >= datetime('now', '-7 days')",
                [game_id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let playtime_month_seconds: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(duration_seconds), 0) FROM game_sessions
                 WHERE game_id = ?1 AND started_at >= datetime('now', '-30 days')",
                [game_id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let playtime_year_seconds: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(duration_seconds), 0) FROM game_sessions
                 WHERE game_id = ?1 AND started_at >= datetime('now', '-365 days')",
                [game_id],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let mut stmt = conn.prepare(
            "SELECT date(started_at) AS d, COALESCE(SUM(duration_seconds), 0)
             FROM game_sessions WHERE game_id = ?1 AND started_at >= datetime('now', '-14 days')
             GROUP BY d ORDER BY d",
        )?;
        let rows = stmt.query_map([game_id], |r| {
            Ok(PlaytimeByDay {
                date: r.get(0)?,
                seconds: r.get(1)?,
            })
        })?;
        let raw = rows.collect::<Result<Vec<_>, _>>()?;
        let today = chrono::Utc::now().date_naive();
        let mut daily = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for i in (0..14).rev() {
            let key = (today - chrono::Duration::days(i))
                .format("%Y-%m-%d")
                .to_string();
            seen.insert(key.clone());
            let seconds = raw
                .iter()
                .find(|p| p.date.starts_with(&key))
                .map(|p| p.seconds)
                .unwrap_or(0);
            daily.push(PlaytimeByDay { date: key, seconds });
        }

        Ok(GameStatistics {
            total_playtime_seconds,
            launch_count,
            sessions_count,
            avg_session_seconds,
            longest_session_seconds,
            last_session_at,
            playtime_week_seconds,
            playtime_month_seconds,
            playtime_year_seconds,
            daily,
        })
    })
}
