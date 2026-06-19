pub mod types;
pub mod logger;
pub mod registry;
pub mod custom_commands;
pub mod parser;
pub mod executor;
pub mod voice;
pub mod voice_setup;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, AppHandle,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[tauri::command]
async fn execute_command(app: AppHandle, command: String) -> Result<String, String> {
    if command.trim().to_lowercase() == "settings" || command.trim().to_lowercase() == "help" {
        if let Some(window) = app.get_webview_window("settings") {
            window.show().unwrap();
            window.set_focus().unwrap();
            return Ok("Opened Settings to view commands".to_string());
        }
        return Err("Settings window not found".to_string());
    }

    if let Some(parsed) = parser::parse_input(&command) {
        executor::execute(&app, parsed).await
    } else {
        Err("Command not recognized. Type 'help' to see all commands.".to_string())
    }
}

#[tauri::command]
fn get_custom_commands_json() -> Result<String, String> {
    let cmds = custom_commands::get_custom_commands();
    serde_json::to_string(&cmds).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_builtin_commands_json() -> Result<String, String> {
    let cmds = &*registry::BUILTIN_COMMANDS;
    serde_json::to_string(cmds).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_custom_commands_json(json: String) -> Result<(), String> {
    let cmds = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    custom_commands::save_custom_commands(cmds)
}

#[tauri::command]
fn open_logs_folder() -> Result<(), String> {
    if let Some(home) = dirs::home_dir() {
        let logs_dir = home.join(".hyde-agent").join("logs");
        open::that(logs_dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_suggestions(query: String) -> Vec<String> {
    let lower_query = query.to_lowercase();
    if lower_query.is_empty() { return vec![]; }
    
    let mut suggestions = Vec::new();
    for cmd in registry::BUILTIN_COMMANDS.iter() {
        if cmd.keyword.to_lowercase().starts_with(&lower_query) {
            suggestions.push(cmd.keyword.clone());
        }
    }
    
    // Sort and limit to 5
    suggestions.sort();
    suggestions.truncate(5);
    suggestions
}

#[tauri::command]
async fn start_native_listening(app: tauri::AppHandle) -> Result<String, String> {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};
    use std::io::{BufRead, BufReader};
    use tauri::Emitter;
    use std::path::Path;

    let py_script = if Path::new("src-tauri/listen_once.py").exists() {
        "src-tauri/listen_once.py"
    } else if Path::new("listen_once.py").exists() {
        "listen_once.py"
    } else {
        return Err("listen_once.py not found".to_string());
    };

    let mut child = Command::new("python")
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .arg("-u")
        .arg(py_script)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn python: {}", e))?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    
    // Spawn a thread to read stderr so it doesn't block and we can log it
    std::thread::spawn(move || {
        let err_reader = BufReader::new(stderr);
        for line in err_reader.lines() {
            if let Ok(line) = line {
                eprintln!("PYTHON ERROR: {}", line);
            }
        }
    });

    let reader = BufReader::new(stdout);
    let mut final_text = String::new();

    for line in reader.lines() {
        if let Ok(line) = line {
            let line = line.trim();
            println!("PYTHON OUT: {}", line);
            if line == "READY" {
                app.emit("voice-state", "READY").unwrap_or(());
            } else if line == "SPEAKING" {
                app.emit("voice-state", "SPEAKING").unwrap_or(());
            } else if line == "TIMEOUT" {
                app.emit("voice-state", "TIMEOUT").unwrap_or(());
                break;
            } else if line == "ERROR" {
                app.emit("voice-state", "ERROR").unwrap_or(());
                break;
            } else if line.starts_with("TEXT:") {
                final_text = line.trim_start_matches("TEXT:").to_string();
                break;
            }
        }
    }
    
    // forcefully kill the child process if it's still running
    let _ = child.kill();
    let _ = child.wait();

    if final_text.is_empty() {
        Err("TIMEOUT".to_string())
    } else {
        Ok(final_text)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            execute_command,
            get_custom_commands_json,
            save_custom_commands_json,
            open_logs_folder,
            get_builtin_commands_json,
            get_suggestions,
            voice_setup::download_vosk_model,
            start_native_listening
        ])
        .setup(|app| {
            logger::init();
            custom_commands::load_custom_commands();
            
            // Setup Tray Icon
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Open Hyde Agent", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &settings_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("settings") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("overlay") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    _ => {}
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;

            // Register Global Shortcut (Alt + Space)
            let alt_space = Shortcut::new(Some(Modifiers::ALT), Code::Space);
            
            // We register the global shortcut dynamically after setup
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, shortcut, event| {
                        if shortcut == &alt_space && event.state() == ShortcutState::Pressed {
                            if let Some(window) = app.get_webview_window("overlay") {
                                if window.is_visible().unwrap_or(false) {
                                    window.hide().unwrap();
                                } else {
                                    window.show().unwrap();
                                    window.set_focus().unwrap();
                                }
                            }
                        }
                    })
                    .build(),
            )?;

            app.global_shortcut().register(alt_space)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
