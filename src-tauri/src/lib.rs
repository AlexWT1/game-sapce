mod commands;
mod database;
mod error;
mod game;
mod logger;
mod models;
mod process;
mod services;
mod tray;

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use tauri::menu::MenuItem;
use tauri::{Emitter, Manager};

use database::Database;
use process::monitor::ActiveMap;

pub struct AppState {
    pub db: Database,
    pub active: ActiveMap,
    pub tray_session_item: Mutex<Option<MenuItem<tauri::Wry>>>,
    /// Guards `sync_steam_library` against a second overlapping run.
    pub steam_syncing: Arc<AtomicBool>,
    /// Guards the launcher update download against a second overlapping run.
    pub update_downloading: Arc<AtomicBool>,
    /// Signals the running update download to cancel.
    pub update_cancel: Arc<AtomicBool>,
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            log::info!("Second instance detected — focusing existing window");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            logger::init(&data_dir.join("logs"))?;
            log::info!("Game Space starting (data dir: {})", data_dir.display());

            let db = Database::open(data_dir.join("game-space.db"))?;
            db.with(|conn| {
                game::seed::seed_if_empty(conn)?;
                game::achievements::ensure_app_achievements(conn)?;
                // Standard per-game set (launch + 10/100/1000 h) for older libraries.
                game::achievements::ensure_all_game_achievements(conn)?;
                Ok::<(), error::AppError>(())
            })?;

            app.manage(AppState {
                db,
                active: Default::default(),
                tray_session_item: Mutex::new(None),
                steam_syncing: Arc::new(AtomicBool::new(false)),
                update_downloading: Arc::new(AtomicBool::new(false)),
                update_cancel: Arc::new(AtomicBool::new(false)),
            });

            tray::setup(app.handle())?;
            process::monitor::recover_sessions(app.handle());
            process::monitor::start_monitor(app.handle().clone());
            log::info!("Setup complete");
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::DragDrop(drop) = event {
                if let tauri::DragDropEvent::Drop { paths, .. } = drop {
                    let payload: Vec<String> = paths
                        .iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect();
                    log::info!("Files dropped: {payload:?}");
                    let _ = window.emit(
                        "file-dropped",
                        serde_json::json!({ "paths": payload }),
                    );
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::games::get_games,
            commands::games::get_game,
            commands::games::create_game,
            commands::games::update_game,
            commands::games::delete_game,
            commands::games::delete_games,
            commands::games::toggle_favorite,
            commands::games::set_game_executable,
            commands::games::get_game_theme,
            commands::games::set_game_theme,
            commands::launch::launch_game,
            commands::launch::stop_game,
            commands::launch::get_running_games,
            commands::launch::is_game_running,
            commands::launch::start_game_session,
            commands::launch::finish_game_session,
            commands::launch::get_game_sessions,
            commands::stats::get_statistics,
            commands::stats::get_game_statistics,
            commands::store::get_store_account,
            commands::store::link_steam_account,
            commands::store::unlink_steam_account,
            commands::store::sync_steam_library,
            commands::achievements::get_achievements,
            commands::achievements::unlock_achievement,
            commands::igdb::igdb_search_games,
            commands::igdb::igdb_get_game,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::get_setting,
            commands::system::window_minimize,
            commands::system::window_toggle_maximize,
            commands::system::window_close,
            commands::system::get_app_info,
            commands::system::select_game_executable,
            commands::system::select_game_directory,
            commands::system::select_any_file,
            commands::system::export_data,
            commands::system::import_data,
            commands::system::reset_data,
            commands::system::search_all,
            commands::system::open_path,
            commands::updater::check_for_updates,
            commands::updater::download_update,
            commands::updater::cancel_update_download,
            commands::updater::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Game Space");
}
