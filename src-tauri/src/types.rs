use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    // App Operations
    OpenApp,
    CloseApp,
    
    // Website & URL
    OpenUrl,
    
    // Search
    WebSearch,
    YoutubeSearch,
    GithubSearch,
    RedditSearch,
    MailSearch,
    
    // File Operations
    OpenFile,
    OpenFolder,
    CreateFolder,
    
    // System Controls
    SystemControl,
    
    // Clipboard
    Clipboard,
    
    // Timer & Reminders
    Timer,
    Reminder,
    CancelTimer,
    ListTimers,
    
    // System Info
    Info,
    
    // Media Controls
    MediaControl,
    
    // AI Intents
    AiChat,
    AiWrite,
    AiExplain,
    AiSummarize,
    AiResearch,
    
    // Custom
    CustomRunCommand,
    
    // Conversational
    SmallTalk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
    pub keyword: String,
    pub aliases: Vec<String>,
    pub action_type: ActionType,
    pub parameters: std::collections::HashMap<String, String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParseResult {
    pub action_type: ActionType,
    pub parameters: std::collections::HashMap<String, String>,
    pub confidence: f32, // 1.0 exact, 0.9 prefix, 0.8 fuzzy, 0.0 no match
}
