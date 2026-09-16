use crate::pipeline::{detector::detect_type, dedupe::content_hash, color_parser::{detect_color, convert_color}, transformers::apply_transform};

#[test]
fn test_detect_url() {
    assert_eq!(detect_type("https://example.com", None), "url");
    assert_eq!(detect_type("http://test.org/path", None), "url");
}

#[test]
fn test_detect_color() {
    assert_eq!(detect_type("#FF0000", None), "color");
    assert_eq!(detect_type("#abc", None), "color");
    assert_eq!(detect_type("rgb(255, 0, 0)", None), "color");
    assert_eq!(detect_type("hsl(120, 50%, 50%)", None), "color");
}

#[test]
fn test_detect_code() {
    let code = "function test() {\n  return true;\n}";
    assert_eq!(detect_type(code, None), "code");

    let rust_code = "fn main() {\n    println!(\"Hello\");\n}";
    assert_eq!(detect_type(rust_code, None), "code");
}

#[test]
fn test_detect_text() {
    assert_eq!(detect_type("Hello world", None), "text");
    assert_eq!(detect_type("Just some plain text", None), "text");
}

#[test]
fn test_content_hash() {
    let hash1 = content_hash("test");
    let hash2 = content_hash("test");
    let hash3 = content_hash("different");

    assert_eq!(hash1, hash2);
    assert_ne!(hash1, hash3);
}

#[test]
fn test_color_conversion() {
    let result = convert_color("#FF0000").unwrap();
    assert_eq!(result.hex, "#FF0000");
    assert!(result.rgb.contains("255"));
    assert!(result.hsl.contains("0"));

    let result = convert_color("rgb(0, 255, 0)").unwrap();
    assert_eq!(result.hex, "#00FF00");

    let result = convert_color("hsl(240, 100%, 50%)").unwrap();
    assert_eq!(result.hex, "#0000FF");
}

#[test]
fn test_transformers() {
    assert_eq!(apply_transform("hello", "uppercase").unwrap(), "HELLO");
    assert_eq!(apply_transform("HELLO", "lowercase").unwrap(), "hello");
    assert_eq!(apply_transform("hello world", "title_case").unwrap(), "Hello World");
    assert_eq!(apply_transform("helloWorld", "snake_case").unwrap(), "hello_world");
    assert_eq!(apply_transform("helloWorld", "kebab_case").unwrap(), "hello-world");
    assert_eq!(apply_transform("hello_world", "camel_case").unwrap(), "helloWorld");
    assert_eq!(apply_transform("hello_world", "pascal_case").unwrap(), "HelloWorld");
    assert_eq!(apply_transform("  hello  ", "trim").unwrap(), "hello");
    assert_eq!(apply_transform("Hello World!", "slug").unwrap(), "hello-world");
    assert_eq!(apply_transform("hello", "base64_encode").unwrap(), "aGVsbG8=");
    assert_eq!(apply_transform("aGVsbG8=", "base64_decode").unwrap(), "hello");
}