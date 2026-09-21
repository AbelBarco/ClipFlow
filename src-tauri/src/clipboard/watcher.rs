use crate::pipeline::{process_clipboard_content, process_image_content};
use crate::storage::repository::{add_clip, ClipItem};
use crate::clipboard::exclusion::should_exclude;
use tauri::{AppHandle, Emitter};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, debug, error};

/// Max image pixels to store (avoid freezing on giant bitmaps).
const MAX_IMAGE_PIXELS: usize = 24_000_000;

pub async fn start_watcher(app: AppHandle) -> Result<(), String> {
    // Text polling is cheap and needs to feel instant; image polling opens
    // and decodes a bitmap, so it runs 4x less often (every ~2s).
    const IMAGE_EVERY_N_TICKS: u64 = 4;

    let mut ticker = interval(Duration::from_millis(500));
    let mut last_text_hash: Option<String> = None;
    let mut last_image_hash: Option<u64> = None;
    let mut tick: u64 = 0;

    tokio::spawn(async move {
        loop {
            ticker.tick().await;
            tick += 1;

            // --- Text (and colors, urls, code — all travel as text) ---
            match get_clipboard_text().await {
                Ok(content) => {
                    if !content.trim().is_empty() {
                        let hash = crate::pipeline::dedupe::content_hash(&content);
                        if last_text_hash.as_ref() != Some(&hash) {
                            last_text_hash = Some(hash.clone());
                            if should_exclude(&content).await {
                                debug!("Excluded clipboard content from history");
                            } else {
                                let item =
                                    process_clipboard_content(content.as_bytes(), None);
                                emit_clip(&app, ClipItem::from(item)).await;
                            }
                        }
                    }
                }
                Err(_) => {
                    // No text in clipboard — normal, keep polling quietly.
                }
            }

            // --- Images (screenshots, copied photos) — sampled, not every tick ---
            if tick % IMAGE_EVERY_N_TICKS == 0 {
                match get_clipboard_image().await {
                    Ok((width, height, rgba)) => {
                        if width > 0 && height > 0 && width * height <= MAX_IMAGE_PIXELS {
                            let hash = image_hash(width, height, &rgba);
                            if last_image_hash != Some(hash) {
                                last_image_hash = Some(hash);
                                let byte_len = rgba.len();
                                match crate::storage::image_store::save_rgba_png(
                                    &app, width, height, &rgba,
                                )
                                .await
                                {
                                    Ok(path) => {
                                        let item = process_image_content(
                                            path, width, height, byte_len,
                                        );
                                        emit_clip(&app, ClipItem::from(item)).await;
                                    }
                                    Err(e) => error!("Failed to save clipboard image: {}", e),
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // No image in clipboard — the common case.
                    }
                }
            }
        }
    });

    Ok(())
}

async fn emit_clip(app: &AppHandle, clip_item: ClipItem) {
    match add_clip(clip_item.clone()).await {
        Ok(()) => {
            info!("Added new clipboard item: {} ({})", clip_item.id, clip_item.r#type);
            if let Err(e) = app.emit("clipboard-item-added", &clip_item) {
                error!("Failed to emit clipboard event: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to add clip to storage: {}", e);
        }
    }
}

/// Fast, lossy hash of image bytes (dimensions + length + samples).
fn image_hash(width: usize, height: usize, rgba: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    width.hash(&mut hasher);
    height.hash(&mut hasher);
    rgba.len().hash(&mut hasher);
    // First 32 KiB + last 4 KiB is plenty to tell images apart.
    let head = &rgba[..rgba.len().min(32 * 1024)];
    head.hash(&mut hasher);
    if rgba.len() > 32 * 1024 {
        rgba[rgba.len() - 4096..].hash(&mut hasher);
    }
    hasher.finish()
}

async fn get_clipboard_text() -> Result<String, String> {
    tokio::task::spawn_blocking(|| {
        let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        clipboard.get_text().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

async fn get_clipboard_image() -> Result<(usize, usize, Vec<u8>), String> {
    tokio::task::spawn_blocking(|| {
        let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        let img = clipboard.get_image().map_err(|e| e.to_string())?;
        Ok((img.width, img.height, img.bytes.to_vec()))
    })
    .await
    .map_err(|e| e.to_string())?
}
