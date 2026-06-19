use std::collections::HashMap;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Clone, serde::Serialize)]
struct ToastPayload {
    message: String,
    is_error: bool,
}

pub fn execute(app: &AppHandle, params: &HashMap<String, String>) -> Result<String, String> {
    let minutes_str = params.get("minutes").ok_or("No minutes provided")?;
    let minutes: u64 = minutes_str.parse().map_err(|_| "Invalid minutes format")?;
    
    let message = params.get("message").cloned().unwrap_or_else(|| "Timer finished!".to_string());
    
    let app_handle = app.clone();
    
    let msg_clone = message.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(minutes * 60)).await;
        
        let _ = app_handle.emit("show-toast", ToastPayload {
            message: msg_clone,
            is_error: false,
        });
        
        // System beep (optional, but requested by spec)
        // A simple beep on Windows can be done with `enigo` or just `\x07` print, but 
        // to keep it simple, we just print the bell character which might ring terminal bell.
        print!("\x07");
    });
    
    if message == "Timer finished!" {
        Ok(format!("Timer set for {} minutes", minutes))
    } else {
        Ok(format!("Reminder set for {} minutes: {}", minutes, message))
    }
}
