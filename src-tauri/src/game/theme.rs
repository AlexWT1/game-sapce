use rusqlite::{Connection, OptionalExtension};

use crate::error::AppResult;
use crate::models::GameTheme;

const DEFAULT: (&str, &str, &str, &str, &str) = (
    "#7c5cff",
    "#2dd4bf",
    "#a78bfa",
    "#0a0c1c",
    "#7c5cff",
);

pub fn get(conn: &Connection, game_id: i64) -> AppResult<GameTheme> {
    let existing = conn
        .query_row(
            "SELECT game_id, primary_color, secondary_color, accent_color, background_color,
                    glow_color, font_family
             FROM game_themes WHERE game_id = ?1",
            [game_id],
            map_row,
        )
        .optional()?;

    if let Some(theme) = existing {
        return Ok(theme);
    }

    Ok(GameTheme {
        game_id,
        primary_color: DEFAULT.0.into(),
        secondary_color: DEFAULT.1.into(),
        accent_color: DEFAULT.2.into(),
        background_color: DEFAULT.3.into(),
        glow_color: DEFAULT.4.into(),
        font_family: "Inter".into(),
    })
}

pub fn upsert(conn: &Connection, theme: &GameTheme) -> AppResult<GameTheme> {
    conn.execute(
        "INSERT INTO game_themes (game_id, primary_color, secondary_color, accent_color,
            background_color, glow_color, font_family)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(game_id) DO UPDATE SET
            primary_color = excluded.primary_color,
            secondary_color = excluded.secondary_color,
            accent_color = excluded.accent_color,
            background_color = excluded.background_color,
            glow_color = excluded.glow_color,
            font_family = excluded.font_family",
        rusqlite::params![
            theme.game_id,
            theme.primary_color,
            theme.secondary_color,
            theme.accent_color,
            theme.background_color,
            theme.glow_color,
            theme.font_family
        ],
    )?;
    get(conn, theme.game_id)
}

/// Derives a default theme from the game genre/title when none is set.
pub fn auto_theme(conn: &Connection, game_id: i64, title: &str, genre: &str) -> AppResult<GameTheme> {
    let hue = title
        .bytes()
        .fold(5381u32, |h, b| h.wrapping_mul(33).wrapping_add(b as u32))
        % 360;
    let primary = format!("#{:02x}{:02x}{:02x}", (hue % 200 + 40), 60, 200);
    let accent = format!("#{:02x}{:02x}{:02x}", 160, (hue % 150 + 60), 220);
    let glow = match genre {
        "RPG" => "#c9a227".to_string(),
        "Экшен" => "#ef4444".to_string(),
        "Стратегия" => "#22c55e".to_string(),
        _ => primary.clone(),
    };
    let theme = GameTheme {
        game_id,
        primary_color: "#0f1119".into(),
        secondary_color: primary,
        accent_color: accent,
        background_color: "#0a0c1c".into(),
        glow_color: glow,
        font_family: "Inter".into(),
    };
    upsert(conn, &theme)
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<GameTheme> {
    Ok(GameTheme {
        game_id: row.get(0)?,
        primary_color: row.get(1)?,
        secondary_color: row.get(2)?,
        accent_color: row.get(3)?,
        background_color: row.get(4)?,
        glow_color: row.get(5)?,
        font_family: row.get(6)?,
    })
}
