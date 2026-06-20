pub mod app;
pub mod browser;
pub mod media;
pub mod system;
pub mod clipboard;
pub mod timer;
pub mod info;

pub mod ai;

use crate::types::{ActionType, ParseResult};
use tauri::AppHandle;

pub async fn execute(app_handle: &AppHandle, cmd: ParseResult) -> Result<String, String> {
    crate::logger::log_action("EXECUTE", &format!("Executing command: {:?}", cmd.action_type), true);
    
    match cmd.action_type {
        ActionType::OpenApp => app::launch(&cmd.parameters),
        ActionType::CloseApp => {
            let name = cmd.parameters.get("app_name").cloned().unwrap_or_default();
            Err(format!("Close app '{}' is handled by the Neural Engine.", name))
        }
        ActionType::OpenUrl => browser::open_url(&cmd.parameters),
        ActionType::WebSearch => browser::web_search(&cmd.parameters),
        ActionType::YoutubeSearch => media::youtube_search(&cmd.parameters),
        ActionType::GithubSearch => {
            let query = cmd.parameters.get("query").cloned().unwrap_or_default();
            let url = format!("https://github.com/search?q={}", urlencoding::encode(&query));
            open::that(&url).map_err(|e| format!("Failed to search GitHub: {}", e))?;
            Ok(format!("Searching GitHub for '{}'", query))
        }
        ActionType::RedditSearch => {
            let query = cmd.parameters.get("query").cloned().unwrap_or_default();
            let url = format!("https://www.reddit.com/search/?q={}", urlencoding::encode(&query));
            open::that(&url).map_err(|e| format!("Failed to search Reddit: {}", e))?;
            Ok(format!("Searching Reddit for '{}'", query))
        }
        ActionType::MailSearch => browser::mail_search(&cmd.parameters),
        ActionType::OpenFile => browser::open_path(&cmd.parameters, false),
        ActionType::OpenFolder => browser::open_path(&cmd.parameters, true),
        ActionType::CreateFolder => {
            Err("Folder creation is handled by the Neural Engine.".to_string())
        }
        ActionType::SystemControl => system::control(&cmd.parameters),
        ActionType::Clipboard => clipboard::execute(app_handle, &cmd.parameters),
        ActionType::Timer | ActionType::Reminder => timer::execute(app_handle, &cmd.parameters),
        ActionType::CancelTimer => {
            Ok("Timer cancellation not yet implemented.".to_string())
        }
        ActionType::ListTimers => {
            Ok("No active timers. (Timer listing coming soon)".to_string())
        }
        ActionType::Info => info::execute(&cmd.parameters),
        ActionType::MediaControl => media::control(&cmd.parameters),
        ActionType::AiChat | ActionType::AiWrite | ActionType::AiExplain |
        ActionType::AiSummarize | ActionType::AiResearch => {
            ai::execute(&cmd.parameters).await
        }
        ActionType::CustomRunCommand => {
            Err("Custom shell commands are not fully supported yet.".to_string())
        }
        ActionType::SmallTalk => {
            let msg = cmd.parameters.get("message").map(|s| s.as_str()).unwrap_or("");
            let reply = match msg {
                "hi" | "hello" | "hey" => "Hello, Sir. How can I assist you today?",
                "how are you" => "I'm operating at peak performance, Sir. Thank you for asking.",
                "who are you" | "what are you" => "I'm Hyde Agent, your personal AI desktop assistant. I can open apps, search the web, set reminders, answer questions, and much more.",
                "good morning" => "Good morning, Sir. Ready to conquer the day.",
                "good evening" => "Good evening, Sir. Need help winding down?",
                "goodnight" | "good night" => "Good night, Sir. Rest well.",
                "thank you" | "thanks" => "You're welcome, Sir. Happy to help.",
                "bye" | "goodbye" | "see you" => "Goodbye, Sir. I'll be here when you need me.",
                "what can you do" | "help" | "commands" => "I can open apps, launch websites, search the web, set timers and reminders, control volume and brightness, answer questions, write documents, and much more. Type 'settings' to see all commands.",
                _ => "Hello, Sir. How can I help?"
            };
            Ok(reply.to_string())
        }
    }
}
