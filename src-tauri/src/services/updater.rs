use std::io::Write as _;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::error::{AppError, AppResult};

const GITHUB_REPO: &str = "AlexWT1/game-sapce";
const USER_AGENT: &str = concat!("GameSpace/", env!("CARGO_PKG_VERSION"));

/// Progress payload for the `update-download-progress` event.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub percent: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GhAsset {
    pub id: u64,
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GhRelease {
    pub tag_name: String,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub html_url: Option<String>,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub assets: Vec<GhAsset>,
}

fn api_client() -> AppResult<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Other(format!("HTTP client: {e}")))
}

/// Compares `a` and `b` as dotted numeric versions (`v0.2.1` vs `0.1.3`).
/// Returns `true` when `a > b`.
pub fn is_newer(a: &str, b: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> {
        s.trim()
            .trim_start_matches('v')
            .trim_start_matches('V')
            .split(['.', '-', '+'])
            .map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>())
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    let (a, b) = (parse(a), parse(b));
    let len = a.len().max(b.len());
    for i in 0..len {
        let (x, y) = (a.get(i).copied().unwrap_or(0), b.get(i).copied().unwrap_or(0));
        if x != y {
            return x > y;
        }
    }
    false
}

/// Picks the release asset matching the current platform.
pub fn pick_asset(release: &GhRelease) -> Option<&GhAsset> {
    let mut names: Vec<&GhAsset> = release.assets.iter().collect();
    if cfg!(target_os = "windows") {
        names.retain(|a| a.name.to_ascii_lowercase().ends_with(".msi"));
        // Prefer x64 builds.
        names.sort_by_key(|a| !a.name.to_ascii_lowercase().contains("x64"));
    } else if cfg!(target_os = "macos") {
        names.retain(|a| a.name.to_ascii_lowercase().ends_with(".dmg"));
    } else {
        // Linux: AppImage if we run as AppImage, otherwise .deb.
        let appimage = std::env::var_os("APPIMAGE").is_some();
        names.retain(|a| {
            let n = a.name.to_ascii_lowercase();
            n.ends_with(".appimage") || n.ends_with(".deb")
        });
        names.sort_by_key(|a| {
            let is_appimage = a.name.to_ascii_lowercase().ends_with(".appimage");
            is_appimage != appimage
        });
    }
    names.into_iter().next()
}

/// Fetches the latest published release from GitHub.
///
/// `token` is an optional GitHub PAT: without it the repo must be public,
/// with it private-repo releases (and assets) are accessible too.
pub async fn fetch_latest_release(token: Option<&str>) -> AppResult<GhRelease> {
    let url = format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest");
    let request = request_set_auth(
        api_client()?.get(&url).header("Accept", "application/vnd.github+json"),
        token,
    );
    let release: GhRelease = request
        .send()
        .await
        .map_err(|e| AppError::Other(format!("GitHub: {e}")))?
        .error_for_status()
        .map_err(github_friendly_error)?
        .json()
        .await
        .map_err(|e| AppError::Other(format!("GitHub: {e}")))?;
    Ok(release)
}

/// Applies a `Authorization: Bearer <token>` header when a token is present.
fn request_set_auth(
    builder: reqwest::RequestBuilder,
    token: Option<&str>,
) -> reqwest::RequestBuilder {
    match token.map(str::trim).filter(|t| !t.is_empty()) {
        Some(token) => builder.bearer_auth(token),
        None => builder,
    }
}

/// 404 from GitHub means "no access / no release" — for private repos the
/// user-facing hint is to provide an access token in the settings.
fn github_friendly_error(e: reqwest::Error) -> AppError {
    if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
        AppError::Other(
            "GitHub: релизы недоступны (404). Для приватного репозитория укажите токен доступа в настройках".into(),
        )
    } else {
        AppError::Other(format!("GitHub: {e}"))
    }
}

/// API endpoint streaming the raw asset bytes (works for private repos with a token).
pub fn asset_download_url(asset_id: u64) -> String {
    format!("https://api.github.com/repos/{GITHUB_REPO}/releases/assets/{asset_id}")
}

/// Target file path for the downloaded installer (inside the app data dir).
pub fn download_target_path(app: &AppHandle, asset_name: &str) -> AppResult<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Other(e.to_string()))?
        .join("updates");
    std::fs::create_dir_all(&dir).map_err(AppError::Io)?;
    Ok(dir.join(asset_name))
}

/// Streams `url` to `target`, emitting `update-download-progress` events.
/// Returns the path of the finished file. Cancels when `cancel` flips to true.
/// `url` should be the assets API endpoint (`asset_download_url`) and
/// `token` the GitHub PAT when the repository is private.
pub async fn download_with_progress(
    app: &AppHandle,
    url: &str,
    token: Option<&str>,
    total: u64,
    target: PathBuf,
    cancel: Arc<AtomicBool>,
) -> AppResult<PathBuf> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Other(format!("HTTP client: {e}")))?;
    let request = request_set_auth(
        client.get(url).header("Accept", "application/octet-stream"),
        token,
    );
    let mut response = request
        .send()
        .await
        .map_err(|e| AppError::Other(format!("Скачивание: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::Other(format!("Скачивание: {e}")))?;

    let part = target.with_extension("part");
    let mut file = std::fs::File::create(&part).map_err(AppError::Io)?;
    let mut downloaded: u64 = 0;
    let mut last_emit = std::time::Instant::now();

    while let Some(chunk) = response.chunk().await.map_err(|e| AppError::Other(e.to_string()))? {
        if cancel.load(Ordering::Relaxed) {
            drop(file);
            let _ = std::fs::remove_file(&part);
            app.emit("update-download-cancelled", ())
                .map_err(|e| AppError::Other(e.to_string()))?;
            return Err(AppError::Other("Загрузка отменена".to_string()));
        }
        file.write_all(&chunk).map_err(AppError::Io)?;
        downloaded += chunk.len() as u64;
        if last_emit.elapsed() >= Duration::from_millis(100) {
            last_emit = std::time::Instant::now();
            let percent = if total > 0 { downloaded as f64 / total as f64 * 100.0 } else { 0.0 };
            let _ = app.emit(
                "update-download-progress",
                DownloadProgress { downloaded, total, percent },
            );
        }
    }
    file.flush().map_err(AppError::Io)?;
    drop(file);
    std::fs::rename(&part, &target).map_err(AppError::Io)?;
    let _ = app.emit(
        "update-download-progress",
        DownloadProgress { downloaded, total, percent: 100.0 },
    );
    Ok(target)
}

/// Installs the downloaded update and restarts the launcher.
///
/// - Windows: runs `msiexec /i <file> /passive /norestart`, waits for it to
///   finish in a detached helper, then relaunches the updated executable.
/// - Linux AppImage: atomically replaces the running AppImage and relaunches it.
/// - Other Linux packages: hands the file to the system installer (`xdg-open`).
pub fn install_and_restart(app: &AppHandle, path: &std::path::Path) -> AppResult<()> {
    if !path.exists() {
        return Err(AppError::not_found(format!("Файл не найден: {}", path.display())));
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        const DETACHED_PROCESS: u32 = 0x0000_0008;

        // msiexec never launches the app after an update, so chain the
        // relaunch in one detached cmd: wait for the installer to finish,
        // then start the updated executable. `&` (not `&&`) so the app also
        // comes back if the user cancels the UAC prompt — in that case the
        // previously installed version stays intact. The MSI only replaces
        // files once this process is gone, which app.exit(0) ensures.
        let exe = std::env::current_exe().map_err(AppError::Io)?;
        let script = format!(
            "msiexec /i \"{}\" /passive /norestart & start \"\" \"{}\"",
            path.display(),
            exe.display(),
        );
        std::process::Command::new("cmd")
            .raw_arg(format!("/C {script}"))
            .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| AppError::Other(format!("Не удалось запустить установку: {e}")))?;
        app.exit(0);
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        open_path_generic(path)?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        if let (Some(current), true) = (
            std::env::var_os("APPIMAGE").map(PathBuf::from),
            path.to_string_lossy().to_ascii_lowercase().ends_with(".appimage"),
        ) {
            let tmp = current.with_extension("appimage.new");
            std::fs::copy(path, &tmp).map_err(|e| AppError::Io(e))?;
            std::fs::rename(&tmp, &current).map_err(|e| AppError::Io(e))?;
            std::process::Command::new(&current)
                .spawn()
                .map_err(|e| AppError::Other(format!("Не удалось перезапустить: {e}")))?;
            app.exit(0);
            return Ok(());
        }
        open_path_generic(path)?;
        Ok(())
    }
}

/// Fallback for non-self-updatable formats: reveal the file in the file manager.
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn open_path_generic(path: &std::path::Path) -> AppResult<()> {
    let dir = path.parent().unwrap_or(path);
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(target_os = "linux")]
    let cmd = "xdg-open";
    std::process::Command::new(cmd)
        .arg(dir)
        .spawn()
        .map_err(|e| AppError::Other(format!("Не удалось открыть папку: {e}")))?;
    Ok(())
}


