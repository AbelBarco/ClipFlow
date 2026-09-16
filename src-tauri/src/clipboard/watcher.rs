use crate::pipeline::{process_clipboard_content, ClipItem};
use crate::storage::repository::add_clip;
use crate::clipboard::exclusion::should_exclude;
use tauri::{AppHandle, Manager};
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, debug, error};

pub async fn start_watcher(app: AppHandle) -> Result<(), String> {
    let mut interval = interval(Duration::from_millis(300));
    let mut last_hash: Option<String> = None;

    tokio::spawn(async move {
        loop {
            interval.tick().await;

            if let Ok(content) = get_clipboard_content().await {
                let hash = crate::pipeline::dedupe::content_hash(&content);

                if last_hash.as_ref() == Some(&hash) {
                    continue;
                }

                // Check if we should exclude this content
                if should_exclude(&content).await {
                    debug!("Excluded clipboard content from history");
                    continue;
                }

                last_hash = Some(hash.clone());

                let clip_item = process_clipboard_content(content.as_bytes(), None);

                if let Err(e) = add_clip(clip_item.clone()).await {
                    error!("Failed to add clip to storage: {}", e);
                } else {
                    info!("Added new clipboard item: {}", clip_item.id);

                    // Notify frontend
                    if let Err(e) = app.emit("clipboard-item-added", &clip_item) {
                        error!("Failed to emit clipboard event: {}", e);
                    }
                }
            }
        }
    });

    Ok(())
}

async fn get_clipboard_content() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        get_clipboard_windows().await
    }
    #[cfg(target_os = "macos")]
    {
        get_clipboard_macos().await
    }
    #[cfg(target_os = "linux")]
    {
        get_clipboard_linux().await
    }
}

#[cfg(target_os = "windows")]
async fn get_clipboard_windows() -> Result<String, String> {
    use windows::Win32::UI::WindowsAndMessaging::{OpenClipboard, CloseClipboard, GetClipboardData, CF_UNICODETEXT};
    use windows::Win32::Foundation::HWND;
    use windows::core::PCWSTR;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    unsafe {
        if OpenClipboard(HWND(0)).is_err() {
            return Err("Failed to open clipboard".to_string());
        }

        let handle = GetClipboardData(CF_UNICODETEXT);
        let _ = CloseClipboard();

        if handle.0.is_null() {
            return Err("No text data in clipboard".to_string());
        }

        let ptr = handle.0 as *const u16;
        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(ptr, len);
        let string = OsString::from_wide(slice).to_string_lossy().to_string();

        Ok(string)
    }
}

#[cfg(target_os = "macos")]
async fn get_clipboard_macos() -> Result<String, String> {
    use objc2::{rc::Retained, runtime::AnyObject};
    use objc2_foundation::{NSString, NSPasteboard, NSPasteboardTypeString};
    use objc2_app_kit::NSPasteboardTypeString;

    let pasteboard = NSPasteboard::generalPasteboard();
    let string = pasteboard.stringForType(&NSPasteboardTypeString);

    string.map(|s| s.to_string()).ok_or("No text in clipboard".to_string())
}

#[cfg(target_os = "linux")]
async fn get_clipboard_linux() -> Result<String, String> {
    // Using xclip or wl-paste via command
    let output = tokio::process::Command::new("wl-paste")
        .arg("--no-newline")
        .output()
        .await
        .or_else(|_| {
            tokio::process::Command::new("xclip")
                .args(["-selection", "clipboard", "-o"])
                .output()
        })
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err("Failed to read clipboard".to_string())
    }
}