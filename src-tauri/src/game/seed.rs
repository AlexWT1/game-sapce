use rusqlite::{Connection, OptionalExtension};

use crate::services::placeholder::placeholder_cover;

struct SeedGame {
    title: &'static str,
    description: &'static str,
    developer: &'static str,
    genre: &'static str,
    year: i64,
    hours: i64,
    sessions: i64,
    favorite: bool,
    theme: Option<(&'static str, &'static str, &'static str)>,
}

const SEED: &[SeedGame] = &[
    SeedGame {
        title: "The Witcher 3: Wild Hunt",
        description: "Эпическое приключение в открытом мире, полное захватывающих историй и незабываемых персонажей.",
        developer: "CD Projekt RED",
        genre: "RPG",
        year: 2015,
        hours: 248,
        sessions: 42,
        favorite: true,
        theme: Some(("#1a1c24", "#8b1e2d", "#c9a227")),
    },
    SeedGame {
        title: "Cyberpunk 2077",
        description: "Открытый мир Найт-Сити — мегаполиса будущего, где власть, роскошь и кибернетика.",
        developer: "CD Projekt RED",
        genre: "RPG",
        year: 2020,
        hours: 96,
        sessions: 21,
        favorite: true,
        theme: Some(("#0d0f14", "#fcee0a", "#00f0ff")),
    },
    SeedGame {
        title: "Hogwarts Legacy",
        description: "Погрузись в мир магии XIX века и стань волшебником Хогвартса.",
        developer: "Avalanche Software",
        genre: "Приключения",
        year: 2023,
        hours: 64,
        sessions: 15,
        favorite: false,
        theme: Some(("#12100c", "#c9a227", "#6ea8fe")),
    },
    SeedGame {
        title: "Red Dead Redemption 2",
        description: "Эпическая история Артура Моргана и банды Ван дер Линде на Диком Западе.",
        developer: "Rockstar Games",
        genre: "Экшен",
        year: 2018,
        hours: 132,
        sessions: 30,
        favorite: true,
        theme: Some(("#140f0c", "#b7410e", "#e0c097")),
    },
    SeedGame {
        title: "Elden Ring",
        description: "Тёмное фэнтези от FromSoftware в открытом мире Благословенных Земель.",
        developer: "FromSoftware",
        genre: "Экшен",
        year: 2022,
        hours: 88,
        sessions: 27,
        favorite: false,
        theme: Some(("#0c0b09", "#c9a227", "#f4e4bc")),
    },
    SeedGame {
        title: "Baldur's Gate 3",
        description: "Тактическая RPG по миру Dungeons & Dragons с глубокими выборами.",
        developer: "Larian Studios",
        genre: "RPG",
        year: 2023,
        hours: 110,
        sessions: 24,
        favorite: false,
        theme: Some(("#100c14", "#7c3aed", "#f59e0b")),
    },
    SeedGame {
        title: "Starfield",
        description: "Космическое исследование от Bethesda — сотни планет и свобода выбора.",
        developer: "Bethesda Game Studios",
        genre: "Приключения",
        year: 2023,
        hours: 54,
        sessions: 12,
        favorite: false,
        theme: Some(("#0a0e18", "#38bdf8", "#94a3b8")),
    },
    SeedGame {
        title: "God of War",
        description: "Кратос и Атрей в путешествии по миру скандинавской мифологии.",
        developer: "Santa Monica Studio",
        genre: "Экшен",
        year: 2018,
        hours: 42,
        sessions: 11,
        favorite: false,
        theme: Some(("#140d0d", "#b91c1c", "#f8fafc")),
    },
    SeedGame {
        title: "The Last of Us Part I",
        description: "История выживания Джоэл и Элли в мире, погубленном эпидемией.",
        developer: "Naughty Dog",
        genre: "Приключения",
        year: 2022,
        hours: 30,
        sessions: 9,
        favorite: false,
        theme: Some(("#0e1210", "#4d7c0f", "#d6d3d1")),
    },
    SeedGame {
        title: "Assassin's Creed Valhalla",
        description: "Стань эйнхерри и возглави викингов в набегах на Англию.",
        developer: "Ubisoft Montreal",
        genre: "Экшен",
        year: 2020,
        hours: 74,
        sessions: 18,
        favorite: false,
        theme: Some(("#0b1214", "#0ea5e9", "#a3e635")),
    },
];

const GAME_ACHIEVEMENTS: &[(&str, &str, &str)] = &[
    ("Начало пути", "Заверши первую сессию", "Common"),
    ("Исследователь", "Проведи 20 часов в игре", "Rare"),
    ("Преданный", "Сыграй 10 сессий", "Epic"),
    ("Марафонец", "Одна сессия дольше 3 часов", "Legendary"),
];

fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, rusqlite::Error> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        [key],
        |r| r.get(0),
    )
    .optional()
}

fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

fn seed_titles_clause() -> String {
    let titles: Vec<String> = SEED.iter().map(|g| format!("'{e}'", e = g.title.replace('\'', "''"))).collect();
    format!("title IN ({})", titles.join(", "))
}

/// One-time migration: old seed used source=Manual and forced demo_mode=off after seeding.
fn migrate_legacy_demo(conn: &Connection) -> Result<(), rusqlite::Error> {
    if get_setting(conn, "demo_seeded")?.is_some() {
        return Ok(());
    }

    let clause = seed_titles_clause();
    let legacy: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM games WHERE executable_path IS NULL AND source = 'Manual' AND {clause}"),
        [],
        |r| r.get(0),
    )?;

    if legacy > 0 {
        conn.execute(
            &format!("UPDATE games SET source = 'Demo' WHERE executable_path IS NULL AND source = 'Manual' AND {clause}"),
            [],
        )?;
        set_setting(conn, "demo_seeded", "1")?;
        // Old flow set demo_mode=off as a "seeded" marker; restore visibility so the toggle works.
        if get_setting(conn, "demo_mode")?.as_deref() == Some("off") {
            set_setting(conn, "demo_mode", "on")?;
        }
        log::info!("Migrated {legacy} legacy demo games to source=Demo");
        return Ok(());
    }

    let any_games: i64 = conn.query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))?;
    if any_games > 0 {
        set_setting(conn, "demo_seeded", "1")?;
    }
    Ok(())
}

/// Seeds demo data when the library is empty. Visibility is controlled by `demo_mode`.
pub fn seed_if_empty(conn: &Connection) -> Result<(), rusqlite::Error> {
    migrate_legacy_demo(conn)?;

    if get_setting(conn, "demo_mode")?.as_deref() == Some("off") {
        return Ok(());
    }

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))?;
    if count > 0 {
        if get_setting(conn, "demo_seeded")?.is_none() {
            set_setting(conn, "demo_seeded", "1")?;
        }
        return Ok(());
    }

    log::info!("Seeding demo library");
    let now = chrono::Utc::now();
    for (i, g) in SEED.iter().enumerate() {
        let created = (now - chrono::Duration::days(120 - i as i64)).to_rfc3339();
        let last_played = (now - chrono::Duration::days(i as i64 % 14)).to_rfc3339();
        let cover = placeholder_cover(g.title);
        conn.execute(
            "INSERT INTO games (title, description, executable_path, cover_path, developer, publisher,
                genre, release_year, platform, source, is_installed, is_favorite, is_hidden,
                created_at, updated_at, last_played_at, total_playtime_seconds, launch_count)
             VALUES (?1, ?2, NULL, ?3, ?4, ?4, ?5, ?6, 'PC', 'Demo', 1, ?7, 0,
                ?8, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                g.title,
                g.description,
                cover,
                g.developer,
                g.genre,
                g.year,
                g.favorite as i64,
                created,
                last_played,
                g.hours * 3600,
                g.sessions
            ],
        )?;
        let game_id = conn.last_insert_rowid();

        if let Some((primary, glow, accent)) = g.theme {
            conn.execute(
                "INSERT INTO game_themes (game_id, primary_color, secondary_color, accent_color,
                    background_color, glow_color, font_family)
                 VALUES (?1, ?2, ?3, ?4, '#0a0c1c', ?5, 'Inter')",
                rusqlite::params![game_id, primary, accent, accent, glow],
            )?;
        }

        for (title, desc, rarity) in GAME_ACHIEVEMENTS {
            let unlocked = i < 3;
            let unlocked_at = unlocked.then(|| last_played.clone());
            conn.execute(
                "INSERT INTO achievements (game_id, title, description, rarity, unlocked, unlocked_at,
                    progress_current, progress_target, scope)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'game')",
                rusqlite::params![
                    game_id,
                    title,
                    desc,
                    rarity,
                    unlocked as i64,
                    unlocked_at,
                    if *rarity == "Common" { 1 } else { (g.sessions / 2).max(1) },
                    if *rarity == "Common" { 1 } else { 10 }
                ],
            )?;
        }

        // Sessions history for charts.
        for s in 0..g.sessions.min(30) {
            let started = now
                - chrono::Duration::days((s as i64 * 3 + i as i64) % 90)
                + chrono::Duration::hours((s as i64 * 5) % 12);
            let duration = 1800 + ((s as i64 * 733 + i as i64 * 151) % 7200);
            conn.execute(
                "INSERT INTO game_sessions (game_id, started_at, ended_at, duration_seconds, pid)
                 VALUES (?1, ?2, ?3, ?4, NULL)",
                rusqlite::params![
                    game_id,
                    started.to_rfc3339(),
                    (started + chrono::Duration::seconds(duration)).to_rfc3339(),
                    duration
                ],
            )?;
        }
    }

    set_setting(conn, "demo_seeded", "1")?;
    if get_setting(conn, "demo_mode")?.is_none() {
        set_setting(conn, "demo_mode", "on")?;
    }
    Ok(())
}
