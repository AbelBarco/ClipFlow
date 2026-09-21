use tauri::{AppHandle, Listener, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tracing::info;

pub fn setup(app: AppHandle) -> Result<(), String> {
    // Windows are declared in tauri.conf.json. Only create them programmatically
    // if they are missing (e.g. config changed), otherwise just wire behaviour.
    ensure_window(&app, "main", "ClipFlow", 520.0, 600.0, true, false)?;
    ensure_window(&app, "spotlight", "ClipFlow Spotlight", 560.0, 400.0, false, true)?;
    ensure_window(&app, "settings", "ClipFlow Settings", 520.0, 500.0, true, false)?;

    // Wire close-to-hide behaviour for all windows.
    for label in ["main", "spotlight", "settings"] {
        if let Some(window) = app.get_webview_window(label) {
            let win = window.clone();
            let is_spotlight = label == "spotlight";
            window.on_window_event(move |event| match event {
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = win.hide();
                }
                WindowEvent::Focused(false) if is_spotlight => {
                    let _ = win.hide();
                }
                _ => {}
            });
        }
    }

    // Listen for show events from tray / global shortcut.
    let app_main = app.clone();
    app.listen("show-main-window", move |_| {
        show_main_window(&app_main).unwrap_or(());
    });
    let app_spot = app.clone();
    app.listen("show-spotlight", move |_| {
        show_spotlight_window(&app_spot).unwrap_or(());
    });
    let app_set = app.clone();
    app.listen("show-settings", move |_| {
        show_settings_window(&app_set).unwrap_or(());
    });

    info!("Windows initialized");
    Ok(())
}

fn ensure_window(
    app: &AppHandle,
    label: &str,
    title: &str,
    width: f64,
    height: f64,
    decorations: bool,
    always_on_top: bool,
) -> Result<(), String> {
    if app.get_webview_window(label).is_some() {
        return Ok(());
    }

    let mut builder = WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
        .title(title)
        .inner_size(width, height)
        .resizable(true)
        .decorations(decorations)
        .always_on_top(always_on_top)
        .skip_taskbar(label == "spotlight")
        .visible(false)
        .center();

    if label == "spotlight" {
        builder = builder.focused(true);
    }

    builder.build().map_err(|e| e.to_string())?;
    info!("{} window created", label);
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
