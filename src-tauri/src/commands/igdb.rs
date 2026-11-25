use chrono::Datelike;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::models::IgdbGameMeta;
use crate::services::igdb::{self, IgdbClient};
use crate::AppState;

fn to_meta(game: igdb::IgdbGame) -> IgdbGameMeta {
    let developer = game
        .involved_companies
        .iter()
        .filter(|ic| ic.developer)
        .find_map(|ic| ic.company.as_ref().map(|c| c.name.clone()))
        .or_else(|| {
            game.involved_companies
                .iter()
                .find_map(|ic| ic.company.as_ref().map(|c| c.name.clone()))
        })
        .unwrap_or_default();

    let publisher = game
        .involved_companies
        .iter()
        .filter(|ic| ic.publisher)
        .find_map(|ic| ic.company.as_ref().map(|c| c.name.clone()))
        .unwrap_or_default();

    let cover_url = game
        .cover
        .as_ref()
        .and_then(|c| igdb::cover_big_url(&c.image_id));

    let release_year = game
        .first_release_date
        .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0).map(|dt| dt.year()))
        .map(|y| y as i32);

    let genres = game
        .genres
        .iter()
        .map(|g| g.name.clone())
        .filter(|n| !n.is_empty())
        .collect();

    IgdbGameMeta {
        igdb_id: game.id,
        name: game.name,
        summary: game.summary,
        release_year,
        developer,
        publisher,
        genres,
        cover_url,
    }
}

/// Searches IGDB by title and returns the best match, if IGDB is configured.
pub(crate) async fn search_first(state: &State<'_, AppState>, query: &str) -> Option<IgdbGameMeta> {
    let query = query.trim();
    if query.is_empty() {
        return None;
    }
    let client = IgdbClient::from_settings(state).ok()?;
    let games = client.query(state, igdb::search_body(query, 1)).await.ok()?;
    games.into_iter().next().map(to_meta)
}

/// Searches IGDB by title and returns ready-to-use metadata for the form.
#[tauri::command]
pub async fn igdb_search_games(
    state: State<'_, AppState>,
    query: String,
    limit: Option<u32>,
) -> AppResult<Vec<IgdbGameMeta>> {
    let query = query.trim().to_string();
    if query.is_empty() {
        return Err(AppError::validation("Укажите название игры"));
    }
    let limit = limit.unwrap_or(8).clamp(1, 20) as usize;
    let client = IgdbClient::from_settings(&state)?;
    let games = client
        .query(&state, igdb::search_body(&query, limit))
        .await?;
    Ok(games.into_iter().map(to_meta).collect())
}

/// Fetches full metadata for one IGDB game id.
#[tauri::command]
pub async fn igdb_get_game(
    state: State<'_, AppState>,
    game_id: i64,
) -> AppResult<Option<IgdbGameMeta>> {
    let client = IgdbClient::from_settings(&state)?;
    let mut games = client.query(&state, igdb::id_body(game_id)).await?;
    Ok(games.pop().map(to_meta))
}
