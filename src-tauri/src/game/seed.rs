use rusqlite::{Connection, OptionalExtension};

use crate::services::placeholder::placeholder_cover;

struct SeedGame {
    title: &'static str,
    description: &'static str,
    description_en: &'static str,
    developer: &'static str,
    genre: &'static str,
    genre_en: &'static str,
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
        description_en: "An epic open-world adventure full of captivating stories and unforgettable characters.",
        developer: "CD Projekt RED",
        genre: "RPG",
        genre_en: "RPG",
        year: 2015,
        hours: 248,
        sessions: 42,
        favorite: true,
        theme: Some(("#1a1c24", "#8b1e2d", "#c9a227")),
    },
    SeedGame {
        title: "Cyberpunk 2077",
        description: "Открытый мир Найт-Сити — мегаполиса будущего, где власть, роскошь и кибернетика.",
        description_en: "The open world of Night City — a megacity of the future where power, luxury, and cybernetics rule.",
        developer: "CD Projekt RED",
        genre: "RPG",
        genre_en: "RPG",
        year: 2020,
        hours: 96,
        sessions: 21,
        favorite: true,
        theme: Some(("#0d0f14", "#fcee0a", "#00f0ff")),
    },
    SeedGame {
        title: "Hogwarts Legacy",
        description: "Погрузись в мир магии XIX века и стань волшебником Хогвартса.",
        description_en: "Immerse yourself in the wizarding world of the 19th century and become a Hogwarts student.",
        developer: "Avalanche Software",
        genre: "Приключения",
        genre_en: "Adventure",
        year: 2023,
        hours: 64,
        sessions: 15,
        favorite: false,
        theme: Some(("#12100c", "#c9a227", "#6ea8fe")),
    },
    SeedGame {
        title: "Red Dead Redemption 2",
        description: "Эпическая история Артура Моргана и банды Ван дер Линде на Диком Западе.",
        description_en: "The epic story of Arthur Morgan and the Van der Linde gang in the Wild West.",
        developer: "Rockstar Games",
        genre: "Экшен",
        genre_en: "Action",
        year: 2018,
        hours: 132,
        sessions: 30,
        favorite: true,
        theme: Some(("#140f0c", "#b7410e", "#e0c097")),
    },
    SeedGame {
        title: "Elden Ring",
        description: "Тёмное фэнтези от FromSoftware в открытом мире Благословенных Земель.",
        description_en: "Dark fantasy by FromSoftware in the open world of the Lands Between.",
        developer: "FromSoftware",
        genre: "Экшен",
        genre_en: "Action",
        year: 2022,
        hours: 88,
        sessions: 27,
        favorite: false,
        theme: Some(("#0c0b09", "#c9a227", "#f4e4bc")),
    },
    SeedGame {
        title: "Baldur's Gate 3",
        description: "Тактическая RPG по миру Dungeons & Dragons с глубокими выборами.",
        description_en: "A tactical RPG set in the Dungeons & Dragons universe with deep choices.",
        developer: "Larian Studios",
        genre: "RPG",
        genre_en: "RPG",
        year: 2023,
        hours: 110,
        sessions: 24,
        favorite: false,
        theme: Some(("#100c14", "#7c3aed", "#f59e0b")),
    },
    SeedGame {
        title: "Starfield",
        description: "Космическое исследование от Bethesda — сотни планет и свобода выбора.",
        description_en: "Space exploration by Bethesda — hundreds of planets and freedom of choice.",
        developer: "Bethesda Game Studios",
        genre: "Приключения",
        genre_en: "Adventure",
        year: 2023,
        hours: 54,
        sessions: 12,
        favorite: false,
        theme: Some(("#0a0e18", "#38bdf8", "#94a3b8")),
    },
    SeedGame {
        title: "God of War",
        description: "Кратос и Атрей в путешествии по миру скандинавской мифологии.",
        description_en: "Kratos and Atreus journey through the world of Norse mythology.",
        developer: "Santa Monica Studio",
        genre: "Экшен",
        genre_en: "Action",
        year: 2018,
        hours: 42,
        sessions: 11,
        favorite: false,
        theme: Some(("#140d0d", "#b91c1c", "#f8fafc")),
    },
    SeedGame {
        title: "The Last of Us Part I",
        description: "История выживания Джоэл и Элли в мире, погубленном эпидемией.",
        description_en: "The survival story of Joel and Ellie in a world ravaged by a pandemic.",
        developer: "Naughty Dog",
        genre: "Приключения",
        genre_en: "Adventure",
        year: 2022,
        hours: 30,
        sessions: 9,
        favorite: false,
        theme: Some(("#0e1210", "#4d7c0f", "#d6d3d1")),
    },
    SeedGame {
        title: "Assassin's Creed Valhalla",
        description: "Стань эйнхерри и возглави викингов в набегах на Англию.",
        description_en: "Become an einherjar and lead the Vikings in raids on England.",
        developer: "Ubisoft Montreal",
        genre: "Экшен",
        genre_en: "Action",
        year: 2020,
        hours: 74,
        sessions: 18,
        favorite: false,
        theme: Some(("#0b1214", "#0ea5e9", "#a3e635")),
    },
];

struct SeedAchievement {
    title_ru: &'static str,
    title_en: &'static str,
    desc_ru: &'static str,
    desc_en: &'static str,
    rarity: &'static str,
}

const GAME_ACHIEVEMENTS: &[SeedAchievement] = &[
    SeedAchievement {
        title_ru: "Начало пути",
        title_en: "First Steps",
        desc_ru: "Заверши первую сессию",
        desc_en: "Complete your first session",
        rarity: "Common",
    },
    SeedAchievement {
        title_ru: "Исследователь",
        title_en: "Explorer",
        desc_ru: "Проведи 20 часов в игре",
        desc_en: "Spend 20 hours in the game",
        rarity: "Rare",
    },
    SeedAchievement {
        title_ru: "Преданный",
        title_en: "Devoted",
        desc_ru: "Сыграй 10 сессий",
        desc_en: "Play 10 sessions",
        rarity: "Epic",
    },
    SeedAchievement {
        title_ru: "Марафонец",
        title_en: "Marathon Runner",
        desc_ru: "Одна сессия дольше 3 часов",
        desc_en: "A single session over 3 hours",
        rarity: "Legendary",
    },
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
    let locale_en = get_setting(conn, "locale")?.as_deref() == Some("en")
        || get_setting(conn, "language")?.as_deref() == Some("en");
    let now = chrono::Utc::now();
    for (i, g) in SEED.iter().enumerate() {
        let created = (now - chrono::Duration::days(120 - i as i64)).to_rfc3339();
        let last_played = (now - chrono::Duration::days(i as i64 % 14)).to_rfc3339();
        let cover = placeholder_cover(g.title);
        let desc = if locale_en { g.description_en } else { g.description };
        let genre = if locale_en { g.genre_en } else { g.genre };
        conn.execute(
            "INSERT INTO games (title, description, executable_path, cover_path, developer, publisher,
                genre, release_year, platform, source, is_installed, is_favorite, is_hidden,
                created_at, updated_at, last_played_at, total_playtime_seconds, launch_count)
             VALUES (?1, ?2, NULL, ?3, ?4, ?4, ?5, ?6, 'PC', 'Demo', 1, ?7, 0,
                ?8, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                g.title,
                desc,
                cover,
                g.developer,
                genre,
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

        for ach in GAME_ACHIEVEMENTS {
            let ach_title = if locale_en { ach.title_en } else { ach.title_ru };
            let ach_desc = if locale_en { ach.desc_en } else { ach.desc_ru };
            let unlocked = i < 3;
            let unlocked_at = unlocked.then(|| last_played.clone());
            conn.execute(
                "INSERT INTO achievements (game_id, title, description, rarity, unlocked, unlocked_at,
                    progress_current, progress_target, scope)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'game')",
                rusqlite::params![
                    game_id,
                    ach_title,
                    ach_desc,
                    ach.rarity,
                    unlocked as i64,
                    unlocked_at,
                    if ach.rarity == "Common" { 1 } else { (g.sessions / 2).max(1) },
                    if ach.rarity == "Common" { 1 } else { 10 }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::migrations;

    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().expect("memory db");
        conn.pragma_update(None, "foreign_keys", "ON").expect("pragma");
        migrations::run(&conn).expect("migrations");
        conn
    }

    #[test]
    fn seed_respects_locale_english() {
        let conn = memory_db();
        set_setting(&conn, "locale", "en").expect("set locale");
        seed_if_empty(&conn).expect("seed");

        let witcher_desc: String = conn
            .query_row(
                "SELECT description FROM games WHERE title = 'The Witcher 3: Wild Hunt'",
                [],
                |r| r.get(0),
            )
            .expect("witcher desc");
        assert!(witcher_desc.starts_with("An epic open-world adventure"));

        let hogwarts_genre: String = conn
            .query_row(
                "SELECT genre FROM games WHERE title = 'Hogwarts Legacy'",
                [],
                |r| r.get(0),
            )
            .expect("hogwarts genre");
        assert_eq!(hogwarts_genre, "Adventure");

        let first_ach: String = conn
            .query_row(
                "SELECT title FROM achievements WHERE title = 'First Steps'",
                [],
                |r| r.get(0),
            )
            .expect("ach title");
        assert_eq!(first_ach, "First Steps");
    }

    #[test]
    fn seed_respects_locale_russian_by_default() {
        let conn = memory_db();
        seed_if_empty(&conn).expect("seed");

        let witcher_desc: String = conn
            .query_row(
                "SELECT description FROM games WHERE title = 'The Witcher 3: Wild Hunt'",
                [],
                |r| r.get(0),
            )
            .expect("witcher desc");
        assert!(witcher_desc.starts_with("Эпическое приключение"));

        let hogwarts_genre: String = conn
            .query_row(
                "SELECT genre FROM games WHERE title = 'Hogwarts Legacy'",
                [],
                |r| r.get(0),
            )
            .expect("hogwarts genre");
        assert_eq!(hogwarts_genre, "Приключения");
    }
}
