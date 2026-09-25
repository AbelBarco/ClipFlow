use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref HEX_REGEX: Regex =
        Regex::new(r"^#([0-9a-fA-F]{3}|[0-9a-fA-F]{4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$").unwrap();
    static ref RGB_REGEX: Regex =
        Regex::new(r"^rgb\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)$").unwrap();
    static ref RGBA_REGEX: Regex =
        Regex::new(r"^rgba\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*,\s*([\d.]+)\s*\)$").unwrap();
    static ref HSL_REGEX: Regex =
        Regex::new(r"^hsl\(\s*(\d+)\s*,\s*(\d+)%\s*,\s*(\d+)%\s*\)$").unwrap();
    static ref HSLA_REGEX: Regex =
        Regex::new(r"^hsla\(\s*(\d+)\s*,\s*(\d+)%\s*,\s*(\d+)%\s*,\s*([\d.]+)\s*\)$").unwrap();
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorConversion {
    pub hex: String,
    pub rgb: String,
    pub hsl: String,
    pub css: String,
}

pub fn detect_color(input: &str) -> bool {
    let trimmed = input.trim();
    HEX_REGEX.is_match(trimmed)
        || RGB_REGEX.is_match(trimmed)
        || RGBA_REGEX.is_match(trimmed)
        || HSL_REGEX.is_match(trimmed)
        || HSLA_REGEX.is_match(trimmed)
}

pub fn convert_color(input: &str) -> Result<ColorConversion, String> {
    let trimmed = input.trim();

    if let Some(caps) = HEX_REGEX.captures(trimmed) {
        return parse_hex(&caps[1]);
    }
    if let Some(caps) = RGB_REGEX.captures(trimmed) {
        return parse_rgb(
            parse_num(&caps[1], "red")?,
            parse_num(&caps[2], "green")?,
            parse_num(&caps[3], "blue")?,
            None,
        );
    }
    if let Some(caps) = RGBA_REGEX.captures(trimmed) {
        return parse_rgb(
            parse_num(&caps[1], "red")?,
            parse_num(&caps[2], "green")?,
            parse_num(&caps[3], "blue")?,
            Some(parse_num(&caps[4], "alpha")?),
        );
    }
    if let Some(caps) = HSL_REGEX.captures(trimmed) {
        return parse_hsl(
            parse_num(&caps[1], "hue")?,
            parse_num(&caps[2], "saturation")?,
            parse_num(&caps[3], "lightness")?,
            None,
        );
    }
    if let Some(caps) = HSLA_REGEX.captures(trimmed) {
        return parse_hsl(
            parse_num(&caps[1], "hue")?,
            parse_num(&caps[2], "saturation")?,
            parse_num(&caps[3], "lightness")?,
            Some(parse_num(&caps[4], "alpha")?),
        );
    }

    Err("Invalid color format".to_string())
}

/// Parse a regex-captured component without panicking: out-of-range input
/// like `rgb(999, 0, 0)` is a user error, not a crash.
fn parse_num<T: std::str::FromStr>(s: &str, what: &str) -> Result<T, String> {
    s.parse().map_err(|_| format!("Invalid {what} value: {s}"))
}

fn parse_hex_digit(s: &str) -> Result<u8, String> {
    u8::from_str_radix(s, 16).map_err(|_| format!("Invalid hex digit: {s}"))
}

fn parse_hex(hex: &str) -> Result<ColorConversion, String> {
    let (r, g, b, a) = match hex.len() {
        3 => (
            parse_hex_digit(&hex[0..1].repeat(2))?,
            parse_hex_digit(&hex[1..2].repeat(2))?,
            parse_hex_digit(&hex[2..3].repeat(2))?,
            255,
        ),
        4 => (
            parse_hex_digit(&hex[0..1].repeat(2))?,
            parse_hex_digit(&hex[1..2].repeat(2))?,
            parse_hex_digit(&hex[2..3].repeat(2))?,
            parse_hex_digit(&hex[3..4].repeat(2))?,
        ),
        6 => (
            parse_hex_digit(&hex[0..2])?,
            parse_hex_digit(&hex[2..4])?,
            parse_hex_digit(&hex[4..6])?,
            255,
        ),
        8 => (
            parse_hex_digit(&hex[0..2])?,
            parse_hex_digit(&hex[2..4])?,
            parse_hex_digit(&hex[4..6])?,
            parse_hex_digit(&hex[6..8])?,
        ),
        _ => return Err("Invalid hex length".to_string()),
    };
    build_conversion(r, g, b, a)
}

fn parse_rgb(r: u8, g: u8, b: u8, a: Option<f32>) -> Result<ColorConversion, String> {
    let a = a.unwrap_or(1.0).clamp(0.0, 1.0);
    let a_byte = (a * 255.0).round() as u8;
    build_conversion(r, g, b, a_byte)
}

fn parse_hsl(h: u16, s: u8, l: u8, a: Option<f32>) -> Result<ColorConversion, String> {
    let h = h as f32;
    let s = s as f32 / 100.0;
    let l = l as f32 / 100.0;
    let a = a.unwrap_or(1.0).clamp(0.0, 1.0);

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let r = ((r1 + m) * 255.0).round() as u8;
    let g = ((g1 + m) * 255.0).round() as u8;
    let b = ((b1 + m) * 255.0).round() as u8;
    let a_byte = (a * 255.0).round() as u8;

    build_conversion(r, g, b, a_byte)
}

fn build_conversion(r: u8, g: u8, b: u8, a: u8) -> Result<ColorConversion, String> {
    let hex = if a == 255 {
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    } else {
        format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
    };

    let rgb = if a == 255 {
        format!("rgb({}, {}, {})", r, g, b)
    } else {
        format!("rgba({}, {}, {}, {:.2})", r, g, b, a as f32 / 255.0)
    };

    let (h, s, l) = rgb_to_hsl(r, g, b);
    let hsl = if a == 255 {
        format!("hsl({}, {}%, {}%)", h, s, l)
    } else {
        format!("hsla({}, {}%, {}%, {:.2})", h, s, l, a as f32 / 255.0)
    };

    let css = if a == 255 {
        format!("rgb({} {} {})", r, g, b)
    } else {
        format!("rgb({} {} {} / {:.2})", r, g, b, a as f32 / 255.0)
    };

    Ok(ColorConversion { hex, rgb, hsl, css })
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (u16, u8, u8) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let l = (max + min) / 2.0;

    if delta == 0.0 {
        return (0, 0, (l * 100.0).round() as u8);
    }

    let s = if l < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    let h = if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };

    (
        h.round() as u16,
        (s * 100.0).round() as u8,
        (l * 100.0).round() as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_hex_and_detects_formats() {
        assert!(detect_color("#0ea5e9"));
        assert!(!detect_color("not a color"));
        let conv = convert_color("#ff0000").expect("valid hex");
        assert_eq!(conv.hex, "#FF0000");
        assert_eq!(conv.rgb, "rgb(255, 0, 0)");
    }

    #[test]
    fn out_of_range_values_are_errors_not_panics() {
        assert!(convert_color("rgb(999, 0, 0)").is_err());
        assert!(convert_color("hsl(0, 0%, 0%)").is_ok());
    }
}
