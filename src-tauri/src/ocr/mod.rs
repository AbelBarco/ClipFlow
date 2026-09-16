pub mod macos;
pub mod windows;
pub mod linux;

use crate::storage::repository::update_ocr_text;
use crate::storage::image_store::load_image;
use tauri::AppHandle;
use std::sync::Arc;
use tokio::sync::Mutex;

pub trait OcrProvider: Send + Sync {
    async fn recognize(&self, image_path: &str) -> Result<String, String>;
}

pub struct OcrEngine {
    provider: Arc<Mutex<Box<dyn OcrProvider>>>,
}

impl OcrEngine {
    pub fn new() -> Self {
        #[cfg(target_os = "macos")]
        let provider = Box::new(macos::MacOcrProvider::new());

        #[cfg(target_os = "windows")]
        let provider = Box::new(windows::WindowsOcrProvider::new());

        #[cfg(target_os = "linux")]
        let provider = Box::new(linux::LinuxOcrProvider::new());

        Self {
            provider: Arc::new(Mutex::new(provider)),
        }
    }

    pub async fn recognize(&self, image_path: &str) -> Result<String, String> {
        let provider = self.provider.lock().await;
        provider.recognize(image_path).await
    }
}

pub async fn run_ocr(item_id: &str) -> Result<String, String> {
    // Get image path from database
    // For now, we'll use a placeholder
    let image_path = format!("/tmp/clipflow_{}.png", item_id);

    let engine = OcrEngine::new();
    let text = engine.recognize(&image_path).await?;

    // Update database with OCR text
    update_ocr_text(item_id, text.clone()).await?;

    Ok(text)
}

pub async fn get_ocr_engine() -> OcrEngine {
    OcrEngine::new()
}