use tauri::{AppHandle, Manager, GlobalShortcutExt};
use tracing::{info, error};

pub async fn register_global_shortcut(app: AppHandle) -> Result<(), String> {
    let shortcut = "Ctrl+Shift+V"; // Default, should come from settings

    app.global_shortcut()
        .on_shortcut(shortcut, move |app, _shortcut, _event| {
            let _ = app.emit("show-spotlight", ());
        })
        .map_err(|e| format!("Failed to register global shortcut: {}", e))?;

    info!("Registered global shortcut: {}", shortcut);
    Ok(())
}

pub fn update_shortcut(app: &AppHandle, new_shortcut: &str) -> Result<(), String> {
    // Unregister all and re-register
    app.global_shortcut().unregister_all().map_err(|e| e.to_string())?;

    app.global_shortcut()
        .on_shortcut(new_shortcut, move |app, _shortcut, _event| {
            let _ = app.emit("show-spotlight", ());
        })
        .map_err(|e| format!("Failed to register global shortcut: {}", e))?;

    Ok(())
}