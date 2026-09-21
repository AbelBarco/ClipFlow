use crate::storage::db::with_db;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug)]
struct ClipRecord {
    id: String,
    r#type: String,
    content: String,
    preview: String,
    timestamp: i64,
    size: i64,
    ocr_text: Option<String>,
    metadata: Option<String>,
}

impl From<ClipRecord> for ClipItem {
    fn from(record: ClipRecord) -> Self {
        Self {
            id: record.id,
            r#type: record.r#type,
            content: record.content,
            preview: record.preview,
            timestamp: record.timestamp,
            size: record.size,
            ocr_text: record.ocr_text.filter(|s| !s.is_empty()),
            metadata: record
                .metadata
                .filter(|s| !s.is_empty())
                .and_then(|m| serde_json::from_str(&m).ok()),
        }
    }
}

impl From<crate::pipeline::ClipItem> for ClipItem {
    fn from(item: crate::pipeline::ClipItem) -> Self {
        Self {
            id: item.id,
            r#type: item.r#type,
            content: item.content,
            preview: item.preview,
            timestamp: item.timestamp,
            size: item.size,
            ocr_text: item.ocr_text,
            metadata: item.metadata,
        }
    }
}

pub async fn add_clip(item: ClipItem) -> Result<(), String> {
    let ocr_text = item.ocr_text.clone().unwrap_or_default();
    let metadata = item
        .metadata
        .clone()
        .map(|m| m.to_string())
        .unwrap_or_default();
    let id = item.id.clone();
    let clip_type = item.r#type.clone();
    let content = item.content.clone();
    let preview = item.preview.clone();
    let timestamp = item.timestamp;
    let size = item.size;

    with_db(|conn| {
        // Skip exact duplicates (same content as most recent with same content)
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM clip_items WHERE content = ?1 LIMIT 1",
                params![content],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if exists {
            return Ok(());
        }

        conn.execute(
            "INSERT INTO clip_items (id, type, content, preview, timestamp, size, ocr_text, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, clip_type, content, preview, timestamp, size, ocr_text, metadata],
        )?;

        // Rotate if needed (keep max 500)
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM clip_items", [], |row| row.get(0))?;
        if count > 500 {
            conn.execute(
                "DELETE FROM clip_items WHERE id IN (
                    SELECT id FROM clip_items ORDER BY timestamp ASC LIMIT ?1
                )",
                params![count - 500],
            )?;
        }

        Ok(())
    })
}

pub async fn get_all_clips() -> Result<Vec<ClipItem>, String> {
    with_db(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, type, content, preview, timestamp, size, ocr_text, metadata
             FROM clip_items ORDER BY timestamp DESC LIMIT 500",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ClipRecord {
                id: row.get(0)?,
                r#type: row.get(1)?,
                content: row.get(2)?,
                preview: row.get(3)?,
                timestamp: row.get(4)?,
                size: row.get(5)?,
                ocr_text: row.get(6)?,
                metadata: row.get(7)?,
            })
        })?;

        let mut clips = Vec::new();
        for r in rows {
            if let Ok(rec) = r {
                clips.push(ClipItem::from(rec));
            }
        }
        Ok(clips)
    })
}

pub async fn delete_clip(item_id: &str) -> Result<(), String> {
    let item_id = item_id.to_string();
    with_db(|conn| {
        conn.execute("DELETE FROM clip_items WHERE id = ?1", params![item_id])?;
        Ok(())
    })
}

pub async fn clear_all_clips() -> Result<(), String> {
    with_db(|conn| {
        conn.execute("DELETE FROM clip_items", [])?;
        Ok(())
    })
}

pub async fn update_ocr_text(item_id: &str, ocr_text: String) -> Result<(), String> {
    let item_id = item_id.to_string();
    with_db(|conn| {
        conn.execute(
            "UPDATE clip_items SET ocr_text = ?1 WHERE id = ?2",
            params![ocr_text, item_id],
        )?;
        Ok(())
    })
}
