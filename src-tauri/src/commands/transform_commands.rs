use crate::pipeline::transformers::{
    apply_transform as apply_impl, list_transformers as list_impl,
};

#[tauri::command]
pub async fn transform_apply(text: String, transformer: String) -> Result<String, String> {
    apply_impl(&text, &transformer).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn transform_list() -> Result<Vec<String>, String> {
    Ok(list_impl())
}
