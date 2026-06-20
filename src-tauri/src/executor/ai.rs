use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Deserialize)]
struct GeminiResponsePart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

// Global static for the API key (for now, will be replaced with settings)
static API_KEY: std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub fn set_api_key(key: String) {
    let _ = API_KEY.set(key);
}

pub async fn execute(params: &HashMap<String, String>) -> Result<String, String> {
    let query = params.get("query").ok_or("No query provided")?;
    let ai_type = params.get("ai_type").cloned().unwrap_or_else(|| "chat".to_string());
    
    // Check if key exists
    let key = match API_KEY.get() {
        Some(k) if !k.is_empty() => k,
        _ => return Ok(format!("🤖 [AI Mode]\n\nYour query: \"{}\"\n\nTo use AI features, please set your Gemini API key in Settings.", query)),
    };
    
    // Pre-prompt based on ai_type
    let prompt = match ai_type.as_str() {
        "explain" => format!("Explain the following simply and concisely:\n\n{}", query),
        "summarize" => format!("Summarize the following:\n\n{}", query),
        "write" => format!("Write the following:\n\n{}", query),
        "research" => format!("Research and provide facts about:\n\n{}", query),
        _ => format!("You are Hyde, a helpful AI assistant. Answer concisely and politely.\n\nUser: {}", query),
    };
    
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}", key);
    
    let request_body = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart {
                text: prompt,
            }],
        }],
    };
    
    let client = reqwest::Client::new();
    let res = client.post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to AI: {}", e))?;
        
    if !res.status().is_success() {
        return Err(format!("AI API Error: {}", res.status()));
    }
    
    let data: GeminiResponse = res.json().await
        .map_err(|e| format!("Failed to parse AI response: {}", e))?;
        
    if let Some(candidates) = data.candidates {
        if let Some(first) = candidates.first() {
            if let Some(part) = first.content.parts.first() {
                return Ok(part.text.clone());
            }
        }
    }
    
    Ok("I received an empty response from the AI.".to_string())
}
