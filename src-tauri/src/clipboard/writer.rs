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

async fn write_text_to_clipboard(content: &str) -> Result<(), String> {
    let content = content.to_string();
    tokio::task::spawn_blocking(move || {
        let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        clipboard.set_text(content).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

async fn write_image_to_clipboard(path: &str) -> Result<(), String> {
    let path = path.to_string();
    tokio::task::spawn_blocking(move || {
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
    .map_err(|e| e.to_string())?
}
