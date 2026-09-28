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
