use std::path::Path;

pub struct WindowsOcrProvider;

#[allow(clippy::new_without_default)]
impl WindowsOcrProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn recognize(&self, image_path: &str, _language: &str) -> Result<String, String> {
        let path = Path::new(image_path);
        if !path.exists() {
            return Err("Image file not found".to_string());
        }

        // TODO: implement WinRT Windows.Media.Ocr integration.
        // When implemented, map the Tesseract `_language` code to the
        // corresponding Windows.Media.Ocr language tag.
        Err("Windows OCR is not implemented yet".to_string())
    }
}
