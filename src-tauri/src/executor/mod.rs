pub mod app;
pub mod browser;
pub mod media;
pub mod system;
pub mod clipboard;
pub mod timer;
pub mod info;

use crate::types::{ActionType, ParseResult};
use tauri::AppHandle;

pub async fn execute(app_handle: &AppHandle, cmd: ParseResult) -> Result<String, String> {
    crate::logger::log_action("EXECUTE", &format!("Executing command: {:?}", cmd.action_type), true);
    
    match cmd.action_type {
        ActionType::OpenApp => app::launch(&cmd.parameters),
        ActionType::OpenUrl => browser::open_url(&cmd.parameters),
        ActionType::WebSearch => browser::web_search(&cmd.parameters),
        ActionType::YoutubeSearch => media::youtube_search(&cmd.parameters),
        ActionType::MailSearch => browser::mail_search(&cmd.parameters),
        ActionType::OpenFile => browser::open_path(&cmd.parameters, false),
        ActionType::OpenFolder => browser::open_path(&cmd.parameters, true),
        ActionType::SystemControl => system::control(&cmd.parameters),
        ActionType::Clipboard => clipboard::execute(app_handle, &cmd.parameters),
        ActionType::Timer => timer::execute(app_handle, &cmd.parameters),
        ActionType::Info => info::execute(&cmd.parameters),
        ActionType::CustomRunCommand => {
            // Placeholder for user commands shell execution
            // We should ask for approval here, but for v1.0 let's just log it or stub it safely
            Err("Custom shell commands are not fully supported yet".to_string())
        },
        ActionType::SmallTalk => {
            let msg = cmd.parameters.get("message").map(|s| s.as_str()).unwrap_or("");
            let reply = match msg {
                "hi" | "hello" | "hey" => "Hello! How can I help you today?",
                "how are you" => "I'm functioning perfectly, thank you for asking!",
                "who are you" | "what are you" => "I'm Hyde Agent, your personal desktop assistant.",
                "good morning" => "Good morning! Ready to tackle the day?",
                "good evening" => "Good evening! Need help winding down?",
                _ => "Hello there!"
            };
            Ok(reply.to_string())
        }
    }
}
