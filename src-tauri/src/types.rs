use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    OpenApp,
    OpenUrl,
    WebSearch,
    YoutubeSearch,
    OpenFile,
    OpenFolder,
    MailSearch,
    SystemControl,
    Clipboard,
    Timer,
    Info,
    CustomRunCommand,
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
