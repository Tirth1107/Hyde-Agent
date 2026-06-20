use std::collections::HashMap;
use std::process::Command;
use enigo::{Enigo, Keyboard, Key, Settings};

pub fn control(params: &HashMap<String, String>) -> Result<String, String> {
    let action = params.get("action").ok_or("No action provided")?;
    
    match action.as_str() {
        "volume up" => {
            let mut enigo = Enigo::new(&Settings::default()).unwrap();
            for _ in 0..5 { // 5 steps usually equals 10%
                let _ = enigo.key(Key::VolumeUp, enigo::Direction::Click);
            }
            Ok("Increased volume".to_string())
        }
        "volume down" => {
            let mut enigo = Enigo::new(&Settings::default()).unwrap();
            for _ in 0..5 {
                let _ = enigo.key(Key::VolumeDown, enigo::Direction::Click);
            }
            Ok("Decreased volume".to_string())
        }
        "mute" => {
            let mut enigo = Enigo::new(&Settings::default()).unwrap();
            let _ = enigo.key(Key::VolumeMute, enigo::Direction::Click);
            Ok("Toggled mute".to_string())
        }
        "lock screen" => {
            #[cfg(target_os = "windows")]
            {
                Command::new("rundll32.exe")
                    .arg("user32.dll,LockWorkStation")
                    .spawn()
                    .map_err(|e| format!("Failed to lock screen: {}", e))?;
                Ok("Locked screen".to_string())
            }
            #[cfg(not(target_os = "windows"))]
            {
                Err("Lock screen not implemented for this OS".to_string())
            }
        }
        "sleep" => {
            #[cfg(target_os = "windows")]
            {
                Command::new("rundll32.exe")
                    .args(&["powrprof.dll,SetSuspendState", "0,1,0"])
                    .spawn()
                    .map_err(|e| format!("Failed to sleep: {}", e))?;
                Ok("Going to sleep".to_string())
            }
            #[cfg(not(target_os = "windows"))]
            {
                Err("Sleep not implemented for this OS".to_string())
            }
        }
        "shutdown" => {
            #[cfg(target_os = "windows")]
            {
                Command::new("shutdown.exe")
                    .args(&["/s", "/t", "5"])
                    .spawn()
                    .map_err(|e| format!("Failed to shutdown: {}", e))?;
                Ok("Shutting down in 5 seconds...".to_string())
            }
            #[cfg(not(target_os = "windows"))]
            {
                Err("Shutdown not implemented for this OS".to_string())
            }
        }
        "restart" => {
            #[cfg(target_os = "windows")]
            {
                Command::new("shutdown.exe")
                    .args(&["/r", "/t", "5"])
                    .spawn()
                    .map_err(|e| format!("Failed to restart: {}", e))?;
                Ok("Restarting in 5 seconds...".to_string())
            }
            #[cfg(not(target_os = "windows"))]
            {
                Err("Restart not implemented for this OS".to_string())
            }
        }
        "screenshot" => {
            #[cfg(target_os = "windows")]
            {
                // Use PowerShell to simulate Win+PrintScreen (saves to Pictures\Screenshots)
                let ps_cmd = r#"
                    Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public class Keyboard { [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtraInfo); }';
                    [Keyboard]::keybd_event(0x5B, 0, 0, 0);
                    [Keyboard]::keybd_event(0x2C, 0, 0, 0);
                    Start-Sleep -Milliseconds 100;
                    [Keyboard]::keybd_event(0x2C, 0, 2, 0);
                    [Keyboard]::keybd_event(0x5B, 0, 2, 0);
                "#;
                let mut cmd = Command::new("powershell");
                cmd.args(&["-NoProfile", "-WindowStyle", "Hidden", "-Command", ps_cmd]);
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::process::CommandExt;
                    cmd.creation_flags(0x08000000);
                }
                cmd.spawn()
                    .map_err(|e| format!("Failed to take screenshot: {}", e))?;
                Ok("Screenshot taken and saved to Pictures\\Screenshots".to_string())
            }
            #[cfg(not(target_os = "windows"))]
            {
                Err("Screenshot not implemented for this OS".to_string())
            }
        }
        "brightness up" => {
            #[cfg(target_os = "windows")]
            {
                let ps_cmd = r#"
                    $brightness = (Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightness).CurrentBrightness;
                    $new = [Math]::Min(100, $brightness + 10);
                    (Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods).WmiSetBrightness(1, $new)
                "#;
                let mut cmd = Command::new("powershell");
                cmd.args(&["-NoProfile", "-WindowStyle", "Hidden", "-Command", ps_cmd]);
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::process::CommandExt;
                    cmd.creation_flags(0x08000000);
                }
                match cmd.output() {
                    Ok(_) => Ok("Increased brightness".to_string()),
                    Err(e) => Err(format!("Failed to adjust brightness: {}", e)),
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                Err("Brightness control not implemented for this OS".to_string())
            }
        }
        "brightness down" => {
            #[cfg(target_os = "windows")]
            {
                let ps_cmd = r#"
                    $brightness = (Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightness).CurrentBrightness;
                    $new = [Math]::Max(0, $brightness - 10);
                    (Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods).WmiSetBrightness(1, $new)
                "#;
                let mut cmd = Command::new("powershell");
                cmd.args(&["-NoProfile", "-WindowStyle", "Hidden", "-Command", ps_cmd]);
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::process::CommandExt;
                    cmd.creation_flags(0x08000000);
                }
                match cmd.output() {
                    Ok(_) => Ok("Decreased brightness".to_string()),
                    Err(e) => Err(format!("Failed to adjust brightness: {}", e)),
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                Err("Brightness control not implemented for this OS".to_string())
            }
        }
        "empty trash" => {
            #[cfg(target_os = "windows")]
            {
                let mut cmd = Command::new("powershell");
                cmd.args(&["-NoProfile", "-WindowStyle", "Hidden", "-Command",
                    "Clear-RecycleBin -Force -ErrorAction SilentlyContinue"]);
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::process::CommandExt;
                    cmd.creation_flags(0x08000000);
                }
                match cmd.output() {
                    Ok(_) => Ok("Recycle bin emptied".to_string()),
                    Err(e) => Err(format!("Failed to empty recycle bin: {}", e)),
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                Err("Empty trash not implemented for this OS".to_string())
            }
        }
        "set_volume" => {
            let value = params.get("value").ok_or("No volume value provided")?;
            #[cfg(target_os = "windows")]
            {
                let ps_cmd = format!(
                    r#"
                    $vol = {};
                    $wshShell = New-Object -ComObject WScript.Shell;
                    # Reset to 0 then set to desired level
                    for ($i = 0; $i -lt 50; $i++) {{ $wshShell.SendKeys([char]174) }};
                    $steps = [Math]::Floor($vol / 2);
                    for ($i = 0; $i -lt $steps; $i++) {{ $wshShell.SendKeys([char]175) }};
                    "#,
                    value
                );
                let mut cmd = Command::new("powershell");
                cmd.args(&["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_cmd]);
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::process::CommandExt;
                    cmd.creation_flags(0x08000000);
                }
                match cmd.output() {
                    Ok(_) => Ok(format!("Volume set to {}%", value)),
                    Err(e) => Err(format!("Failed to set volume: {}", e)),
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                Err("Volume control not implemented for this OS".to_string())
            }
        }
        _ => Err(format!("Unknown system control: {}", action))
    }
}
