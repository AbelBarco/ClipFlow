use crate::ocr::{run_ocr, OcrProvider};
use tauri::command;

#[command]
pub async fn run_ocr(item_id: String) -> Result<String, String> {
    run_ocr(&item_id).await.map_err(|e| e.to_string())
}