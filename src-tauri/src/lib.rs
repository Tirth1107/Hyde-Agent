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
async fn start_native_listening() -> Result<String, String> {
    let script = r#"
        Add-Type -AssemblyName System.Speech
        $recognizer = New-Object System.Speech.Recognition.SpeechRecognitionEngine
        $recognizer.SetInputToDefaultAudioDevice()
        $grammar = New-Object System.Speech.Recognition.DictationGrammar
        $recognizer.LoadGrammar($grammar)
        $result = $recognizer.Recognize()
        if ($result -ne $null) { Write-Host $result.Text }
    "#;
    use std::os::windows::process::CommandExt;
    let output = std::process::Command::new("powershell")
        .creation_flags(0x08000000)
        .arg("-Command")
        .arg(script)
        .output()
        .map_err(|e| e.to_string())?;

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        Err("No speech detected".to_string())
    } else {
        Ok(text)
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
