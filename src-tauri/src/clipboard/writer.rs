use tauri::{AppHandle, Manager};
use tracing::{info, error};

pub async fn write_to_clipboard(content: &str, mime_type: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        write_clipboard_windows(content).await
    }
    #[cfg(target_os = "macos")]
    {
        write_clipboard_macos(content).await
    }
    #[cfg(target_os = "linux")]
    {
        write_clipboard_linux(content).await
    }
}

#[cfg(target_os = "windows")]
async fn write_clipboard_windows(content: &str) -> Result<(), String> {
    use windows::Win32::UI::WindowsAndMessaging::{OpenClipboard, CloseClipboard, EmptyClipboard, SetClipboardData, CF_UNICODETEXT};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::DataExchange::GlobalAlloc;
    use windows::Win32::System::Memory::{GMEM_MOVEABLE, GMEM_ZEROINIT};
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    unsafe {
        if OpenClipboard(HWND(0)).is_err() {
            return Err("Failed to open clipboard".to_string());
        }

        if EmptyClipboard().is_err() {
            let _ = CloseClipboard();
            return Err("Failed to empty clipboard".to_string());
        }

        let wide: Vec<u16> = OsString::from(content).encode_wide().chain(Some(0)).collect();
        let size = wide.len() * 2;
        let handle = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, size);
        if handle.0.is_null() {
            let _ = CloseClipboard();
            return Err("Failed to allocate clipboard memory".to_string());
        }

        let ptr = handle.0 as *mut u16;
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());

        if SetClipboardData(CF_UNICODETEXT, handle).is_err() {
            let _ = CloseClipboard();
            return Err("Failed to set clipboard data".to_string());
        }

        let _ = CloseClipboard();
        Ok(())
    }
}

#[cfg(target_os = "macos")]
async fn write_clipboard_macos(content: &str) -> Result<(), String> {
    use objc2_foundation::{NSString, NSPasteboard, NSPasteboardTypeString};

    let pasteboard = NSPasteboard::generalPasteboard();
    pasteboard.clearContents();
    let string = NSString::from_str(content);
    pasteboard.setStringForType(&string, &NSPasteboardTypeString)
        .map_err(|e| format!("Failed to write to clipboard: {:?}", e))
}

#[cfg(target_os = "linux")]
async fn write_clipboard_linux(content: &str) -> Result<(), String> {
    // Try wl-copy first (Wayland), then xclip (X11)
    let result = tokio::process::Command::new("wl-copy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(content.as_bytes()).map_err(|e| e.into())
            } else {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "No stdin"))
            }
        });

    if result.is_ok() {
        return Ok(());
    }

    tokio::process::Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(content.as_bytes()).map_err(|e| e.into())
            } else {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "No stdin"))
            }
        })
        .map_err(|e| e.to_string())?;

    Ok(())
}