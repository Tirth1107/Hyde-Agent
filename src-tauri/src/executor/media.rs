use std::collections::HashMap;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::process::Command;

pub fn youtube_search(params: &HashMap<String, String>) -> Result<String, String> {
    let query = params.get("query").ok_or("No video name provided")?;
    let encoded = utf8_percent_encode(query, NON_ALPHANUMERIC).to_string();
    let url = format!("https://www.youtube.com/results?search_query={}", encoded);
    
    open::that(&url).map_err(|e| format!("Failed to open YouTube: {}", e))?;
    Ok(format!("Opened YouTube results for '{}'", query))
}

pub fn control(params: &HashMap<String, String>) -> Result<String, String> {
    let action = params.get("action").ok_or("No media action provided")?;
    
    #[cfg(target_os = "windows")]
    {
        let vk = match action.as_str() {
            "pause" | "play" | "play_pause" => "0xB3", // VK_MEDIA_PLAY_PAUSE
            "next" => "0xB0", // VK_MEDIA_NEXT_TRACK
            "previous" => "0xB1", // VK_MEDIA_PREV_TRACK
            _ => return Err(format!("Unknown media action: {}", action)),
        };

        let ps_cmd = format!(
            "Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public class Keyboard {{ [DllImport(\"user32.dll\")] public static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtraInfo); }}';\n\
            [Keyboard]::keybd_event({}, 0, 0, 0);\n\
            [Keyboard]::keybd_event({}, 0, 2, 0);",
            vk, vk
        );

        let mut cmd = Command::new("powershell");
        cmd.args(&["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_cmd]);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        cmd.spawn()
            .map_err(|e| format!("Failed to execute media control: {}", e))?;
            
        let msg = match action.as_str() {
            "pause" | "play" | "play_pause" => "Toggled playback",
            "next" => "Skipped to next track",
            "previous" => "Went to previous track",
            _ => "Media controlled",
        };
        Ok(format!("{}, Sir.", msg))
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("Media controls not implemented for this OS".to_string())
    }
}
