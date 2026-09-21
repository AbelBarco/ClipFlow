use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tracing::info;

pub fn register_global_shortcut(app: &AppHandle) -> Result<(), String> {
    // Try to read the configured shortcut, fall back to default.
    let shortcut = std::env::var("CLIPFLOW_SHORTCUT").unwrap_or_else(|_| "Ctrl+Shift+V".to_string());

    register_shortcut(app, &shortcut)
}

pub fn register_shortcut(app: &AppHandle, shortcut: &str) -> Result<(), String> {
    app.global_shortcut()
        .on_shortcut(shortcut, |app, _shortcut, event| {
            // Only react on key press, not release.
            use tauri_plugin_global_shortcut::ShortcutState;
            if event.state == ShortcutState::Pressed {
                let _ = app.emit("show-spotlight", ());
                crate::windows::show_spotlight_window(app).unwrap_or(());
            }
        })
        .map_err(|e| format!("Failed to register global shortcut '{shortcut}': {e}"))?;

    info!("Registered global shortcut: {}", shortcut);
    Ok(())
}

pub fn update_shortcut(app: &AppHandle, new_shortcut: &str) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;
    register_shortcut(app, new_shortcut)
}
