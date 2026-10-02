use tauri::{AppHandle, Emitter, State};

use crate::error::AppResult;
use crate::game::achievements;
use crate::models::Achievement;
use crate::AppState;

pub const SELECT: &str = "SELECT a.id, a.game_id, g.title, a.title, a.description, a.icon_path,
    a.rarity, a.unlocked, a.unlocked_at, a.progress_current, a.progress_target, a.scope, a.key
    FROM achievements a LEFT JOIN games g ON g.id = a.game_id";

#[tauri::command]
pub fn get_achievements(
    app: AppHandle,
    state: State<'_, AppState>,
    game_id: Option<i64>,
) -> AppResult<Vec<Achievement>> {
    achievements::evaluate(&app);
    state.db.with(|conn| {
        let demo_off = super::settings::demo_disabled(conn)?;
        let demo_filter = if demo_off {
            " AND (a.game_id IS NULL OR g.source != 'Demo')"
        } else {
            ""
        };
        let sql = match game_id {
            Some(_) => format!(
                "{SELECT} WHERE a.game_id = ?1{demo_filter} ORDER BY a.unlocked DESC, a.id"
            ),
            None => format!("{SELECT} WHERE 1=1{demo_filter} ORDER BY a.unlocked DESC, a.scope, a.id"),
        };
        let mut stmt = conn.prepare(&sql)?;
        let rows = match game_id {
            Some(id) => stmt.query_map([id], achievements::map_achievement)?,
            None => stmt.query_map([], achievements::map_achievement)?,
        };
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    })
}

#[tauri::command]
pub fn unlock_achievement(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<Achievement> {
    let ach = state.db.with(|conn| achievements::unlock(conn, id))?;
    let _ = app.emit("achievement-unlocked", &ach);
    Ok(ach)
}
