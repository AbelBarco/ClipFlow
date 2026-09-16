pub mod watcher;
pub mod writer;
pub mod exclusion;

use crate::clipboard::watcher::start_watcher;
use crate::clipboard::writer::write_to_clipboard;
use tauri::Manager;

pub async fn start(app: tauri::AppHandle) -> Result<(), String> {
    start_watcher(app).await
}

pub async fn write(content: &str, mime_type: &str) -> Result<(), String> {
    write_to_clipboard(content, mime_type).await
}