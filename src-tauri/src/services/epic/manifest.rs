use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct InstalledEpicGame {
    pub app_name: String,
    pub display_name: String,
    pub catalog_item_id: String,
    pub namespace: String,
    pub install_location: String,
    pub executable_path: Option<String>,
    pub description: Option<String>,
    pub developer: Option<String>,
    pub cover_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawManifestItem {
    #[serde(default)]
    app_name: String,
    #[serde(default)]
    display_name: String,
    #[serde(default)]
    catalog_item_id: String,
    #[serde(default)]
    catalog_namespace: String,
    #[serde(default)]
    install_location: String,
    #[serde(default)]
    launch_executable: String,
    #[serde(default)]
    main_game_app_name: String,
    #[serde(default, rename = "bIsIncompleteInstall")]
    is_incomplete_install: bool,
    #[serde(default)]
    app_categories: Vec<String>,
}

/// Locates the Epic Games Launcher v1 Manifests directory.
pub fn find_manifests_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        for app_data in read_registry_app_data_paths() {
            let manifests = app_data.join("Manifests");
            if manifests.is_dir() {
                return Some(manifests);
            }
        }
        if let Ok(program_data) = std::env::var("PROGRAMDATA") {
            let candidate = PathBuf::from(program_data)
                .join("Epic")
                .join("EpicGamesLauncher")
                .join("Data")
                .join("Manifests");
            if candidate.is_dir() {
                return Some(candidate);
            }
        }
        let default = PathBuf::from(r"C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests");
        if default.is_dir() {
            return Some(default);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(home) = std::env::var("HOME") {
            let heroic = PathBuf::from(home).join(".config/legendary/installed.json");
            if heroic.parent().map(|p| p.is_dir()).unwrap_or(false) {
                return heroic.parent().map(|p| p.to_path_buf());
            }
        }
    }

    None
}

/// Locates the Epic Games Launcher v2 (`com.epicgames.eglv2`) data directory.
pub fn find_v2_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        for app_data in read_registry_app_data_paths() {
            if app_data.join("installations.jsonl").is_file() {
                return Some(app_data);
            }
        }
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let candidate = PathBuf::from(local_app_data).join("com.epicgames.eglv2");
            if candidate.join("installations.jsonl").is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn read_registry_app_data_paths() -> Vec<PathBuf> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    let mut paths = Vec::new();
    let hives = [
        (RegKey::predef(HKEY_CURRENT_USER), r"Software\Epic Games\EpicGamesLauncher"),
        (
            RegKey::predef(HKEY_LOCAL_MACHINE),
            r"SOFTWARE\WOW6432Node\Epic Games\EpicGamesLauncher",
        ),
        (
            RegKey::predef(HKEY_LOCAL_MACHINE),
            r"SOFTWARE\Epic Games\EpicGamesLauncher",
        ),
    ];

    for (hive, subkey) in hives {
        if let Ok(key) = hive.open_subkey(subkey) {
            if let Ok(path) = key.get_value::<String, _>("AppDataPath") {
                let trimmed = path.trim();
                if !trimmed.is_empty() {
                    paths.push(PathBuf::from(trimmed));
                }
            }
        }
    }
    paths
}

fn is_game_manifest(raw: &RawManifestItem) -> bool {
    if raw.app_name.trim().is_empty() || raw.display_name.trim().is_empty() {
        return false;
    }
    if raw.is_incomplete_install {
        return false;
    }
    if raw.install_location.trim().is_empty() {
        return false;
    }
    // Skip DLCs / add-ons that belong to another main game.
    if !raw.main_game_app_name.trim().is_empty() && raw.main_game_app_name != raw.app_name {
        return false;
    }
    // Skip Unreal Engine installations.
    if raw.app_name.starts_with("UE_") {
        return false;
    }
    if !raw.app_categories.is_empty() {
        let has_games = raw
            .app_categories
            .iter()
            .any(|c| c.eq_ignore_ascii_case("games"));
        if !has_games {
            return false;
        }
    }
    true
}

fn resolve_exe(install_dir: &Path, launch_executable: &str) -> Option<String> {
    let rel = launch_executable.trim();
    if rel.is_empty() {
        return None;
    }
    let full = install_dir.join(rel);
    if full.exists() {
        Some(full.to_string_lossy().into_owned())
    } else {
        None
    }
}

pub fn parse_manifest_str(json_str: &str) -> Option<InstalledEpicGame> {
    let raw: RawManifestItem = serde_json::from_str(json_str).ok()?;
    if !is_game_manifest(&raw) {
        return None;
    }
    let install_dir = Path::new(raw.install_location.trim());
    if !install_dir.exists() {
        return None;
    }

    let executable_path = resolve_exe(install_dir, &raw.launch_executable);

    Some(InstalledEpicGame {
        app_name: raw.app_name.trim().to_string(),
        display_name: raw.display_name.trim().to_string(),
        catalog_item_id: raw.catalog_item_id.trim().to_string(),
        namespace: raw.catalog_namespace.trim().to_string(),
        install_location: raw.install_location.trim().to_string(),
        executable_path,
        description: None,
        developer: None,
        cover_url: None,
    })
}

/// Reads all `.item` manifests in `dir` and returns installed Epic games keyed by `app_name`.
pub fn scan_installed_games_in(dir: &Path) -> HashMap<String, InstalledEpicGame> {
    let mut result = HashMap::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return result;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("item") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Some(game) = parse_manifest_str(&content) {
            result.insert(game.app_name.clone(), game);
        }
    }
    result
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct V2InstallEntry {
    #[serde(default)]
    installation_id: String,
    #[serde(default)]
    complete: bool,
    #[serde(default)]
    removed: bool,
    #[serde(default)]
    dir: String,
    #[serde(default)]
    namespace: String,
    #[serde(default)]
    item_id: String,
    #[serde(default)]
    artifact_id: String,
    #[serde(default)]
    launch_executable: String,
}

#[derive(Deserialize)]
struct V2CatalogCache {
    #[serde(default)]
    items: HashMap<String, V2CatalogWrapper>,
}

#[derive(Deserialize)]
struct V2CatalogWrapper {
    item: Option<V2CatalogItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct V2CatalogItem {
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    developer: String,
    #[serde(default)]
    key_images: Vec<V2KeyImage>,
    #[serde(default)]
    categories: Vec<V2Category>,
    #[serde(default)]
    main_game_item: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct V2KeyImage {
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(default)]
    url: String,
}

#[derive(Deserialize)]
struct V2Category {
    #[serde(default)]
    path: String,
}

fn pick_v2_cover(images: &[V2KeyImage]) -> Option<String> {
    const PRIORITY: &[&str] = &[
        "DieselGameBoxTall",
        "OfferImageTall",
        "DieselStoreFrontTall",
        "DieselGameBox",
        "OfferImageWide",
        "Thumbnail",
    ];
    for kind in PRIORITY {
        if let Some(found) = images.iter().find(|img| img.kind == *kind && !img.url.is_empty()) {
            return Some(found.url.clone());
        }
    }
    images
        .iter()
        .find(|img| !img.url.is_empty())
        .map(|img| img.url.clone())
}

/// Scans Epic Games Launcher v2 (`com.epicgames.eglv2`) `installations.jsonl` + `cache/prod/catalog-cache.json`.
pub fn scan_v2_installations(v2_dir: &Path) -> HashMap<String, InstalledEpicGame> {
    let mut result = HashMap::new();
    let jsonl_path = v2_dir.join("installations.jsonl");
    let Ok(content) = std::fs::read_to_string(&jsonl_path) else {
        return result;
    };

    // JSONL is an append-only log; later lines for the same `installationId` supersede earlier ones.
    let mut latest_by_id: HashMap<String, V2InstallEntry> = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(entry) = serde_json::from_str::<V2InstallEntry>(trimmed) else {
            continue;
        };
        if entry.installation_id.trim().is_empty() {
            continue;
        }
        latest_by_id.insert(entry.installation_id.clone(), entry);
    }

    let catalog_items: HashMap<String, V2CatalogItem> = {
        let cache_path = v2_dir.join("cache").join("prod").join("catalog-cache.json");
        std::fs::read_to_string(&cache_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<V2CatalogCache>(&raw).ok())
            .map(|cache| {
                cache
                    .items
                    .into_iter()
                    .filter_map(|(k, w)| w.item.map(|item| (k, item)))
                    .collect()
            })
            .unwrap_or_default()
    };

    for (_id, entry) in latest_by_id {
        if !entry.complete || entry.removed {
            continue;
        }
        let dir_trimmed = entry.dir.trim();
        if dir_trimmed.is_empty() {
            continue;
        }
        let install_dir = Path::new(dir_trimmed);
        if !install_dir.exists() {
            continue;
        }

        let app_name = if !entry.artifact_id.trim().is_empty() {
            entry.artifact_id.trim().to_string()
        } else {
            entry.installation_id.trim().to_string()
        };
        if app_name.starts_with("UE_") {
            continue;
        }

        let cat = catalog_items.get(entry.item_id.trim());
        if let Some(item) = cat {
            // Skip DLCs and non-game items (e.g. Unreal Engine).
            if item.main_game_item.is_some() {
                continue;
            }
            if !item.categories.is_empty()
                && !item
                    .categories
                    .iter()
                    .any(|c| c.path.eq_ignore_ascii_case("games"))
            {
                continue;
            }
        }

        let fallback_title = install_dir
            .file_name()
            .and_then(|n| n.to_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(&app_name)
            .to_string();

        let display_name = cat
            .map(|c| c.title.trim())
            .filter(|t| !t.is_empty())
            .unwrap_or(&fallback_title)
            .to_string();

        let executable_path = resolve_exe(install_dir, &entry.launch_executable);

        result.insert(
            app_name.clone(),
            InstalledEpicGame {
                app_name,
                display_name,
                catalog_item_id: entry.item_id.trim().to_string(),
                namespace: entry.namespace.trim().to_string(),
                install_location: dir_trimmed.to_string(),
                executable_path,
                description: cat
                    .map(|c| c.description.trim().to_string())
                    .filter(|d| !d.is_empty()),
                developer: cat
                    .map(|c| c.developer.trim().to_string())
                    .filter(|d| !d.is_empty()),
                cover_url: cat.and_then(|c| pick_v2_cover(&c.key_images)),
            },
        );
    }

    result
}

/// Scans Epic Games Launcher (both v1 manifests and v2 `com.epicgames.eglv2`) for installed games.
pub fn scan_installed_games() -> Option<HashMap<String, InstalledEpicGame>> {
    let v1_dir = find_manifests_dir();
    let v2_dir = find_v2_data_dir();

    if v1_dir.is_none() && v2_dir.is_none() {
        return None;
    }

    let mut combined = HashMap::new();
    if let Some(dir) = v1_dir {
        combined.extend(scan_installed_games_in(&dir));
    }
    if let Some(dir) = v2_dir {
        combined.extend(scan_v2_installations(&dir));
    }
    Some(combined)
}

#[derive(Deserialize)]
struct V2PlaytimeCache {
    #[serde(default)]
    items: Vec<crate::services::epic::api::PlaytimeEntry>,
}

/// Reads Epic Games Launcher v2's local `playtime-cache.json` for `account_id` as an offline fallback.
pub fn read_v2_playtime_cache(account_id: &str) -> HashMap<String, i64> {
    let mut map = HashMap::new();
    if account_id.trim().is_empty() {
        return map;
    }
    let Some(v2_dir) = find_v2_data_dir() else {
        return map;
    };
    let path = v2_dir
        .join("cache")
        .join("prod")
        .join("account")
        .join(account_id.trim())
        .join("playtime-cache.json");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return map;
    };
    if let Ok(cache) = serde_json::from_str::<V2PlaytimeCache>(&raw) {
        for entry in cache.items {
            if !entry.artifact_id.is_empty() && entry.total_time > 0 {
                map.insert(entry.artifact_id, entry.total_time);
            }
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_installed_epic_game() {
        let dir = std::env::temp_dir().join("gs_epic_test_valid");
        let _ = std::fs::create_dir_all(&dir);
        let exe = dir.join("Game.exe");
        std::fs::write(&exe, b"mz").expect("write dummy exe");

        let json = serde_json::json!({
            "AppName": "Sugar",
            "DisplayName": "Rocket League",
            "CatalogItemId": "cat123",
            "CatalogNamespace": "9773aa1aa54f4f7b80e44bef04986cea",
            "InstallLocation": dir.to_string_lossy(),
            "LaunchExecutable": "Game.exe",
            "MainGameAppName": "Sugar",
            "bIsIncompleteInstall": false,
            "AppCategories": ["public", "games"]
        })
        .to_string();

        let parsed = parse_manifest_str(&json).expect("should parse valid game");
        assert_eq!(parsed.app_name, "Sugar");
        assert_eq!(parsed.display_name, "Rocket League");
        assert_eq!(parsed.catalog_item_id, "cat123");
        assert_eq!(parsed.namespace, "9773aa1aa54f4f7b80e44bef04986cea");
        assert_eq!(parsed.executable_path.as_deref(), Some(exe.to_string_lossy().as_ref()));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn skips_incomplete_dlc_and_unreal_engine() {
        let dir = std::env::temp_dir().join("gs_epic_test_skip");
        let _ = std::fs::create_dir_all(&dir);

        let incomplete = serde_json::json!({
            "AppName": "Test1",
            "DisplayName": "Incomplete Game",
            "InstallLocation": dir.to_string_lossy(),
            "bIsIncompleteInstall": true,
            "AppCategories": ["games"]
        })
        .to_string();
        assert!(parse_manifest_str(&incomplete).is_none());

        let dlc = serde_json::json!({
            "AppName": "TestDlc",
            "DisplayName": "Some DLC",
            "InstallLocation": dir.to_string_lossy(),
            "MainGameAppName": "MainGame",
            "bIsIncompleteInstall": false,
            "AppCategories": ["games"]
        })
        .to_string();
        assert!(parse_manifest_str(&dlc).is_none());

        let engine = serde_json::json!({
            "AppName": "UE_5.4",
            "DisplayName": "Unreal Engine 5.4",
            "InstallLocation": dir.to_string_lossy(),
            "bIsIncompleteInstall": false,
            "AppCategories": ["engines"]
        })
        .to_string();
        assert!(parse_manifest_str(&engine).is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn skips_when_install_location_missing_on_disk() {
        let json = serde_json::json!({
            "AppName": "MissingApp",
            "DisplayName": "Missing Game",
            "InstallLocation": "Z:\\NonExistent\\Path\\For\\Epic\\Test",
            "bIsIncompleteInstall": false,
            "AppCategories": ["games"]
        })
        .to_string();
        assert!(parse_manifest_str(&json).is_none());
    }

    #[test]
    fn scans_v2_installations_jsonl_and_catalog_cache() {
        let root = std::env::temp_dir().join("gs_epic_test_v2");
        let game_dir = root.join("GTAV");
        let cache_dir = root.join("cache").join("prod");
        let _ = std::fs::create_dir_all(&game_dir);
        let _ = std::fs::create_dir_all(&cache_dir);
        let exe = game_dir.join("PlayGTAV.exe");
        std::fs::write(&exe, b"mz").expect("write exe");

        let jsonl = format!(
            "{{\"version\":1,\"entries\":2}}\n\
             {{\"installationId\":\"id1\",\"complete\":false,\"dir\":{},\"namespace\":\"ns1\",\"itemId\":\"item1\",\"artifactId\":\"art1\",\"launchExecutable\":\"PlayGTAV.exe\"}}\n\
             {{\"installationId\":\"id1\",\"complete\":true,\"dir\":{},\"namespace\":\"ns1\",\"itemId\":\"item1\",\"artifactId\":\"art1\",\"launchExecutable\":\"PlayGTAV.exe\"}}\n\
             {{\"installationId\":\"id2\",\"complete\":false,\"dir\":{},\"namespace\":\"ns1\",\"itemId\":\"item2\",\"artifactId\":\"art2\",\"launchExecutable\":\"\"}}\n",
            serde_json::to_string(&game_dir.to_string_lossy()).unwrap(),
            serde_json::to_string(&game_dir.to_string_lossy()).unwrap(),
            serde_json::to_string(&game_dir.to_string_lossy()).unwrap(),
        );
        std::fs::write(root.join("installations.jsonl"), jsonl).expect("write jsonl");

        let catalog = serde_json::json!({
            "items": {
                "item1": {
                    "item": {
                        "title": "Grand Theft Auto V",
                        "description": "Los Santos",
                        "developer": "Rockstar Games",
                        "categories": [{"path": "games"}],
                        "keyImages": [{"type": "DieselGameBoxTall", "url": "https://cdn.example/gtav.jpg"}]
                    }
                }
            }
        });
        std::fs::write(cache_dir.join("catalog-cache.json"), catalog.to_string()).expect("write cache");

        let games = scan_v2_installations(&root);
        assert_eq!(games.len(), 1);
        let g = games.get("art1").expect("art1 present");
        assert_eq!(g.display_name, "Grand Theft Auto V");
        assert_eq!(g.developer.as_deref(), Some("Rockstar Games"));
        assert_eq!(g.cover_url.as_deref(), Some("https://cdn.example/gtav.jpg"));
        assert_eq!(g.executable_path.as_deref(), Some(exe.to_string_lossy().as_ref()));

        let _ = std::fs::remove_dir_all(&root);
    }
}
