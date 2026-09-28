//! Local Steam installation discovery: VDF/ACF parsing and library scanning.

use std::path::{Path, PathBuf};

/// A parsed VDF (Valve Data Format) value.
#[derive(Debug, Clone, PartialEq)]
pub enum VdfValue {
    Str(String),
    Map(Vec<(String, VdfValue)>),
}

impl VdfValue {
    pub fn get(&self, key: &str) -> Option<&VdfValue> {
        match self {
            VdfValue::Map(entries) => entries
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .map(|(_, v)| v),
            VdfValue::Str(_) => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            VdfValue::Str(s) => Some(s),
            VdfValue::Map(_) => None,
        }
    }

    pub fn as_map(&self) -> Option<&[(String, VdfValue)]> {
        match self {
            VdfValue::Map(entries) => Some(entries),
            VdfValue::Str(_) => None,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Str(String),
    Open,
    Close,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '{' => {
                tokens.push(Token::Open);
                i += 1;
            }
            '}' => {
                tokens.push(Token::Close);
                i += 1;
            }
            '"' => {
                i += 1;
                let mut s = String::new();
                let mut closed = false;
                while i < chars.len() {
                    let ch = chars[i];
                    if ch == '\\' && i + 1 < chars.len() {
                        let next = chars[i + 1];
                        if next == '"' || next == '\\' {
                            s.push(next);
                            i += 2;
                            continue;
                        }
                    }
                    if ch == '"' {
                        closed = true;
                        i += 1;
                        break;
                    }
                    s.push(ch);
                    i += 1;
                }
                if !closed {
                    return Err("unterminated string in VDF".into());
                }
                tokens.push(Token::Str(s));
            }
            c if c.is_whitespace() => i += 1,
            '/' if i + 1 < chars.len() && chars[i + 1] == '/' => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            _ => {
                let start = i;
                while i < chars.len()
                    && !chars[i].is_whitespace()
                    && chars[i] != '{'
                    && chars[i] != '}'
                    && chars[i] != '"'
                {
                    i += 1;
                }
                tokens.push(Token::Str(chars[start..i].iter().collect()));
            }
        }
    }
    Ok(tokens)
}

fn parse_object(tokens: &[Token], pos: &mut usize) -> Result<Vec<(String, VdfValue)>, String> {
    let mut out: Vec<(String, VdfValue)> = Vec::new();
    loop {
        match tokens.get(*pos) {
            Some(Token::Close) => {
                *pos += 1;
                return Ok(out);
            }
            Some(Token::Str(key)) => {
                let key = key.clone();
                *pos += 1;
                let value = match tokens.get(*pos) {
                    Some(Token::Open) => {
                        *pos += 1;
                        VdfValue::Map(parse_object(tokens, pos)?)
                    }
                    Some(Token::Str(v)) => {
                        *pos += 1;
                        VdfValue::Str(v.clone())
                    }
                    Some(Token::Close) => return Err(format!("missing value for key {key}")),
                    None => return Err("unexpected end of VDF".into()),
                };
                out.push((key, value));
            }
            Some(_) => return Err("unexpected token in VDF object".into()),
            None => return Err("unexpected end of VDF".into()),
        }
    }
}

/// Parses a VDF document and returns its root object.
pub fn parse_vdf(input: &str) -> Result<VdfValue, String> {
    let tokens = tokenize(input)?;
    let mut pos = 0;
    match tokens.get(pos) {
        Some(Token::Str(_)) => {
            // Named root: "RootName" { ... }
            pos += 1;
            if matches!(tokens.get(pos), Some(Token::Open)) {
                pos += 1;
                Ok(VdfValue::Map(parse_object(&tokens, &mut pos)?))
            } else {
                // A bare key/value pair at the top level.
                let key = match tokens.first() {
                    Some(Token::Str(k)) => k.clone(),
                    _ => return Err("empty VDF".into()),
                };
                let value = match tokens.get(pos) {
                    Some(Token::Str(v)) => VdfValue::Str(v.clone()),
                    _ => return Err(format!("missing value for key {key}")),
                };
                Ok(VdfValue::Map(vec![(key, value)]))
            }
        }
        Some(Token::Open) => {
            pos += 1;
            Ok(VdfValue::Map(parse_object(&tokens, &mut pos)?))
        }
        _ => Err("empty VDF".into()),
    }
}

/// An app found in a Steam library `appmanifest_*.acf` file that is actually installed.
#[derive(Debug, Clone, PartialEq)]
pub struct InstalledApp {
    pub appid: u32,
    pub name: String,
    pub install_path: PathBuf,
}

/// Locates the Steam installation root for the current platform.
pub fn find_steam_root() -> Option<PathBuf> {
    let candidates = platform_steam_roots();
    candidates.into_iter().find(|c| is_steam_root(c))
}

#[cfg(windows)]
fn platform_steam_roots() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(p) = registry_steam_root() {
        out.push(p);
    }
    for var in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Ok(base) = std::env::var(var) {
            out.push(PathBuf::from(base).join("Steam"));
        }
    }
    out
}

#[cfg(windows)]
fn registry_steam_root() -> Option<PathBuf> {
    use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
    use winreg::RegKey;

    const KEYS: [&str; 2] = [
        r"SOFTWARE\WOW6432Node\Valve\Steam",
        r"SOFTWARE\Valve\Steam",
    ];
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    for sub in KEYS {
        if let Ok(key) = hklm.open_subkey_with_flags(sub, KEY_READ) {
            if let Ok(value) = key.get_value::<String, _>("InstallPath") {
                if !value.trim().is_empty() {
                    return Some(PathBuf::from(value));
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn platform_steam_roots() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        let home = PathBuf::from(home);
        out.push(home.join(".steam/steam"));
        out.push(home.join(".steam/root"));
        out.push(home.join(".local/share/Steam"));
        out.push(home.join(".var/app/com.valvesoftware.Steam/data/Steam"));
    }
    out
}

#[cfg(target_os = "macos")]
fn platform_steam_roots() -> Vec<PathBuf> {
    let mut out = vec![PathBuf::from("/Applications/Steam.app/Contents/MacOS")];
    if let Ok(home) = std::env::var("HOME") {
        out.push(PathBuf::from(home).join("Library/Application Support/Steam"));
    }
    out
}

fn is_steam_root(path: &Path) -> bool {
    path.join("steamapps").is_dir() || path.join("steamapps/libraryfolders.vdf").is_file()
}

/// Returns every Steam library folder (root + extra libraries from `libraryfolders.vdf`).
pub fn library_paths(steam_root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = vec![steam_root.to_path_buf()];
    let candidates = [
        steam_root.join("steamapps").join("libraryfolders.vdf"),
        steam_root.join("Library").join("libraryfolders.vdf"),
    ];
    for file in candidates {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let Ok(root) = parse_vdf(&text) else {
            log::warn!("Failed to parse {}", file.display());
            continue;
        };
        let Some(entries) = root.as_map() else {
            continue;
        };
        for (key, value) in entries {
            let path = match value {
                // Modern format: "0" { "path" "..." }
                VdfValue::Map(fields) => fields
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("path"))
                    .and_then(|(_, v)| v.as_str()),
                // Legacy format: "1" "D:\\SteamLibrary"
                VdfValue::Str(s) if key.parse::<u32>().is_ok() => Some(s.as_str()),
                VdfValue::Str(_) => None,
            };
            if let Some(path) = path {
                if !path.trim().is_empty() {
                    out.push(PathBuf::from(path));
                }
            }
        }
    }

    let mut seen: Vec<PathBuf> = Vec::new();
    out.into_iter()
        .filter(|p| {
            let key = p.to_string_lossy().to_lowercase();
            if seen.contains(&PathBuf::from(&key)) {
                return false;
            }
            seen.push(PathBuf::from(key));
            true
        })
        .collect()
}

/// Scans all libraries for installed apps (`appmanifest_*.acf` + existing install dir).
pub fn installed_apps(steam_root: &Path) -> Vec<InstalledApp> {
    let mut out: Vec<InstalledApp> = Vec::new();
    for library in library_paths(steam_root) {
        let steamapps = library.join("steamapps");
        let Ok(entries) = std::fs::read_dir(&steamapps) else {
            continue;
        };
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if !file_name.starts_with("appmanifest_") || !file_name.ends_with(".acf") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            let Ok(root) = parse_vdf(&text) else {
                log::warn!("Failed to parse {}", entry.path().display());
                continue;
            };
            let appid = root
                .get("appid")
                .and_then(|v| v.as_str())
                .and_then(|v| v.parse::<u32>().ok());
            let name = root
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let installdir = root.get("installdir").and_then(|v| v.as_str());
            let (Some(appid), Some(installdir)) = (appid, installdir) else {
                continue;
            };
            let install_path = steamapps.join("common").join(installdir);
            if !install_path.is_dir() {
                continue;
            }
            out.push(InstalledApp {
                appid,
                name,
                install_path,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn parses_modern_libraryfolders() {
        let text = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"C:\\Program Files (x86)\\Steam"
		"contentid"		"111"
		"apps"
		{
			"228980"		"463658718"
		}
	}
	"1"
	{
		"path"		"D:\\Games\\SteamLibrary"
	}
}
"#;
        let root = parse_vdf(text).expect("parse");
        let libs = {
            let mut paths = vec![PathBuf::from("/root")];
            if let Some(entries) = root.as_map() {
                for (_, value) in entries {
                    if let VdfValue::Map(fields) = value {
                        if let Some(p) = fields
                            .iter()
                            .find(|(k, _)| k == "path")
                            .and_then(|(_, v)| v.as_str())
                        {
                            paths.push(PathBuf::from(p));
                        }
                    }
                }
            }
            paths
        };
        assert_eq!(libs.len(), 3);
        assert_eq!(
            libs[1],
            PathBuf::from(r"C:\Program Files (x86)\Steam")
        );
        assert_eq!(libs[2], PathBuf::from(r"D:\Games\SteamLibrary"));
    }

    #[test]
    fn parses_legacy_libraryfolders() {
        let text = r#"
"LibraryFolders"
{
	"TimeNextStatsUpdate"		"0"
	"1"		"D:\\SteamLibrary"
	"ContentStatsID"		"-1234"
}
"#;
        let root = parse_vdf(text).expect("parse");
        let entries = root.as_map().expect("map");
        let legacy: Vec<&str> = entries
            .iter()
            .filter_map(|(k, v)| match v {
                VdfValue::Str(s) if k.parse::<u32>().is_ok() => Some(s.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(legacy, vec![r"D:\SteamLibrary"]);
    }

    #[test]
    fn parses_appmanifest() {
        let text = r#"
"AppState"
{
	"appid"		"730"
	"universe"		"1"
	"name"		"Counter-Strike 2"
	"StateFlags"		"4"
	"installdir"		"Counter-Strike Global Offensive"
	"LastUpdated"		"1700000000"
}
"#;
        let root = parse_vdf(text).expect("parse");
        assert_eq!(root.get("appid").and_then(|v| v.as_str()), Some("730"));
        assert_eq!(
            root.get("name").and_then(|v| v.as_str()),
            Some("Counter-Strike 2")
        );
        assert_eq!(
            root.get("installdir").and_then(|v| v.as_str()),
            Some("Counter-Strike Global Offensive")
        );
    }

    #[test]
    fn keeps_single_backslashes_in_strings() {
        let root = parse_vdf("\"a\"\n{\n\"p\" \"D:\\Games\"\n}").expect("parse");
        assert_eq!(root.get("p").and_then(|v| v.as_str()), Some(r"D:\Games"));
    }

    #[test]
    fn reports_unterminated_strings() {
        assert!(parse_vdf("\"broken").is_err());
    }

    static SEQ: AtomicU32 = AtomicU32::new(0);

    #[test]
    fn detects_installed_apps() {
        let id = SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!("gs-steam-test-{}-{id}", std::process::id()));
        let steamapps = root.join("steamapps");
        let installed_dir = steamapps.join("common").join("Test Game");
        std::fs::create_dir_all(&installed_dir).expect("create install dir");

        std::fs::write(
            steamapps.join("appmanifest_100.acf"),
            r#""AppState"
{
	"appid"		"100"
	"name"		"Test Game"
	"installdir"		"Test Game"
}
"#,
        )
        .expect("write manifest");
        std::fs::write(
            steamapps.join("appmanifest_200.acf"),
            r#""AppState"
{
	"appid"		"200"
	"name"		"Missing Game"
	"installdir"		"Missing Game"
}
"#,
        )
        .expect("write manifest");
        let escaped = root.to_string_lossy().replace('\\', "\\\\");
        std::fs::write(
            steamapps.join("libraryfolders.vdf"),
            format!(
                "\"libraryfolders\"\n{{\n\"0\"\n{{\n\"path\"\t\t\"{escaped}\"\n}}\n}}\n"
            ),
        )
        .expect("write vdf");

        assert!(is_steam_root(&root));
        let apps = installed_apps(&root);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].appid, 100);
        assert_eq!(apps[0].name, "Test Game");
        assert_eq!(apps[0].install_path, installed_dir);

        // The root is listed once even when libraryfolders.vdf points at it.
        let libs = library_paths(&root);
        assert_eq!(libs, vec![root.clone()]);

        let _ = std::fs::remove_dir_all(&root);
    }
}
