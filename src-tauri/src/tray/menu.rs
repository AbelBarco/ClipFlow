use tauri::{AppHandle, Manager, menu::{MenuBuilder, MenuItemBuilder}};
use tracing::info;

pub fn setup(app: AppHandle) -> Result<(), String> {
    let show_main = MenuItemBuilder::new("Show ClipFlow")
        .id("show_main")
        .accelerator("Ctrl+Shift+V")
        .build(&app)
        .map_err(|e| e.to_string())?;

    let settings = MenuItemBuilder::new("Settings")
        .id("settings")
        .build(&app)
        .map_err(|e| e.to_string())?;

    let quit = MenuItemBuilder::new("Quit")
        .id("quit")
        .accelerator("Ctrl+Q")
        .build(&app)
        .map_err(|e| e.to_string())?;

    let menu = MenuBuilder::new(&app)
        .items(&[&show_main, &settings, &quit])
        .build()
        .map_err(|e| e.to_string())?;

    app.tray()
        .set_menu(Some(menu))
        .map_err(|e| e.to_string())?;

    app.tray()
        .set_icon(app.default_window_icon().cloned())
        .map_err(|e| e.to_string())?;

    app.tray()
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "show_main" => {
                    let _ = app.emit("show-main-window", ());
                }
                "settings" => {
                    let _ = app.emit("show-settings", ());
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        });

    app.tray()
        .on_tray_icon_event(|_tray, event| {
            if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                // Left click could show main window
            }
        });

    info!("System tray initialized");
    Ok(())
}