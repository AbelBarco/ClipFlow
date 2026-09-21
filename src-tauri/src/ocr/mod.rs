#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "linux")]
pub mod linux;

use crate::storage::repository::update_ocr_text;

pub async fn run_ocr(item_id: &str) -> Result<String, String> {
    // Look up the clip; images store a filesystem path in `content`.
    let clips = crate::storage::repository::get_all_clips()
        .await
        .map_err(|e| e.to_string())?;
    let clip = clips
        .into_iter()
        .find(|c| c.id == item_id)
        .ok_or("Item not found")?;

    let image_path = clip.content.clone();
    let text = recognize_image(&image_path).await?;

    // Update database with OCR text
    update_ocr_text(item_id, text.clone()).await?;

    Ok(text)
}

async fn recognize_image(image_path: &str) -> Result<String, String> {
    #[cfg(target_os = "linux")]
    {
        return linux::LinuxOcrProvider::new().recognize(image_path).await;
    }
    #[cfg(target_os = "windows")]
    {
        return windows::WindowsOcrProvider::new()
            .recognize(image_path)
            .await;
    }
    #[cfg(target_os = "macos")]
    {
        return macos::MacOcrProvider::new().recognize(image_path).await;
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        let _ = image_path;
        Err("OCR not supported on this platform".to_string())
    }
}
