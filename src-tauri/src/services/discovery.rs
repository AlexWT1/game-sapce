use std::path::Path;

use walkdir::WalkDir;

use crate::models::DiscoveredGame;

/// Scans directories for executables that could be games.
pub fn discover(dirs: &[String]) -> Vec<DiscoveredGame> {
    let mut results = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for dir in dirs {
        let root = Path::new(dir);
        if !root.exists() {
            continue;
        }
        for entry in WalkDir::new(root)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() || is_macos_app(e.path()))
        {
            let path = entry.path();
            if !is_candidate(path) {
                continue;
            }
            let path_str = path.to_string_lossy().to_string();
            if !seen.insert(path_str.clone()) {
                continue;
            }
            let title = guess_title(path);
            if title.is_empty() {
                continue;
            }
            results.push(DiscoveredGame {
                title,
                path: path_str,
                install_dir: path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                platform: platform_label(),
            });
            if results.len() >= 200 {
                return results;
            }
        }
    }
    results
}

fn platform_label() -> String {
    if cfg!(target_os = "windows") {
        "Windows".into()
    } else if cfg!(target_os = "macos") {
        "macOS".into()
    } else {
        "Linux".into()
    }
}

fn is_macos_app(path: &Path) -> bool {
    path.is_dir() && path.extension().map(|e| e == "app").unwrap_or(false)
}

fn is_candidate(path: &Path) -> bool {
    if path.is_dir() {
        return is_macos_app(path);
    }
    if path.components().any(|c| {
        let s = c.as_os_str().to_string_lossy().to_lowercase();
        s.starts_with('.') || s == "unins000.exe" || s == "uninstall"
    }) {
        return false;
    }
    if cfg!(target_os = "windows") {
        let Some(ext) = path.extension().map(|e| e.to_string_lossy().to_lowercase()) else {
            return false;
        };
        return matches!(ext.as_str(), "exe" | "bat");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = path.metadata() {
            let mode = meta.permissions().mode();
            let executable = mode & 0o111 != 0;
            if !executable {
                return false;
            }
            // Prefer files that look like ELF binaries.
            if let Ok(header) = std::fs::read(path) {
                if header.len() >= 4 && &header[0..4] == b"\x7fELF" {
                    return true;
                }
            }
            // Accept scripts with shebang.
            if let Ok(text) = std::fs::read(path) {
                if text.starts_with(b"#!") {
                    return true;
                }
            }
            return false;
        }
    }
    false
}

fn guess_title(path: &Path) -> String {
    if is_macos_app(path) {
        return path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
    }
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let cleaned = stem
        .replace(['-', '_'], " ")
        .split_whitespace()
        .filter(|w| {
            let lw = w.to_lowercase();
            !matches!(
                lw.as_str(),
                "x64" | "x86" | "win64" | "win32" | "linux" | "setup" | "launcher" | "bin"
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    cleaned
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
