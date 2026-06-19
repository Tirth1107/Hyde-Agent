use std::collections::HashMap;
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

pub fn execute(app: &AppHandle, params: &HashMap<String, String>) -> Result<String, String> {
    let action = params.get("action").ok_or("No action provided")?;
    let clipboard = app.clipboard();

    match action.as_str() {
        "copy" => {
            let text = params.get("text").ok_or("No text provided to copy")?;
            clipboard.write_text(text).map_err(|e| format!("Failed to copy: {}", e))?;
            Ok(format!("Copied '{}' to clipboard", text))
        }
        "history" => {
            // For v1.0, we just show current clipboard content if possible, or stub it
            let text = clipboard.read_text().map_err(|e| format!("Failed to read clipboard: {}", e))?;
            // Real history requires a background listener which is complex for v1.0
            Ok(format!("Current clipboard: {}", text))
        }
        _ => Err(format!("Unknown clipboard action: {}", action))
    }
}
