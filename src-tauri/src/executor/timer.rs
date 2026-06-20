use std::collections::HashMap;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use std::sync::Mutex;
use lazy_static::lazy_static;
use tauri::async_runtime::JoinHandle;

struct TimerInfo {
    message: String,
    task: JoinHandle<()>,
}

lazy_static! {
    static ref TIMERS: Mutex<HashMap<usize, TimerInfo>> = Mutex::new(HashMap::new());
    static ref TIMER_ID_COUNTER: Mutex<usize> = Mutex::new(0);
}

#[derive(Clone, serde::Serialize)]
struct ToastPayload {
    message: String,
    is_error: bool,
}

pub fn execute(app: &AppHandle, params: &HashMap<String, String>) -> Result<String, String> {
    let action = params.get("action").cloned().unwrap_or_else(|| "set_timer".to_string());
    
    if action == "cancel_timer" {
        let mut timers = TIMERS.lock().unwrap();
        if timers.is_empty() {
            return Ok("No active timers to cancel.".to_string());
        }
        for (_, info) in timers.drain() {
            info.task.abort();
        }
        return Ok("All timers cancelled.".to_string());
    }
    
    if action == "list_timers" {
        let timers = TIMERS.lock().unwrap();
        if timers.is_empty() {
            return Ok("There are currently no active timers.".to_string());
        }
        let mut resp = format!("You have {} active timers:\n", timers.len());
        for (id, info) in timers.iter() {
            resp.push_str(&format!("- Timer {}: {}\n", id, info.message));
        }
        return Ok(resp);
    }

    // Default to set_timer
    let minutes_str = params.get("minutes").ok_or("No minutes provided")?;
    let minutes: u64 = minutes_str.parse().map_err(|_| "Invalid minutes format")?;
    
    let message = params.get("message").cloned().unwrap_or_else(|| "Timer finished!".to_string());
    
    let mut id_counter = TIMER_ID_COUNTER.lock().unwrap();
    *id_counter += 1;
    let timer_id = *id_counter;
    
    let app_handle = app.clone();
    let msg_clone = message.clone();
    
    let task = tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(minutes * 60)).await;
        
        let _ = app_handle.emit("show-toast", ToastPayload {
            message: msg_clone,
            is_error: false,
        });
        
        print!("\x07"); // System beep
        
        // Remove self from active timers
        let mut timers = TIMERS.lock().unwrap();
        timers.remove(&timer_id);
    });
    
    let mut timers = TIMERS.lock().unwrap();
    timers.insert(timer_id, TimerInfo { message: message.clone(), task });
    
    if message == "Timer finished!" {
        Ok(format!("Timer {} set for {} minutes", timer_id, minutes))
    } else {
        Ok(format!("Reminder {} set for {} minutes: {}", timer_id, minutes, message))
    }
}
