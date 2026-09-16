use crate::config::app_config::{get_config, set_config, reset_config, AppConfig};
use tauri::command;

#[command]
pub async fn get_config() -> Result<AppConfig, String> {
    get_config().await.map_err(|e| e.to_string())
}

#[command]
pub async fn set_config(config: AppConfig) -> Result<(), String> {
    set_config(config).await.map_err(|e| e.to_string())
}

#[command]
pub async fn reset_config() -> Result<AppConfig, String> {
    reset_config().await.map_err(|e| e.to_string())
}