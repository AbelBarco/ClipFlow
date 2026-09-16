#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clipflow::init;

fn main() {
    tauri::Builder::default()
        .plugin(init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}