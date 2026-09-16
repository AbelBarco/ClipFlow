use crate::config::app_config::get_excluded_apps;
use std::process::Command;

pub async fn should_exclude(content: &str) -> bool {
    // Check for concealed/password-like content
    if is_concealed_content(content) {
        return true;
    }

    // Check if source app is excluded
    if let Ok(excluded_apps) = get_excluded_apps().await {
        if let Some(current_app) = get_current_app().await {
            if excluded_apps.iter().any(|app| current_app.contains(app)) {
                return true;
            }
        }
    }

    false
}

fn is_concealed_content(content: &str) -> bool {
    // Heuristics for detecting passwords/secrets
    let lower = content.to_lowercase();

    // Common password field indicators
    if lower.contains("password") && content.len() < 100 {
        return true;
    }

    // High entropy strings (likely secrets)
    if content.len() > 20 && content.len() < 200 {
        let entropy = calculate_entropy(content);
        if entropy > 4.5 {
            return true;
        }
    }

    // Common secret patterns
    let secret_patterns = [
        "api_key", "apikey", "secret", "token", "private_key",
        "aws_access", "aws_secret", "github_token", "ghp_",
        "sk_live", "rk_live", "pk_live", "-----BEGIN",
    ];

    for pattern in &secret_patterns {
        if lower.contains(pattern) {
            return true;
        }
    }

    false
}

fn calculate_entropy(s: &str) -> f64 {
    let mut freq = std::collections::HashMap::new();
    for c in s.chars() {
        *freq.entry(c).or_insert(0) += 1;
    }

    let len = s.len() as f64;
    freq.values()
        .map(|&count| {
            let p = count as f64 / len;
            -p * p.log2()
        })
        .sum()
}

async fn get_current_app() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        get_current_app_windows().await
    }
    #[cfg(target_os = "macos")]
    {
        get_current_app_macos().await
    }
    #[cfg(target_os = "linux")]
    {
        get_current_app_linux().await
    }
}

#[cfg(target_os = "windows")]
async fn get_current_app_windows() -> Option<String> {
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    use windows::Win32::System::Threading::OpenProcess;
    use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS};
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            return None;
        }

        let mut process_id = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
        let mut entry = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                if entry.th32ProcessID == process_id {
                    let name = OsString::from_wide(&entry.szExeFile)
                        .to_string_lossy()
                        .trim_end_matches('\0')
                        .to_string();
                    return Some(name);
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        None
    }
}

#[cfg(target_os = "macos")]
async fn get_current_app_macos() -> Option<String> {
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::NSRunningApplication;

    let workspace = NSWorkspace::sharedWorkspace();
    let app = workspace.frontmostApplication()?;
    Some(app.localizedName()?.to_string())
}

#[cfg(target_os = "linux")]
async fn get_current_app_linux() -> Option<String> {
    // Try to get active window using xdotool or similar
    let output = Command::new("xdotool")
        .args(["getactivewindow", "getwindowname"])
        .output()
        .await
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}