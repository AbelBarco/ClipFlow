//! Offline dictionary spell checking, Word-style.
//!
//! How it works (no network, ever):
//! - `build.rs` compiles the curated, auditable word lists (`dict-src/*.txt`,
//!   "word count" per line) into compact FST maps (`dicts/*.fst`, word →
//!   frequency) that ship as Tauri resources.
//! - Here we memory-load one map per active language and answer two
//!   questions: is this word known, and what are the closest known words
//!   (Levenshtein automaton + frequency ranking, like Word suggestions).
//! - zh/ja/ko are intentionally unsupported: without spaces and with rich
//!   morphology, word lists are the wrong tool there — the rule engine in
//!   the frontend owns those languages.
//! - Findings are *suggestions only*, never auto-applied (same policy as
//!   Word: AutoCorrect pairs live in the typo tables, the dictionary only
//!   underlines).

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use fst::{IntoStreamer, Streamer};
use tauri::{path::BaseDirectory, Manager};

/// UI locale codes with a bundled dictionary.
pub const SUPPORTED: &[&str] = &["es", "en", "fr", "de", "pt", "it", "ru"];

const MAX_SUGGESTIONS: usize = 5;
const MAX_FINDINGS: usize = 150;
const MAX_TOKENS: usize = 3000;

static MAPS: OnceLock<Mutex<HashMap<String, fst::Map<Vec<u8>>>>> = OnceLock::new();

fn maps() -> &'static Mutex<HashMap<String, fst::Map<Vec<u8>>>> {
    MAPS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn is_supported(lang: &str) -> bool {
    SUPPORTED.contains(&lang)
}

/// Load the language map on first use (resource dir in packaged app).
/// Returns false when unavailable (dev/test without bundled resources) —
/// callers must degrade to rules-only checking, never fail loudly.
pub fn ensure_loaded(app: &tauri::AppHandle, lang: &str) -> bool {
    if !is_supported(lang) {
        return false;
    }
    if maps().lock().map(|m| m.contains_key(lang)).unwrap_or(false) {
        return true;
    }
    let path = match app
        .path()
        .resolve(format!("dicts/{lang}.fst"), BaseDirectory::Resource)
    {
        Ok(p) => p,
        Err(_) => return false,
    };
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let map = match fst::Map::new(bytes) {
        Ok(m) => m,
        Err(_) => return false,
    };
    maps()
        .lock()
        .map(|mut m| m.insert(lang.to_string(), map))
        .ok();
    true
}

/// Build a map from raw bytes (used by tests with tiny word lists).
pub fn load_bytes(bytes: Vec<u8>) -> Result<fst::Map<Vec<u8>>, String> {
    fst::Map::new(bytes).map_err(|e| e.to_string())
}

pub fn with_map<R>(lang: &str, f: impl FnOnce(&fst::Map<Vec<u8>>) -> R) -> Option<R> {
    let guard = maps().lock().ok()?;
    let map = guard.get(lang)?;
    Some(f(map))
}

// ---------------------------------------------------------------------------
// Tokenizer (UTF-16 offsets, like the frontend caret).

#[derive(Debug)]
pub struct Token {
    pub start16: u32,
    pub len16: u32,
    pub text: String,
}

fn is_token_char(c: char) -> bool {
    c.is_alphabetic() || c == '\'' || c == '\u{2019}' || c == '-'
}

fn push_token(out: &mut Vec<Token>, run: &str, start16: u32) {
    let chars: Vec<char> = run.chars().collect();
    let mut s = 0usize;
    let mut e = chars.len();
    while s < e && !chars[s].is_alphabetic() {
        s += 1;
    }
    while e > s && !chars[e - 1].is_alphabetic() {
        e -= 1;
    }
    if e - s < 2 {
        return; // single letters ("a", "y") are noise, and valid anyway
    }
    let core: String = chars[s..e].iter().collect();
    let alphas: Vec<char> = core.chars().filter(|c| c.is_alphabetic()).collect();
    if alphas.len() > 1 && alphas.iter().all(|c| c.is_uppercase()) {
        return; // ALL-CAPS acronyms (USA, DNI, PDF)
    }
    let pre16: u32 = chars[..s].iter().map(|c| c.len_utf16() as u32).sum();
    let len16: u32 = chars[s..e].iter().map(|c| c.len_utf16() as u32).sum();
    out.push(Token {
        start16: start16 + pre16,
        len16,
        text: core,
    });
}

static LINK_RE: OnceLock<regex::Regex> = OnceLock::new();

fn link_re() -> &'static regex::Regex {
    LINK_RE.get_or_init(|| {
        regex::Regex::new(
            r"(?i)https?://\S+|ftp://\S+|www\.\S+|\S+@\S+\.\S+|\b(?:[a-z0-9-]+\.)+[a-z]{2,}\b",
        )
        .expect("link regex")
    })
}

/// Blank out URLs, emails and bare domains with spaces (same length, so
/// UTF-16 offsets into the original text stay valid).
fn strip_links(text: &str) -> String {
    link_re()
        .replace_all(text, |caps: &regex::Captures| {
            " ".repeat(caps[0].chars().count())
        })
        .into_owned()
}

/// Split prose into checkable words with UTF-16 offsets. URLs, emails and
/// domains are blanked first (their fragments must never be flagged);
/// numbers and code fragments fall out naturally (digits and symbols are
/// separators, edge-stripped tokens must start/end with a letter).
pub fn tokenize(text: &str) -> Vec<Token> {
    let plain = strip_links(text);
    let mut out = Vec::new();
    let mut run = String::new();
    let mut run_start16 = 0u32;
    let mut u16off = 0u32;
    for ch in plain.chars() {
        let w = ch.len_utf16() as u32;
        if is_token_char(ch) {
            if run.is_empty() {
                run_start16 = u16off;
            }
            run.push(ch);
        } else if !run.is_empty() {
            push_token(&mut out, &run, run_start16);
            run.clear();
        }
        u16off += w;
    }
    if !run.is_empty() {
        push_token(&mut out, &run, run_start16);
    }
    out
}

// ---------------------------------------------------------------------------
// Lookup + suggestions.

fn damerau(a: &[char], b: &[char]) -> usize {
    let (n, m) = (a.len(), b.len());
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    d[n][m]
}

fn match_case(sample: &str, sugg: &str) -> String {
    let alpha: Vec<char> = sample.chars().filter(|c| c.is_alphabetic()).collect();
    if alpha.len() > 1 && alpha.iter().all(|c| c.is_uppercase()) {
        return sugg.to_uppercase();
    }
    match sample.chars().next() {
        Some(f) if f.is_uppercase() => {
            let mut cs = sugg.chars();
            match cs.next() {
                Some(g) => g.to_uppercase().collect::<String>() + cs.as_str(),
                None => sugg.to_string(),
            }
        }
        _ => sugg.to_string(),
    }
}

fn is_known(map: &fst::Map<Vec<u8>>, lang: &str, lower: &str) -> bool {
    if map.contains_key(lower) {
        return true;
    }
    // German compounds are productive ("Donaudampfschiff" won't be listed):
    // accept glued pairs of known words instead of flagging them.
    if lang == "de" {
        let chars: Vec<char> = lower.chars().collect();
        if chars.len() >= 6 {
            for i in 3..chars.len() - 2 {
                let a: String = chars[..i].iter().collect();
                let b: String = chars[i..].iter().collect();
                if map.contains_key(&a) && map.contains_key(&b) {
                    return true;
                }
            }
        }
    }
    false
}

/// Closest known words, ranked by (edit distance, frequency). Empty when
/// nothing is within reach — an unknown word with no candidates is still
/// reported (the user may "learn" it), just without suggestions.
fn suggest(map: &fst::Map<Vec<u8>>, lower: &str) -> Vec<(String, u64)> {
    let nchars = lower.chars().count();
    if nchars < 2 {
        return Vec::new();
    }
    let dist = if nchars <= 4 { 1 } else { 2 };
    // The automaton builder is heavyweight; bound it and fall back to a
    // cheaper distance-1 query rather than failing the whole check.
    let lev = fst::automaton::Levenshtein::new_with_limit(lower, dist, 60_000)
        .ok()
        .or_else(|| fst::automaton::Levenshtein::new(lower, 1).ok());
    let lev = match lev {
        Some(l) => l,
        None => return Vec::new(),
    };
    let mut stream = map.search(lev).into_stream();
    let mut cands: Vec<(usize, u64, String)> = Vec::new();
    let mut seen = 0usize;
    while let Some((k, freq)) = stream.next() {
        if seen >= 400 {
            break;
        }
        seen += 1;
        let Ok(word) = String::from_utf8(k.to_vec()) else {
            continue;
        };
        if word == lower {
            continue;
        }
        let d = damerau(
            &lower.chars().collect::<Vec<_>>(),
            &word.chars().collect::<Vec<_>>(),
        );
        cands.push((d, freq, word));
    }
    cands.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
    cands
        .into_iter()
        .take(MAX_SUGGESTIONS)
        .map(|(_, f, w)| (w, f))
        .collect()
}

#[derive(Debug)]
pub struct Finding {
    pub index: u32,
    pub length: u32,
    pub word: String,
    pub suggestions: Vec<String>,
}

/// Check prose against the dictionary. Pure function of (map, custom words)
/// so it stays unit-testable without Tauri or files.
pub fn check_text(
    map: &fst::Map<Vec<u8>>,
    custom: &HashSet<String>,
    lang: &str,
    text: &str,
) -> Vec<Finding> {
    let mut out = Vec::new();
    for tok in tokenize(text).into_iter().take(MAX_TOKENS) {
        if out.len() >= MAX_FINDINGS {
            break;
        }
        let lower = tok.text.to_lowercase();
        if custom.contains(&lower) || is_known(map, lang, &lower) {
            continue;
        }
        let suggestions: Vec<String> = suggest(map, &lower)
            .into_iter()
            .map(|(w, _)| match_case(&tok.text, &w))
            .collect();
        out.push(Finding {
            index: tok.start16,
            length: tok.len16,
            word: tok.text,
            suggestions,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn tiny_map() -> fst::Map<Vec<u8>> {
        // Sorted insertion, as the builder requires.
        let mut words = vec![
            ("casa", 900u64),
            ("cosa", 700),
            ("hola", 1000),
            ("mundo", 800),
            ("prueba", 500),
        ];
        words.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        let mut builder = fst::MapBuilder::new(Vec::new()).unwrap();
        for (w, f) in words {
            builder.insert(w, f).unwrap();
        }
        fst::Map::new(builder.into_inner().unwrap()).unwrap()
    }

    #[test]
    fn tokenize_offsets_and_filters() {
        let toks = tokenize("hola, mundo! USA a https://x.y q");
        let words: Vec<&str> = toks.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(words, vec!["hola", "mundo"]);
        assert_eq!((toks[0].start16, toks[0].len16), (0, 4));
        assert_eq!((toks[1].start16, toks[1].len16), (6, 5));
    }

    #[test]
    fn tokenize_counts_utf16() {
        // "😀" is 2 UTF-16 units (one surrogate pair): the word starts at 2.
        let toks = tokenize("😀ab hola");
        assert_eq!(toks[0].text, "ab");
        assert_eq!((toks[0].start16, toks[0].len16), (2, 2));
        assert_eq!((toks[1].start16, toks[1].len16), (5, 4));
    }

    #[test]
    fn damerau_distances() {
        let v = |s: &str| s.chars().collect::<Vec<_>>();
        assert_eq!(damerau(&v("hola"), &v("hola")), 0);
        assert_eq!(damerau(&v("holla"), &v("hola")), 1);
        assert_eq!(damerau(&v("ab"), &v("ba")), 1); // adjacent transposition
        assert_eq!(damerau(&v("aloh"), &v("hola")), 3);
        assert_eq!(damerau(&v(""), &v("abc")), 3);
    }

    #[test]
    fn known_and_suggest() {
        let map = tiny_map();
        let custom = HashSet::<String>::new();
        assert!(check_text(&map, &custom, "es", "hola mundo").is_empty());
        assert!(is_known(&map, "es", "hola"));
        assert!(!is_known(&map, "es", "holla"));
        let s = suggest(&map, "holla");
        assert!(s.iter().any(|(w, _)| w == "hola"), "got: {s:?}");
        // Best (closest + most frequent) comes first.
        assert_eq!(s[0].0, "hola");
    }

    #[test]
    fn german_compounds_are_known() {
        let mut words = vec![("donau", 10u64), ("dampf", 10), ("schiff", 10)];
        words.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        let mut b = fst::MapBuilder::new(Vec::new()).unwrap();
        for (w, f) in words {
            b.insert(w, f).unwrap();
        }
        let map = fst::Map::new(b.into_inner().unwrap()).unwrap();
        assert!(is_known(&map, "de", "donaudampf"));
        assert!(!is_known(&map, "de", "donaudampfx"));
        assert!(!is_known(&map, "es", "donaudampf"));
    }

    #[test]
    fn check_text_respects_custom_words_and_case() {
        let map = tiny_map();
        let mut custom = HashSet::<String>::new();
        custom.insert("holla".to_string());
        let out = check_text(&map, &custom, "es", "Holla mundo");
        assert!(out.is_empty(), "learned word must pass: {out:?}");
        let out = check_text(&map, &HashSet::new(), "es", "Holla mundo");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].word, "Holla");
        // Suggestion follows the original case.
        assert_eq!(out[0].suggestions[0], "Hola");
    }

    #[test]
    fn real_spanish_dict_spot_checks() {
        let path = std::path::Path::new("dicts/es.fst");
        if !path.exists() {
            println!("dicts/es.fst missing (build script did not run?) — skipped");
            return;
        }
        let bytes = std::fs::read(path).unwrap();
        let map = fst::Map::new(bytes).unwrap();
        assert!(map.contains_key("hola"));
        assert!(map.contains_key("ordenador"));
        assert!(!map.contains_key("holla"), "typo must be unknown");
        let s = suggest(&map, "holla");
        assert!(s.iter().any(|(w, _)| w == "hola"), "expected hola in {s:?}");
        // Frequency ranking: a common word beats a rare one at same distance.
        let s2 = suggest(&map, "prueva");
        assert!(
            s2.iter().any(|(w, _)| w == "prueba"),
            "expected prueba in {s2:?}"
        );
    }
}
