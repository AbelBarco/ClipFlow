use crate::storage::crypto::{decrypt_from_storage, encrypt_to_storage};
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

/// What happened when a freshly copied item reached storage.
#[derive(Debug, Clone)]
pub struct AddClipOutcome {
    /// False when the content already existed and was just refreshed.
    pub inserted: bool,
    /// The stored row (decrypted) the UI should display/move to top.
    pub item: ClipItem,
}

async fn history_limit() -> i64 {
    crate::config::app_config::get_config()
        .await
        .map(|c| c.general.max_history_items as i64)
        .unwrap_or(500)
        .clamp(50, 5000)
}

async fn encryption_enabled() -> bool {
    crate::config::app_config::get_config()
        .await
        .map(|c| c.security.encryption_enabled)
        .unwrap_or(false)
}

pub async fn add_clip(item: ClipItem) -> Result<AddClipOutcome, String> {
    let max_items = history_limit().await;
    let encrypt = encryption_enabled().await;

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

    // Encrypt BEFORE touching the DB so plaintext never hits the page cache
    // when the user opted into encryption. Errors are fatal here on
    // purpose: silently storing plaintext while the UI shows "encrypted"
    // would be worse than failing loudly.
    let stored_content = maybe_encrypt(&content, encrypt)?;
    let stored_ocr = maybe_encrypt(&ocr_text, encrypt)?;

    with_db(|conn| {
        // Re-copy of known content: refresh the existing row (new timestamp)
        // instead of ignoring it — the item moves back to the top.
        let existing: Option<String> = conn
            .query_row(
                "SELECT id FROM clip_items WHERE content = ?1 ORDER BY timestamp DESC LIMIT 1",
                params![stored_content],
                |row| row.get(0),
            )
            .ok();
        if let Some(existing_id) = existing {
            conn.execute(
                "UPDATE clip_items SET timestamp = ?1 WHERE id = ?2",
                params![timestamp, existing_id],
            )?;
            let refreshed =
                get_record(conn, &existing_id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
            return Ok(AddClipOutcome {
                inserted: false,
                item: decrypt_record(refreshed)?,
            });
        }

        conn.execute(
            "INSERT INTO clip_items (id, type, content, preview, timestamp, size, ocr_text, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id,
                clip_type,
                stored_content,
                preview,
                timestamp,
                size,
                stored_ocr,
                metadata
            ],
        )?;

        // Rotate if needed (keep the configured max)
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM clip_items", [], |row| row.get(0))?;
        if count > max_items {
            conn.execute(
                "DELETE FROM clip_items WHERE id IN (
                    SELECT id FROM clip_items ORDER BY timestamp ASC LIMIT ?1
                )",
                params![count - max_items],
            )?;
        }

        Ok(AddClipOutcome {
            inserted: true,
            item: ClipItem {
                id,
                r#type: clip_type,
                content,
                preview,
                timestamp,
                size,
                ocr_text: item.ocr_text.clone(),
                metadata: item.metadata.clone(),
            },
        })
    })
}

fn maybe_encrypt(value: &str, encrypt: bool) -> Result<String, String> {
    if value.is_empty() || !encrypt {
        return Ok(value.to_string());
    }
    encrypt_to_storage(value)
}

fn decrypt_record(record: ClipRecord) -> Result<ClipItem, rusqlite::Error> {
    let content = decrypt_from_storage(&record.content).map_err(|e| {
        tracing::error!("Cannot decrypt stored clip {}: {e}", record.id);
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, e.into())
    })?;
    let ocr_text = record
        .ocr_text
        .map(|o| {
            if o.is_empty() {
                Ok(String::new())
            } else {
                decrypt_from_storage(&o)
            }
        })
        .transpose()
        .map_err(|e: String| {
            rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, e.into())
        })?;
    Ok(ClipItem::from(ClipRecord {
        content,
        ocr_text,
        ..record
    }))
}

fn get_record(
    conn: &rusqlite::Connection,
    id: &str,
) -> Result<Option<ClipRecord>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, type, content, preview, timestamp, size, ocr_text, metadata
         FROM clip_items WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
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
    match rows.next() {
        Some(record) => Ok(Some(record?)),
        None => Ok(None),
    }
}

/// Fetch a single item by id (decrypted). Used by paste/delete/preview so
/// those paths don't scan the whole table.
pub async fn get_clip_by_id(item_id: &str) -> Result<Option<ClipItem>, String> {
    let item_id = item_id.to_string();
    with_db(|conn| {
        let record = get_record(conn, &item_id)?;
        record.map(decrypt_record).transpose()
    })
}

pub async fn get_all_clips() -> Result<Vec<ClipItem>, String> {
    // The list query is bounded by the user's history setting instead of a
    // hardcoded constant, so Spotlight never pulls more rows than the app
    // is configured to keep.
    let limit = history_limit().await;
    with_db(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, type, content, preview, timestamp, size, ocr_text, metadata
             FROM clip_items ORDER BY timestamp DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit], |row| {
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
            match r {
                Ok(rec) => match decrypt_record(rec) {
                    Ok(item) => clips.push(item),
                    Err(e) => tracing::error!("Skipping undecryptable clip row: {e}"),
                },
                Err(e) => tracing::error!("Skipping malformed clip row: {e}"),
            }
        }
        Ok(clips)
    })
}

pub async fn delete_clip(item_id: &str) -> Result<(), String> {
    // Tombstone first (decrypted content, so it matches future clipboard
    // reads): without this, deleting an item whose content is still in the
    // OS clipboard resurrects it on the next boot.
    if let Ok(Some(item)) = get_clip_by_id(item_id).await {
        tombstone_item(&item);
    }
    let item_id = item_id.to_string();
    with_db(|conn| {
        conn.execute("DELETE FROM clip_items WHERE id = ?1", params![item_id])?;
        Ok(())
    })
}

pub async fn clear_all_clips() -> Result<(), String> {
    // Tombstone everything currently stored, for the same reason as above.
    if let Ok(clips) = get_all_clips().await {
        for item in &clips {
            tombstone_item(item);
        }
    }
    with_db(|conn| {
        conn.execute("DELETE FROM clip_items", [])?;
        Ok(())
    })
}

/// Record a content hash as user-deleted. Namespaced by kind ("t:" text,
/// "i:" image pixels) since both share the table.
fn tombstone_item(item: &ClipItem) {
    let hash = if item.r#type == "image" {
        image_content_hash(&item.content)
    } else {
        Some(text_tombstone_key(&item.content))
    };
    if let Some(hash) = hash {
        tombstone_hash(&hash);
    }
}

/// Tombstone key for text content (stable across restarts).
pub fn text_tombstone_key(content: &str) -> String {
    format!(
        "t:{}",
        crate::pipeline::dedupe::stable_content_hash(content)
    )
}

/// Tombstone key for raw image pixels (stable across re-saves, unlike paths).
pub fn image_tombstone_key(width: usize, height: usize, rgba: &[u8]) -> String {
    format!(
        "i:{}",
        crate::pipeline::dedupe::stable_image_hash(width, height, rgba)
    )
}

/// Stable fingerprint of a stored image file's pixels (paths change on every
/// re-save, pixels don't). `None` when the file is already gone — then there
/// is nothing that could resurrect it anyway.
fn image_content_hash(path: &str) -> Option<String> {
    let img = image::open(path).ok()?;
    let rgba = img.to_rgba8();
    Some(format!(
        "i:{}",
        crate::pipeline::dedupe::stable_image_hash(
            rgba.width() as usize,
            rgba.height() as usize,
            &rgba.into_raw()
        )
    ))
}

fn tombstone_hash(hash: &str) {
    let hash = hash.to_string();
    let now = chrono::Utc::now().timestamp();
    let res = with_db(|conn| {
        conn.execute(
            "INSERT OR IGNORE INTO tombstones (content_hash, deleted_at) VALUES (?1, ?2)",
            rusqlite::params![hash, now],
        )?;
        // Hygiene: forget very old deletions (their content is long gone
        // from any clipboard by now).
        conn.execute(
            "DELETE FROM tombstones WHERE deleted_at < ?1",
            rusqlite::params![now - 90 * 86400],
        )?;
        Ok(())
    });
    if let Err(e) = res {
        tracing::warn!("Could not record delete tombstone: {e}");
    }
}

/// Was this content hash explicitly deleted by the user? Consulted ONLY on
/// the watcher's boot-baseline poll (see watcher.rs): in-session re-copies
/// must always be (re-)added, so later polls never ask.
pub fn is_tombstoned(hash: &str) -> bool {
    with_db(|conn| {
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM tombstones WHERE content_hash = ?1",
            rusqlite::params![hash],
            |row| row.get(0),
        )?;
        Ok(n > 0)
    })
    .unwrap_or(false)
}

pub async fn update_ocr_text(item_id: &str, ocr_text: String) -> Result<(), String> {
    let item_id = item_id.to_string();
    let stored = if encryption_enabled().await && !ocr_text.is_empty() {
        encrypt_to_storage(&ocr_text)?
    } else {
        ocr_text
    };
    with_db(|conn| {
        conn.execute(
            "UPDATE clip_items SET ocr_text = ?1 WHERE id = ?2",
            params![stored, item_id],
        )?;
        Ok(())
    })
}

/// Re-encrypt (`enable = true`) or decrypt (`false`) every stored row.
///
/// Idempotent and restartable: the `cf1:` envelope marks encrypted rows,
/// so re-running after an interruption only touches remaining rows.
/// Returns the number of rows converted.
pub async fn migrate_encryption(enable: bool) -> Result<usize, String> {
    let rows: Vec<(String, String, String)> = with_db(|conn| {
        let mut stmt =
            conn.prepare("SELECT id, content, COALESCE(ocr_text, '') FROM clip_items")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    })?;

    let mut converted = 0usize;
    for (id, content, ocr) in rows {
        let (new_content, new_ocr) = if enable {
            let c = if content.is_empty() || crate::storage::crypto::is_encrypted(&content) {
                content.clone()
            } else {
                encrypt_to_storage(&content)?
            };
            let o = if ocr.is_empty() || crate::storage::crypto::is_encrypted(&ocr) {
                ocr.clone()
            } else {
                encrypt_to_storage(&ocr)?
            };
            (c, o)
        } else {
            (decrypt_from_storage(&content)?, decrypt_from_storage(&ocr)?)
        };
        if new_content != content || new_ocr != ocr {
            let idc = id.clone();
            with_db(move |conn| {
                conn.execute(
                    "UPDATE clip_items SET content = ?1, ocr_text = ?2 WHERE id = ?3",
                    params![new_content, new_ocr, idc],
                )?;
                Ok(())
            })?;
            converted += 1;
        }
    }
    Ok(converted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::init_test_db;
    use std::sync::Mutex;

    // Unit tests share one global in-memory DB handle: serialize them.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn test_item(id: &str, content: &str, timestamp: i64) -> ClipItem {
        ClipItem {
            id: id.to_string(),
            r#type: "text".to_string(),
            content: content.to_string(),
            preview: content.to_string(),
            timestamp,
            size: content.len() as i64,
            ocr_text: None,
            metadata: None,
        }
    }

    #[test]
    fn duplicate_copy_refreshes_timestamp() {
        let _guard = TEST_LOCK.lock().unwrap();
        init_test_db();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            clear_all_clips().await.unwrap();
            let first = add_clip(test_item("dup-1", "same content", 1000))
                .await
                .unwrap();
            assert!(first.inserted);
            let second = add_clip(test_item("dup-2", "same content", 2000))
                .await
                .unwrap();
            assert!(!second.inserted);
            // The original row is refreshed, not duplicated.
            assert_eq!(second.item.id, "dup-1");
            assert_eq!(second.item.timestamp, 2000);
            let clips = get_all_clips().await.unwrap();
            assert_eq!(clips.len(), 1);
            assert_eq!(clips[0].id, "dup-1");
            assert_eq!(clips[0].timestamp, 2000);
        });
    }

    #[test]
    fn get_clip_by_id_roundtrip() {
        let _guard = TEST_LOCK.lock().unwrap();
        init_test_db();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            clear_all_clips().await.unwrap();
            add_clip(test_item("by-id", "find me", 3000)).await.unwrap();
            let found = get_clip_by_id("by-id").await.unwrap();
            assert_eq!(found.map(|c| c.content), Some("find me".to_string()));
            assert!(get_clip_by_id("missing").await.unwrap().is_none());
        });
    }

    #[test]
    fn delete_leaves_tombstone() {
        let _guard = TEST_LOCK.lock().unwrap();
        init_test_db();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            clear_all_clips().await.unwrap();
            add_clip(test_item("t-1", "gone", 1000)).await.unwrap();
            assert!(!is_tombstoned(&text_tombstone_key("gone")));
            delete_clip("t-1").await.unwrap();
            assert!(get_clip_by_id("t-1").await.unwrap().is_none());
            assert!(is_tombstoned(&text_tombstone_key("gone")));
            assert!(!is_tombstoned(&text_tombstone_key("never deleted")));
        });
    }

    #[test]
    fn clear_tombstones_everything() {
        let _guard = TEST_LOCK.lock().unwrap();
        init_test_db();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            clear_all_clips().await.unwrap();
            add_clip(test_item("c-1", "one", 1000)).await.unwrap();
            add_clip(test_item("c-2", "two", 2000)).await.unwrap();
            clear_all_clips().await.unwrap();
            assert!(get_all_clips().await.unwrap().is_empty());
            assert!(is_tombstoned(&text_tombstone_key("one")));
            assert!(is_tombstoned(&text_tombstone_key("two")));
        });
    }
}
