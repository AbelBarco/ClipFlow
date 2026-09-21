use serde_json;

pub fn apply_transform(text: &str, transformer: &str) -> Result<String, String> {
    match transformer {
        "uppercase" => Ok(text.to_uppercase()),
        "lowercase" => Ok(text.to_lowercase()),
        "title_case" => Ok(to_title_case(text)),
        "snake_case" => Ok(to_snake_case(text)),
        "kebab_case" => Ok(to_kebab_case(text)),
        "camel_case" => Ok(to_camel_case(text)),
        "pascal_case" => Ok(to_pascal_case(text)),
        "trim" => Ok(text.trim().to_string()),
        "slug" => Ok(to_slug(text)),
        "json_pretty" => pretty_json(text),
        "json_minify" => minify_json(text),
        "url_encode" => Ok(urlencoding::encode(text).to_string()),
        "url_decode" => urlencoding::decode(text).map(|s| s.to_string()).map_err(|e| e.to_string()),
        "base64_encode" => {
            use base64::Engine as _;
            Ok(base64::engine::general_purpose::STANDARD.encode(text))
        }
        "base64_decode" => {
            use base64::Engine as _;
            base64::engine::general_purpose::STANDARD
                .decode(text.trim())
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .map_err(|e| e.to_string())
        }
        _ => Err(format!("Unknown transformer: {}", transformer)),
    }
}

pub fn list_transformers() -> Vec<String> {
    vec![
        "uppercase".to_string(),
        "lowercase".to_string(),
        "title_case".to_string(),
        "snake_case".to_string(),
        "kebab_case".to_string(),
        "camel_case".to_string(),
        "pascal_case".to_string(),
        "trim".to_string(),
        "slug".to_string(),
        "json_pretty".to_string(),
        "json_minify".to_string(),
        "url_encode".to_string(),
        "url_decode".to_string(),
        "base64_encode".to_string(),
        "base64_decode".to_string(),
    ]
}

fn to_title_case(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn to_snake_case(text: &str) -> String {
    let mut result = String::new();
    let mut prev_was_upper = false;
    let mut prev_was_lower = false;

    for (i, c) in text.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && (prev_was_lower || (prev_was_upper && i + 1 < text.len() && text.chars().nth(i + 1).map_or(false, |n| n.is_lowercase()))) {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_was_upper = true;
            prev_was_lower = false;
        } else if c.is_alphanumeric() {
            result.push(c);
            prev_was_lower = true;
            prev_was_upper = false;
        } else if !result.is_empty() && !result.ends_with('_') {
            result.push('_');
            prev_was_upper = false;
            prev_was_lower = false;
        }
    }

    result.trim_matches('_').to_string()
}

fn to_kebab_case(text: &str) -> String {
    to_snake_case(text).replace('_', "-")
}

fn to_camel_case(text: &str) -> String {
    let snake = to_snake_case(text);
    let parts: Vec<&str> = snake.split('_').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return String::new();
    }
    let mut result = parts[0].to_lowercase();
    for part in &parts[1..] {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            result.push(first.to_uppercase().next().unwrap());
            result.extend(chars);
        }
    }
    result
}

fn to_pascal_case(text: &str) -> String {
    let camel = to_camel_case(text);
    if camel.is_empty() {
        return String::new();
    }
    let mut chars = camel.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn to_slug(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn pretty_json(text: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    serde_json::to_string_pretty(&value).map_err(|e| e.to_string())
}

fn minify_json(text: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    serde_json::to_string(&value).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transforms_text() {
        assert_eq!(apply_transform("hola", "uppercase").unwrap(), "HOLA");
        assert_eq!(apply_transform("Hola Mundo", "snake_case").unwrap(), "hola_mundo");
        assert!(apply_transform("x", "no_existe").is_err());
        assert_eq!(list_transformers().len(), 15);
    }
}