use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::types::CommandEntry;

lazy_static! {
    static ref CUSTOM_COMMANDS: Mutex<Vec<CommandEntry>> = Mutex::new(Vec::new());
}

fn get_custom_commands_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".hyde-agent").join("custom_commands.json"))
}

pub fn load_custom_commands() {
    if let Some(path) = get_custom_commands_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cmds) = serde_json::from_str::<Vec<CommandEntry>>(&content) {
                    let mut lock = CUSTOM_COMMANDS.lock().unwrap();
                    *lock = cmds;
                    crate::logger::log_action("LOAD_CUSTOM_COMMANDS", "Successfully loaded custom commands.", true);
                    return;
                }
            }
        }
    }
    // If we fail or it doesn't exist, we just start with empty
    let mut lock = CUSTOM_COMMANDS.lock().unwrap();
    *lock = Vec::new();
}

pub fn save_custom_commands(cmds: Vec<CommandEntry>) -> Result<(), String> {
    if let Some(path) = get_custom_commands_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        
        let json = serde_json::to_string_pretty(&cmds).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())?;
        
        // Update memory
        let mut lock = CUSTOM_COMMANDS.lock().unwrap();
        *lock = cmds;
        
        crate::logger::log_action("SAVE_CUSTOM_COMMANDS", "Successfully saved custom commands.", true);
        Ok(())
    } else {
        Err("Could not resolve home directory".to_string())
    }
}

pub fn get_custom_commands() -> Vec<CommandEntry> {
    let lock = CUSTOM_COMMANDS.lock().unwrap();
    lock.clone()
}
