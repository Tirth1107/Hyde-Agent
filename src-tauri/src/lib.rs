pub mod types;
pub mod logger;
pub mod registry;
pub mod custom_commands;
pub mod parser;
pub mod executor;
pub mod ipc;
pub mod core;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, AppHandle,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use std::sync::Arc;

/// Global engine IPC handle (set during setup)
static ENGINE_IPC: std::sync::OnceLock<Arc<ipc::EngineIpc>> = std::sync::OnceLock::new();

fn get_engine() -> Option<&'static Arc<ipc::EngineIpc>> {
    ENGINE_IPC.get()
}

#[tauri::command]
async fn execute_command(app: AppHandle, command: String) -> Result<String, String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Err("Empty command".to_string());
    }

    let lower = trimmed.to_lowercase();

    // Fast intercept: Settings/Help opens the settings window
    if lower == "settings" || lower == "help" {
        if let Some(window) = app.get_webview_window("settings") {
            window.show().unwrap();
            window.set_focus().unwrap();
            return Ok("Opened Settings to view commands".to_string());
        }
        return Err("Settings window not found".to_string());
    }

    // ── PHASE 1: Try fast-path (Rust exact match on built-in keywords) ──
    // This avoids IPC latency for simple, unambiguous commands like
    // "youtube", "chrome", "volume up", "mute", etc.
    if let Some(parsed) = parser::try_exact_match(&lower) {
        if parsed.confidence >= 1.0 {
            return executor::execute(&app, parsed).await;
        }
    }

    // ── PHASE 2: Route through Python Neural Engine via IPC ──
    if let Some(engine) = get_engine() {
        match engine.classify(trimmed).await {
            Ok(result) => {
                // Execute via the V2 Router Pipeline
                return crate::core::router::execute(&app, result).await;
            }
            Err(ipc_err) => {
                // IPC failed — fall back to Rust parser
                eprintln!("[IPC Error] {}", ipc_err);
                if let Some(parsed) = parser::parse_input(trimmed) {
                    return executor::execute(&app, parsed).await;
                }
                return Err("Command not recognized. Type 'help' to see all commands.".to_string());
            }
        }
    }

    // ── PHASE 3: No engine available — use Rust parser only ──
    if let Some(parsed) = parser::parse_input(trimmed) {
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
fn get_ai_settings() -> Result<serde_json::Value, String> {
    let provider = crate::core::db::get_setting("ai_provider").unwrap_or_else(|| "gemini".to_string());
    let key = crate::core::db::get_setting("ai_api_key").unwrap_or_default();
    Ok(serde_json::json!({
        "provider": provider,
        "key": key
    }))
}

#[tauri::command]
fn save_ai_settings(provider: String, key: String) -> Result<(), String> {
    crate::core::db::set_setting("ai_provider", &provider).map_err(|e| e.to_string())?;
    crate::core::db::set_setting("ai_api_key", &key).map_err(|e| e.to_string())?;
    Ok(())
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
async fn start_native_listening(_app: tauri::AppHandle) -> Result<String, String> {
    // The Hyde Engine daemon runs continuously in the background.
    Ok("Hyde Neural Engine is always running.".to_string())
}

#[tauri::command]
fn get_gemini_api_key() -> Result<String, String> {
    if let Some(home) = dirs::home_dir() {
        let key_file = home.join(".hyde-agent").join("gemini_key.txt");
        if key_file.exists() {
            if let Ok(key) = std::fs::read_to_string(key_file) {
                let trimmed = key.trim().to_string();
                executor::ai::set_api_key(trimmed.clone());
                return Ok(trimmed);
            }
        }
    }
    Ok("".to_string())
}

#[tauri::command]
fn save_gemini_api_key(key: String) -> Result<(), String> {
    if let Some(home) = dirs::home_dir() {
        let config_dir = home.join(".hyde-agent");
        std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
        std::fs::write(config_dir.join("gemini_key.txt"), &key).map_err(|e| e.to_string())?;
        executor::ai::set_api_key(key);
        Ok(())
    } else {
        Err("Could not find home directory".to_string())
    }
}

fn start_hyde_engine_daemon(app: tauri::AppHandle) {
    use std::io::{BufRead, BufReader};
    use tauri::Emitter;

    // Spawn the Python engine and set up IPC
    match ipc::EngineIpc::spawn() {
        Ok((engine_ipc, mut child)) => {
            let engine_arc = Arc::new(engine_ipc);
            
            // Store globally for execute_command to use
            let _ = ENGINE_IPC.set(engine_arc.clone());

            // ── Stderr logger thread ──
            if let Some(stderr) = child.stderr.take() {
                std::thread::spawn(move || {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines().flatten() {
                        eprintln!("ENGINE ERR: {}", line);
                    }
                });
            }

            // ── Stdout reader thread ──
            // Reads both voice state events AND JSON-RPC responses
            if let Some(stdout) = child.stdout.take() {
                let engine_for_reader = engine_arc.clone();
                std::thread::spawn(move || {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines().flatten() {
                        let line = line.trim().to_string();
                        if line.is_empty() { continue; }
                        
                        println!("ENGINE OUT: {}", line);

                        // JSON-RPC responses are prefixed with [JSON]
                        if line.starts_with("[JSON]") {
                            let json_str = &line[6..]; // Strip [JSON] prefix
                            match serde_json::from_str::<ipc::IpcResponse>(json_str) {
                                Ok(response) => {
                                    engine_for_reader.handle_response(response);
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse IPC response: {} — raw: {}", e, json_str);
                                }
                            }
                            continue;
                        }

                        // Voice state events (unchanged from v1)
                        if line == "[STATE: READY]" {
                            app.emit("voice-state", "READY").unwrap_or(());
                        } else if line == "[STATE: SPEAKING]" {
                            app.emit("voice-state", "SPEAKING").unwrap_or(());
                        } else if line == "[STATE: TIMEOUT]" || line == "[STATE: IDLE]" {
                            app.emit("voice-state", "TIMEOUT").unwrap_or(());
                        } else if line == "[STATE: ERROR]" {
                            app.emit("voice-state", "ERROR").unwrap_or(());
                        } else if line == "[STATE: SUCCESS]" {
                            app.emit("voice-state", "SUCCESS").unwrap_or(());
                        } else if line.starts_with("[STATE: EXECUTE:") {
                            let text = line.trim_start_matches("[STATE: EXECUTE:").trim();
                            app.emit("voice-execute", text).unwrap_or(());
                        } else if line.starts_with("[User]:") {
                            let text = line.trim_start_matches("[User]:").trim();
                            app.emit("voice-state", format!("TEXT:{}", text)).unwrap_or(());
                        }
                    }

                    let _ = child.wait();
                    eprintln!("Hyde Engine daemon exited.");
                });
            }
        }
        Err(e) => {
            eprintln!("Failed to start Hyde Engine daemon: {}", e);
            eprintln!("Falling back to Rust-only mode (no NLU, no voice).");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            start_native_listening,
            get_ai_settings,
            save_ai_settings
        ])
        .setup(|app| {
            logger::init();
            custom_commands::load_custom_commands();
            if let Err(e) = crate::core::db::init() {
                eprintln!("Failed to init SQLite DB: {}", e);
            }
            
            crate::core::scheduler::start_scheduler(app.handle().clone());
            
            // Start the Hyde Neural Engine Daemon with IPC
            start_hyde_engine_daemon(app.handle().clone());
            
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
