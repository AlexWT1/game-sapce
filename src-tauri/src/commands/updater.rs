use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::error::{AppError, AppResult};
use crate::models::UpdateCheck;
use crate::services::updater;
use crate::AppState;

/// Compares the current version against the latest GitHub release.
#[tauri::command]
pub async fn check_for_updates() -> AppResult<UpdateCheck> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let release = updater::fetch_latest_release().await?;
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

    tauri::async_runtime::spawn(async move {
        let result = run_download(&app2, cancel).await;
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
    cancel: Arc<AtomicBool>,
) -> AppResult<std::path::PathBuf> {
    let release = updater::fetch_latest_release().await?;
    let asset = updater::pick_asset(&release).ok_or_else(|| {
        AppError::not_found("В последнем релизе нет файла для этой платформы")
    })?;
    let target = updater::download_target_path(app, &asset.name)?;
    let path = updater::download_with_progress(
        app,
        &asset.browser_download_url,
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
