use tauri::Manager;
use uuid::Uuid;
use image::ImageFormat;

pub async fn save_image(app: &tauri::AppHandle, data: &[u8], format: ImageFormat) -> Result<String, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let images_dir = app_dir.join("images");
    std::fs::create_dir_all(&images_dir).map_err(|e| e.to_string())?;

    let filename = format!("{}.{}", Uuid::new_v4(), format_ext(format));
    let path = images_dir.join(&filename);

    std::fs::write(&path, data).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

pub fn format_ext(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        ImageFormat::Gif => "gif",
        ImageFormat::WebP => "webp",
        ImageFormat::Bmp => "bmp",
        ImageFormat::Tiff => "tiff",
        _ => "png",
    }
}

pub async fn load_image(path: &str) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| e.to_string())
}

pub async fn delete_image(path: &str) -> Result<(), String> {
    if std::path::Path::new(path).exists() {
        std::fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}