use std::collections::HashMap;
use std::process::Command;
use enigo::{Enigo, Keyboard, Key, Settings};

pub fn control(params: &HashMap<String, String>) -> Result<String, String> {
    let action = params.get("action").ok_or("No action provided")?;
    
    match action.as_str() {
        "volume up" => {
            let mut enigo = Enigo::new(&Settings::default()).unwrap();
            for _ in 0..5 { // 5 steps usually equals 10%
                let _ = enigo.key(Key::VolumeUp, enigo::Direction::Click);
            }
            Ok("Increased volume".to_string())
        }
        "volume down" => {
            let mut enigo = Enigo::new(&Settings::default()).unwrap();
            for _ in 0..5 {
                let _ = enigo.key(Key::VolumeDown, enigo::Direction::Click);
            }
            Ok("Decreased volume".to_string())
        }
        "mute" => {
            let mut enigo = Enigo::new(&Settings::default()).unwrap();
            let _ = enigo.key(Key::VolumeMute, enigo::Direction::Click);
            Ok("Toggled mute".to_string())
        }
        "lock screen" => {
            #[cfg(target_os = "windows")]
            {
                Command::new("rundll32.exe")
                    .arg("user32.dll,LockWorkStation")
                    .spawn()
                    .map_err(|e| format!("Failed to lock screen: {}", e))?;
                Ok("Locked screen".to_string())
            }
            #[cfg(not(target_os = "windows"))]
            {
                Err("Lock screen not implemented for this OS".to_string())
            }
        }
        "sleep" => {
            #[cfg(target_os = "windows")]
            {
                Command::new("rundll32.exe")
                    .args(&["powrprof.dll,SetSuspendState", "0,1,0"])
                    .spawn()
                    .map_err(|e| format!("Failed to sleep: {}", e))?;
                Ok("Going to sleep".to_string())
            }
            #[cfg(not(target_os = "windows"))]
            {
                Err("Sleep not implemented for this OS".to_string())
            }
        }
        "shutdown" => {
            #[cfg(target_os = "windows")]
            {
                Command::new("shutdown.exe")
                    .args(&["/s", "/t", "5"])
                    .spawn()
                    .map_err(|e| format!("Failed to shutdown: {}", e))?;
                Ok("Shutting down in 5 seconds...".to_string())
            }
            #[cfg(not(target_os = "windows"))]
            {
                Err("Shutdown not implemented for this OS".to_string())
            }
        }
        "restart" => {
            #[cfg(target_os = "windows")]
            {
                Command::new("shutdown.exe")
                    .args(&["/r", "/t", "5"])
                    .spawn()
                    .map_err(|e| format!("Failed to restart: {}", e))?;
                Ok("Restarting in 5 seconds...".to_string())
            }
            #[cfg(not(target_os = "windows"))]
            {
                Err("Restart not implemented for this OS".to_string())
            }
        }
        "screenshot" => {
            Err("Screenshot not implemented yet".to_string())
        }
        _ => Err(format!("Unknown system control: {}", action))
    }
}
