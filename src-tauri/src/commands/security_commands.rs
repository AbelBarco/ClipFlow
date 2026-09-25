//! At-rest encryption controls.
//!
//! `security_set_encryption` migrates every stored row and then persists the
//! flag, so the database and the setting can never disagree for long: the
//! migration is idempotent and safe to retry after an interruption.

use crate::config::app_config::{get_config, set_config};
use crate::storage::repository::migrate_encryption;

/// Enable/disable encryption of `content`/`ocr_text` at rest.
/// Returns the number of rows converted.
pub async fn set_encryption(enabled: bool) -> Result<usize, String> {
    // Migrate first: if the OS keychain is unavailable this fails BEFORE
    // we persist anything, so the UI never shows "encrypted" while rows
    // are still plaintext.
    let converted = migrate_encryption(enabled).await?;

    let mut config = get_config().await?;
    config.security.encryption_enabled = enabled;
    set_config(config).await?;

    tracing::info!("At-rest encryption set to {enabled} ({converted} rows converted)");
    Ok(converted)
}

#[tauri::command]
pub async fn security_set_encryption(enabled: bool) -> Result<usize, String> {
    set_encryption(enabled).await.map_err(|e| e.to_string())
}

/// Reports whether encryption is currently enabled.
#[tauri::command]
pub async fn security_status() -> Result<bool, String> {
    Ok(get_config()
        .await
        .map(|c| c.security.encryption_enabled)
        .unwrap_or(false))
}
