use crate::pipeline::transformers::{apply_transform, list_transformers};
use tauri::command;

#[command]
pub async fn apply_transform(text: String, transformer: String) -> Result<String, String> {
    apply_transform(&text, &transformer).map_err(|e| e.to_string())
}

#[command]
pub async fn list_transformers() -> Result<Vec<String>, String> {
    Ok(list_transformers())
}