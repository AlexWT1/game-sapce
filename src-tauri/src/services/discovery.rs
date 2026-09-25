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

/// Directory names that never contain a game executable (redists, engines, launchers' helpers...).
const SKIP_DIR_EXACT: &[&str] = &[
    "setup",
    "install",
    "installer",
    "installers",
    "engine",
    "uninstall",
    "uninstaller",
    "prerequisites",
    "node_modules",
    "__pycache__",
    "system32",
    "syswow64",
    "windowsapps",
    "cache",
    "logs",
    "temp",
    "tmp",
];

/// Directory names that mark support content wherever they appear in the path.
const SKIP_DIR_SUBSTR: &[&str] = &[
    "redist",
    "prereq",
    "directx",
    "vcredist",
    "vc_redist",
    "easyanticheat",
    "anticheat",
    "battleye",
    "crashhandler",
    "unins",
    "uninstall",
    "dotnet",
];

/// Executable names that are installers, redistributables or plain applications.
const JUNK_STEMS: &[&str] = &[
    "setup",
    "install",
    "uninstall",
    "unins000",
    "unins001",
    "unins002",
    "isuninstall",
    "iscc",
    "dxsetup",
    "dxwebsetup",
    "vcredist",
    "vcredist_x64",
    "vcredist_x86",
    "vc_redist",
    "vc_redist_x64",
    "vc_redist_x86",
    "dotnetfx",
    "ndp45",
    "ndp46",
    "ndp47",
    "ndp48",
    "unitycrashhandler",
    "unitycrashhandler32",
    "unitycrashhandler64",
    "crashreportclient",
    "crashreportclienteditor",
    "easyanticheatsetup",
    "eacsetup",
    "steam",
    "epicgameslauncher",
    "origin",
    "uplay",
    "ubisoftconnect",
    "battlenet",
    "goggalaxy",
    "webhelper",
    "epicwebhelper",
    "galaxyclienthelper",
];

const JUNK_SUBSTR: &[&str] = &[
    "redist",
    "vcredist",
    "vc_redist",
    "vcruntime",
    "directx",
    "unins",
    "uninstall",
    "prereq",
    "webhelper",
    "crashreport",
    "crashhandler",
    "crashpad",
    "anticheat",
    "dotnetfx",
];

fn has_skipped_dir(path: &Path) -> bool {
    path.components().any(|c| {
        let mut s = c.as_os_str().to_string_lossy().to_lowercase();
        if s.starts_with('.') {
            return true;
        }
        if let Some(stem) = s.strip_suffix(".exe") {
            s = stem.to_string();
        }
        SKIP_DIR_EXACT.contains(&s.as_str())
            || SKIP_DIR_SUBSTR.iter().any(|d| s.contains(d))
    })
}

fn is_junk_executable(path: &Path) -> bool {
    let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_lowercase()) else {
        return false;
    };
    let stem = name
        .rsplit_once('.')
        .map(|(s, _)| s.to_string())
        .unwrap_or_else(|| name.clone());
    JUNK_STEMS.contains(&stem.as_str()) || JUNK_SUBSTR.iter().any(|j| stem.contains(j))
}

fn is_candidate(path: &Path) -> bool {
    if path.is_dir() {
        return is_macos_app(path);
    }
    if has_skipped_dir(path) {
        return false;
    }
    if is_junk_executable(path) {
        return false;
    }
    if cfg!(target_os = "windows") {
        let Some(ext) = path.extension().map(|e| e.to_string_lossy().to_lowercase()) else {
            return false;
        };
        return ext == "exe";
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
                "x64"
                    | "x86"
                    | "win64"
                    | "win32"
                    | "linux"
                    | "setup"
                    | "launcher"
                    | "bin"
                    | "shipping"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_junk_executables() {
        let junk = [
            "/g/Game/Setup.exe",
            "/g/Game/unins000.exe",
            "/g/Game/Uninstall.exe",
            "/g/Game/UnityCrashHandler64.exe",
            "/g/Game/CrashReportClient.exe",
            "/g/Game/_CommonRedist/vc_redist.x64.exe",
            "/g/Game/VC_redist.x86.exe",
            "/g/Game/DirectX_Jun2010_Redist.exe",
            "/g/Game/EasyAntiCheat_Setup.exe",
            "/g/Game/ue4prereq-setup_x64.exe",
            "/g/Game/dxwebsetup.exe",
        ];
        for j in junk {
            assert!(is_junk_executable(Path::new(j)), "should skip {j}");
        }

        let real = [
            "/g/Game/Celeste.exe",
            "/g/Game/HollowKnight.exe",
            "/g/Game/Binaries/Win64/MyGame-Win64-Shipping.exe",
            "/g/Game/Portal2/bin/x86/portal2.exe",
        ];
        for r in real {
            assert!(!is_junk_executable(Path::new(r)), "should keep {r}");
        }
    }

    #[test]
    fn skips_support_directories() {
        let skipped = [
            "/g/Game/_CommonRedist/vcredist_x64.exe",
            "/g/Game/Redist/setup.exe",
            "/g/Game/Engine/Binaries/Win64/UE4Editor.exe",
            "/g/Game/EasyAntiCheat/EasyAntiCheat.exe",
            "/g/Game/DirectX/dxsetup.exe",
            "/g/.cache/something.exe",
            "/g/MyGame/node_modules/tool.exe",
        ];
        for s in skipped {
            assert!(has_skipped_dir(Path::new(s)), "should skip {s}");
        }

        let kept = [
            "/g/Game/bin/game.exe",
            "/g/MyGame/MyGame.exe",
            "/g/Hollow Knight/hk.exe",
        ];
        for k in kept {
            assert!(!has_skipped_dir(Path::new(k)), "should keep {k}");
        }
    }

    #[test]
    fn titles_are_cleaned_up() {
        assert_eq!(
            guess_title(Path::new("/g/GameName-Win64-Shipping.exe")),
            "Gamename"
        );
        assert_eq!(guess_title(Path::new("/g/my_cool_game_x64.exe")), "My Cool Game");
    }
}
