//! Launch-at-startup that actually launches at startup.
//!
//! The `launchAtStartup` toggle used to be a placebo: it persisted a boolean
//! nobody read. Now it is synced to the OS on every settings save/reset and
//! on every app start, via `tauri-plugin-autostart` (registry Run key on
//! Windows, login item on macOS, `.desktop` entry on Linux). Fully local,
//! no network involved.
//!
//! The autostart entry carries `--hidden`, so a login-time launch stays in
//! the tray instead of popping the main window (see `lib.rs` setup).

use tauri_plugin_autostart::ManagerExt;

/// Argument passed to the app when the OS launches it at login.
pub const HIDDEN_ARG: &str = "--hidden";

/// Make the OS autostart state match the user's setting. Idempotent: enabling
/// an already-enabled entry (or vice versa) is harmless, so this runs on
/// every settings save/reset and at startup to heal external drift.
///
/// Returns the effective OS state. On failure the persisted setting is left
/// untouched (callers save first, sync second) and the error is loud so the
/// UI never claims an autostart that doesn't exist.
pub fn sync_launch_at_startup(app: &tauri::AppHandle, enabled: bool) -> Result<bool, String> {
    let autolaunch = app.autolaunch();
    if enabled {
        autolaunch
            .enable()
            .map_err(|e| format!("Could not enable launch at startup: {e}"))?;
    } else {
        autolaunch
            .disable()
            .map_err(|e| format!("Could not disable launch at startup: {e}"))?;
    }
    // Verify what the OS actually reports: enable/disable is best-effort on
    // some platforms (sandboxed Linux packages, managed macOS profiles…).
    match autolaunch.is_enabled() {
        Ok(actual) => {
            if actual != enabled {
                tracing::warn!("Autostart requested {enabled} but OS reports {actual}");
            }
            Ok(actual)
        }
        Err(e) => {
            tracing::warn!("Could not verify autostart state: {e}");
            Ok(enabled)
        }
    }
}

/// True when this process was started by the OS at login (vs. by the user).
/// Used to stay in the tray instead of popping the main window.
pub fn launched_hidden() -> bool {
    std::env::args().any(|arg| arg == HIDDEN_ARG)
}
