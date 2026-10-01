//! Dictionary spell-check commands (Word-style underline, offline).
//!
//! `corrector_check_text` returns unknown words with suggestions for the
//! manual suggestion list. Findings are never auto-applied — same policy
//! as Word: only the curated AutoCorrect pairs (typo tables) self-apply.

use crate::config::app_config::get_config;
use crate::spell;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DictFindingDto {
    /// UTF-16 offset, like the frontend caret.
    pub index: u32,
    pub length: u32,
    pub word: String,
    pub suggestions: Vec<String>,
}

#[tauri::command]
pub async fn corrector_check_text(
    app: tauri::AppHandle,
    text: String,
    lang: String,
) -> Result<Vec<DictFindingDto>, String> {
    let lang = lang.trim().to_lowercase();
    if !spell::is_supported(&lang) {
        return Ok(Vec::new());
    }
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }
    // Heavy fuzzy work must not block the async runtime.
    let custom: HashSet<String> = get_config()
        .await
        .map(|c| {
            c.corrector
                .custom_words
                .iter()
                .map(|w| w.to_lowercase())
                .collect()
        })
        .unwrap_or_default();
    tokio::task::spawn_blocking(move || {
        if !spell::ensure_loaded(&app, &lang) {
            return Ok(Vec::new());
        }
        let out = spell::with_map(&lang, |map| spell::check_text(map, &custom, &lang, &text))
            .unwrap_or_default();
        Ok(out
            .into_iter()
            .map(|f| DictFindingDto {
                index: f.index,
                length: f.length,
                word: f.word,
                suggestions: f.suggestions,
            })
            .collect())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Teach the checker a word ("add to dictionary", Word-style). Stored
/// lowercased in settings, effective immediately.
#[tauri::command]
pub async fn corrector_learn_word(word: String) -> Result<Vec<String>, String> {
    let clean = word.trim().to_lowercase();
    if clean.chars().count() < 2
        || clean.chars().count() > 40
        || !clean
            .chars()
            .all(|c| c.is_alphabetic() || c == '\'' || c == '\u{2019}' || c == '-')
    {
        return Err("Not a learnable word".to_string());
    }
    let mut config = crate::config::app_config::get_config().await?;
    if !config.corrector.custom_words.iter().any(|w| w == &clean) {
        config.corrector.custom_words.push(clean);
        // Bound the list; oldest entries are the least relevant.
        if config.corrector.custom_words.len() > 2000 {
            let drop = config.corrector.custom_words.len() - 2000;
            config.corrector.custom_words.drain(..drop);
        }
        crate::config::app_config::set_config(config.clone()).await?;
    }
    Ok(config.corrector.custom_words)
}
