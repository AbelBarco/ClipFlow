#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

use crate::storage::repository::update_ocr_text;

pub async fn run_ocr(item_id: &str) -> Result<String, String> {
    // Look up the clip; images store a filesystem path in `content`.
    let clips = crate::storage::repository::get_all_clips()
        .await
        .map_err(|e| e.to_string())?;
    let clip = clips
        .into_iter()
        .find(|c| c.id == item_id)
        .ok_or("Item not found")?;

    let image_path = clip.content.clone();
    let text = recognize_image(&image_path).await?;

    // Update database with OCR text
    update_ocr_text(item_id, text.clone()).await?;

    Ok(text)
}

async fn recognize_image(image_path: &str) -> Result<String, String> {
    // The OCR language comes from the user settings ("Idiomas" section).
    // Fall back to English when settings can't be read.
    let language = crate::config::app_config::get_config()
        .await
        .map(|cfg| cfg.ocr.language)
        .unwrap_or_else(|_| "eng".to_string());
    let language = normalize_ocr_language(&language);

    #[cfg(target_os = "linux")]
    {
        return linux::LinuxOcrProvider::new()
            .recognize(image_path, &language)
            .await;
    }
    #[cfg(target_os = "windows")]
    {
        return windows::WindowsOcrProvider::new()
            .recognize(image_path, &language)
            .await;
    }
    #[cfg(target_os = "macos")]
    {
        return macos::MacOcrProvider::new()
            .recognize(image_path, &language)
            .await;
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        let _ = image_path;
        let _ = language;
        Err("OCR not supported on this platform".to_string())
    }
}

/// Maps a (possibly combined, e.g. "eng+spa") Tesseract language code to the
/// primary BCP-47 tag used by native OS engines (WinRT `Windows.Media.Ocr`).
/// Unknown codes fall back to English; callers should then try the engine's
/// user-profile/default language before giving up.
pub(crate) fn tesseract_to_bcp47(code: &str) -> &'static str {
    match code.trim().to_lowercase().split('+').next().unwrap_or("") {
        "spa" | "es" => "es",
        "fra" | "fr" => "fr",
        "deu" | "de" => "de",
        "por" | "pt" => "pt",
        "ita" | "it" => "it",
        "rus" | "ru" => "ru",
        "chi_sim" | "zh" => "zh-Hans",
        "chi_tra" => "zh-Hant",
        "jpn" | "ja" => "ja",
        "kor" | "ko" => "ko",
        "eng" | "en" => "en",
        _ => "en",
    }
}

/// Accepts both Tesseract codes (eng, spa, …) and UI locale codes
/// (en, es, …) and normalizes them to a Tesseract language code.
fn normalize_ocr_language(raw: &str) -> String {
    let lower = raw.trim().to_lowercase();
    // Already a Tesseract code? Keep it (supports "eng+spa" combos too).
    if lower
        .chars()
        .all(|c| c == '+' || c == '_' || c.is_ascii_lowercase())
        && !lower.is_empty()
    {
        const KNOWN: &[&str] = &[
            "eng", "spa", "fra", "deu", "por", "ita", "rus", "chi_sim", "chi_tra", "jpn", "kor",
        ];
        let first = lower.split('+').next().unwrap_or("");
        if KNOWN.contains(&first) || lower.contains('+') {
            return lower;
        }
    }
    match lower.as_str() {
        "en" => "eng".to_string(),
        "es" => "spa".to_string(),
        "fr" => "fra".to_string(),
        "de" => "deu".to_string(),
        "pt" => "por".to_string(),
        "it" => "ita".to_string(),
        "ru" => "rus".to_string(),
        "zh" => "chi_sim".to_string(),
        "ja" => "jpn".to_string(),
        "ko" => "kor".to_string(),
        other if !other.is_empty() => other.to_string(),
        _ => "eng".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_tesseract_codes_to_bcp47() {
        assert_eq!(tesseract_to_bcp47("eng"), "en");
        assert_eq!(tesseract_to_bcp47("spa"), "es");
        assert_eq!(tesseract_to_bcp47("fra"), "fr");
        assert_eq!(tesseract_to_bcp47("deu"), "de");
        assert_eq!(tesseract_to_bcp47("por"), "pt");
        assert_eq!(tesseract_to_bcp47("ita"), "it");
        assert_eq!(tesseract_to_bcp47("rus"), "ru");
        assert_eq!(tesseract_to_bcp47("chi_sim"), "zh-Hans");
        assert_eq!(tesseract_to_bcp47("chi_tra"), "zh-Hant");
        assert_eq!(tesseract_to_bcp47("jpn"), "ja");
        assert_eq!(tesseract_to_bcp47("kor"), "ko");
        // Combos resolve to the primary language…
        assert_eq!(tesseract_to_bcp47("eng+spa"), "en");
        // …UI locale codes pass through, unknowns fall back to English.
        assert_eq!(tesseract_to_bcp47("es"), "es");
        assert_eq!(tesseract_to_bcp47("klingon"), "en");
        assert_eq!(tesseract_to_bcp47(""), "en");
    }

    #[test]
    fn normalizes_ui_locales_to_tesseract() {
        assert_eq!(normalize_ocr_language("es"), "spa");
        assert_eq!(normalize_ocr_language("eng"), "eng");
        assert_eq!(normalize_ocr_language("eng+spa"), "eng+spa");
        assert_eq!(normalize_ocr_language(""), "eng");
    }
}
