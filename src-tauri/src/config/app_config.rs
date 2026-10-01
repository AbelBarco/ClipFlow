use crate::storage::db::with_db;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub general: GeneralSettings,
    pub exclusions: ExclusionSettings,
    pub ocr: OcrSettings,
    /// Native offline spell/grammar corrector preferences.
    #[serde(default)]
    pub corrector: CorrectorSettings,
    /// At-rest protection preferences.
    #[serde(default)]
    pub security: SecuritySettings,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SecuritySettings {
    /// Encrypt `content`/`ocr_text` in SQLite with an OS-keychain key.
    pub encryption_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralSettings {
    pub global_shortcut: String,
    pub max_history_items: usize,
    pub launch_at_startup: bool,
    pub show_notifications: bool,
    pub theme: String,
    /// UI locale code: en | es | fr | de | pt | it | zh | ja | ko | ru
    #[serde(default = "default_ui_language")]
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExclusionSettings {
    pub excluded_apps: Vec<String>,
    pub excluded_types: Vec<String>,
    pub respect_concealed: bool,
    /// Secret-heuristic sensitivity: "conservative" (default) or "standard".
    #[serde(default = "default_heuristic_level")]
    pub heuristic_level: String,
    /// Show an OS notification when an item is dropped as a possible secret.
    #[serde(default = "default_true")]
    pub notify_on_exclude: bool,
}

fn default_heuristic_level() -> String {
    "conservative".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrSettings {
    pub enabled: bool,
    pub language: String,
    pub auto_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorrectorSettings {
    pub enabled: bool,
    /// UI locale code or "auto" (follow interface language).
    #[serde(default = "default_corrector_language")]
    pub language: String,
    #[serde(default)]
    pub auto_correct: bool,
    /// User-taught words ("add to dictionary"), always treated as known.
    #[serde(default)]
    pub custom_words: Vec<String>,
}

impl Default for CorrectorSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            language: "auto".to_string(),
            auto_correct: false,
            custom_words: Vec::new(),
        }
    }
}

fn default_ui_language() -> String {
    "es".to_string()
}

fn default_corrector_language() -> String {
    "auto".to_string()
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
                language: default_ui_language(),
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
                heuristic_level: default_heuristic_level(),
                notify_on_exclude: true,
            },
            ocr: OcrSettings {
                enabled: true,
                language: "eng".to_string(),
                auto_run: false,
            },
            corrector: CorrectorSettings::default(),
            security: SecuritySettings::default(),
        }
    }
}

const CONFIG_KEY: &str = "clipflow_config";

pub async fn get_config() -> Result<AppConfig, String> {
    let json: Option<String> = with_db(|conn| {
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![CONFIG_KEY])?;
        if let Some(row) = rows.next()? {
            let v: String = row.get(0)?;
            Ok(Some(v))
        } else {
            Ok(None)
        }
    })?;

    if let Some(json) = json {
        serde_json::from_str(&json).map_err(|e| e.to_string())
    } else {
        Ok(AppConfig::default())
    }
}

pub async fn set_config(config: AppConfig) -> Result<(), String> {
    let json = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    with_db(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![CONFIG_KEY, json],
        )?;
        Ok(())
    })
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
