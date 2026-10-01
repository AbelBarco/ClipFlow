use crate::clipboard::writer::write_to_clipboard;
use crate::storage::repository::{clear_all_clips, delete_clip, get_all_clips, get_clip_by_id};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClipItemDto {
    pub id: String,
    pub r#type: String,
    pub content: String,
    pub preview: String,
    pub timestamp: i64,
    pub size: i64,
    pub ocr_text: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl From<crate::storage::repository::ClipItem> for ClipItemDto {
    fn from(c: crate::storage::repository::ClipItem) -> Self {
        Self {
            id: c.id,
            r#type: c.r#type,
            content: c.content,
            preview: c.preview,
            timestamp: c.timestamp,
            size: c.size,
            ocr_text: c.ocr_text,
            metadata: c.metadata,
        }
    }
}

#[tauri::command]
pub async fn clipboard_get_history() -> Result<Vec<ClipItemDto>, String> {
    let clips = get_all_clips().await?;
    Ok(clips.into_iter().map(Into::into).collect())
}

/// Fetch one item with its full (decrypted) content, e.g. for the preview
/// panel, without scanning the whole history table.
#[tauri::command]
#[allow(non_snake_case)]
pub async fn clipboard_get_item(itemId: String) -> Result<Option<ClipItemDto>, String> {
    Ok(get_clip_by_id(&itemId).await?.map(Into::into))
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn clipboard_delete_item(itemId: String) -> Result<(), String> {
    // If it was an image, remove its file too (best effort).
    if let Some(clip) = get_clip_by_id(&itemId).await? {
        if clip.r#type == "image" {
            let _ = crate::storage::image_store::delete_image(&clip.content).await;
        }
    }
    delete_clip(&itemId).await
}

#[tauri::command]
pub async fn clipboard_clear_history(app: tauri::AppHandle) -> Result<(), String> {
    clear_all_clips().await?;
    // Clean orphaned image files as well (best effort).
    let _ = crate::storage::image_store::clean_images_dir(&app).await;
    Ok(())
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn clipboard_paste_item(itemId: String) -> Result<(), String> {
    if let Some(clip) = get_clip_by_id(&itemId).await? {
        write_to_clipboard(&clip.content, &clip.r#type).await
    } else {
        Err("Item not found".to_string())
    }
}

#[tauri::command]
pub async fn clipboard_copy_to_clipboard(content: String, r#type: String) -> Result<(), String> {
    write_to_clipboard(&content, &r#type).await
}

/// Return an image history item as a `data:` URL for preview in the webview.
/// `content` of image items is an absolute file path, which the webview
/// cannot load directly.
#[tauri::command]
#[allow(non_snake_case)]
pub async fn clipboard_get_image(itemId: String) -> Result<String, String> {
    let clip = get_clip_by_id(&itemId).await?.ok_or("Item not found")?;
    if clip.r#type != "image" {
        return Err("Item is not an image".to_string());
    }
    crate::storage::image_store::load_data_url(&clip.content).await
}
