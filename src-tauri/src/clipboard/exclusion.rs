use crate::config::app_config::get_excluded_apps;

pub async fn should_exclude(content: &str) -> bool {
    // Check for concealed/password-like content
    if is_concealed_content(content) {
        return true;
    }

    // Check if source app is excluded (best effort; currently disabled
    // because reliable foreground-app detection needs platform-specific
    // native code — content heuristics above cover the main cases).
    if let Ok(excluded_apps) = get_excluded_apps().await {
        if excluded_apps.is_empty() {
            return false;
        }
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
    // TODO: implement foreground-app detection per platform.
    None
}
