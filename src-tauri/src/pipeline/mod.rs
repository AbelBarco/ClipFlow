pub mod detector;
pub mod dedupe;
pub mod color_parser;
pub mod transformers;

use crate::pipeline::detector::detect_type;

pub fn process_clipboard_content(content: &[u8], mime_type: Option<&str>) -> ClipItem {
    let content_str = String::from_utf8_lossy(content).to_string();
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

/// Build a history item for a captured image stored at `path`.
pub fn process_image_content(path: String, width: usize, height: usize, byte_len: usize) -> ClipItem {
    ClipItem {
        id: uuid::Uuid::new_v4().to_string(),
        r#type: "image".to_string(),
        content: path,
        preview: format!("Imagen {}×{} px", width, height),
        timestamp: chrono::Utc::now().timestamp_millis(),
        size: byte_len as i64,
        ocr_text: None,
        metadata: Some(serde_json::json!({ "width": width, "height": height })),
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

#[cfg(test)]
mod tests {
    use super::process_image_content;

    #[test]
    fn builds_image_item() {
        let item = process_image_content("/tmp/x.png".to_string(), 800, 600, 123);
        assert_eq!(item.r#type, "image");
        assert_eq!(item.preview, "Imagen 800×600 px");
        assert!(!item.id.is_empty());
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