pub mod db;
pub mod repository;
pub mod rotation;
pub mod image_store;

use crate::storage::db::init_db;
use tauri::Manager;

pub async fn init(app: &tauri::AppHandle) -> Result<(), String> {
    init_db(app).await
}