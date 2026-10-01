use crate::clipboard::writer::{synthesize_paste, write_to_clipboard};
use crate::storage::repository::{clear_all_clips, delete_clip, get_all_clips, get_clip_by_id};
use serde::{Deserialize, Serialize};
use tauri::Manager;

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

/// Real paste: copy the item to the OS clipboard, hide the spotlight so the
/// keystrokes land in the previously focused app, then synthesize the paste
/// shortcut (Ctrl+V, Cmd+V on macOS).
///
/// Returns whether the keystroke was synthesized. When false (e.g. Wayland
/// without the right portal) the content is still in the clipboard — the
/// frontend tells the user to paste manually with Ctrl+V.
#[tauri::command]
#[allow(non_snake_case)]
pub async fn clipboard_paste_item(app: tauri::AppHandle, itemId: String) -> Result<bool, String> {
    let clip = get_clip_by_id(&itemId).await?.ok_or("Item not found")?;
    write_to_clipboard(&clip.content, &clip.r#type).await?;
    if let Some(win) = app.get_webview_window("spotlight") {
        let _ = win.hide();
    }
    // Let the OS return focus to the previous app before typing into it.
    tokio::time::sleep(std::time::Duration::from_millis(180)).await;
    let pasted = synthesize_paste().await;
    if !pasted {
        // Show the window again so the frontend can tell the user to paste
        // manually: the content is in the clipboard either way.
        if let Some(win) = app.get_webview_window("spotlight") {
            let _ = win.show();
            let _ = win.set_focus();
        }
    }
    Ok(pasted)
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
