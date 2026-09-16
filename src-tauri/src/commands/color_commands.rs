use crate::pipeline::color_parser::{convert_color, detect_color};
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorConversion {
    pub hex: String,
    pub rgb: String,
    pub hsl: String,
    pub css: String,
}

#[command]
pub async fn convert_color(input: String) -> Result<ColorConversion, String> {
    convert_color(&input).map_err(|e| e.to_string())
}

#[command]
pub async fn detect_color(input: String) -> Result<bool, String> {
    Ok(detect_color(&input))
}