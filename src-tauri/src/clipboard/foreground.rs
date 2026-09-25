//! Foreground-application detection for per-app clipboard exclusion.
//!
//! Cross-platform via `active-win-pos-rs`:
//!
//! - Windows: `GetForegroundWindow` + owning process name.
//! - macOS: `NSWorkspace.frontmostApplication`.
//! - Linux: X11 active window, plus Hyprland/KWin native Wayland support.
//!
//! On compositors without an API the query returns `None` and exclusion
//! degrades to content heuristics only (documented in `SECURITY.md`).
//!
//! `None` always means "unknown", never "safe".

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForegroundApp {
    /// Lowercased process name, e.g. `1password`, `bitwarden`.
    pub process_name: String,
    /// Application name as reported by the OS (may be empty on X11).
    pub app_name: String,
    /// Raw window title (may contain PII — never logged verbatim).
    pub title: String,
}

/// Best-effort snapshot of the currently focused application.
///
/// Returns `None` when the platform cannot provide one or the query fails.
/// Callers must treat `None` as "unknown", never as "safe".
pub async fn get_foreground_app() -> Option<ForegroundApp> {
    tokio::task::spawn_blocking(|| {
        let window = active_win_pos_rs::get_active_window().ok()?;
        // Prefer the executable stem (`…/1password.exe` → `1password`);
        // fall back to the reported application name.
        let from_path = window
            .process_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .trim()
            .to_lowercase();
        let process_name = if from_path.is_empty() {
            window.app_name.trim().to_lowercase()
        } else {
            from_path
        };
        if process_name.is_empty() && window.title.trim().is_empty() {
            return None;
        }
        Some(ForegroundApp {
            process_name,
            app_name: window.app_name,
            title: window.title,
        })
    })
    .await
    .ok()
    .flatten()
}

/// Case-insensitive "process/app/title contains entry" match.
///
/// Entries are user-controlled (`1Password`, `Bitwarden`, …) and are
/// matched against the process name, the OS app name and the window title
/// so that `KeePass` matches `keepassxc` as well as a `KeePassXC` title.
pub fn matches_excluded(app: &ForegroundApp, excluded_apps: &[String]) -> Option<String> {
    let app_name_lower = app.app_name.to_lowercase();
    let title_lower = app.title.to_lowercase();
    for entry in excluded_apps {
        let needle = entry.trim().to_lowercase();
        if needle.is_empty() {
            continue;
        }
        if app.process_name.contains(&needle)
            || app_name_lower.contains(&needle)
            || title_lower.contains(&needle)
        {
            return Some(entry.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(process: &str, title: &str) -> ForegroundApp {
        ForegroundApp {
            process_name: process.to_string(),
            app_name: String::new(),
            title: title.to_string(),
        }
    }

    #[test]
    fn matches_process_name_case_insensitively() {
        let excluded = vec!["1Password".to_string(), "Bitwarden".to_string()];
        let hit = matches_excluded(&app("1password", "Vault"), &excluded);
        assert_eq!(hit.as_deref(), Some("1Password"));
    }

    #[test]
    fn matches_window_title() {
        let excluded = vec!["KeePass".to_string()];
        let hit = matches_excluded(&app("unknown-host", "KeePassXC — Database"), &excluded);
        assert_eq!(hit.as_deref(), Some("KeePass"));
    }

    #[test]
    fn matches_os_app_name() {
        let excluded = vec!["Bitwarden".to_string()];
        let mut a = app("", "Login");
        a.app_name = "Bitwarden".to_string();
        let hit = matches_excluded(&a, &excluded);
        assert_eq!(hit.as_deref(), Some("Bitwarden"));
    }

    #[test]
    fn no_match_returns_none() {
        let excluded = vec!["1Password".to_string()];
        assert!(matches_excluded(&app("notepad", "Untitled"), &excluded).is_none());
    }

    #[test]
    fn empty_entries_are_ignored() {
        let excluded = vec!["   ".to_string()];
        assert!(matches_excluded(&app("app", "App"), &excluded).is_none());
    }
}
