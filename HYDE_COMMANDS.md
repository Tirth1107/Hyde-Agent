# Hyde Agent — Complete Command Inventory & Documentation

> **Generated**: June 20, 2026  
> **Version**: 1.0.0 (Pre-Redesign Audit)  
> **Purpose**: Official command reference, gap analysis, and migration guide

---

## Table of Contents

1. [Command Summary](#command-summary)
2. [App Launching Commands](#1-app-launching-commands)
3. [Website Commands](#2-website-commands)
4. [System Control Commands](#3-system-control-commands)
5. [Search Commands](#4-search-commands)
6. [File & Folder Commands](#5-file--folder-commands)
7. [Timer & Reminder Commands](#6-timer--reminder-commands)
8. [Clipboard Commands](#7-clipboard-commands)
9. [System Information Commands](#8-system-information-commands)
10. [Small Talk / Conversational](#9-small-talk--conversational-commands)
11. [Voice-Only Commands](#10-voice-only-commands-python-engine)
12. [Hidden / Internal Commands](#11-hidden--internal-commands)
13. [Custom Commands](#12-custom-commands)
14. [Missing Commands](#13-missing-commands-gap-analysis)
15. [Duplicate Commands](#14-duplicate-commands)
16. [Broken Commands](#15-broken-commands)
17. [Unreachable Commands](#16-unreachable-commands)
18. [Commands That Should Become Intents](#17-commands-that-should-become-intents)

---

## Command Summary

| Category | Count | Pipeline | Status |
|:---------|:------|:---------|:-------|
| App Launching | 22 | Rust | ✅ Working |
| Websites | 51 | Rust | ✅ Working |
| System Controls | 11 | Rust | ⚠️ Partial (3 stubs) |
| Search (Web/YouTube/Mail) | 6 prefixes | Rust | ✅ Working |
| File & Folder | 2 prefixes | Rust | ✅ Working |
| Timer | 2 prefixes | Rust | ⚠️ Partial (no cancel/list) |
| Clipboard | 2 | Rust | ✅ Working |
| System Info | 5 | Rust | ✅ Working |
| Small Talk | 11 | Rust | ⚠️ Hardcoded strings |
| Voice Intents (Python) | 16 categories | Python | ⚠️ Partially implemented |
| **Total Registered** | **~128** | | |

---

## 1. App Launching Commands

**Pipeline**: Rust (`registry.rs` → `executor/app.rs`)  
**Trigger**: Exact keyword match, or `open <app_name>` prefix  
**Fallback**: Windows Start Menu search → Google web search

| Keyword | Aliases | Executable | Description |
|:--------|:--------|:-----------|:------------|
| `brave` | `brave browser` | `brave.exe` | Launch Brave Browser |
| `chrome` | `google chrome` | `chrome.exe` | Launch Google Chrome |
| `firefox` | `mozilla`, `ff` | `firefox.exe` | Launch Firefox |
| `edge` | `microsoft edge`, `ms edge` | `msedge.exe` | Launch Microsoft Edge |
| `notepad` | `text editor` | `notepad` (via `open`) | Launch Notepad |
| `vscode` | `visual studio code`, `code` | `code` | Launch VS Code |
| `terminal` | `cmd`, `command prompt`, `powershell` | `terminal` (via `open`) | Launch System Terminal |
| `explorer` | `file explorer`, `files`, `folder` | `explorer` (via `open`) | Launch File Explorer |
| `calculator` | `calc` | `calculator` (via `open`) | Launch Calculator |
| `spotify` | `music` | `spotify.exe` | Launch Spotify |
| `discord` | `dc` | `Update.exe --processStart Discord.exe` | Launch Discord |
| `steam` | `games` | `steam` (via `open`) | Launch Steam |
| `task manager` | `taskmgr`, `processes` | via Start Menu search | Launch Task Manager |
| `settings` | `control panel`, `options` | Opens Settings window* | OS Settings / Hyde Settings** |
| `paint` | `mspaint`, `draw` | via Start Menu search | Launch MS Paint |
| `word` | `ms word`, `microsoft word` | via Start Menu search | Launch Microsoft Word |
| `excel` | `ms excel`, `spreadsheet` | via Start Menu search | Launch Excel |
| `powerpoint` | `ppt`, `slides` | via Start Menu search | Launch PowerPoint |
| `telegram` | `tg` | via Start Menu search | Launch Telegram |
| `slack` | *(none)* | via Start Menu search | Launch Slack |
| `obs` | `obs studio`, `screen record` | via Start Menu search | Launch OBS Studio |
| `vlc` | `media player` | via Start Menu search | Launch VLC |
| `zoom` | `meeting` | via Start Menu search | Launch Zoom |

> **\*\* Bug**: `settings` keyword is intercepted by `execute_command()` in `lib.rs` to open the **Hyde Settings window**, but it's also registered in `registry.rs` as `ActionType::OpenApp` with aliases `control panel`, `options`. The `lib.rs` intercept wins, so `open settings` opens Hyde Settings, not Windows Settings. `control panel` and `options` correctly try to open the OS app.

**Natural Language Examples**:
- `open chrome` ✅
- `launch vscode` ✅
- `open notepad` ✅
- `Could you please open Chrome` ✅ (filler words stripped)
- `start spotify` ❌ (not a recognized prefix — falls through to web search)
- `run terminal` ❌ (not a recognized prefix — falls through to web search)

---

## 2. Website Commands

**Pipeline**: Rust (`registry.rs` → `executor/browser.rs`)  
**Trigger**: Exact keyword match, alias match, or `open <site>` prefix  

| Keyword | Aliases | URL | Description |
|:--------|:--------|:----|:------------|
| `youtube` | `yt` | https://www.youtube.com | Open YouTube |
| `github` | `gh` | https://www.github.com | Open GitHub |
| `google` | `search` | https://www.google.com | Open Google |
| `gmail` | `mail` | https://mail.google.com | Open Gmail |
| `yahoo mail` | `yahoo` | https://mail.yahoo.com | Open Yahoo Mail |
| `outlook` | `hotmail` | https://outlook.live.com | Open Outlook |
| `reddit` | *(none)* | https://www.reddit.com | Open Reddit |
| `twitter` | `x` | https://www.twitter.com | Open Twitter/X |
| `instagram` | `insta`, `ig` | https://www.instagram.com | Open Instagram |
| `whatsapp web` | `whatsapp`, `wa` | https://web.whatsapp.com | Open WhatsApp Web |
| `chatgpt` | `gpt`, `ai` | https://chat.openai.com | Open ChatGPT |
| `claude` | `anthropic` | https://claude.ai | Open Claude |
| `netflix` | `movies` | https://www.netflix.com | Open Netflix |
| `amazon` | `shop`, `shopping` | https://www.amazon.com | Open Amazon |
| `facebook` | `fb` | https://www.facebook.com | Open Facebook |
| `linkedin` | *(none)* | https://www.linkedin.com | Open LinkedIn |
| `twitch` | `stream` | https://www.twitch.tv | Open Twitch |
| `pinterest` | `pin` | https://www.pinterest.com | Open Pinterest |
| `stackoverflow` | `so` | https://stackoverflow.com | Open Stack Overflow |
| `wikipedia` | `wiki` | https://www.wikipedia.org | Open Wikipedia |
| `ebay` | *(none)* | https://www.ebay.com | Open eBay |
| `spotify web` | `web spotify` | https://open.spotify.com | Open Spotify Web |
| `discord web` | `web discord` | https://discord.com/app | Open Discord Web |
| `figma` | `design` | https://www.figma.com | Open Figma |
| `notion` | `notes` | https://www.notion.so | Open Notion |
| `canva` | *(none)* | https://www.canva.com | Open Canva |
| `medium` | `blog` | https://medium.com | Open Medium |
| `quora` | *(none)* | https://www.quora.com | Open Quora |
| `imgur` | *(none)* | https://imgur.com | Open Imgur |
| `bing` | *(none)* | https://www.bing.com | Open Bing |
| `yahoo` | *(none)* | https://www.yahoo.com | Open Yahoo |
| `duckduckgo` | `ddg` | https://duckduckgo.com | Open DuckDuckGo |
| `weather` | `forecast` | https://weather.com | Open Weather |
| `maps` | `google maps` | https://maps.google.com | Open Google Maps |
| `calendar` | `google calendar` | https://calendar.google.com | Open Google Calendar |
| `drive` | `google drive` | https://drive.google.com | Open Google Drive |
| `photos` | `google photos` | https://photos.google.com | Open Google Photos |
| `translate` | `google translate` | https://translate.google.com | Open Google Translate |
| `news` | `google news` | https://news.google.com | Open Google News |
| `meet` | `google meet` | https://meet.google.com | Open Google Meet |
| `zoom web` | `web zoom` | https://zoom.us | Open Zoom Web |
| `tiktok` | `tt` | https://www.tiktok.com | Open TikTok |
| `snapchat` | `snap` | https://www.snapchat.com | Open Snapchat |
| `vimeo` | *(none)* | https://vimeo.com | Open Vimeo |
| `soundcloud` | *(none)* | https://soundcloud.com | Open SoundCloud |
| `apple music` | `music apple` | https://music.apple.com | Open Apple Music |
| `hulu` | *(none)* | https://www.hulu.com | Open Hulu |
| `disney plus` | `disney+` | https://www.disneyplus.com | Open Disney+ |
| `hbo max` | `max` | https://www.max.com | Open HBO Max |
| `prime video` | `amazon prime` | https://www.primevideo.com | Open Prime Video |

> **Alias Conflicts**:
> - `yahoo` is aliased to **Yahoo Mail** (https://mail.yahoo.com), but also registered as keyword `yahoo` pointing to https://www.yahoo.com. The alias match in `yahoo mail` wins because it's checked first.
> - `search` is aliased to **Google** (https://www.google.com), which conflicts with the `search <query>` prefix rule. Exact match wins.
> - `music` is aliased to **Spotify** (app), but `apple music` is a website. No conflict since "music" alone matches the app.

---

## 3. System Control Commands

**Pipeline**: Rust (`registry.rs` → `executor/system.rs`)  
**Trigger**: Exact keyword match or alias

| Keyword | Aliases | Status | Description |
|:--------|:--------|:-------|:------------|
| `volume up` | `louder`, `increase volume` | ✅ Working | Increase volume (~10%) |
| `volume down` | `quieter`, `decrease volume` | ✅ Working | Decrease volume (~10%) |
| `mute` | `silence` | ✅ Working | Toggle mute |
| `lock screen` | `lock pc`, `lock` | ✅ Working | Lock workstation |
| `sleep` | `hibernate`, `suspend` | ✅ Working | Put PC to sleep |
| `shutdown` | `turn off`, `power off` | ✅ Working | Shutdown (5s delay) |
| `restart` | `reboot` | ✅ Working | Restart (5s delay) |
| `screenshot` | `print screen`, `capture` | ❌ **Stub** | Returns "not implemented" |
| `empty trash` | `empty recycle bin`, `clear trash` | ❌ **Not implemented** | Registered but no executor code |
| `brightness up` | `brighter`, `increase brightness` | ❌ **Not implemented** | Registered but no executor code |
| `brightness down` | `dimmer`, `decrease brightness` | ❌ **Not implemented** | Registered but no executor code |

**Prefix Commands**:
| Prefix | Example | Status |
|:-------|:--------|:-------|
| `set volume <value>` | `set volume 50` | ✅ Registered in parser but **no executor handler** — falls to `Unknown system control` error |

---

## 4. Search Commands

**Pipeline**: Rust (`parser.rs` prefix rules → `executor/browser.rs` or `executor/media.rs`)  
**Trigger**: Prefix matching on normalized input

| Prefix Pattern | Action | Example |
|:---------------|:-------|:--------|
| `search <query>` | Google Search | `search rust programming` |
| `google <query>` | Google Search | `google how to learn python` |
| `look up <query>` | Google Search | `look up weather today` |
| `play <query>` | YouTube Search | `play lofi music` |
| `play <query> on youtube` | YouTube Search | `play cooking tutorial on youtube` |
| `search youtube <query>` | YouTube Search | `search youtube react tutorial` |
| `youtube <query>` | YouTube Search | `youtube guitar lesson` |
| `go to <url>` | Open URL | `go to example.com` |

**Mail Search (Regex)**:
| Pattern | Example | Action |
|:--------|:--------|:-------|
| `find/search/show emails from <sender>` | `find emails from boss` | Gmail search: `from:boss` |
| `find/search/show emails about <topic>` | `search emails about project` | Gmail search: `project` |

---

## 5. File & Folder Commands

**Pipeline**: Rust (`parser.rs` prefix rules → `executor/browser.rs`)

| Prefix Pattern | Example | Action |
|:---------------|:--------|:-------|
| `open file <absolute_path>` | `open file D:\projects\notes.txt` | Opens file with default app |
| `open folder <absolute_path>` | `open folder C:\Users` | Opens folder in Explorer |

> **Limitation**: Paths must be absolute. No relative path resolution or file search.

---

## 6. Timer & Reminder Commands

**Pipeline**: Rust (`parser.rs` prefix rules → `executor/timer.rs`)

| Prefix Pattern | Example | Status |
|:---------------|:--------|:-------|
| `set timer <N> minutes` | `set timer 5 minutes` | ✅ Working |
| `timer <N> minutes` | `timer 10 minutes` | ✅ Working |

> **Missing**:
> - `remind me to <X> in <N> minutes` — Currently falls to web search
> - `cancel timer` — Not implemented
> - `list timers` — Not implemented
> - `set timer <N> seconds` — Not supported
> - `set timer <N> hours` — Not supported
> - Natural language time: "in half an hour", "in 2 hours" — Not supported

---

## 7. Clipboard Commands

**Pipeline**: Rust (`parser.rs` prefix + `registry.rs` exact → `executor/clipboard.rs`)

| Command | Aliases | Action |
|:--------|:--------|:-------|
| `copy <text>` | *(prefix rule)* | Copies text to clipboard |
| `show clipboard` | `clipboard`, `clip history` | Shows current clipboard content |

> **Missing**: `paste` action, actual clipboard history (only shows current)

---

## 8. System Information Commands

**Pipeline**: Rust (`registry.rs` exact → `executor/info.rs`)

| Keyword | Aliases | Output |
|:--------|:--------|:-------|
| `time` | `current time`, `what time is it` | `Current time is 10:30 AM` |
| `date` | `today's date`, `what is the date` | `Today is Saturday, June 20, 2026` |
| `ip address` | `my ip`, `ip` | `Local IP: 192.168.1.x` |
| `ram` | `memory usage`, `memory` | `RAM: 8192 MB used / 16384 MB total` |
| `battery` | `power level` | `Battery: 85% (Discharging)` |

---

## 9. Small Talk / Conversational Commands

**Pipeline**: Rust (`registry.rs` → `executor/mod.rs` inline match)

| Keyword | Aliases | Response |
|:--------|:--------|:---------|
| `hi` | *(none)* | "Hello! How can I help you today?" |
| `hello` | `hallo`, `hiya` | "Hello! How can I help you today?" |
| `hey` | `heya` | "Hello! How can I help you today?" |
| `how are you` | `how are you doing`, `how's it going` | "I'm functioning perfectly, thank you for asking!" |
| `who are you` | `what are you`, `tell me about yourself` | "I'm Hyde Agent, your personal desktop assistant." |
| `good morning` | `morning` | "Good morning! Ready to tackle the day?" |
| `good evening` | `evening` | "Good evening! Need help winding down?" |
| `goodnight` | `good night` | "Hello there!" (default fallback) |
| `thank you` | `thanks` | "Hello there!" (default fallback) |
| `bye` | `goodbye`, `see you` | "Hello there!" (default fallback) |
| `what can you do` | `help`, `commands` | "Hello there!" (default fallback) |

> **Bug**: `goodnight`, `thank you`, `bye`, and `what can you do` all return "Hello there!" which is incorrect. They should have proper responses.
>
> **Bug**: `help` and `commands` are aliased to SmallTalk but `settings`/`help` is intercepted in `lib.rs` line 19 to open Settings. So `help` opens Settings (correct) but `what can you do` and `commands` return "Hello there!" (broken).

---

## 10. Voice-Only Commands (Python Engine)

These intents exist **only** in the Python `intent_parser.py` and are **not accessible via chat/typed input**:

| Intent | Regex Patterns | Status |
|:-------|:---------------|:-------|
| `OPEN_WEBSITE` | `open/launch/go to <target>` | ✅ Executed via Python `executor.py` |
| `OPEN_APP` | `open/launch/start/run <target>` | ✅ Executed via Python `executor.py` |
| `CLOSE_APP` | `close/kill/quit/stop/exit <target>` | ❌ **Not implemented** in executor |
| `WEB_SEARCH` | `search/google/look up/find <query>` | ✅ Executed |
| `PLAY_MUSIC` | `play <song> [by <artist>]` | ✅ Searches YouTube |
| `PAUSE_MUSIC` | `pause/stop the music` | ❌ **Not implemented** in executor |
| `NEXT_TRACK` | `next/skip the song` | ❌ **Not implemented** in executor |
| `PREVIOUS_TRACK` | `previous/last/go back the song` | ❌ **Not implemented** in executor |
| `INCREASE_VOLUME` | `turn up/increase/raise volume` | ❌ **Not implemented** in executor |
| `DECREASE_VOLUME` | `turn down/decrease/lower volume` | ❌ **Not implemented** in executor |
| `WEATHER_QUERY` | `what's the weather / is it raining` | ❌ **Not implemented** in executor |
| `TIME_QUERY` | `what time is it` | ✅ Executed via TTS |
| `SYSTEM_INFO` | `system info / battery / cpu / ram` | ❌ **Not implemented** in executor |
| `SHUTDOWN_PC` | `shutdown/turn off the computer` | ❌ **Not implemented** in executor |
| `RESTART_PC` | `restart/reboot the computer` | ❌ **Not implemented** in executor |
| `LOCK_PC` | `lock the computer/screen` | ❌ **Not implemented** in executor |
| `OPEN_FOLDER` | `open/show the <X> folder` | ❌ **Not implemented** in executor |
| `FILE_SEARCH` | `find/search for the file <X>` | ❌ **Not implemented** in executor |
| `GENERAL_AI_CHAT` | *(default fallback)* | ❌ **Not implemented** (prints warning) |

> **Critical Issue**: 13 out of 18 Python intents have **no executor implementation**. They print `[WARN] Intent X not fully implemented yet.` and speak "I understand your intent, but I haven't been programmed to execute that action yet."

---

## 11. Hidden / Internal Commands

| Command | Location | Description |
|:--------|:---------|:------------|
| `settings` / `help` | `lib.rs:19` | Opens Settings window (intercepted before parser) |
| `open <url.com>` | `parser.rs:57-64` | Auto-detects URLs ending in `.com`, `.org`, `.net`, `.io` |
| `open <anything>` | `parser.rs:88-93` | Treats any `open X` as app launch if X isn't a known keyword |
| Filler word stripping | `parser.rs:258` | Strips: "can you please", "could you", "would you", "please", "i want to", "let's", "can you", "hey hyde" |
| Fuzzy matching | `parser.rs:102-111` | Levenshtein distance ≤ 2 on keywords > 3 chars |
| Global web search fallback | `parser.rs:114-118` | **Everything unrecognized becomes a Google search** |
| App web search fallback | `app.rs:47-52` | If app not found locally, Google searches the app name |

---

## 12. Custom Commands

**Location**: `~/.hyde-agent/custom_commands.json`  
**Format**: Same as `CommandEntry` struct  
**Management**: Settings UI → Custom Commands tab  
**Types supported**: `open_url`, `open_app`

---

## 13. Missing Commands (Gap Analysis)

### Critical Missing (Most Requested)

| Category | Missing Command | Priority |
|:---------|:----------------|:---------|
| **Reminders** | `remind me to <X> in <N> minutes` | 🔴 Critical |
| **AI Chat** | `explain <topic>` | 🔴 Critical |
| **AI Writing** | `write <document_type>` | 🔴 Critical |
| **AI Summary** | `summarize <text>` | 🔴 Critical |
| **Context** | Follow-up commands based on previous action | 🔴 Critical |

### Important Missing

| Category | Missing Command | Priority |
|:---------|:----------------|:---------|
| Timer | `cancel timer` | 🟡 High |
| Timer | `list timers` / `show timers` | 🟡 High |
| Timer | Timer with seconds/hours | 🟡 High |
| System | `screenshot` implementation | 🟡 High |
| System | `brightness up/down` implementation | 🟡 High |
| System | `empty trash` implementation | 🟡 High |
| System | `set volume <N>` executor handler | 🟡 High |
| Media | Play/Pause/Next/Previous track controls | 🟡 High |
| App | `close <app>` / `kill <app>` | 🟡 High |

### Nice to Have

| Category | Missing Command | Priority |
|:---------|:----------------|:---------|
| Search | `search github <query>` | 🟢 Medium |
| Search | `search reddit <query>` | 🟢 Medium |
| Search | `search stackoverflow <query>` | 🟢 Medium |
| Files | `create folder <name>` | 🟢 Medium |
| Files | `delete file <path>` | 🟢 Medium |
| Files | `find file <name>` | 🟢 Medium |
| Developer | `run command <cmd>` | 🟢 Medium |
| Developer | `git status` | 🟢 Medium |
| System | `wifi on/off` | 🟢 Medium |
| System | `bluetooth on/off` | 🟢 Medium |
| Clipboard | `paste` | 🟢 Medium |
| Clipboard | Real clipboard history | 🟢 Medium |

---

## 14. Duplicate Commands

| Keyword/Alias | Registered As | Conflict With |
|:--------------|:--------------|:--------------|
| `yahoo` | Website alias → Yahoo Mail | Website keyword → Yahoo.com |
| `search` | Website alias → Google.com | Prefix rule → Web Search |
| `music` | App alias → Spotify | Website → Apple Music |
| `meeting` | App alias → Zoom (app) | — (no conflict, but ambiguous) |
| `notes` | Website alias → Notion | — (no conflict with apps) |
| `settings` | App keyword (registry) | Internal command (lib.rs intercept) |
| `help` | SmallTalk alias | Internal command (lib.rs intercept) |
| `commands` | SmallTalk alias | — (no handler, returns "Hello there!") |
| `design` | Website alias → Figma | — (could mean Canva too) |

---

## 15. Broken Commands

| Command | Expected | Actual | Root Cause |
|:--------|:---------|:-------|:-----------|
| `goodnight` | "Good night!" | "Hello there!" | Default SmallTalk fallback |
| `thank you` | "You're welcome!" | "Hello there!" | Default SmallTalk fallback |
| `bye` | "Goodbye!" | "Hello there!" | Default SmallTalk fallback |
| `what can you do` | Help text | "Hello there!" | SmallTalk instead of help |
| `screenshot` | Take screenshot | "not implemented" error | Stub in system.rs |
| `brightness up` | Increase brightness | "Unknown system control" error | No executor handler |
| `brightness down` | Decrease brightness | "Unknown system control" error | No executor handler |
| `empty trash` | Empty recycle bin | "Unknown system control" error | No executor handler |
| `set volume 50` | Set volume to 50% | "Unknown system control" error | No `set_volume` handler |
| `remind me to X in Y minutes` | Create reminder | Google Search | No intent recognition |
| `explain quantum computing` | AI explanation | Google Search | No AI intent |
| `write me a letter` | Generate letter | Google Search | No AI intent |

---

## 16. Unreachable Commands

| Command | Why Unreachable |
|:--------|:----------------|
| `commands` (SmallTalk alias) | `help` triggers Settings intercept in `lib.rs`, but `commands` goes to SmallTalk with "Hello there!" |
| Python `CLOSE_APP` intent | Never executed — no Python executor handler |
| Python `PAUSE_MUSIC` intent | Never executed — no Python executor handler |
| Python `NEXT_TRACK` intent | Never executed — no Python executor handler |
| Python `PREVIOUS_TRACK` intent | Never executed — no Python executor handler |
| Python `INCREASE_VOLUME` intent | Never executed — no Python executor handler |
| Python `DECREASE_VOLUME` intent | Never executed — no Python executor handler |
| Python `WEATHER_QUERY` intent | Never executed — no Python executor handler |
| Python `SYSTEM_INFO` intent | Never executed — no Python executor handler |
| Python `SHUTDOWN_PC` intent | Never executed — no Python executor handler |
| Python `RESTART_PC` intent | Never executed — no Python executor handler |
| Python `LOCK_PC` intent | Never executed — no Python executor handler |
| Python `OPEN_FOLDER` intent | Never executed — no Python executor handler |
| Python `FILE_SEARCH` intent | Never executed — no Python executor handler |
| Python `GENERAL_AI_CHAT` intent | Never executed — no LLM integration |
| Any command via voice that Rust handles | Voice pipeline uses Python executor, not Rust |

---

## 17. Commands That Should Become Intents

These currently rely on exact keyword/prefix matching but should be reclassified as **natural language intents**:

| Current Behavior | Should Become Intent | Natural Language Examples |
|:-----------------|:--------------------|:--------------------------|
| `volume up` (exact match) | `VOLUME_UP` intent | "turn up the volume", "make it louder", "increase sound" |
| `mute` (exact match) | `MUTE` intent | "mute the sound", "silence everything", "turn off audio" |
| `set timer 5 minutes` (prefix) | `SET_TIMER` intent | "set a 5 minute timer", "countdown 5 minutes", "wake me in 5" |
| `copy <text>` (prefix) | `COPY_CLIPBOARD` intent | "copy this to clipboard", "put this in my clipboard" |
| `show clipboard` (exact match) | `SHOW_CLIPBOARD` intent | "what's in my clipboard", "show me what I copied" |
| `time` (exact match) | `TIME_QUERY` intent | "what time is it", "tell me the time", "current time" |
| `battery` (exact match) | `SYSTEM_INFO` intent | "how's my battery", "battery level", "am I charging" |
| `search <query>` (prefix) | `WEB_SEARCH` intent | "look this up", "find information about", "I need to research" |
| `open youtube` (prefix + registry) | `OPEN_WEBSITE` intent | "take me to youtube", "I want to watch something", "go to youtube" |
| `shutdown` (exact match) | `SHUTDOWN_PC` intent | "turn off the computer", "shut it down", "power off" |
| **ALL unknown input** (fallback) | **Multi-class classification** | Should determine if it's AI chat, search, app, etc. — NOT default to web search |

---

## Appendix: Filler Words Stripped by Parser

The following phrases are stripped from the beginning of input before matching:

```
"can you please "
"could you "
"would you "
"please "
"i want to "
"let's "
"can you "
"hey hyde "
```

**Missing filler words** that should also be stripped:
```
"kindly "
"just "
"go ahead and "
"i need you to "
"i'd like to "
"would you mind "
"for me "
"do me a favor and "
"i need to "
```

---

## Appendix: URL Detection Suffixes

The parser auto-detects URLs ending in:
- `.com`
- `.org`
- `.net`
- `.io`

**Missing**:
- `.dev`
- `.app`
- `.ai`
- `.co`
- `.me`
- `.xyz`
- `.edu`
- `.gov`
