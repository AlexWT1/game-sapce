use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::commands::settings::read_setting;
use crate::error::{AppError, AppResult};
use crate::models::UpdateCheck;
use crate::services::updater;
use crate::AppState;

/// Reads the optional GitHub PAT stored in settings (`github_token`).
/// Empty/missing token means anonymous access (public repos only).
fn github_token(state: &AppState) -> Option<String> {
    state
        .db
        .with(|conn| Ok(read_setting(conn, "github_token")?))
        .ok()
        .flatten()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
}

/// Compares the current version against the latest GitHub release.
/// Works anonymously for public repos; pass a token in settings for private ones.
#[tauri::command]
pub async fn check_for_updates(state: State<'_, AppState>) -> AppResult<UpdateCheck> {
    let token = github_token(&state);
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let release = updater::fetch_latest_release(token.as_deref()).await?;
    let asset = updater::pick_asset(&release);
    let asset_name = asset.map(|a| a.name.clone());
    let download_url = asset.map(|a| a.browser_download_url.clone());
    let download_size = asset.map(|a| a.size);
    let update_available = updater::is_newer(&release.tag_name, &current_version);
    Ok(UpdateCheck {
        update_available,
        current_version,
        latest_version: Some(release.tag_name.trim_start_matches('v').to_string()),
        release_notes: release.body.filter(|b| !b.trim().is_empty()),
        release_url: release.html_url,
        asset_name,
        download_url,
        download_size,
        published_at: release.published_at,
    })
}

/// Starts a background download of the latest release asset.
/// Emits `update-download-progress` / `update-download-done` /
/// `update-download-cancelled` / `update-download-error` events.
#[tauri::command]
pub async fn download_update(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    if state
        .update_downloading
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_err()
    {
        return Err(AppError::Conflict("Загрузка обновления уже идёт".into()));
    }
    state.update_cancel.store(false, Ordering::Relaxed);

    let app2 = app.clone();
    let downloading = state.update_downloading.clone();
    let cancel = state.update_cancel.clone();
    let token = github_token(&state);

    tauri::async_runtime::spawn(async move {
        let result = run_download(&app2, token, cancel).await;
        downloading.store(false, Ordering::Relaxed);
        if let Err(e) = result {
            let msg = e.to_string();
            if !msg.contains("отменена") {
                log::error!("Update download failed: {msg}");
                let _ = app2.emit("update-download-error", msg);
            }
        }
    });
    Ok(())
}

async fn run_download(
    app: &AppHandle,
    token: Option<String>,
    cancel: Arc<AtomicBool>,
) -> AppResult<std::path::PathBuf> {
    let release = updater::fetch_latest_release(token.as_deref()).await?;
    let asset = updater::pick_asset(&release).ok_or_else(|| {
        AppError::not_found("В последнем релизе нет файла для этой платформы")
    })?;
    // For private repos `browser_download_url` requires auth; the assets API
    // endpoint streams raw bytes and works with (and without) a token.
    let has_token = token.as_deref().is_some_and(|t| !t.trim().is_empty());
    let url = if has_token {
        updater::asset_download_url(asset.id)
    } else {
        asset.browser_download_url.clone()
    };
    let target = updater::download_target_path(app, &asset.name)?;
    let path = updater::download_with_progress(
        app,
        &url,
        token.as_deref(),
        asset.size,
        target,
        cancel,
    )
    .await?;
    let _ = app.emit("update-download-done", path.to_string_lossy().to_string());
    Ok(path)
}

/// Requests cancellation of the running download.
#[tauri::command]
pub async fn cancel_update_download(state: State<'_, AppState>) -> AppResult<()> {
    state.update_cancel.store(true, Ordering::Relaxed);
    Ok(())
}

/// Installs the downloaded package and restarts the launcher.
#[tauri::command]
pub async fn install_update(app: AppHandle, path: String) -> AppResult<()> {
    let path = std::path::PathBuf::from(path);
    updater::install_and_restart(&app, &path)
}
