use super::OcrProvider;
use std::path::Path;

pub struct WindowsOcrProvider;

impl WindowsOcrProvider {
    pub fn new() -> Self {
        Self
    }
}

impl OcrProvider for WindowsOcrProvider {
    async fn recognize(&self, image_path: &str) -> Result<String, String> {
        let path = Path::new(image_path);
        if !path.exists() {
            return Err("Image file not found".to_string());
        }

        // Use Windows.Media.Ocr via WinRT
        // This is a placeholder implementation

        Ok("Windows OCR result placeholder".to_string())
    }
}