use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref LOG_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
}

pub fn init() {
    if let Some(home) = dirs::home_dir() {
        let agent_dir = home.join(".hyde-agent");
        let logs_dir = agent_dir.join("logs");
        
        if let Err(e) = fs::create_dir_all(&logs_dir) {
            eprintln!("Failed to create logs directory: {}", e);
            return;
        }

        let mut dir_lock = LOG_DIR.lock().unwrap();
        *dir_lock = Some(logs_dir);
    }
}

pub fn log_action(action: &str, details: &str, success: bool) {
    let dir_lock = LOG_DIR.lock().unwrap();
    if let Some(logs_dir) = &*dir_lock {
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let log_file = logs_dir.join(format!("{}.log", date_str));
        
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let status = if success { "SUCCESS" } else { "ERROR" };
        let log_entry = format!("[{}] [{}] {}: {}\n", timestamp, status, action, details);

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
        {
            let _ = file.write_all(log_entry.as_bytes());
        }
    }
}
