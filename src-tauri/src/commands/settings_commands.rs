use crate::config::app_config::{get_config as get_impl, set_config as set_impl, reset_config as reset_impl, AppConfig};

#[tauri::command]
pub async fn settings_get() -> Result<AppConfig, String> {
    get_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn settings_set(config: AppConfig) -> Result<(), String> {
    // If the shortcut changed, update the global shortcut registration.
    // We can't access AppHandle here; the frontend emits an event the
    // backend listens to (see lib.rs). For now just persist.
    set_impl(config).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn settings_reset() -> Result<AppConfig, String> {
    reset_impl().await.map_err(|e| e.to_string())
}

// Backwards-compatible aliases for the old names.
#[tauri::command]
pub async fn get_config() -> Result<AppConfig, String> {
    get_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_config(config: AppConfig) -> Result<(), String> {
    set_impl(config).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_config() -> Result<AppConfig, String> {
    reset_impl().await.map_err(|e| e.to_string())
}
