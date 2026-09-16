use crate::storage::db::{get_connection, init_db};
use crate::pipeline::{detector::detect_type, dedupe::content_hash};
use serde::{Deserialize, Serialize};
use tauri::Manager;
use chrono::Utc;
use uuid::Uuid;

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

impl From<crate::storage::repository::ClipRecord> for ClipItem {
    fn from(record: crate::storage::repository::ClipRecord) -> Self {
        Self {
            id: record.id,
            r#type: record.r#type,
            content: record.content,
            preview: record.preview,
            timestamp: record.timestamp,
            size: record.size,
            ocr_text: record.ocr_text,
            metadata: record.metadata,
        }
    }
}

pub async fn add_clip(item: ClipItem) -> Result<(), String> {
    let conn = get_connection().ok_or("Database not initialized")?;
    let conn = conn.lock().unwrap();

    let hash = content_hash(&item.content);

    // Check for duplicate
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM clip_items WHERE content = ?1 LIMIT 1",
            [&hash],
            |_| Ok(true),
        )
        .unwrap_or(false);

    if exists {
        return Ok(()); // Duplicate, ignore
    }

    conn.execute(
        "INSERT INTO clip_items (id, type, content, preview, timestamp, size, ocr_text, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        [
            &item.id,
            &item.r#type,
            &item.content,
            &item.preview,
            &item.timestamp.to_string(),
            &item.size.to_string(),
            &item.ocr_text.unwrap_or_default(),
            &item.metadata.map(|m| m.to_string()).unwrap_or_default(),
        ],
    ).map_err(|e| e.to_string())?;

    // Rotate if needed
    rotate_if_needed(&conn).map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn get_all_clips() -> Result<Vec<ClipItem>, String> {
    let conn = get_connection().ok_or("Database not initialized")?;
    let conn = conn.lock().unwrap();

    let mut stmt = conn.prepare(
        "SELECT id, type, content, preview, timestamp, size, ocr_text, metadata
         FROM clip_items ORDER BY timestamp DESC LIMIT 500"
    ).map_err(|e| e.to_string())?;

    let clips = stmt.query_map([], |row| {
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
    }).map_err(|e| e.to_string())?
    .filter_map(Result::ok)
    .collect();

    Ok(clips)
}

pub async fn delete_clip(item_id: &str) -> Result<(), String> {
    let conn = get_connection().ok_or("Database not initialized")?;
    let conn = conn.lock().unwrap();

    conn.execute("DELETE FROM clip_items WHERE id = ?1", [item_id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn clear_all_clips() -> Result<(), String> {
    let conn = get_connection().ok_or("Database not initialized")?;
    let conn = conn.lock().unwrap();

    conn.execute("DELETE FROM clip_items", [])
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn update_ocr_text(item_id: &str, ocr_text: String) -> Result<(), String> {
    let conn = get_connection().ok_or("Database not initialized")?;
    let conn = conn.lock().unwrap();

    conn.execute(
        "UPDATE clip_items SET ocr_text = ?1 WHERE id = ?2",
        [&ocr_text, item_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

fn rotate_if_needed(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM clip_items", [], |row| row.get(0))?;
    if count > 500 {
        conn.execute(
            "DELETE FROM clip_items WHERE id IN (
                SELECT id FROM clip_items ORDER BY timestamp ASC LIMIT ?1
            )",
            [count - 500],
        )?;
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
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
            ocr_text: record.ocr_text,
            metadata: record.metadata.and_then(|m| serde_json::from_str(&m).ok()),
        }
    }
}