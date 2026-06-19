use std::collections::HashMap;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

pub fn youtube_search(params: &HashMap<String, String>) -> Result<String, String> {
    let query = params.get("query").ok_or("No video name provided")?;
    let encoded = utf8_percent_encode(query, NON_ALPHANUMERIC).to_string();
    let url = format!("https://www.youtube.com/results?search_query={}", encoded);
    
    open::that(&url).map_err(|e| format!("Failed to open YouTube: {}", e))?;
    Ok(format!("Opened YouTube results for '{}'", query))
}
