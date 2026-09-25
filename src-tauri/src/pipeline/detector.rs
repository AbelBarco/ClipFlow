use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
    static ref HEX_COLOR_REGEX: Regex = Regex::new(r"^#([0-9a-fA-F]{3}|[0-9a-fA-F]{4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$").unwrap();
    static ref RGB_REGEX: Regex = Regex::new(r"^rgb\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*\)$").unwrap();
    static ref RGBA_REGEX: Regex = Regex::new(r"^rgba\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*,\s*[\d.]+\s*\)$").unwrap();
    static ref HSL_REGEX: Regex = Regex::new(r"^hsl\(\s*\d+\s*,\s*\d+%\s*,\s*\d+%\s*\)$").unwrap();
    static ref HSLA_REGEX: Regex = Regex::new(r"^hsla\(\s*\d+\s*,\s*\d+%\s*,\s*\d+%\s*,\s*[\d.]+\s*\)$").unwrap();
    static ref CODE_INDICATORS: Regex = Regex::new(r"(?m)^\s*(fn|function|const|let|var|class|interface|type|import|export|return|if|else|for|while|match|switch|case|default|struct|enum|impl|trait|mod|use|pub|async|await|try|catch|finally|throw|new|this|super|extends|implements|interface|abstract|final|static|public|private|protected|override|virtual|sealed|readonly|nullable|nonnull|@|#|//|/\*|\*/|<!--|-->|<\?|\?>)").unwrap();
}

pub fn detect_type(content: &str, mime_type: Option<&str>) -> String {
    // Check MIME type first
    if let Some(mime) = mime_type {
        if mime.starts_with("image/") {
            return "image".to_string();
        }
        if mime.starts_with("text/") || mime == "application/json" || mime == "application/xml" {
            // Continue to content-based detection
        }
    }

    let trimmed = content.trim();

    // Check for color formats
    if is_color(trimmed) {
        return "color".to_string();
    }

    // Check for URL
    if URL_REGEX.is_match(trimmed) {
        return "url".to_string();
    }

    // Check for code
    if is_code(trimmed) {
        return "code".to_string();
    }

    // Default to text
    "text".to_string()
}

fn is_color(content: &str) -> bool {
    HEX_COLOR_REGEX.is_match(content)
        || RGB_REGEX.is_match(content)
        || RGBA_REGEX.is_match(content)
        || HSL_REGEX.is_match(content)
        || HSLA_REGEX.is_match(content)
}

fn is_code(content: &str) -> bool {
    // Heuristic: check for common code patterns
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() > 1 {
        let indicator_count = lines
            .iter()
            .filter(|line| CODE_INDICATORS.is_match(line))
            .count();
        if indicator_count > 0 {
            return true;
        }
    }

    // Check for common code characters
    let code_chars = content
        .chars()
        .filter(|c| {
            matches!(
                c,
                '{' | '}' | '[' | ']' | '(' | ')' | ';' | ':' | '=' | '<' | '>'
            )
        })
        .count();
    if code_chars > content.len() / 20 {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_color_url_and_text() {
        assert_eq!(detect_type("#ff0000", None), "color");
        assert_eq!(detect_type("rgb(255, 0, 0)", None), "color");
        assert_eq!(detect_type("https://example.com", None), "url");
        assert_eq!(detect_type("hello world", None), "text");
    }
}
