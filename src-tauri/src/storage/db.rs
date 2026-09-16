use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use once_cell::sync::Lazy;

static DB_POOL: Lazy<Arc<Mutex<Option<Connection>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

pub async fn init_db(app: &tauri::AppHandle) -> Result<(), String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    let db_path = app_dir.join("clipflow.db");
    let conn = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_URI,
    ).map_err(|e| e.to_string())?;

    // Enable WAL mode for better concurrency
    conn.execute("PRAGMA journal_mode=WAL;", []).map_err(|e| e.to_string())?;
    conn.execute("PRAGMA synchronous=NORMAL;", []).map_err(|e| e.to_string())?;

    // Run migrations
    run_migrations(&conn).map_err(|e| e.to_string())?;

    *DB_POOL.lock().unwrap() = Some(conn);
    Ok(())
}

fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS clip_items (
            id TEXT PRIMARY KEY,
            type TEXT NOT NULL,
            content TEXT NOT NULL,
            preview TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            size INTEGER NOT NULL,
            ocr_text TEXT,
            metadata TEXT
        );",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_clip_items_timestamp ON clip_items(timestamp DESC);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_clip_items_type ON clip_items(type);",
        [],
    )?;

    Ok(())
}

pub fn get_connection() -> Option<Arc<Mutex<Connection>>> {
    DB_POOL.lock().unwrap().as_ref().map(|c| Arc::new(Mutex::new(c.try_clone().ok()?)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_path() {
        let dir = std::env::temp_dir().join("clipflow_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.db");
        assert!(path.to_string_lossy().ends_with("test.db"));
    }
}