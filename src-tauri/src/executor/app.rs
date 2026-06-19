use std::collections::HashMap;
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use urlencoding::encode;

pub fn launch(params: &HashMap<String, String>) -> Result<String, String> {
    let app_name = params.get("app_name").ok_or("No app name provided")?;
    
    // 1. First, try executing the app name or its typical executable directly
    let exe_name = match app_name.as_str() {
        "brave" => "brave.exe",
        "chrome" => "chrome.exe",
        "firefox" => "firefox.exe",
        "edge" => "msedge.exe",
        "vscode" => "code",
        "discord" => "Update.exe --processStart Discord.exe",
        "spotify" => "spotify.exe",
        _ => app_name.as_str(),
    };
    
    if open::that(exe_name).is_ok() {
        return Ok(format!("Opened {}", app_name));
    }

    // 2. Dynamic Fallback: Search Windows Start Menu using PowerShell
    #[cfg(target_os = "windows")]
    {
        let ps_cmd = format!("Get-StartApps '*{}*' | Select-Object -First 1 -ExpandProperty AppID", app_name);
        let mut cmd = Command::new("powershell");
        cmd.args(&["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_cmd]);
        cmd.creation_flags(0x08000000);
        
        if let Ok(output) = cmd.output() 
        {
            let app_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !app_id.is_empty() {
                let launch_cmd = format!("shell:AppsFolder\\{}", app_id);
                if open::that(&launch_cmd).is_ok() {
                    return Ok(format!("Found and opened {} via Windows Search", app_name));
                }
            }
        }
    }

    // 3. Graceful Fallback: Web Search instead of error
    let query = encode(app_name);
    let search_url = format!("https://www.google.com/search?q={}", query);
    match open::that(&search_url) {
        Ok(_) => Ok(format!("I couldn't find '{}' locally, so I searched the web for it.", app_name)),
        Err(_) => Ok(format!("Sorry, I couldn't find '{}' or open the web browser.", app_name))
    }
}
