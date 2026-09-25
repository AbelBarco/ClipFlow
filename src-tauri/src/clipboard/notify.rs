//! User-visible notice when an item is dropped as a possible secret.
//!
//! The old behavior discarded matches silently, so hashes/UUIDs "vanished".
//! Now the watcher sends one OS notification (rate-limited, opt-out via
//! `notifyOnExclude`, silenced when `showNotifications` is off). Strings are
//! localized in the backend because the frontend notification plugin has no
//! JS binding in this project — the UI language comes from settings.

use crate::config::app_config::get_config;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

// At most one exclusion notice per interval — copying a password manager
// dump must not machine-gun the notification center.
const NOTIFY_COOLDOWN: Duration = Duration::from_secs(60);

static LAST_NOTIFY: Mutex<Option<Instant>> = Mutex::new(None);

/// Localized (title, body) for the "possible secret excluded" notice.
fn strings_for(language: &str) -> (&'static str, &'static str) {
    let base = language
        .split(['-', '_'])
        .next()
        .unwrap_or("es")
        .to_lowercase();
    match base.as_str() {
        "en" => (
            "ClipFlow — possible secret skipped",
            "An item looked like a password or token and was not saved. Adjust this in Settings → Excluded applications.",
        ),
        "fr" => (
            "ClipFlow — secret possible ignoré",
            "Un élément ressemblait à un mot de passe ou un jeton et n’a pas été enregistré. Réglages → Applications exclues.",
        ),
        "de" => (
            "ClipFlow — mögliches Geheimnis übersprungen",
            "Ein Element sah wie ein Passwort oder Token aus und wurde nicht gespeichert. Einstellungen → Ausgeschlossene Apps.",
        ),
        "pt" => (
            "ClipFlow — possível segredo ignorado",
            "Um item parecia uma palavra-passe ou token e não foi guardado. Definições → Aplicações excluídas.",
        ),
        "it" => (
            "ClipFlow — possibile segreto ignorato",
            "Un elemento sembrava una password o un token e non è stato salvato. Impostazioni → App escluse.",
        ),
        "zh" => (
            "ClipFlow — 已跳过疑似密钥",
            "某个项目疑似密码或令牌，未保存。可在“设置 → 排除的应用”中调整。",
        ),
        "ja" => (
            "ClipFlow — 機密の可能性のためスキップ",
            "パスワードやトークンの可能性がある項目は保存されませんでした。設定 → 除外するアプリで調整できます。",
        ),
        "ko" => (
            "ClipFlow — 의심 시크릿 건너뜀",
            "비밀번호나 토큰으로 보이는 항목을 저장하지 않았습니다. 설정 → 제외된 앱에서 조정하세요.",
        ),
        "ru" => (
            "ClipFlow — возможный секрет пропущен",
            "Элемент похож на пароль или токен и не был сохранён. Настройки → Исключённые приложения.",
        ),
        _ => (
            "ClipFlow — posible secreto omitido",
            "Un elemento parecía una contraseña o token y no se guardó. Puedes ajustarlo en Ajustes → Aplicaciones excluidas.",
        ),
    }
}

/// Fire-and-forget: never blocks or fails the watcher.
pub async fn notify_secret_excluded(app: &AppHandle) {
    // Rate limit.
    {
        let mut last = LAST_NOTIFY.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        if let Some(prev) = *last {
            if now.duration_since(prev) < NOTIFY_COOLDOWN {
                return;
            }
        }
        *last = Some(now);
    }

    // Honor both the global and the specific toggle.
    let config = get_config().await.unwrap_or_default();
    if !config.general.show_notifications || !config.exclusions.notify_on_exclude {
        return;
    }
    let (title, body) = strings_for(&config.general.language);

    if let Err(e) = app.notification().builder().title(title).body(body).show() {
        tracing::debug!("Exclusion notice could not be shown: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_cover_all_ui_locales() {
        for lang in ["es", "en", "fr", "de", "pt", "it", "zh", "ja", "ko", "ru"] {
            let (title, body) = strings_for(lang);
            assert!(!title.is_empty() && !body.is_empty(), "{lang}");
        }
        // BCP-47 variants fall back to the base language.
        assert_eq!(strings_for("es-ES").0, strings_for("es").0);
        assert_eq!(strings_for("zh-CN").0, strings_for("zh").0);
    }
}
