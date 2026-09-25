use tauri::{AppHandle, Listener, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_store::StoreExt;
use tracing::info;

const WINDOW_STATE_STORE: &str = "window-state.json";
const SPOTLIGHT_POS_KEY: &str = "spotlight-position";

pub fn setup(app: AppHandle) -> Result<(), String> {
    // Windows are declared in tauri.conf.json. Only create them programmatically
    // if they are missing (e.g. config changed), otherwise just wire behaviour.
    ensure_window(&app, "main", "ClipFlow", 520.0, 600.0, true, false)?;
    ensure_window(
        &app,
        "spotlight",
        "ClipFlow Spotlight",
        560.0,
        400.0,
        false,
        true,
    )?;
    ensure_window(
        &app,
        "settings",
        "ClipFlow Settings",
        520.0,
        500.0,
        true,
        false,
    )?;

    // Wire close-to-hide behaviour for all windows.
    for label in ["main", "spotlight", "settings"] {
        if let Some(window) = app.get_webview_window(label) {
            let win = window.clone();
            let app_handle = app.clone();
            let is_spotlight = label == "spotlight";
            if is_spotlight {
                // Reopen where the user left it (draggable, see spotlight view).
                restore_spotlight_position(&app_handle, &win);
            }
            window.on_window_event(move |event| match event {
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    if is_spotlight {
                        persist_spotlight_position(&app_handle, &win);
                    }
                    let _ = win.hide();
                }
                WindowEvent::Focused(false) if is_spotlight => {
                    persist_spotlight_position(&app_handle, &win);
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

/// Remember where the user dragged the spotlight window so it reopens in
/// the comfortable spot instead of jumping back to the screen center.
fn persist_spotlight_position(app: &AppHandle, window: &tauri::WebviewWindow) {
    let position = match window.outer_position() {
        Ok(pos) => pos,
        Err(e) => {
            tracing::debug!("Could not read spotlight position: {e}");
            return;
        }
    };
    match app.store(WINDOW_STATE_STORE) {
        Ok(store) => {
            store.set(
                SPOTLIGHT_POS_KEY,
                serde_json::json!({ "x": position.x, "y": position.y }),
            );
            if let Err(e) = store.save() {
                tracing::debug!("Could not save spotlight position: {e}");
            }
        }
        Err(e) => tracing::debug!("Window-state store unavailable: {e}"),
    }
}

/// Restore the saved spotlight position when it still fits on a connected
/// monitor (guards against stale coordinates after monitor changes);
/// otherwise leave the default centered position.
fn restore_spotlight_position(app: &AppHandle, window: &tauri::WebviewWindow) {
    let store = match app.store(WINDOW_STATE_STORE) {
        Ok(store) => store,
        Err(_) => return,
    };
    let saved = match store.get(SPOTLIGHT_POS_KEY) {
        Some(value) => value,
        None => return,
    };
    let (x, y) = match (
        saved.get("x").and_then(|v| v.as_i64()),
        saved.get("y").and_then(|v| v.as_i64()),
    ) {
        (Some(x), Some(y)) => (x as i32, y as i32),
        _ => return,
    };

    let on_screen = window
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .any(|monitor| {
            let origin = monitor.position();
            let size = monitor.size();
            // Tolerate partially off-screen positions (shadows, rounding).
            const MARGIN: i32 = 80;
            x + MARGIN >= origin.x
                && y + MARGIN >= origin.y
                && x - MARGIN < origin.x + size.width as i32
                && y - MARGIN < origin.y + size.height as i32
        });
    if !on_screen {
        return;
    }

    if let Err(e) = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
    {
        tracing::debug!("Could not restore spotlight position: {e}");
    }
}
