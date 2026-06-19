use std::collections::HashMap;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

pub fn open_url(params: &HashMap<String, String>) -> Result<String, String> {
    let url = params.get("url").ok_or("No URL provided")?;
    let mut target = url.clone();
    if !target.starts_with("http") {
        target = format!("https://{}", target);
    }
    
    open::that(&target).map_err(|e| format!("Failed to open URL: {}", e))?;
    Ok(format!("Opened {}", target))
}

pub fn web_search(params: &HashMap<String, String>) -> Result<String, String> {
    let query = params.get("query").ok_or("No search query provided")?;
    let encoded = utf8_percent_encode(query, NON_ALPHANUMERIC).to_string();
    let url = format!("https://www.google.com/search?q={}", encoded);
    
    open::that(&url).map_err(|e| format!("Failed to search: {}", e))?;
    Ok(format!("Searching Google for '{}'", query))
}

pub fn mail_search(params: &HashMap<String, String>) -> Result<String, String> {
    let query = params.get("query").ok_or("No query provided")?;
    let encoded = utf8_percent_encode(query, NON_ALPHANUMERIC).to_string();
    let provider = params.get("provider").map(|s| s.as_str()).unwrap_or("gmail");
    
    let url = match provider {
        "gmail" => format!("https://mail.google.com/mail/u/0/#search/{}", encoded),
        "yahoo" => format!("https://mail.yahoo.com/d/search/keyword={}", encoded),
        "outlook" => "https://outlook.live.com/mail/0/inbox".to_string(), // Outlook doesn't support direct URL search well
        _ => return Err("Unknown mail provider".to_string()),
    };
    
    open::that(&url).map_err(|e| format!("Failed to open mail: {}", e))?;
    Ok(format!("Searching {} for '{}'", provider, query))
}

pub fn open_path(params: &HashMap<String, String>, is_folder: bool) -> Result<String, String> {
    let path = params.get("path").ok_or("No path provided")?;
    open::that(path).map_err(|e| format!("Could not open {}: {}", if is_folder { "folder" } else { "file" }, e))?;
    Ok(format!("Opened {}", path))
}
