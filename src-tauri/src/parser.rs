//! Hyde Agent — Fast-Path Parser (Rust)
//!
//! This module provides a FAST exact-match parser for built-in commands.
//! It serves as an optimization layer: exact keyword matches are handled
//! instantly in Rust without IPC latency to the Python engine.
//!
//! For anything that isn't an exact match, the input flows to the
//! Python Neural Engine for NLU classification.
//!
//! IMPORTANT: This module does NOT have a web-search fallback.
//! If nothing matches, it returns None, and the caller routes to Python.

use std::collections::HashMap;
use strsim::levenshtein;
use regex::Regex;
use crate::types::{ActionType, ParseResult};
use crate::registry::BUILTIN_COMMANDS;
use crate::custom_commands::get_custom_commands;

/// Try ONLY exact keyword/alias matches. Returns None if no exact match.
/// This is the fast-path used before IPC to Python.
pub fn try_exact_match(normalized: &str) -> Option<ParseResult> {
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

    // 2. Exact built-in keyword or alias
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

    None
}

/// Full parser with prefix rules, fuzzy matching, etc.
/// Used as fallback when IPC to Python fails.
/// NOTE: No longer has a web-search fallback — returns None if nothing matches.
pub fn parse_input(input: &str) -> Option<ParseResult> {
    let normalized = normalize(input);
    if normalized.is_empty() {
        return None;
    }

    // 1. Exact match (custom + built-in)
    if let Some(result) = try_exact_match(&normalized) {
        return Some(result);
    }

    // 2. Check for URLs or "open " command which could be a URL or App
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
       target_str.ends_with(".net") || target_str.ends_with(".io") ||
       target_str.ends_with(".dev") || target_str.ends_with(".app") ||
       target_str.ends_with(".ai") || target_str.ends_with(".co") {
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

    // 3. Prefix Rules
    if let Some(res) = check_prefixes(&normalized) {
        return Some(res);
    }

    // 4. Fuzzy built-in
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

    // NO web-search fallback!
    // Return None so the caller knows to route through Python NLU.
    None
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

    // Mail regex patterns
    let mail_from_re = Regex::new(r"^(?:find|search|show)(?: me)?(?: the)? (?:mail|mails|email|emails) (?:from|by) (.+)").unwrap();
    if let Some(caps) = mail_from_re.captures(n) {
        let sender = caps.get(1).unwrap().as_str();
        return Some(ParseResult {
            action_type: ActionType::MailSearch,
            parameters: HashMap::from([("provider".to_string(), "gmail".to_string()), ("query".to_string(), format!("from:{}", sender))]),
            confidence: 0.9,
        });
    }

    let mail_about_re = Regex::new(r"^(?:find|search|show)(?: me)?(?: the)? (?:(?:mail|mails|email|emails) (?:about|for|regarding|of|with) (.+)|(.+) (?:in|on) (?:mail|mails|email|emails))").unwrap();
    if let Some(caps) = mail_about_re.captures(n) {
        let topic = caps.get(1).or_else(|| caps.get(2)).unwrap().as_str();
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
    let fillers = [
        "can you please ", "could you please ", "would you please ",
        "could you ", "would you ", "please ", "i want to ", "i'd like to ",
        "i need you to ", "i need to ", "let's ", "can you ", "hey hyde ",
        "hi hyde ", "okay hyde ", "go ahead and ", "just ", "kindly ",
    ];
    let mut lower = cleaned.trim().to_lowercase();
    for f in fillers.iter() {
        if lower.starts_with(f) {
            lower = lower.replacen(f, "", 1);
        }
    }
    
    lower.split_whitespace().collect::<Vec<&str>>().join(" ")
}
