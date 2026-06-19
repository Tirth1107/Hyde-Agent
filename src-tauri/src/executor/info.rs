use std::collections::HashMap;
use chrono::Local;

pub fn execute(params: &HashMap<String, String>) -> Result<String, String> {
    let info_type = params.get("type").ok_or("No info type provided")?;

    match info_type.as_str() {
        "time" => {
            let time = Local::now().format("%I:%M %p").to_string();
            Ok(format!("Current time is {}", time))
        }
        "date" => {
            let date = Local::now().format("%A, %B %d, %Y").to_string();
            Ok(format!("Today is {}", date))
        }
        "ip address" => {
            match local_ip_address::local_ip() {
                Ok(ip) => Ok(format!("Local IP: {}", ip)),
                Err(_) => Err("Could not determine local IP".to_string())
            }
        }
        "ram" => {
            use sysinfo::System;
            let mut sys = System::new_all();
            sys.refresh_all();
            let used = sys.used_memory() / 1024 / 1024;
            let total = sys.total_memory() / 1024 / 1024;
            Ok(format!("RAM: {} MB used / {} MB total", used, total))
        }
        "battery" => {
            match battery::Manager::new() {
                Ok(manager) => {
                    if let Ok(mut batteries) = manager.batteries() {
                        if let Some(Ok(batt)) = batteries.next() {
                            let percent = batt.state_of_charge().value * 100.0;
                            let state = batt.state();
                            return Ok(format!("Battery: {:.0}% ({})", percent, state));
                        }
                    }
                    Err("No battery found".to_string())
                }
                Err(_) => Err("Battery API error".to_string())
            }
        }
        _ => Err(format!("Unknown info type: {}", info_type))
    }
}
