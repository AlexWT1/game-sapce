use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use crate::error::{AppError, AppResult};

/// Platform-specific game launching helpers.
pub fn spawn_game(executable: &Path) -> AppResult<Child> {
    if !executable.exists() {
        return Err(AppError::launch(format!(
            "Файл игры не найден: {}",
            executable.display()
        )));
    }

    #[cfg(target_os = "macos")]
    {
        if executable.extension().map(|e| e == "app").unwrap_or(false) {
            let child = Command::new("open")
                .arg("-a")
                .arg(executable)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
            return Ok(child);
        }
    }

    let mut cmd = Command::new(executable);
    if let Some(dir) = executable.parent() {
        cmd.current_dir(dir);
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const DETACHED_PROCESS: u32 = 0x00000008;
        cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS);
    }

    let child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| {
            log::error!("Failed to spawn {}: {e}", executable.display());
            AppError::launch(format!("Не удалось запустить процесс: {e}"))
        })?;

    log::info!(
        "Spawned {} with pid {}",
        executable.display(),
        child.id()
    );
    Ok(child)
}

/// Directories that commonly contain installed games on this platform.
pub fn default_game_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_default();

    #[cfg(target_os = "linux")]
    {
        dirs.push(home.join("Games"));
        dirs.push(home.join(".steam/steam/steamapps/common"));
        dirs.push(home.join(".local/share/Steam/steamapps/common"));
        dirs.push(home.join("Heroic/legendaryPrefix"));
        dirs.push(PathBuf::from("/usr/games"));
    }

    #[cfg(target_os = "macos")]
    {
        dirs.push(PathBuf::from("/Applications"));
        dirs.push(home.join("Applications"));
    }

    #[cfg(target_os = "windows")]
    {
        for drive in ["C", "D", "E"] {
            dirs.push(PathBuf::from(format!(
                r"{drive}:\Program Files (x86)\Steam\steamapps\common"
            )));
            dirs.push(PathBuf::from(format!(
                r"{drive}:\Program Files\Steam\steamapps\common"
            )));
            dirs.push(PathBuf::from(format!(
                r"{drive}:\Program Files (x86)\Epic Games"
            )));
            dirs.push(PathBuf::from(format!(r"{drive}:\GOG Games")));
            dirs.push(PathBuf::from(format!(r"{drive}:\Games")));
        }
    }

    dirs.retain(|d| d.exists());
    dirs
}

/// Validates that the path points to something launchable.
pub fn validate_executable(path: &str) -> AppResult<PathBuf> {
    if path.trim().is_empty() {
        return Err(AppError::validation("Executable path is empty"));
    }
    let p = PathBuf::from(path);
    if !p.exists() {
        return Err(AppError::launch(format!(
            "Файл игры не найден: {path}"
        )));
    }
    if p.is_dir()
        && !p
            .extension()
            .map(|e| e.eq_ignore_ascii_case("app"))
            .unwrap_or(false)
    {
        return Err(AppError::validation(
            "Указанный путь является папкой, а не исполняемым файлом",
        ));
    }
    Ok(p)
}
