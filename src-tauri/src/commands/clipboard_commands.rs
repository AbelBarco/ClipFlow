use crate::storage::repository::{get_all_clips, delete_clip, clear_all_clips};
use crate::clipboard::writer::write_to_clipboard;
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClipItem {
    pub id: String,
    pub r#type: String,
    pub content: String,
    pub preview: String,
    pub timestamp: i64,
    pub size: i64,
    pub ocr_text: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[command]
pub async fn get_history() -> Result<Vec<ClipItem>, String> {
    let clips = get_all_clips().await?;
    Ok(clips.into_iter().map(Into::into).collect())
}

#[command]
pub async fn delete_item(item_id: String) -> Result<(), String> {
    delete_clip(&item_id).await
}

#[command]
pub async fn clear_history() -> Result<(), String> {
    clear_all_clips().await
}

#[command]
pub async fn paste_item(item_id: String) -> Result<(), String> {
    let clips = get_all_clips().await?;
    if let Some(clip) = clips.into_iter().find(|c| c.id == item_id) {
        write_to_clipboard(&clip.content, &clip.r#type).await
    } else {
        Err("Item not found".to_string())
    }
}

#[command]
pub async fn copy_to_clipboard(content: String, r#type: String) -> Result<(), String> {
    write_to_clipboard(&content, &r#type).await
}