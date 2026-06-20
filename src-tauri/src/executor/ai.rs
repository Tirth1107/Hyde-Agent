use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::core::db;

use tauri_plugin_clipboard_manager::ClipboardExt;

pub async fn execute(app: &tauri::AppHandle, params: &HashMap<String, String>) -> Result<String, String> {
    let query = params.get("query").ok_or("No query provided")?;
    let ai_type = params.get("ai_type").cloned().unwrap_or_else(|| "chat".to_string());
    
    // Get provider from DB, default to Gemini
    let provider = db::get_setting("ai_provider").unwrap_or_else(|| "gemini".to_string());
    
    // For Ollama we don't strictly need a key, for others we do
    let key = if provider != "ollama" {
        match db::get_setting("ai_api_key") {
            Some(k) if !k.is_empty() => k,
            _ => {
                // Mode 1: Browser AI Mode (Default Fallback)
                let encoded_query = percent_encoding::utf8_percent_encode(&query, percent_encoding::NON_ALPHANUMERIC).to_string();
                
                // Put query in clipboard just in case the provider doesn't auto-fill
                let _ = app.clipboard().write_text(query.clone());
                
                let url = match provider.as_str() {
                    "chatgpt" => format!("https://chatgpt.com/?q={}", encoded_query),
                    "claude" => format!("https://claude.ai/new?q={}", encoded_query),
                    "perplexity" => format!("https://www.perplexity.ai/search?q={}", encoded_query),
                    "grok" => format!("https://grok.com/?q={}", encoded_query),
                    "gemini" | _ => format!("https://gemini.google.com/app?q={}", encoded_query),
                };
                
                open::that(&url).map_err(|e| format!("Failed to open AI in browser: {}", e))?;
                
                let provider_clone = provider.clone();
                tokio::spawn(async move {
                    // Wait for the browser to open and the page to load
                    tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
                    
                    if provider_clone == "gemini" || provider_clone == "claude" {
                        use enigo::{Enigo, Settings, Keyboard, Direction, Key};
                        if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
                            // Ctrl+V to paste
                            let _ = enigo.key(Key::Control, Direction::Press);
                            let _ = enigo.key(Key::Unicode('v'), Direction::Click);
                            let _ = enigo.key(Key::Control, Direction::Release);
                            
                            // Wait a moment for the paste to register
                            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                            
                            // Press Enter to submit the prompt
                            let _ = enigo.key(Key::Return, Direction::Click);
                        }
                    }
                });
                
                let display_provider = match provider.as_str() {
                    "chatgpt" => "ChatGPT",
                    "claude" => "Claude",
                    "perplexity" => "Perplexity",
                    "grok" => "Grok",
                    _ => "Gemini",
                };
                
                return Ok(format!("Opening {} to answer your question. (Prompt copied to clipboard)", display_provider));
            }
        }
    } else {
        String::new()
    };
    
    let prompt = match ai_type.as_str() {
        "explain" => format!("Explain the following simply and concisely:\n\n{}", query),
        "summarize" => format!("Summarize the following:\n\n{}", query),
        "write" => format!("Write the following:\n\n{}", query),
        "research" => format!("Research and provide facts about:\n\n{}", query),
        _ => format!("You are Hyde, a helpful AI assistant. Answer concisely and politely.\n\nUser: {}", query),
    };
    
    // Route to appropriate provider
    let response = match provider.as_str() {
        "chatgpt" => call_openai(&key, &prompt).await,
        "claude" => call_claude(&key, &prompt).await,
        "perplexity" => call_perplexity(&key, &prompt).await,
        "ollama" => call_ollama(&prompt).await,
        "gemini" | _ => call_gemini(&key, &prompt).await,
    };
    
    if let Ok(ref res_text) = response {
        let _ = db::add_memory("default", "user", query);
        let _ = db::add_memory("default", "assistant", res_text);
    }
    
    response
}

// ─── Gemini Implementation ───
#[derive(Serialize)] struct GeminiPart { text: String }
#[derive(Serialize)] struct GeminiContent { parts: Vec<GeminiPart> }
#[derive(Serialize)] struct GeminiRequest { contents: Vec<GeminiContent> }
#[derive(Deserialize)] struct GeminiResponsePart { text: String }
#[derive(Deserialize)] struct GeminiResponseContent { parts: Vec<GeminiResponsePart> }
#[derive(Deserialize)] struct GeminiCandidate { content: GeminiResponseContent }
#[derive(Deserialize)] struct GeminiResponse { candidates: Option<Vec<GeminiCandidate>> }

async fn call_gemini(key: &str, prompt: &str) -> Result<String, String> {
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}", key);
    let req = GeminiRequest { contents: vec![GeminiContent { parts: vec![GeminiPart { text: prompt.to_string() }] }] };
    let res = reqwest::Client::new().post(&url).json(&req).send().await.map_err(|e| e.to_string())?;
    
    if !res.status().is_success() { return Err(format!("API Error: {}", res.status())); }
    let data: GeminiResponse = res.json().await.map_err(|e| e.to_string())?;
    
    if let Some(cands) = data.candidates {
        if let Some(first) = cands.first() {
            if let Some(part) = first.content.parts.first() {
                return Ok(part.text.clone());
            }
        }
    }
    Err("Empty AI response".to_string())
}

// ─── OpenAI Implementation ───
#[derive(Serialize, Deserialize)] struct OaiMsg { role: String, content: String }
#[derive(Serialize)] struct OaiReq { model: String, messages: Vec<OaiMsg> }
#[derive(Deserialize)] struct OaiChoice { message: OaiMsg }
#[derive(Deserialize)] struct OaiRes { choices: Vec<OaiChoice> }

async fn call_openai(key: &str, prompt: &str) -> Result<String, String> {
    let mut msgs = Vec::new();
    
    // Add history
    let history = db::get_memory("default", 10);
    for h in history {
        msgs.push(OaiMsg { role: h.role, content: h.content });
    }
    
    msgs.push(OaiMsg { role: "user".to_string(), content: prompt.to_string() });

    let req = OaiReq {
        model: "gpt-4o".to_string(),
        messages: msgs
    };
    let res = reqwest::Client::new().post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(key).json(&req).send().await.map_err(|e| e.to_string())?;
        
    if !res.status().is_success() { return Err(format!("API Error: {}", res.status())); }
    let data: OaiRes = res.json().await.map_err(|e| e.to_string())?;
    
    data.choices.first().map(|c| c.message.content.clone()).ok_or_else(|| "Empty response".to_string())
}

// ─── Claude Implementation ───
#[derive(Serialize)] struct ClaudeReq { model: String, max_tokens: u32, messages: Vec<OaiMsg> }
#[derive(Deserialize)] struct ClaudeContent { text: String }
#[derive(Deserialize)] struct ClaudeRes { content: Vec<ClaudeContent> }

async fn call_claude(key: &str, prompt: &str) -> Result<String, String> {
    let req = ClaudeReq {
        model: "claude-3-5-sonnet-20240620".to_string(),
        max_tokens: 1024,
        messages: vec![OaiMsg { role: "user".to_string(), content: prompt.to_string() }]
    };
    let res = reqwest::Client::new().post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .json(&req).send().await.map_err(|e| e.to_string())?;
        
    if !res.status().is_success() { return Err(format!("API Error: {}", res.status())); }
    let data: ClaudeRes = res.json().await.map_err(|e| e.to_string())?;
    
    data.content.first().map(|c| c.text.clone()).ok_or_else(|| "Empty response".to_string())
}

// ─── Perplexity Implementation ───
async fn call_perplexity(key: &str, prompt: &str) -> Result<String, String> {
    let req = OaiReq {
        model: "llama-3-sonar-large-32k-online".to_string(),
        messages: vec![OaiMsg { role: "user".to_string(), content: prompt.to_string() }]
    };
    let res = reqwest::Client::new().post("https://api.perplexity.ai/chat/completions")
        .bearer_auth(key).json(&req).send().await.map_err(|e| e.to_string())?;
        
    if !res.status().is_success() { return Err(format!("API Error: {}", res.status())); }
    let data: OaiRes = res.json().await.map_err(|e| e.to_string())?;
    
    data.choices.first().map(|c| c.message.content.clone()).ok_or_else(|| "Empty response".to_string())
}

// ─── Ollama Implementation (Local) ───
#[derive(Serialize)] struct OllamaReq { model: String, prompt: String, stream: bool }
#[derive(Deserialize)] struct OllamaRes { response: String }

async fn call_ollama(prompt: &str) -> Result<String, String> {
    let req = OllamaReq {
        model: "llama3".to_string(),
        prompt: prompt.to_string(),
        stream: false,
    };
    let res = reqwest::Client::new().post("http://localhost:11434/api/generate")
        .json(&req).send().await.map_err(|e| e.to_string())?;
        
    if !res.status().is_success() { return Err(format!("API Error: {}", res.status())); }
    let data: OllamaRes = res.json().await.map_err(|e| e.to_string())?;
    
    Ok(data.response)
}

pub fn set_api_key(key: String) {
    let _ = db::set_setting("ai_api_key", &key);
}
