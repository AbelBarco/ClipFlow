use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    AppHandle, Emitter,
};
use tracing::info;

pub fn setup(app: &AppHandle) -> Result<(), String> {
    let show_main = MenuItemBuilder::new("Show ClipFlow")
        .id("show_main")
        .accelerator("Ctrl+Shift+V")
        .build(app)
        .map_err(|e| e.to_string())?;

    let spotlight = MenuItemBuilder::new("Open Spotlight")
        .id("show_spotlight")
        .build(app)
        .map_err(|e| e.to_string())?;

    let settings = MenuItemBuilder::new("Settings")
        .id("settings")
        .build(app)
        .map_err(|e| e.to_string())?;

    let quit = MenuItemBuilder::new("Quit")
        .id("quit")
        .accelerator("Ctrl+Q")
        .build(app)
        .map_err(|e| e.to_string())?;

    let menu = MenuBuilder::new(app)
        .items(&[&show_main, &spotlight, &settings, &quit])
        .build()
        .map_err(|e| e.to_string())?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or("No default window icon")?;

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("ClipFlow")
        .on_menu_event(|app: &AppHandle, event| match event.id.as_ref() {
            "show_main" => {
                let _ = app.emit("show-main-window", ());
                crate::windows::show_main_window(app).unwrap_or(());
            }
            "show_spotlight" => {
                let _ = app.emit("show-spotlight", ());
                crate::windows::show_spotlight_window(app).unwrap_or(());
            }
            "settings" => {
                let _ = app.emit("show-settings", ());
                crate::windows::show_settings_window(app).unwrap_or(());
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let _ = app.emit("show-spotlight", ());
                crate::windows::show_spotlight_window(app).unwrap_or(());
            }
        })
        .build(app)
        .map_err(|e| e.to_string())?;

    info!("System tray initialized");
    Ok(())
}
