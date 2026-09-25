pub mod crypto;
pub mod db;
pub mod image_store;
pub mod repository;
pub mod rotation;

use crate::storage::db::init_db;

pub async fn init(app: &tauri::AppHandle) -> Result<(), String> {
    init_db(app).await
}
