pub mod detector;
pub mod dedupe;
pub mod color_parser;
pub mod transformers;

use crate::pipeline::detector::detect_type;
use crate::pipeline::dedupe::content_hash;

pub fn process_clipboard_content(content: &[u8], mime_type: Option<&str>) -> ClipItem {
    let content_str = String::from_utf8_lossy(content).to_string();
    let hash = content_hash(&content_str);
    let clip_type = detect_type(&content_str, mime_type);
    let preview = create_preview(&content_str, &clip_type);

    ClipItem {
        id: uuid::Uuid::new_v4().to_string(),
        r#type: clip_type,
        content: content_str,
        preview,
        timestamp: chrono::Utc::now().timestamp_millis(),
        size: content.len() as i64,
        ocr_text: None,
        metadata: None,
    }
}

fn create_preview(content: &str, clip_type: &str) -> String {
    match clip_type {
        "image" => "[Image]".to_string(),
        "color" => content.to_string(),
        _ => {
            let lines: Vec<&str> = content.lines().collect();
            if lines.len() > 3 {
                format!("{}...", lines[..3].join("\n"))
            } else {
                content.to_string()
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipItem {
    pub id: String,
    pub r#type: String,
    pub content: String,
    pub preview: String,
    pub timestamp: i64,
    pub size: i64,
    pub ocr_text: Option<String>,
    pub metadata: Option<serde_json::Value>,
}