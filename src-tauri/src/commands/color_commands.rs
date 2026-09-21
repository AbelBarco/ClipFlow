use crate::pipeline::color_parser::{convert_color as convert_impl, detect_color as detect_impl};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorConversion {
    pub hex: String,
    pub rgb: String,
    pub hsl: String,
    pub css: String,
}

#[tauri::command]
pub async fn color_convert(input: String) -> Result<ColorConversion, String> {
    let c = convert_impl(&input).map_err(|e| e.to_string())?;
    Ok(ColorConversion {
        hex: c.hex,
        rgb: c.rgb,
        hsl: c.hsl,
        css: c.css,
    })
}

#[tauri::command]
pub async fn color_detect(input: String) -> Result<bool, String> {
    Ok(detect_impl(&input))
}
