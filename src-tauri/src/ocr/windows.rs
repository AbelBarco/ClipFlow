//! Native on-device OCR for Windows via WinRT `Windows.Media.Ocr`.
//!
//! Fully offline-first: recognition runs locally, no network involved.
//! The only requirement is the OS OCR language pack for the requested
//! language (Settings → Time & Language → Language → Add a language; most
//! installs already include English). When the requested pack is missing we
//! fall back to the user's profile languages, then to any installed engine.

use std::path::Path;
use windows::{
    Foundation::Collections::IVectorView,
    Globalization::Language,
    Graphics::Imaging::{BitmapDecoder, SoftwareBitmap},
    Media::Ocr::OcrEngine,
    Storage::Streams::{DataWriter, InMemoryRandomAccessStream},
    Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED},
};

/// Fallback cap when the engine won't tell us its own limit.
const FALLBACK_MAX_SIDE_PX: u32 = 3000;

pub struct WindowsOcrProvider;

#[allow(clippy::new_without_default)]
impl WindowsOcrProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn recognize(&self, image_path: &str, language: &str) -> Result<String, String> {
        let path = Path::new(image_path);
        if !path.exists() {
            return Err("Image file not found".to_string());
        }

        // WinRT needs COM initialized on the calling thread. Tokio workers
        // are fresh OS threads: first call returns S_OK, later ones S_FALSE,
        // both fine. Even on failure the calls below surface a clear error.
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }

        let max_side = Self::max_side_px();
        let bytes = tokio::fs::read(path)
            .await
            .map_err(|e| format!("Cannot read image file: {e}"))?;
        let bitmap = Self::decode(bytes, max_side).await?;
        let engine = Self::engine_for(language)?;
        let result = engine
            .RecognizeAsync(&bitmap)
            .map_err(|e| format!("OCR recognition failed to start: {e}"))?
            .await
            .map_err(|e| format!("OCR recognition failed: {e}"))?;

        let mut lines = Vec::new();
        for line in result
            .Lines()
            .map_err(|e| format!("Cannot read OCR result: {e}"))?
        {
            let text = line
                .Text()
                .map_err(|e| format!("Cannot read OCR line: {e}"))?;
            lines.push(text.to_string());
        }
        Ok(lines.join("\n").trim().to_string())
    }

    /// Decode PNG/JPEG bytes into a `SoftwareBitmap` the engine accepts,
    /// downscaling huge screenshots to the engine's own size limit first.
    async fn decode(bytes: Vec<u8>, max_side: u32) -> Result<SoftwareBitmap, String> {
        let bytes = tokio::task::spawn_blocking(move || downscale_if_needed(bytes, max_side))
            .await
            .map_err(|e| format!("Image preprocessing panicked: {e}"))??;

        let stream = InMemoryRandomAccessStream::new().map_err(|e| format!("Stream error: {e}"))?;
        let writer =
            DataWriter::CreateDataWriter(&stream).map_err(|e| format!("Stream error: {e}"))?;
        writer
            .WriteBytes(&bytes)
            .map_err(|e| format!("Stream error: {e}"))?;
        writer
            .StoreAsync()
            .map_err(|e| format!("Stream error: {e}"))?
            .await
            .map_err(|e| format!("Stream error: {e}"))?;
        writer
            .DetachStream()
            .map_err(|e| format!("Stream error: {e}"))?;
        stream.Seek(0).map_err(|e| format!("Stream error: {e}"))?;

        let decoder = BitmapDecoder::CreateAsync(&stream)
            .map_err(|e| format!("Cannot decode image (unsupported format?): {e}"))?
            .await
            .map_err(|e| format!("Cannot decode image (unsupported format?): {e}"))?;
        decoder
            .GetSoftwareBitmapAsync()
            .map_err(|e| format!("Cannot build bitmap: {e}"))?
            .await
            .map_err(|e| format!("Cannot build bitmap: {e}"))
    }

    /// The engine's own bitmap size limit (WinRT rejects oversized images).
    fn max_side_px() -> u32 {
        OcrEngine::MaxImageDimension()
            .ok()
            .filter(|&m| m >= 512)
            .unwrap_or(FALLBACK_MAX_SIDE_PX)
    }

    /// Pick an engine for the requested Tesseract code, with graceful
    /// fallbacks when its language pack isn't installed.
    fn engine_for(language: &str) -> Result<OcrEngine, String> {
        let wanted = crate::ocr::tesseract_to_bcp47(language);
        let available = Self::available_languages()?;

        // 1. Exact tag ("es" → "es", "zh-Hans" → "zh-Hans").
        for lang in &available {
            let tag = language_tag(lang)?;
            if tag.eq_ignore_ascii_case(wanted) {
                tracing::info!("WinRT OCR engine: {tag}");
                return Self::create_engine(lang);
            }
        }
        // 2. Same base language ("zh" ↔ "zh-Hans").
        let base = wanted.split('-').next().unwrap_or(wanted).to_lowercase();
        for lang in &available {
            let tag = language_tag(lang)?.to_lowercase();
            if tag == base || tag.starts_with(&format!("{base}-")) {
                tracing::info!("WinRT OCR engine (base match): {tag} for {wanted}");
                return Self::create_engine(lang);
            }
        }
        // 3-4. User profile languages, then whatever is installed.
        if let Ok(engine) = OcrEngine::TryCreateFromUserProfileLanguages() {
            tracing::info!("WinRT OCR engine: user profile fallback for {wanted}");
            return Ok(engine);
        }
        for lang in &available {
            if let Ok(engine) = Self::create_engine(lang) {
                let tag = language_tag(lang).unwrap_or_default();
                tracing::info!("WinRT OCR engine: first-available fallback ({tag}) for {wanted}");
                return Ok(engine);
            }
        }
        Err("No OCR engine available. Install an OCR language pack in Windows Settings → Time & Language → Language.".to_string())
    }

    fn available_languages() -> Result<Vec<Language>, String> {
        let view: IVectorView<Language> = OcrEngine::AvailableRecognizerLanguages()
            .map_err(|e| format!("Cannot list OCR languages: {e}"))?;
        let size = view
            .Size()
            .map_err(|e| format!("Cannot list OCR languages: {e}"))?;
        let mut out = Vec::with_capacity(size as usize);
        for i in 0..size {
            out.push(
                view.GetAt(i)
                    .map_err(|e| format!("Cannot list OCR languages: {e}"))?,
            );
        }
        Ok(out)
    }

    fn create_engine(lang: &Language) -> Result<OcrEngine, String> {
        OcrEngine::TryCreateFromLanguage(lang).map_err(|e| format!("Cannot create OCR engine: {e}"))
    }
}

fn language_tag(lang: &Language) -> Result<String, String> {
    lang.LanguageTag()
        .map(|t| t.to_string())
        .map_err(|e| format!("Cannot read OCR language: {e}"))
}

/// Shrink images whose longest side exceeds `max_side` (CPU work, runs on a
/// blocking thread). Returns the original bytes when already small enough.
fn downscale_if_needed(bytes: Vec<u8>, max_side: u32) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(&bytes).map_err(|e| format!("Cannot decode image: {e}"))?;
    if img.width() <= max_side && img.height() <= max_side {
        return Ok(bytes);
    }
    let scale = max_side as f32 / img.width().max(img.height()) as f32;
    let resized = img.resize(
        (img.width() as f32 * scale) as u32,
        (img.height() as f32 * scale) as u32,
        image::imageops::FilterType::Triangle,
    );
    let mut out = Vec::new();
    resized
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .map_err(|e| format!("Cannot re-encode image: {e}"))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Render crisp black-on-white text with GDI so the test exercises the
    /// real pipeline (file → decoder → bitmap → engine → text) without any
    /// fixture files or fonts to install (Arial ships with Windows).
    fn render_text_png(text: &str) -> Vec<u8> {
        use windows::core::w;
        use windows::Win32::Graphics::Gdi::*;

        const W: i32 = 900;
        const H: i32 = 260;
        unsafe {
            let screen = GetDC(None);
            assert!(!screen.is_invalid());
            let mem = CreateCompatibleDC(screen);
            assert!(!mem.is_invalid());
            let bmp = CreateCompatibleBitmap(screen, W, H);
            assert!(!bmp.is_invalid());
            let old_bmp = SelectObject(mem, bmp);
            let font = CreateFontW(
                96,
                0,
                0,
                0,
                FW_NORMAL.0 as i32,
                0,
                0,
                0,
                ANSI_CHARSET.0 as u32,
                OUT_DEFAULT_PRECIS.0 as u32,
                CLIP_DEFAULT_PRECIS.0 as u32,
                DEFAULT_QUALITY.0 as u32,
                DEFAULT_PITCH.0 as u32 | FF_DONTCARE.0 as u32,
                w!("Arial"),
            );
            assert!(!font.is_invalid());
            let old_font = SelectObject(mem, font);
            let rect = windows::Win32::Foundation::RECT {
                left: 0,
                top: 0,
                right: W,
                bottom: H,
            };
            FillRect(mem, &rect, HBRUSH(GetStockObject(WHITE_BRUSH).0));
            SetTextColor(mem, windows::Win32::Foundation::COLORREF(0x00000000));
            SetBkMode(mem, OPAQUE);
            let wide: Vec<u16> = text.encode_utf16().collect();
            TextOutW(mem, 40, 70, &wide);

            // Pull the pixels back out (top-down 32bpp) into an RGBA image.
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: W,
                    biHeight: -H,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut pixels = vec![0u8; (W * H * 4) as usize];
            let scanned = GetDIBits(
                mem,
                bmp,
                0,
                H as u32,
                Some(pixels.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );
            assert!(scanned != 0);
            for px in pixels.chunks_exact_mut(4) {
                px.swap(0, 2); // BGRA → RGBA
                px[3] = 255;
            }
            let rgba =
                image::RgbaImage::from_raw(W as u32, H as u32, pixels).expect("pixel buffer size");
            SelectObject(mem, old_bmp);
            SelectObject(mem, old_font);
            DeleteObject(font);
            DeleteObject(bmp);
            DeleteDC(mem);
            ReleaseDC(None, screen);

            let mut png = Vec::new();
            image::DynamicImage::ImageRgba8(rgba)
                .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
                .expect("png encode");
            png
        }
    }

    fn english_available() -> bool {
        WindowsOcrProvider::engine_for("eng").is_ok()
    }

    #[tokio::test]
    async fn recognizes_rendered_text_end_to_end() {
        let png = render_text_png("Hello ClipFlow 123");
        let dir = std::env::temp_dir().join("clipflow_ocr_test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("hello.png");
        std::fs::write(&path, &png).expect("write fixture");

        // Decoding must work regardless of installed language packs.
        let bitmap = WindowsOcrProvider::decode(png, 3000).await.expect("decode");
        let info = bitmap.PixelWidth().expect("width");
        assert!(info > 0);

        // Full recognition needs the English pack; where it exists we assert
        // on content, otherwise the decode above is the coverage.
        if !english_available() {
            println!("English OCR pack missing — content assertion skipped");
            return;
        }
        let text = WindowsOcrProvider::new()
            .recognize(path.to_str().expect("path"), "eng")
            .await
            .expect("recognize");
        let normalized: String = text
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { ' ' })
            .collect();
        let normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            normalized.contains("hello"),
            "unexpected OCR output: {text:?}"
        );
        assert!(
            normalized.contains("clipflow") || normalized.contains("123"),
            "unexpected OCR output: {text:?}"
        );
    }

    #[test]
    fn missing_file_is_a_clean_error() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let err = WindowsOcrProvider::new()
                .recognize("C:\\definitely\\not\\here.png", "eng")
                .await
                .expect_err("must fail");
            assert!(err.contains("not found"));
        });
    }
}
