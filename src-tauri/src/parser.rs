use std::collections::HashMap;
use strsim::levenshtein;
use regex::Regex;
use crate::types::{ActionType, ParseResult};
use crate::registry::BUILTIN_COMMANDS;
use crate::custom_commands::get_custom_commands;

pub fn parse_input(input: &str) -> Option<ParseResult> {
    let normalized = normalize(input);
    if normalized.is_empty() {
        return None;
    }

    // 1. Exact custom commands
    for cmd in get_custom_commands() {
        if cmd.keyword.to_lowercase() == normalized {
            return Some(ParseResult {
                action_type: cmd.action_type,
                parameters: cmd.parameters,
                confidence: 1.0,
            });
        }
    }

    // 2. Exact built-in
    for cmd in BUILTIN_COMMANDS.iter() {
        if cmd.keyword.to_lowercase() == normalized {
            return Some(ParseResult {
                action_type: cmd.action_type.clone(),
                parameters: cmd.parameters.clone(),
                confidence: 1.0,
            });
        }
        for alias in &cmd.aliases {
            if alias.to_lowercase() == normalized {
                return Some(ParseResult {
                    action_type: cmd.action_type.clone(),
                    parameters: cmd.parameters.clone(),
                    confidence: 1.0,
                });
            }
        }
    }

    // 3. Check for URLs or "open " command which could be a URL or App
    let mut target_str = normalized.clone();
    let has_open_prefix = if let Some(rest) = normalized.strip_prefix("open ") {
        target_str = rest.to_string();
        true
    } else if let Some(rest) = normalized.strip_prefix("go to ") {
        target_str = rest.to_string();
        false
    } else {
        false
    };

    if target_str.starts_with("http://") || target_str.starts_with("https://") || 
       target_str.ends_with(".com") || target_str.ends_with(".org") || 
       target_str.ends_with(".net") || target_str.ends_with(".io") {
        return Some(ParseResult {
            action_type: ActionType::OpenUrl,
            parameters: HashMap::from([("url".to_string(), target_str)]),
            confidence: 1.0,
        });
    }

    if has_open_prefix {
        // Check if the target is a builtin command (like a URL)
        for cmd in BUILTIN_COMMANDS.iter() {
            if cmd.keyword.to_lowercase() == target_str {
                return Some(ParseResult {
                    action_type: cmd.action_type.clone(),
                    parameters: cmd.parameters.clone(),
                    confidence: 1.0,
                });
            }
            for alias in &cmd.aliases {
                if alias.to_lowercase() == target_str {
                    return Some(ParseResult {
                        action_type: cmd.action_type.clone(),
                        parameters: cmd.parameters.clone(),
                        confidence: 1.0,
                    });
                }
            }
        }

        // Assume app name if just "open X" and not a URL
        return Some(ParseResult {
            action_type: ActionType::OpenApp,
            parameters: HashMap::from([("app_name".to_string(), target_str)]),
            confidence: 0.9,
        });
    }

    // 4. Prefix Rules
    if let Some(res) = check_prefixes(&normalized) {
        return Some(res);
    }

    // 5. Fuzzy built-in
    for cmd in BUILTIN_COMMANDS.iter() {
        let dist = levenshtein(&cmd.keyword.to_lowercase(), &normalized);
        if dist <= 2 && cmd.keyword.len() > 3 {
            return Some(ParseResult {
                action_type: cmd.action_type.clone(),
                parameters: cmd.parameters.clone(),
                confidence: 0.8,
            });
        }
    }

    // Global Fallback
    Some(ParseResult {
        action_type: ActionType::WebSearch,
        parameters: HashMap::from([("query".to_string(), input.to_string())]),
        confidence: 0.1,
    })
}

fn check_prefixes(n: &str) -> Option<ParseResult> {
    // Media / Search
    if let Some(rest) = n.strip_prefix("play ") {
        let q = rest.strip_suffix(" on youtube").unwrap_or(rest);
        return Some(ParseResult {
            action_type: ActionType::YoutubeSearch,
            parameters: HashMap::from([("query".to_string(), q.to_string())]),
            confidence: 0.9,
        });
    }
    if let Some(rest) = n.strip_prefix("search youtube ") {
        return Some(ParseResult {
            action_type: ActionType::YoutubeSearch,
            parameters: HashMap::from([("query".to_string(), rest.to_string())]),
            confidence: 0.9,
        });
    }
    if let Some(rest) = n.strip_prefix("youtube ") {
        return Some(ParseResult {
            action_type: ActionType::YoutubeSearch,
            parameters: HashMap::from([("query".to_string(), rest.to_string())]),
            confidence: 0.9,
        });
    }
    if let Some(rest) = n.strip_prefix("search ") {
        return Some(ParseResult {
            action_type: ActionType::WebSearch,
            parameters: HashMap::from([("query".to_string(), rest.to_string())]),
            confidence: 0.9,
        });
    }
    if let Some(rest) = n.strip_prefix("google ") {
        return Some(ParseResult {
            action_type: ActionType::WebSearch,
            parameters: HashMap::from([("query".to_string(), rest.to_string())]),
            confidence: 0.9,
        });
    }
    if let Some(rest) = n.strip_prefix("look up ") {
        return Some(ParseResult {
            action_type: ActionType::WebSearch,
            parameters: HashMap::from([("query".to_string(), rest.to_string())]),
            confidence: 0.9,
        });
    }

    // Advanced NLU Regex Intent matching for Mail
    let mail_from_re = Regex::new(r"^(?:find|search|show)(?: me)?(?: the)? (?:mail|mails|email|emails) (?:from|by) (.+)").unwrap();
    if let Some(caps) = mail_from_re.captures(n) {
        let sender = caps.get(1).unwrap().as_str();
        return Some(ParseResult {
            action_type: ActionType::MailSearch,
            parameters: HashMap::from([("provider".to_string(), "gmail".to_string()), ("query".to_string(), format!("from:{}", sender))]),
            confidence: 0.9,
        });
    }

    let mail_about_re = Regex::new(r"^(?:find|search|show)(?: me)?(?: the)? (?:mail|mails|email|emails) (?:about|for|regarding|of) (.+)").unwrap();
    if let Some(caps) = mail_about_re.captures(n) {
        let topic = caps.get(1).unwrap().as_str();
        return Some(ParseResult {
            action_type: ActionType::MailSearch,
            parameters: HashMap::from([("provider".to_string(), "gmail".to_string()), ("query".to_string(), topic.to_string())]),
            confidence: 0.9,
        });
    }

    // Open App/File/Folder
    if let Some(rest) = n.strip_prefix("open folder ") {
        return Some(ParseResult {
            action_type: ActionType::OpenFolder,
            parameters: HashMap::from([("path".to_string(), rest.to_string())]),
            confidence: 0.9,
        });
    }
    if let Some(rest) = n.strip_prefix("open file ") {
        return Some(ParseResult {
            action_type: ActionType::OpenFile,
            parameters: HashMap::from([("path".to_string(), rest.to_string())]),
            confidence: 0.9,
        });
    }
    if let Some(rest) = n.strip_prefix("go to ") {
        return Some(ParseResult {
            action_type: ActionType::OpenUrl,
            parameters: HashMap::from([("url".to_string(), rest.to_string())]),
            confidence: 0.9,
        });
    }

    // Clipboard
    if let Some(rest) = n.strip_prefix("copy ") {
        return Some(ParseResult {
            action_type: ActionType::Clipboard,
            parameters: HashMap::from([("action".to_string(), "copy".to_string()), ("text".to_string(), rest.to_string())]),
            confidence: 0.9,
        });
    }

    // Timer
    if let Some(rest) = n.strip_prefix("set timer ") {
        if let Some(mins) = rest.strip_suffix(" minutes") {
            return Some(ParseResult {
                action_type: ActionType::Timer,
                parameters: HashMap::from([("minutes".to_string(), mins.to_string())]),
                confidence: 0.9,
            });
        }
    }
    if let Some(rest) = n.strip_prefix("timer ") {
        if let Some(mins) = rest.strip_suffix(" minutes") {
            return Some(ParseResult {
                action_type: ActionType::Timer,
                parameters: HashMap::from([("minutes".to_string(), mins.to_string())]),
                confidence: 0.9,
            });
        }
    }

    // System Volume Set
    if let Some(rest) = n.strip_prefix("set volume ") {
        return Some(ParseResult {
            action_type: ActionType::SystemControl,
            parameters: HashMap::from([("action".to_string(), "set_volume".to_string()), ("value".to_string(), rest.to_string())]),
            confidence: 0.9,
        });
    }

    None
}

fn normalize(input: &str) -> String {
    let mut cleaned = input.to_string();
    cleaned = cleaned.replace(".", "").replace(",", "").replace("!", "").replace("?", "");
    
    // NLU filler word stripping
    let fillers = ["can you please ", "could you ", "would you ", "please ", "i want to ", "let's ", "can you ", "hey hyde "];
    let mut lower = cleaned.trim().to_lowercase();
    for f in fillers.iter() {
        if lower.starts_with(f) {
            lower = lower.replacen(f, "", 1);
        }
    }
    
    lower.split_whitespace().collect::<Vec<&str>>().join(" ")
}
