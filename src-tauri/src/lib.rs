mod commands;
mod database;
mod error;
mod game;
mod logger;
mod models;
mod process;
mod services;
mod tray;

use std::sync::Mutex;

use tauri::menu::MenuItem;
use tauri::{Emitter, Manager};

use database::Database;
use process::monitor::ActiveMap;

pub struct AppState {
    pub db: Database,
    pub active: ActiveMap,
    pub tray_session_item: Mutex<Option<MenuItem<tauri::Wry>>>,
}

pub fn run() {
    tauri::Builder::default()
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
                Ok::<(), error::AppError>(())
            })?;

            app.manage(AppState {
                db,
                active: Default::default(),
                tray_session_item: Mutex::new(None),
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
            commands::games::toggle_favorite,
            commands::games::set_game_executable,
            commands::games::get_game_theme,
            commands::games::set_game_theme,
            commands::games::game_exists,
            commands::launch::launch_game,
            commands::launch::stop_game,
            commands::launch::get_running_games,
            commands::launch::is_game_running,
            commands::launch::start_game_session,
            commands::launch::finish_game_session,
            commands::launch::get_game_sessions,
            commands::stats::get_statistics,
            commands::stats::get_game_statistics,
            commands::achievements::get_achievements,
            commands::achievements::unlock_achievement,
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
            commands::system::discover_games,
            commands::system::export_data,
            commands::system::import_data,
            commands::system::reset_data,
            commands::system::search_all,
            commands::system::open_path,
            commands::system::get_default_game_dirs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Game Space");
}
