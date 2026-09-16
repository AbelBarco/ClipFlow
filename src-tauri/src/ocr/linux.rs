use super::OcrProvider;
use std::path::Path;
use tokio::process::Command;

pub struct LinuxOcrProvider;

impl LinuxOcrProvider {
    pub fn new() -> Self {
        Self
    }
}

impl OcrProvider for LinuxOcrProvider {
    async fn recognize(&self, image_path: &str) -> Result<String, String> {
        let path = Path::new(image_path);
        if !path.exists() {
            return Err("Image file not found".to_string());
        }

        // Use Tesseract via command line
        let output = Command::new("tesseract")
            .args([image_path, "stdout", "-l", "eng"])
            .output()
            .await
            .map_err(|e| format!("Failed to run tesseract: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Tesseract failed: {}", stderr))
        }
    }
}