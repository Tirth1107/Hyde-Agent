use std::collections::HashMap;
use crate::ipc::IpcResult;
use crate::types::ActionType;
use crate::executor;
use tauri::AppHandle;

pub async fn execute(app: &AppHandle, cmd: IpcResult) -> Result<String, String> {
    let intent_str = cmd.intent.clone().unwrap_or_else(|| "GENERAL_AI_CHAT".to_string());
    crate::logger::log_action("ROUTER", &format!("Executing intent: {}", intent_str), true);
    
    // We can map IPC string intents to Rust ActionType, or execute directly based on intent string
    // Let's create a basic mapping to ActionType for the Rust executors:
    let action_type = match intent_str.as_str() {
        "OPEN_APP" => ActionType::OpenApp,
        "CLOSE_APP" => ActionType::CloseApp,
        "OPEN_WEBSITE" => ActionType::OpenUrl,
        "WEB_SEARCH" => ActionType::WebSearch,
        "YOUTUBE_SEARCH" => ActionType::YoutubeSearch,
        "GITHUB_SEARCH" => ActionType::GithubSearch,
        "REDDIT_SEARCH" => ActionType::RedditSearch,
        "MAIL_SEARCH" => ActionType::MailSearch,
        "OPEN_FILE" => ActionType::OpenFile,
        "OPEN_FOLDER" => ActionType::OpenFolder,
        "COPY_CLIPBOARD" | "VIEW_CLIPBOARD" => ActionType::Clipboard,
        "SET_TIMER" => ActionType::Timer,
        "SET_REMINDER" => ActionType::Reminder,
        "CANCEL_TIMER" => ActionType::CancelTimer,
        "LIST_TIMERS" => ActionType::ListTimers,
        "TIME_QUERY" | "DATE_QUERY" | "BATTERY_QUERY" | "NETWORK_QUERY" => ActionType::Info,
        "VOLUME_UP" | "VOLUME_DOWN" | "VOLUME_MUTE" | "PLAY_MUSIC" | "PAUSE_MUSIC" | "NEXT_TRACK" | "PREV_TRACK" => ActionType::MediaControl,
        "SLEEP_PC" | "SHUTDOWN_PC" | "RESTART_PC" | "LOCK_PC" | "EMPTY_TRASH" | "SCREENSHOT" => ActionType::SystemControl,
        "AI_CHAT" | "GENERAL_AI_CHAT" => ActionType::AiChat,
        "AI_WRITE" | "AI_GENERATION" => ActionType::AiWrite,
        "AI_EXPLAIN" => ActionType::AiExplain,
        "AI_SUMMARIZE" => ActionType::AiSummarize,
        "AI_RESEARCH" => ActionType::AiResearch,
        _ => ActionType::AiChat,
    };
    
    // Convert HashMap<String, serde_json::Value> to HashMap<String, String>
    let mut str_params = HashMap::new();
    if let Some(params) = cmd.parameters {
        for (k, v) in params {
            if let Some(s) = v.as_str() {
                str_params.insert(k, s.to_string());
            } else {
                str_params.insert(k, v.to_string());
            }
        }
    }
    
    let confidence = cmd.confidence.unwrap_or(0.0) as f32;
    
    let rust_cmd = crate::types::ParseResult {
        action_type: action_type.clone(),
        parameters: str_params.clone(),
        confidence,
    };
    
    // 1. Exact OS Execution (Fast Path & Rust Handlers)
    // Only execute natively if confidence is decent
    if confidence >= 0.70 && !is_ai_intent(&action_type) {
        let execution_result = executor::execute(app, rust_cmd.clone()).await;
        match execution_result {
            Ok(response) => {
                if !response.is_empty() {
                    return Ok(response);
                }
            }
            Err(e) => {
                crate::logger::log_action("ROUTER_WARN", &format!("Execution returned error: {}", e), true);
                // Return high-confidence native execution errors directly to the user
                return Err(e);
            }
        }
    }
    
    // 2. AI Fallback Engine
    if is_ai_intent(&action_type) || confidence < 0.70 {
        let ai_query = cmd.ai_query.clone().unwrap_or_else(|| {
            str_params.get("query").cloned().unwrap_or_else(|| str_params.get("message").cloned().unwrap_or_default())
        });
        
        let ai_type = match action_type {
            ActionType::AiExplain => "explain",
            ActionType::AiSummarize => "summarize",
            ActionType::AiWrite => "write",
            ActionType::AiResearch => "research",
            _ => "chat"
        }.to_string();
        
        let mut params = HashMap::new();
        params.insert("query".to_string(), ai_query);
        params.insert("ai_type".to_string(), ai_type);
        
        match executor::ai::execute(app, &params).await {
            Ok(ai_response) => return Ok(ai_response),
            Err(e) => {
                crate::logger::log_action("ROUTER_AI_WARN", &format!("AI Failed: {}", e), true);
                return Err(format!("AI Engine Error: {}", e));
            }
        }
    }
    
    // 3. Absolute Fallback
    // If we reach here (which shouldn't happen with the new parser), return a generic error.
    Err("Command not recognized. Type 'help' to see all commands.".to_string())
}

fn is_ai_intent(action: &ActionType) -> bool {
    matches!(
        action,
        ActionType::AiChat | ActionType::AiWrite | ActionType::AiExplain | 
        ActionType::AiSummarize | ActionType::AiResearch
    )
}
