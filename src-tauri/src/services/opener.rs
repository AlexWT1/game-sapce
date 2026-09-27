//! Opens external links with the OS default handler.

use crate::error::{AppError, AppResult};

/// Opens a url (including custom protocols such as `steam://`).
pub fn open_url(url: &str) -> AppResult<()> {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| AppError::Other(format!("Не удалось открыть ссылку: {e}")))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| AppError::Other(format!("Не удалось открыть ссылку: {e}")))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .spawn()
            .map_err(|e| AppError::Other(format!("Не удалось открыть ссылку: {e}")))?;
    }
    log::info!("Opened external link {url}");
    Ok(())
}
