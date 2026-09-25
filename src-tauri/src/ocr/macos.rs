use std::path::Path;

pub struct MacOcrProvider;

#[allow(clippy::new_without_default)]
impl MacOcrProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn recognize(&self, image_path: &str, _language: &str) -> Result<String, String> {
        let path = Path::new(image_path);
        if !path.exists() {
            return Err("Image file not found".to_string());
        }

        // TODO: implement Vision VNRecognizeTextRequest integration.
        // When implemented, pass `_language` as the recognition language.
        Err("macOS OCR is not implemented yet".to_string())
    }
}
