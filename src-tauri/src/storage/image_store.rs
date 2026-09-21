use tauri::Manager;
use uuid::Uuid;

pub fn images_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let dir = app_dir.join("images");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub async fn save_image(app: &tauri::AppHandle, data: &[u8], ext: &str) -> Result<String, String> {
    let dir = images_dir(app)?;
    let filename = format!("{}.{}", Uuid::new_v4(), ext);
    let path = dir.join(&filename);
    std::fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

/// Save raw RGBA pixels as a PNG file. Returns the absolute file path.
pub async fn save_rgba_png(
    app: &tauri::AppHandle,
    width: usize,
    height: usize,
    rgba: &[u8],
) -> Result<String, String> {
    let dir = images_dir(app)?;
    let filename = format!("{}.png", Uuid::new_v4());
    let path = dir.join(&filename);

    let (w, h, pixels) = (width, height, rgba.to_vec());
    tokio::task::spawn_blocking(move || {
        let img: image::RgbaImage = image::ImageBuffer::from_raw(w as u32, h as u32, pixels)
            .ok_or("Invalid image dimensions")?;
        img.save(&path).map_err(|e| e.to_string())?;
        Ok::<String, String>(path.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Read an image file and return it as a `data:image/...;base64,...` URL
/// so the webview can display it without extra file-protocol configuration.
pub async fn load_data_url(path: &str) -> Result<String, String> {
    let bytes = tokio::fs::read(path).await.map_err(|e| e.to_string())?;
    let mime = guess_mime(path, &bytes);
    Ok(format!(
        "data:{};base64,{}",
        mime,
        base64::engine::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &bytes
        )
    ))
}

fn guess_mime(path: &str, bytes: &[u8]) -> &'static str {
    // Magic bytes first, extension as fallback.
    if bytes.len() >= 8 && &bytes[..8] == b"\x89PNG\r\n\x1a\n" {
        return "image/png";
    }
    if bytes.len() >= 3 && &bytes[..3] == b"\xff\xd8\xff" {
        return "image/jpeg";
    }
    if bytes.len() >= 6 && (&bytes[..6] == b"GIF87a" || &bytes[..6] == b"GIF89a") {
        return "image/gif";
    }
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return "image/webp";
    }
    if bytes.len() >= 2 && &bytes[..2] == b"BM" {
        return "image/bmp";
    }
    let lower = path.to_lowercase();
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".bmp") {
        "image/bmp"
    } else {
        "image/png"
    }
}

pub async fn load_image(path: &str) -> Result<Vec<u8>, String> {
    tokio::fs::read(path).await.map_err(|e| e.to_string())
}

pub async fn delete_image(path: &str) -> Result<(), String> {
    if std::path::Path::new(path).exists() {
        tokio::fs::remove_file(path).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Remove every file inside the images directory (used on "clear history").
pub async fn clean_images_dir(app: &tauri::AppHandle) -> Result<(), String> {
    let dir = images_dir(app)?;
    let mut entries = tokio::fs::read_dir(&dir).await.map_err(|e| e.to_string())?;
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let _ = tokio::fs::remove_file(entry.path()).await;
    }
    Ok(())
}
