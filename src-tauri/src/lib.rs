pub mod commands;
pub mod config;
pub mod clipboard;
pub mod storage;
pub mod pipeline;
pub mod ocr;
pub mod hotkeys;
pub mod tray;
pub mod windows;

use tauri::Manager;

pub fn init() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("clipflow")
        .invoke_handler(tauri::generate_handler![
            commands::clipboard::get_history,
            commands::clipboard::delete_item,
            commands::clipboard::clear_history,
            commands::clipboard::paste_item,
            commands::clipboard::copy_to_clipboard,
            commands::color::convert_color,
            commands::color::detect_color,
            commands::transform::apply_transform,
            commands::transform::list_transformers,
            commands::ocr::run_ocr,
            commands::settings::get_config,
            commands::settings::set_config,
            commands::settings::reset_config,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            // Initialize storage
            storage::init(&handle)?;

            // Initialize clipboard watcher
            clipboard::watcher::start(handle.clone())?;

            // Register global shortcut
            hotkeys::register_global_shortcut(handle.clone())?;

            // Setup system tray
            tray::setup(handle.clone())?;

            // Setup windows
            windows::setup(handle)?;

            Ok(())
        })
        .build()
}