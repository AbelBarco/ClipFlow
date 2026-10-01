pub mod clipboard;
pub mod commands;
pub mod config;
pub mod hotkeys;
pub mod ocr;
pub mod pipeline;
pub mod storage;
pub mod tray;
pub mod windows;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("clipflow=info")),
        )
        .try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:clipflow.db", vec![])
                .build(),
        )
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    use tauri_plugin_global_shortcut::ShortcutState;
                    if event.state == ShortcutState::Pressed {
                        use tauri::Emitter;
                        let _ = app.emit("show-spotlight", ());
                        crate::windows::show_spotlight_window(app).unwrap_or(());
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::clipboard_commands::clipboard_get_history,
            commands::clipboard_commands::clipboard_delete_item,
            commands::clipboard_commands::clipboard_clear_history,
            commands::clipboard_commands::clipboard_paste_item,
            commands::clipboard_commands::clipboard_copy_to_clipboard,
            commands::clipboard_commands::clipboard_get_image,
            commands::clipboard_commands::clipboard_get_item,
            commands::color_commands::color_convert,
            commands::color_commands::color_detect,
            commands::transform_commands::transform_apply,
            commands::transform_commands::transform_list,
            commands::ocr_commands::ocr_run_ocr,
            commands::ocr_commands::run_ocr,
            commands::settings_commands::settings_get,
            commands::settings_commands::settings_set,
            commands::settings_commands::settings_reset,
            commands::settings_commands::get_config,
            commands::settings_commands::set_config,
            commands::settings_commands::reset_config,
            commands::security_commands::security_set_encryption,
            commands::security_commands::security_status,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            // Initialize storage (async -> block)
            tauri::async_runtime::block_on(async {
                if let Err(e) = storage::db::init_db(&handle).await {
                    tracing::error!("Failed to init database: {}", e);
                }
            });

            // Start clipboard watcher (spawn, don't block)
            {
                let h = handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = clipboard::watcher::start_watcher(h).await {
                        tracing::error!("Failed to start clipboard watcher: {}", e);
                    }
                });
            }

            // Register default global shortcut (Ctrl+Shift+V).
            // Failures here shouldn't prevent the app from starting
            // (e.g. shortcut already taken).
            if let Err(e) = hotkeys::register_global_shortcut(&handle) {
                tracing::warn!("Global shortcut registration failed: {}", e);
            }

            // Setup system tray (tolerate failure in dev without icon)
            if let Err(e) = tray::setup(&handle) {
                tracing::warn!("Tray setup failed: {}", e);
            }

            // Setup windows (idempotent — windows exist via tauri.conf.json)
            if let Err(e) = windows::setup(handle.clone()) {
                tracing::error!("Window setup failed: {}", e);
            }

            // Show main window on first start so the UI is visible.
            // (Spotlight/settings stay hidden until invoked.)
            crate::windows::show_main_window(&handle).unwrap_or(());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
