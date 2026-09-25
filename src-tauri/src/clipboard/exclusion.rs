//! Clipboard exclusion engine: per-app focus detection + secret heuristics.
//!
//! Two independent layers (both best-effort, fail-open with a log):
//! 1. **Application exclusion** — the focused app (process/title) is matched
//!    against the user's list. Backed by [`crate::clipboard::foreground`].
//! 2. **Concealed-content heuristics** — password/secret-shaped text is
//!    skipped. Gated by `respectConcealed` and tuned by `heuristicLevel`
//!    (`conservative` default, `standard` legacy). Common non-secret shapes
//!    (UUIDs, hex hashes, JWTs, publishable IDs) are allow-listed so normal
//!    copies — git SHAs, Stripe `cus_`/`pk_` IDs — are never dropped.

use crate::clipboard::foreground::{get_foreground_app, matches_excluded};
use crate::config::app_config::{get_config, AppConfig, ExclusionSettings};

/// Why a clipboard item was dropped (or not).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExclusionDecision {
    Keep,
    /// Copied while `matched_entry`'s app was focused.
    ExcludedApp {
        matched_entry: String,
    },
    /// Looks like a secret (`reason` is a short machine-readable tag).
    ConcealedHeuristic {
        reason: &'static str,
    },
}

impl ExclusionDecision {
    pub fn excluded(&self) -> bool {
        !matches!(self, ExclusionDecision::Keep)
    }
}

/// Load settings (fresh-install defaults on error) and decide.
pub async fn decide(content: &str) -> ExclusionDecision {
    let config = get_config().await.unwrap_or_default();
    decide_with_config(content, &config).await
}

pub async fn decide_with_config(content: &str, config: &AppConfig) -> ExclusionDecision {
    // Layer 1: focused-app exclusion (always active, independent toggle).
    let excluded_apps = &config.exclusions.excluded_apps;
    if !excluded_apps.is_empty() {
        if let Some(app) = get_foreground_app().await {
            if let Some(matched) = matches_excluded(&app, excluded_apps) {
                return ExclusionDecision::ExcludedApp {
                    matched_entry: matched,
                };
            }
        }
    }

    // Layer 2: content heuristics — honors the user's toggle.
    if config.exclusions.respect_concealed {
        if let Some(reason) = concealed_reason(content, &config.exclusions) {
            return ExclusionDecision::ConcealedHeuristic { reason };
        }
    }

    ExclusionDecision::Keep
}

/// Backwards-compatible boolean check (kept for existing callers/tests).
pub async fn should_exclude(content: &str) -> bool {
    decide(content).await.excluded()
}

/// Pure heuristic core — fully unit-testable.
fn concealed_reason(content: &str, settings: &ExclusionSettings) -> Option<&'static str> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Allow-list first: shapes that look random but are routinely copied
    // as *non-secrets* (git SHAs, UUIDs, Stripe object/publishable IDs,
    // truncated JWTs pasted for debugging).
    if looks_like_non_secret(trimmed) {
        return None;
    }

    let lower = trimmed.to_lowercase();

    // Short texts explicitly mentioning passwords (field labels, prompts).
    if lower.contains("password") && trimmed.len() < 100 {
        return Some("password-indicator");
    }

    // High-entropy blobs (likely generated secrets/tokens).
    let (min_len, max_len, min_entropy) = match settings.heuristic_level.as_str() {
        "standard" => (20, 200, 4.5),
        _ => (32, 200, 5.5), // "conservative" (default) and unknown values
    };
    if trimmed.len() > min_len && trimmed.len() < max_len {
        // Single-line only: prose with spaces/newlines is not a token.
        if !trimmed.contains([' ', '\t', '\n', '\r']) && calculate_entropy(trimmed) > min_entropy {
            return Some("high-entropy");
        }
    }

    // Known secret markers. Note: publishable/test IDs (`pk_live_`,
    // `pk_test_`, `cus_`, …) are deliberately NOT here — see allow-list.
    const SECRET_PATTERNS: &[&str] = &[
        "api_key",
        "apikey",
        "secret",
        "private_key",
        "aws_access",
        "aws_secret",
        "github_token",
        "ghp_",
        "gho_",
        "sk_live",
        "sk_test",
        "rk_live",
        "rk_test",
        "-----begin",
        "token",
    ];
    for pattern in SECRET_PATTERNS {
        if lower.contains(pattern) {
            return Some("secret-pattern");
        }
    }

    None
}

/// True for random-looking but non-secret identifiers.
///
/// Checked on the *trimmed, lowercased* text with simple structural tests
/// (no regex crate needed here — the hot path runs on every copy).
fn looks_like_non_secret(text: &str) -> bool {
    let lower = text.to_lowercase();

    // UUID v4-shaped: 8-4-4-4-12 hex.
    if is_uuid(&lower) {
        return true;
    }

    // Pure hex, 7..=128 chars (short/long git SHAs, MD5/SHA digests).
    if (7..=128).contains(&text.len()) && text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return true;
    }

    // JWT-shaped: three base64url segments.
    if is_jwt_shaped(text) {
        return true;
    }

    // Stripe-style non-secret IDs: pk_live_/pk_test_ (publishable),
    // cus_/in_/pi_/ch_/sub_/prod_/price_/acct_ (object references).
    const NON_SECRET_PREFIXES: &[&str] = &[
        "pk_live_", "pk_test_", "cus_", "in_", "pi_", "ch_", "sub_", "prod_", "price_", "acct_",
    ];
    if NON_SECRET_PREFIXES
        .iter()
        .any(|p| lower.starts_with(p) && lower.len() < 100)
    {
        return true;
    }

    false
}

fn is_uuid(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return false;
    }
    const LENS: [usize; 5] = [8, 4, 4, 4, 12];
    parts
        .iter()
        .zip(LENS)
        .all(|(p, n)| p.len() == n && p.bytes().all(|b| b.is_ascii_hexdigit()))
}

fn is_jwt_shaped(s: &str) -> bool {
    if !(32..=4096).contains(&s.len()) {
        return false;
    }
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    parts.iter().all(|p| {
        (8..=2048).contains(&p.len())
            && p.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::app_config::AppConfig;

    fn conservative() -> ExclusionSettings {
        AppConfig::default().exclusions
    }

    fn standard() -> ExclusionSettings {
        ExclusionSettings {
            heuristic_level: "standard".to_string(),
            ..AppConfig::default().exclusions
        }
    }

    #[test]
    fn normal_text_is_kept() {
        assert_eq!(concealed_reason("hello world", &conservative()), None);
        assert_eq!(
            concealed_reason("https://example.com/some/path", &conservative()),
            None
        );
    }

    #[test]
    fn git_sha_uuid_and_ids_are_not_flagged() {
        // Full + short SHAs, UUID, Stripe non-secrets, truncated JWT.
        assert_eq!(
            concealed_reason("9f2c1ab4e3d84f6a92c1d3e5b7a04f6c8d2e1a3b", &conservative()),
            None
        );
        assert_eq!(concealed_reason("9f2c1ab", &conservative()), None);
        assert_eq!(
            concealed_reason("550e8400-e29b-41d4-a716-446655440000", &conservative()),
            None
        );
        assert_eq!(
            concealed_reason("cus_N8fuwb3k2mQa1Z", &conservative()),
            None
        );
        assert_eq!(
            concealed_reason("pk_live_51H7xYZabc123", &conservative()),
            None
        );
        assert_eq!(
            concealed_reason(
                "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0In0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
                &conservative()
            ),
            None
        );
        // …and also under the legacy standard level.
        assert_eq!(
            concealed_reason("550e8400-e29b-41d4-a716-446655440000", &standard()),
            None
        );
    }

    #[test]
    fn real_secrets_are_still_caught() {
        assert_eq!(
            concealed_reason("sk_live_4eC39HqLyjWDarjtT1zdp7dc", &conservative()),
            Some("secret-pattern")
        );
        assert_eq!(
            concealed_reason(
                "ghp_a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
                &conservative()
            ),
            Some("secret-pattern")
        );
        assert_eq!(
            concealed_reason("my api_key = ABCD1234", &conservative()),
            Some("secret-pattern")
        );
        // High-entropy token: 64 unique symbols => exactly 6.0 bits/char.
        assert_eq!(
            concealed_reason(
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
                &conservative()
            ),
            Some("high-entropy")
        );
    }

    #[test]
    fn conservative_is_stricter_than_standard_about_length() {
        // 26-char mixed token, all symbols unique => ~4.7 bits/char:
        // standard (min 20 chars, > 4.5) flags it, conservative
        // (min 32 chars, > 5.5) lets it through.
        let token = "aB3dE5fG7hJ9kL2mN4pQ6rS8tU";
        assert_eq!(token.len(), 26);
        assert_eq!(concealed_reason(token, &standard()), Some("high-entropy"));
        assert_eq!(concealed_reason(token, &conservative()), None);
    }

    #[test]
    fn empty_and_multiline_prose_are_kept() {
        assert_eq!(concealed_reason("   ", &conservative()), None);
        assert_eq!(
            concealed_reason(
                "line one\nline two with X7gT9pL2vN4qR8sK words",
                &conservative()
            ),
            None
        );
    }
}
