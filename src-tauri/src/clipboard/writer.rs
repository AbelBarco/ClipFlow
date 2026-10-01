/// Write a history item back to the OS clipboard.
/// `content` is plain text for every type except `image`,
/// where it is the absolute path of the stored PNG file.
pub async fn write_to_clipboard(content: &str, mime_type: &str) -> Result<(), String> {
    if mime_type == "image" {
        write_image_to_clipboard(content).await
    } else {
        write_text_to_clipboard(content).await
    }
}

/// Synthesize the platform paste shortcut (Ctrl+V, Cmd+V on macOS) so the
/// content lands in the previously focused app. Returns false — never fails —
/// when key synthesis is unavailable (e.g. Wayland compositors without the
/// right portal): the clipboard already holds the content, so the caller can
/// fall back to "paste it yourself with Ctrl+V".
pub async fn synthesize_paste() -> bool {
    tokio::task::spawn_blocking(|| {
        use enigo::{Direction, Enigo, Key, Keyboard, Settings};
        let mut enigo = match Enigo::new(&Settings::default()) {
            Ok(e) => e,
            Err(e) => {
                tracing::debug!("Key synthesis unavailable: {e}");
                return false;
            }
        };
        #[cfg(target_os = "macos")]
        let modifier = Key::Meta;
        #[cfg(not(target_os = "macos"))]
        let modifier = Key::Control;
        if enigo.key(modifier, Direction::Press).is_err() {
            return false;
        }
        if enigo.key(Key::Unicode('v'), Direction::Click).is_err() {
            let _ = enigo.key(modifier, Direction::Release);
            return false;
        }
        if enigo.key(modifier, Direction::Release).is_err() {
            return false;
        }
        true
    })
    .await
    .unwrap_or(false)
}

/// The OS clipboard is a shared resource: another app may hold it open for a
/// moment (notably on Windows). Retry briefly instead of failing at once.
/// Each attempt uses a fresh clipboard handle on a blocking thread.
async fn retry_clipboard<F, T>(label: &'static str, op: F) -> Result<T, String>
where
    F: Fn() -> Result<T, String> + Send + Sync + 'static,
    T: Send + 'static,
{
    let op = std::sync::Arc::new(op);
    let mut attempt = 0;
    loop {
        let op = op.clone();
        let res = tokio::task::spawn_blocking(move || op())
            .await
            .map_err(|e| e.to_string())?;
        match res {
            Ok(value) => return Ok(value),
            Err(e) if attempt < 2 => {
                attempt += 1;
                tracing::debug!("{label} busy (attempt {attempt}/3), retrying: {e}");
                tokio::time::sleep(std::time::Duration::from_millis(60)).await;
            }
            Err(e) => return Err(e),
        }
    }
}

async fn write_text_to_clipboard(content: &str) -> Result<(), String> {
    let content = content.to_string();
    retry_clipboard("clipboard write", move || {
        let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        clipboard
            .set_text(content.clone())
            .map_err(|e| e.to_string())
    })
    .await
}

async fn write_image_to_clipboard(path: &str) -> Result<(), String> {
    let path = path.to_string();
    retry_clipboard("image write", move || {
        let img = image::open(&path).map_err(|e| format!("No se pudo leer la imagen: {e}"))?;
        let rgba = img.to_rgba8();
        let (w, h) = (rgba.width() as usize, rgba.height() as usize);
        let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        clipboard
            .set_image(arboard::ImageData {
                width: w,
                height: h,
                bytes: rgba.into_raw().into(),
            })
            .map_err(|e| format!("No se pudo copiar la imagen: {e}"))?;
        Ok(())
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rt() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    fn read_text() -> Result<String, String> {
        let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        clipboard.get_text().map_err(|e| e.to_string())
    }

    /// Real OS clipboard round-trip. Skips gracefully where no clipboard
    /// backend exists (headless CI): it proves the copy path on dev machines
    /// without failing elsewhere. Restores the previous content afterwards.
    #[test]
    fn clipboard_text_roundtrip_best_effort() {
        let original = match read_text() {
            Ok(t) => t,
            Err(_) => {
                println!("no clipboard backend — skipped");
                return;
            }
        };
        let probe = format!("clipflow-test-{}", uuid::Uuid::new_v4());
        test_rt().block_on(async {
            write_to_clipboard(&probe, "text")
                .await
                .expect("write to clipboard");
            // A retry loop already ran inside; one direct read must match.
            let back = read_text().expect("read back");
            assert_eq!(back, probe);
            // Restore (best effort, must not fail the test).
            let _ = write_to_clipboard(&original, "text").await;
        });
    }
}
