use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref HEX_REGEX: Regex = Regex::new(r"^#([0-9a-fA-F]{3}|[0-9a-fA-F]{4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$").unwrap();
    static ref RGB_REGEX: Regex = Regex::new(r"^rgb\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)$").unwrap();
    static ref RGBA_REGEX: Regex = Regex::new(r"^rgba\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*,\s*([\d.]+)\s*\)$").unwrap();
    static ref HSL_REGEX: Regex = Regex::new(r"^hsl\(\s*(\d+)\s*,\s*(\d+)%\s*,\s*(\d+)%\s*\)$").unwrap();
    static ref HSLA_REGEX: Regex = Regex::new(r"^hsla\(\s*(\d+)\s*,\s*(\d+)%\s*,\s*(\d+)%\s*,\s*([\d.]+)\s*\)$").unwrap();
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
            caps[1].parse().unwrap(),
            caps[2].parse().unwrap(),
            caps[3].parse().unwrap(),
            None,
        );
    }
    if let Some(caps) = RGBA_REGEX.captures(trimmed) {
        return parse_rgb(
            caps[1].parse().unwrap(),
            caps[2].parse().unwrap(),
            caps[3].parse().unwrap(),
            Some(caps[4].parse().unwrap()),
        );
    }
    if let Some(caps) = HSL_REGEX.captures(trimmed) {
        return parse_hsl(
            caps[1].parse().unwrap(),
            caps[2].parse().unwrap(),
            caps[3].parse().unwrap(),
            None,
        );
    }
    if let Some(caps) = HSLA_REGEX.captures(trimmed) {
        return parse_hsl(
            caps[1].parse().unwrap(),
            caps[2].parse().unwrap(),
            caps[3].parse().unwrap(),
            Some(caps[4].parse().unwrap()),
        );
    }

    Err("Invalid color format".to_string())
}

fn parse_hex(hex: &str) -> Result<ColorConversion, String> {
    let (r, g, b, a) = match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap();
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap();
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap();
            (r, g, b, 255)
        }
        4 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap();
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap();
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap();
            let a = u8::from_str_radix(&hex[3..4].repeat(2), 16).unwrap();
            (r, g, b, a)
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
            (r, g, b, 255)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
            let a = u8::from_str_radix(&hex[6..8], 16).unwrap();
            (r, g, b, a)
        }
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

    (h.round() as u16, (s * 100.0).round() as u8, (l * 100.0).round() as u8)
}