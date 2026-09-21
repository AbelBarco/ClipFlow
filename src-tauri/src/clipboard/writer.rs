pub async fn write_to_clipboard(content: &str, _mime_type: &str) -> Result<(), String> {
    let content = content.to_string();
    tokio::task::spawn_blocking(move || {
        let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        clipboard.set_text(content).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
