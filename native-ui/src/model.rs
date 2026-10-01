//! Modelo de datos de la UI nativa.
//!
//! Autocontenido (sin Tauri ni WebView2): el binario final es un único .exe
//! que dibuja por GPU (winit + wgpu). Para datos reales, sustituye
//! `seed_items()` por una lectura al SQLite de ClipFlow.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipType {
    Text,
    Url,
    Code,
    Color,
    Image,
}

impl ClipType {
    pub fn label(&self) -> &'static str {
        match self {
            ClipType::Text => "Texto",
            ClipType::Url => "Enlaces",
            ClipType::Code => "Código",
            ClipType::Color => "Colores",
            ClipType::Image => "Imágenes",
        }
    }

    /// Letra dentro del círculo de color (siempre legible).
    pub fn glyph(&self) -> &'static str {
        match self {
            ClipType::Text => "Aa",
            ClipType::Url => "U",
            ClipType::Code => "</>",
            ClipType::Color => "C",
            ClipType::Image => "I",
        }
    }

    #[allow(dead_code)]
    pub fn all() -> [ClipType; 5] {
        [
            ClipType::Text,
            ClipType::Url,
            ClipType::Code,
            ClipType::Color,
            ClipType::Image,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipItem {
    pub id: String,
    pub kind: ClipType,
    pub content: String,
    pub preview: String,
    pub size: usize,
    /// epoch millis
    pub timestamp: i64,
    pub ocr: Option<String>,
}

impl ClipItem {
    pub fn matches_query(&self, q: &str) -> bool {
        if q.trim().is_empty() {
            return true;
        }
        let q = q.to_lowercase();
        self.content.to_lowercase().contains(&q)
            || self.preview.to_lowercase().contains(&q)
            || self
                .ocr
                .as_ref()
                .is_some_and(|o| o.to_lowercase().contains(&q))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    One(ClipType),
}

impl Filter {
    pub fn matches(&self, kind: ClipType) -> bool {
        match self {
            Filter::All => true,
            Filter::One(k) => *k == kind,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub launch_at_startup: bool,
    pub history_size: usize,
    pub dark_mode: bool,
    pub language: String,
    pub exclude_passwords: bool,
    pub ocr_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            launch_at_startup: false,
            history_size: 200,
            dark_mode: true,
            language: "Español".to_string(),
            exclude_passwords: true,
            ocr_enabled: true,
        }
    }
}

pub fn now_millis() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub fn format_age(timestamp_ms: i64) -> String {
    let diff = now_millis().saturating_sub(timestamp_ms);
    let mins = diff / 60_000;
    let hours = diff / 3_600_000;
    let days = diff / 86_400_000;
    if mins < 1 {
        "ahora".to_string()
    } else if mins < 60 {
        format!("hace {} min", mins)
    } else if hours < 24 {
        format!("hace {} h", hours)
    } else if days < 7 {
        format!("hace {} d", days)
    } else {
        chrono::DateTime::from_timestamp_millis(timestamp_ms)
            .map(|d| d.format("%d/%m/%Y").to_string())
            .unwrap_or_default()
    }
}

pub fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Ejemplos para ver la lista, la vista previa y los filtros desde el
/// primer arranque. Con el historial vacío se muestra la tarjeta de
/// bienvenida ("Tu portapapeles, siempre a mano").
pub fn seed_items() -> Vec<ClipItem> {
    let now = now_millis();
    let m = 60_000;
    vec![
        ClipItem {
            id: "1".to_string(),
            kind: ClipType::Code,
            content: "fn copy_to_clipboard(text: &str) -> anyhow::Result<()> {\n    arboard::Clipboard::new()?.set_text(text)?;\n    Ok(())\n}".to_string(),
            preview: "fn copy_to_clipboard(text: &str) -> anyhow::Result<()> …".to_string(),
            size: 128,
            timestamp: now - 2 * m,
            ocr: None,
        },
        ClipItem {
            id: "2".to_string(),
            kind: ClipType::Url,
            content: "https://github.com/anomalyco/opencode".to_string(),
            preview: "https://github.com/anomalyco/opencode".to_string(),
            size: 38,
            timestamp: now - 9 * m,
            ocr: None,
        },
        ClipItem {
            id: "3".to_string(),
            kind: ClipType::Color,
            content: "#0EA5E9".to_string(),
            preview: "#0EA5E9".to_string(),
            size: 7,
            timestamp: now - 24 * m,
            ocr: None,
        },
        ClipItem {
            id: "4".to_string(),
            kind: ClipType::Text,
            content: "ClipFlow vive en la bandeja: Ctrl+Shift+V abre el acceso rápido y Enter pega al instante.".to_string(),
            preview: "ClipFlow vive en la bandeja: Ctrl+Shift+V abre el acceso rápido…".to_string(),
            size: 96,
            timestamp: now - 51 * m,
            ocr: None,
        },
        ClipItem {
            id: "5".to_string(),
            kind: ClipType::Image,
            content: "<imagen 1280x720>".to_string(),
            preview: "Captura 1280 × 720 · PNG".to_string(),
            size: 412_000,
            timestamp: now - 95 * m,
            ocr: Some("Texto detectado por OCR en la captura".to_string()),
        },
        ClipItem {
            id: "6".to_string(),
            kind: ClipType::Color,
            content: "#7C3AED".to_string(),
            preview: "#7C3AED".to_string(),
            size: 7,
            timestamp: now - 180 * m,
            ocr: None,
        },
        ClipItem {
            id: "7".to_string(),
            kind: ClipType::Code,
            content: "SELECT id, content, created_at FROM clips ORDER BY created_at DESC LIMIT 60;".to_string(),
            preview: "SELECT id, content, created_at FROM clips ORDER BY…".to_string(),
            size: 74,
            timestamp: now - 300 * m,
            ocr: None,
        },
        ClipItem {
            id: "8".to_string(),
            kind: ClipType::Text,
            content: "Reunión mañana 10:00 — llevar propuesta de precios y demo del portapapeles.".to_string(),
            preview: "Reunión mañana 10:00 — llevar propuesta de precios…".to_string(),
            size: 78,
            timestamp: now - 500 * m,
            ocr: None,
        },
    ]
}
