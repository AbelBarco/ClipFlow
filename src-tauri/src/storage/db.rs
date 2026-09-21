use rusqlite::{Connection, OpenFlags};
use std::sync::Mutex;
use tauri::Manager;
use once_cell::sync::Lazy;

static DB: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

pub async fn init_db(app: &tauri::AppHandle) -> Result<(), String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    let db_path = app_dir.join("clipflow.db");
    let conn = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_URI,
    ).map_err(|e| e.to_string())?;

    // Enable WAL mode for better concurrency
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
        .map_err(|e| e.to_string())?;

    // Run migrations
    run_migrations(&conn).map_err(|e| e.to_string())?;

    *DB.lock().map_err(|e| e.to_string())? = Some(conn);
    Ok(())
}

fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS clip_items (
            id TEXT PRIMARY KEY,
            type TEXT NOT NULL,
            content TEXT NOT NULL,
            preview TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            size INTEGER NOT NULL,
            ocr_text TEXT,
            metadata TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_clip_items_timestamp ON clip_items(timestamp DESC);
        CREATE INDEX IF NOT EXISTS idx_clip_items_type ON clip_items(type);
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )?;

    Ok(())
}

/// Run a closure with access to the database connection.
/// Serializes all DB access through a single mutex to keep rusqlite usage safe.
pub fn with_db<T, F>(f: F) -> Result<T, String>
where
    F: FnOnce(&Connection) -> Result<T, rusqlite::Error>,
{
    let guard = DB.lock().map_err(|e| format!("DB lock poisoned: {e}"))?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;
    f(conn).map_err(|e| e.to_string())
}

#[cfg(test)]
pub fn init_test_db() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();
    *DB.lock().unwrap() = Some(conn);
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