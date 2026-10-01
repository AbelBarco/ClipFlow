use crate::config::app_config::{
    get_config as get_impl, reset_config as reset_impl, set_config as set_impl, AppConfig,
};

#[tauri::command]
pub async fn settings_get() -> Result<AppConfig, String> {
    get_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn settings_set(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    // If the shortcut changed, update the global shortcut registration.
    // We can't access AppHandle here; the frontend emits an event the
    // backend listens to (see lib.rs). For now just persist.
    set_impl(config.clone()).await.map_err(|e| e.to_string())?;
    // Launch-at-startup is real now: apply the toggle to the OS right away
    // instead of waiting for the next boot (which also re-syncs).
    crate::autostart::sync_launch_at_startup(&app, config.general.launch_at_startup)?;
    Ok(())
}

#[tauri::command]
pub async fn settings_reset(app: tauri::AppHandle) -> Result<AppConfig, String> {
    let defaults = reset_impl().await.map_err(|e| e.to_string())?;
    crate::autostart::sync_launch_at_startup(&app, defaults.general.launch_at_startup)?;
    Ok(defaults)
}

// Backwards-compatible aliases for the old names.
#[tauri::command]
pub async fn get_config() -> Result<AppConfig, String> {
    get_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    set_impl(config.clone()).await.map_err(|e| e.to_string())?;
    crate::autostart::sync_launch_at_startup(&app, config.general.launch_at_startup)?;
    Ok(())
}

#[tauri::command]
pub async fn reset_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    let defaults = reset_impl().await.map_err(|e| e.to_string())?;
    crate::autostart::sync_launch_at_startup(&app, defaults.general.launch_at_startup)?;
    Ok(defaults)
}
