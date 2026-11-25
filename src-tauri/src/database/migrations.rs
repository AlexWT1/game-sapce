use rusqlite::Connection;

use crate::error::{AppError, AppResult};

/// Ordered list of embedded migrations. New files are appended here.
const MIGRATIONS: &[(&str, &str)] = &[
    ("001_init", include_str!("../../../migrations/001_init.sql")),
    ("002_steam", include_str!("../../../migrations/002_steam.sql")),
    (
        "003_achievements_key",
        include_str!("../../../migrations/003_achievements_key.sql"),
    ),
    ("004_launcher", include_str!("../../../migrations/004_launcher.sql")),
    (
        "005_launcher_args",
        include_str!("../../../migrations/005_launcher_args.sql"),
    ),
    ("006_epic", include_str!("../../../migrations/006_epic.sql")),
];

/// Applies all pending migrations inside a transaction-ish sequence.
pub fn run(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )?;

    let applied: Vec<String> = {
        let mut stmt = conn.prepare("SELECT version FROM schema_migrations")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    for (version, sql) in MIGRATIONS {
        if applied.iter().any(|v| v == version) {
            continue;
        }
        log::info!("Applying migration {version}");
        conn.execute_batch(sql).map_err(|e| {
            log::error!("Migration {version} failed: {e}");
            AppError::Other(format!("migration {version} failed: {e}"))
        })?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            rusqlite::params![version, chrono::Utc::now().to_rfc3339()],
        )?;
        log::info!("Migration {version} applied");
    }
    Ok(())
}
