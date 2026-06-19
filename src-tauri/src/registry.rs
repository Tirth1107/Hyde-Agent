use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::types::{ActionType, CommandEntry};

lazy_static! {
    pub static ref BUILTIN_COMMANDS: Vec<CommandEntry> = load_builtins();
}

fn create_cmd(keyword: &str, aliases: Vec<&str>, action_type: ActionType, desc: &str, params: HashMap<String, String>) -> CommandEntry {
    CommandEntry {
        keyword: keyword.to_string(),
        aliases: aliases.into_iter().map(|s| s.to_string()).collect(),
        action_type,
        parameters: params,
        description: desc.to_string(),
    }
}

fn load_builtins() -> Vec<CommandEntry> {
    let mut cmds = Vec::new();

    // 1. Apps
    let apps = vec![
        ("brave", vec!["brave browser"], "Brave Browser"),
        ("chrome", vec!["google chrome"], "Google Chrome"),
        ("firefox", vec!["mozilla", "ff"], "Firefox"),
        ("edge", vec!["microsoft edge", "ms edge"], "Microsoft Edge"),
        ("notepad", vec!["text editor"], "Notepad"),
        ("vscode", vec!["visual studio code", "code"], "Visual Studio Code"),
        ("terminal", vec!["cmd", "command prompt", "powershell"], "System Terminal"),
        ("explorer", vec!["file explorer", "files", "folder"], "File Explorer"),
        ("calculator", vec!["calc"], "Calculator"),
        ("spotify", vec!["music"], "Spotify"),
        ("discord", vec!["dc"], "Discord"),
        ("steam", vec!["games"], "Steam"),
        ("task manager", vec!["taskmgr", "processes"], "Task Manager"),
        ("settings", vec!["control panel", "options"], "OS Settings"),
        ("paint", vec!["mspaint", "draw"], "MS Paint"),
        ("word", vec!["ms word", "microsoft word"], "Word"),
        ("excel", vec!["ms excel", "spreadsheet"], "Excel"),
        ("powerpoint", vec!["ppt", "slides"], "PowerPoint"),
        ("telegram", vec!["tg"], "Telegram"),
        ("slack", vec![], "Slack"),
        ("obs", vec!["obs studio", "screen record"], "OBS Studio"),
        ("vlc", vec!["media player"], "VLC Media Player"),
        ("zoom", vec!["meeting"], "Zoom"),
    ];
    for (kw, aliases, desc) in apps {
        let mut p = HashMap::new();
        p.insert("app_name".to_string(), kw.to_string());
        cmds.push(create_cmd(kw, aliases, ActionType::OpenApp, &format!("Launch {}", desc), p));
    }

    // 2. Websites
    let sites = vec![
        ("youtube", vec!["yt"], "https://www.youtube.com"),
        ("github", vec!["gh"], "https://www.github.com"),
        ("google", vec!["search"], "https://www.google.com"),
        ("gmail", vec!["mail"], "https://mail.google.com"),
        ("yahoo mail", vec!["yahoo"], "https://mail.yahoo.com"),
        ("outlook", vec!["hotmail"], "https://outlook.live.com"),
        ("reddit", vec![], "https://www.reddit.com"),
        ("twitter", vec!["x"], "https://www.twitter.com"),
        ("instagram", vec!["insta", "ig"], "https://www.instagram.com"),
        ("whatsapp web", vec!["whatsapp", "wa"], "https://web.whatsapp.com"),
        ("chatgpt", vec!["gpt", "ai"], "https://chat.openai.com"),
        ("claude", vec!["anthropic"], "https://claude.ai"),
        ("netflix", vec!["movies"], "https://www.netflix.com"),
        ("amazon", vec!["shop", "shopping"], "https://www.amazon.com"),
        ("facebook", vec!["fb"], "https://www.facebook.com"),
        ("linkedin", vec![], "https://www.linkedin.com"),
        ("twitch", vec!["stream"], "https://www.twitch.tv"),
        ("pinterest", vec!["pin"], "https://www.pinterest.com"),
        ("stackoverflow", vec!["so"], "https://stackoverflow.com"),
        ("wikipedia", vec!["wiki"], "https://www.wikipedia.org"),
        ("ebay", vec![], "https://www.ebay.com"),
        ("spotify web", vec!["web spotify"], "https://open.spotify.com"),
        ("discord web", vec!["web discord"], "https://discord.com/app"),
        ("figma", vec!["design"], "https://www.figma.com"),
        ("notion", vec!["notes"], "https://www.notion.so"),
        ("canva", vec![], "https://www.canva.com"),
        ("medium", vec!["blog"], "https://medium.com"),
        ("quora", vec![], "https://www.quora.com"),
        ("imgur", vec![], "https://imgur.com"),
        ("bing", vec![], "https://www.bing.com"),
        ("yahoo", vec![], "https://www.yahoo.com"),
        ("duckduckgo", vec!["ddg"], "https://duckduckgo.com"),
        ("weather", vec!["forecast"], "https://weather.com"),
        ("maps", vec!["google maps"], "https://maps.google.com"),
        ("calendar", vec!["google calendar"], "https://calendar.google.com"),
        ("drive", vec!["google drive"], "https://drive.google.com"),
        ("photos", vec!["google photos"], "https://photos.google.com"),
        ("translate", vec!["google translate"], "https://translate.google.com"),
        ("news", vec!["google news"], "https://news.google.com"),
        ("meet", vec!["google meet"], "https://meet.google.com"),
        ("zoom web", vec!["web zoom"], "https://zoom.us"),
        ("tiktok", vec!["tt"], "https://www.tiktok.com"),
        ("snapchat", vec!["snap"], "https://www.snapchat.com"),
        ("vimeo", vec![], "https://vimeo.com"),
        ("soundcloud", vec![], "https://soundcloud.com"),
        ("apple music", vec!["music apple"], "https://music.apple.com"),
        ("hulu", vec![], "https://www.hulu.com"),
        ("disney plus", vec!["disney+"], "https://www.disneyplus.com"),
        ("hbo max", vec!["max"], "https://www.max.com"),
        ("prime video", vec!["amazon prime"], "https://www.primevideo.com"),
    ];
    for (kw, aliases, url) in sites {
        let mut p = HashMap::new();
        p.insert("url".to_string(), url.to_string());
        cmds.push(create_cmd(kw, aliases, ActionType::OpenUrl, &format!("Open {}", kw), p));
    }

    // 3. System Controls
    let sys_controls = vec![
        ("volume up", vec!["louder", "increase volume"], "Increase volume"),
        ("volume down", vec!["quieter", "decrease volume"], "Decrease volume"),
        ("mute", vec!["silence"], "Toggle mute"),
        ("screenshot", vec!["print screen", "capture"], "Take screenshot"),
        ("lock screen", vec!["lock pc", "lock"], "Lock workstation"),
        ("sleep", vec!["hibernate", "suspend"], "Sleep"),
        ("empty trash", vec!["empty recycle bin", "clear trash"], "Empty Recycle Bin"),
        ("shutdown", vec!["turn off", "power off"], "Shutdown PC"),
        ("restart", vec!["reboot"], "Restart PC"),
        ("brightness up", vec!["brighter", "increase brightness"], "Increase brightness"),
        ("brightness down", vec!["dimmer", "decrease brightness"], "Decrease brightness"),
    ];
    for (kw, aliases, desc) in sys_controls {
        let mut p = HashMap::new();
        p.insert("action".to_string(), kw.to_string());
        cmds.push(create_cmd(kw, aliases, ActionType::SystemControl, desc, p));
    }

    // 4. Clipboard
    cmds.push(create_cmd("show clipboard", vec!["clipboard", "clip history"], ActionType::Clipboard, "Show clipboard history", HashMap::from([("action".to_string(), "history".to_string())])));

    // 5. Info
    let infos = vec![
        ("battery", vec!["power level"]), 
        ("ram", vec!["memory usage", "memory"]), 
        ("ip address", vec!["my ip", "ip"]), 
        ("time", vec!["current time", "what time is it"]), 
        ("date", vec!["today's date", "what is the date"])
    ];
    for (kw, aliases) in infos {
        let mut p = HashMap::new();
        p.insert("type".to_string(), kw.to_string());
        cmds.push(create_cmd(kw, aliases, ActionType::Info, &format!("Show {}", kw), p));
    }

    // 6. SmallTalk
    let greetings = vec![
        ("hi", vec![]), 
        ("hello", vec!["hallo", "hiya"]), 
        ("hey", vec!["heya"]), 
        ("how are you", vec!["how are you doing", "how's it going"]), 
        ("who are you", vec!["what are you", "tell me about yourself"]), 
        ("good morning", vec!["morning"]), 
        ("good evening", vec!["evening"]),
        ("goodnight", vec!["good night"]),
        ("thank you", vec!["thanks"]),
        ("bye", vec!["goodbye", "see you"]),
        ("what can you do", vec!["help", "commands"]),
    ];
    for (kw, aliases) in greetings {
        let mut p = HashMap::new();
        p.insert("message".to_string(), kw.to_string());
        cmds.push(create_cmd(kw, aliases, ActionType::SmallTalk, "Conversational reply", p));
    }

    cmds
}
