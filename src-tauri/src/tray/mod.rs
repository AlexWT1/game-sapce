use tauri::Emitter;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

use crate::AppState;

pub const MENU_OPEN: &str = "open";
pub const MENU_CONTINUE: &str = "continue";
pub const MENU_SETTINGS: &str = "settings";
pub const MENU_QUIT: &str = "quit";
pub const MENU_SESSION: &str = "session-info";

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, MENU_OPEN, "Открыть Game Space", true, None::<&str>)?;
    let continue_item =
        MenuItem::with_id(app, MENU_CONTINUE, "Продолжить игру", true, None::<&str>)?;
    let session =
        MenuItem::with_id(app, MENU_SESSION, "Нет активной сессии", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, MENU_SETTINGS, "Настройки", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "Выйти", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "title", "Game Space", false, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &continue_item,
            &session,
            &PredefinedMenuItem::separator(app)?,
            &open,
            &settings,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut slot) = state.tray_session_item.lock() {
            *slot = Some(session.clone());
        }
    }

    let Some(icon) = app.default_window_icon().cloned() else {
        log::warn!("Default window icon missing, tray not created");
        return Ok(());
    };

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("Game Space")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            MENU_QUIT => {
                app.exit(0);
            }
            MENU_OPEN | MENU_CONTINUE => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                    let _ = app.emit(
                        "navigate",
                        serde_json::json!({ "path": "/" }),
                    );
                }
            }
            MENU_SETTINGS => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = app.emit("navigate", serde_json::json!({ "path": "/settings" }));
                }
            }
            _ => {}
        })
        .build(app)?;

    log::info!("System tray initialized");
    Ok(())
}

