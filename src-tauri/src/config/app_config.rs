use serde::{Deserialize, Serialize};
use tauri::Manager;
use crate::storage::db::get_connection;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub general: GeneralSettings,
    pub exclusions: ExclusionSettings,
    pub ocr: OcrSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralSettings {
    pub global_shortcut: String,
    pub max_history_items: usize,
    pub launch_at_startup: bool,
    pub show_notifications: bool,
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExclusionSettings {
    pub excluded_apps: Vec<String>,
    pub excluded_types: Vec<String>,
    pub respect_concealed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrSettings {
    pub enabled: bool,
    pub language: String,
    pub auto_run: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralSettings {
                global_shortcut: "Ctrl+Shift+V".to_string(),
                max_history_items: 500,
                launch_at_startup: false,
                show_notifications: true,
                theme: "system".to_string(),
            },
            exclusions: ExclusionSettings {
                excluded_apps: vec![
                    "1Password".to_string(),
                    "Bitwarden".to_string(),
                    "LastPass".to_string(),
                    "KeePass".to_string(),
                ],
                excluded_types: vec!["password".to_string(), "concealed".to_string()],
                respect_concealed: true,
            },
            ocr: OcrSettings {
                enabled: true,
                language: "eng".to_string(),
                auto_run: false,
            },
        }
    }
}

const CONFIG_KEY: &str = "clipflow_config";

pub async fn get_config() -> Result<AppConfig, String> {
    let conn = get_connection().ok_or("Database not initialized")?;
    let conn = conn.lock().unwrap();

    let config_json: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [CONFIG_KEY],
            |row| row.get(0),
        )
        .ok();

    if let Some(json) = config_json {
        serde_json::from_str(&json).map_err(|e| e.to_string())
    } else {
        Ok(AppConfig::default())
    }
}

pub async fn set_config(config: AppConfig) -> Result<(), String> {
    let conn = get_connection().ok_or("Database not initialized")?;
    let conn = conn.lock().unwrap();

    let json = serde_json::to_string(&config).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        [CONFIG_KEY, &json],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn reset_config() -> Result<AppConfig, String> {
    let default = AppConfig::default();
    set_config(default.clone()).await?;
    Ok(default)
}

pub async fn get_excluded_apps() -> Result<Vec<String>, String> {
    let config = get_config().await?;
    Ok(config.exclusions.excluded_apps)
}

pub async fn get_global_shortcut() -> Result<String, String> {
    let config = get_config().await?;
    Ok(config.general.global_shortcut)
}

pub async fn init_settings_table() -> Result<(), String> {
    let conn = get_connection().ok_or("Database not initialized")?;
    let conn = conn.lock().unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
        [],
    ).map_err(|e| e.to_string())?;

    Ok(())
}