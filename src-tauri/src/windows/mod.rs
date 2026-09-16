use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tracing::{info, error};

pub fn setup(app: AppHandle) -> Result<(), String> {
    // Create main window (hidden by default)
    create_main_window(&app)?;

    // Create spotlight window (hidden by default)
    create_spotlight_window(&app)?;

    // Create settings window (hidden by default)
    create_settings_window(&app)?;

    Ok(())
}

fn create_main_window(app: &AppHandle) -> Result<(), String> {
    let window = WebviewWindowBuilder::new(
        app,
        "main",
        WebviewUrl::App("main".into()),
    )
    .title("ClipFlow")
    .inner_size(520.0, 600.0)
    .min_inner_size(400.0, 400.0)
    .resizable(true)
    .decorations(true)
    .always_on_top(false)
    .skip_taskbar(false)
    .visible(false)
    .build()
    .map_err(|e| e.to_string())?;

    window.on_window_event(|event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = event.window().hide();
        }
    });

    info!("Main window created");
    Ok(())
}

fn create_spotlight_window(app: &AppHandle) -> Result<(), String> {
    let window = WebviewWindowBuilder::new(
        app,
        "spotlight",
        WebviewUrl::App("spotlight".into()),
    )
    .title("ClipFlow Spotlight")
    .inner_size(560.0, 400.0)
    .min_inner_size(400.0, 300.0)
    .max_inner_size(800.0, 800.0)
    .resizable(false)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .visible(false)
    .focused(true)
    .build()
    .map_err(|e| e.to_string())?;

    window.on_window_event(|event| {
        if let WindowEvent::Focused(false) = event {
            // Hide when focus is lost
            let _ = event.window().hide();
        }
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = event.window().hide();
        }
    });

    info!("Spotlight window created");
    Ok(())
}

fn create_settings_window(app: &AppHandle) -> Result<(), String> {
    let window = WebviewWindowBuilder::new(
        app,
        "settings",
        WebviewUrl::App("settings".into()),
    )
    .title("ClipFlow Settings")
    .inner_size(520.0, 500.0)
    .min_inner_size(400.0, 400.0)
    .resizable(true)
    .decorations(true)
    .always_on_top(false)
    .skip_taskbar(false)
    .visible(false)
    .build()
    .map_err(|e| e.to_string())?;

    window.on_window_event(|event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = event.window().hide();
        }
    });

    info!("Settings window created");
    Ok(())
}

pub fn show_main_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn show_spotlight_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("spotlight") {
        // Position at center of screen
        if let Some(monitor) = window.current_monitor().map_err(|e| e.to_string())? {
            let monitor_size = monitor.size();
            let window_size = window.inner_size().map_err(|e| e.to_string())?;
            let x = (monitor_size.width as f64 - window_size.width as f64) / 2.0;
            let y = (monitor_size.height as f64 - window_size.height as f64) / 3.0; // Higher up for spotlight
            window.set_outer_position(tauri::PhysicalPosition::new(x as i32, y as i32))
                .map_err(|e| e.to_string())?;
        }
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn show_settings_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn hide_all_windows(app: &AppHandle) -> Result<(), String> {
    for label in ["main", "spotlight", "settings"] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.hide();
        }
    }
    Ok(())
}