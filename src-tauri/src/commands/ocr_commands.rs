use crate::ocr::run_ocr as run_ocr_impl;

#[tauri::command]
#[allow(non_snake_case)]
pub async fn ocr_run_ocr(itemId: String) -> Result<String, String> {
    run_ocr_impl(&itemId).await.map_err(|e| e.to_string())
}

// Keep the old name as an alias so both `run_ocr` and `ocr_run_ocr` work.
#[tauri::command]
#[allow(non_snake_case)]
pub async fn run_ocr(itemId: String) -> Result<String, String> {
    run_ocr_impl(&itemId).await.map_err(|e| e.to_string())
}
