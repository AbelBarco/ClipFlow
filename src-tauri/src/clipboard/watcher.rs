use crate::pipeline::process_clipboard_content;
use crate::storage::repository::{add_clip, ClipItem};
use crate::clipboard::exclusion::should_exclude;
use tauri::{AppHandle, Emitter};
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, debug, error};

pub async fn start_watcher(app: AppHandle) -> Result<(), String> {
    let mut ticker = interval(Duration::from_millis(500));
    let mut last_hash: Option<String> = None;

    tokio::spawn(async move {
        loop {
            ticker.tick().await;

            match get_clipboard_content().await {
                Ok(content) => {
                    if content.trim().is_empty() {
                        continue;
                    }
                    let hash = crate::pipeline::dedupe::content_hash(&content);

                    if last_hash.as_ref() == Some(&hash) {
                        continue;
                    }

                    // Check if we should exclude this content
                    if should_exclude(&content).await {
                        debug!("Excluded clipboard content from history");
                        last_hash = Some(hash);
                        continue;
                    }

                    last_hash = Some(hash);

                    let pipeline_item = process_clipboard_content(content.as_bytes(), None);
                    let clip_item = ClipItem::from(pipeline_item.clone());

                    match add_clip(clip_item.clone()).await {
                        Ok(()) => {
                            info!("Added new clipboard item: {}", clip_item.id);
                            // Notify frontend
                            if let Err(e) = app.emit("clipboard-item-added", &clip_item) {
                                error!("Failed to emit clipboard event: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to add clip to storage: {}", e);
                        }
                    }
                }
                Err(_) => {
                    // No text in clipboard — normal, just keep polling quietly.
                }
            }
        }
    });

    Ok(())
}

async fn get_clipboard_content() -> Result<String, String> {
    tokio::task::spawn_blocking(|| {
        let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        clipboard.get_text().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
