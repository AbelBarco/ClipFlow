use std::path::Path;

pub struct WindowsOcrProvider;

impl WindowsOcrProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn recognize(&self, image_path: &str) -> Result<String, String> {
        let path = Path::new(image_path);
        if !path.exists() {
            return Err("Image file not found".to_string());
        }

        // TODO: implement WinRT Windows.Media.Ocr integration.
        Err("Windows OCR is not implemented yet".to_string())
    }
}
