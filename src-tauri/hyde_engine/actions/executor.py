"""
Hyde Neural Engine — Action Executor

Unified action executor for both chat and voice pipelines.
Routes intents to appropriate handlers: local OS actions, AI responses, or web fallback.

Execution Strategy:
  - LOCAL intents (OPEN_APP, VOLUME_UP, SET_TIMER, etc.) → OS commands / IPC to Rust
  - AI intents (AI_WRITE, AI_EXPLAIN, etc.) → respond via JSON-RPC (Rust handles LLM)
  - WEB intents (WEB_SEARCH, YOUTUBE_SEARCH) → browser URL
  - SMALL_TALK → predefined responses
"""

import os
import sys
import webbrowser
import subprocess
import urllib.parse
from datetime import datetime
from pathlib import Path


# ─── Rich SmallTalk Responses ──────────────────────────────────────────────
SMALL_TALK_RESPONSES = {
    "greeting": "Hello, Sir. How can I assist you today?",
    "how_are_you": "I'm operating at peak performance, Sir. Thank you for asking.",
    "who_are_you": "I'm Hyde, your personal AI desktop assistant. I can open apps, search the web, set reminders, answer questions, and much more. Just ask.",
    "good_morning": "Good morning, Sir. Ready to conquer the day.",
    "good_afternoon": "Good afternoon, Sir. What can I help you with?",
    "good_evening": "Good evening, Sir. Need help winding down?",
    "goodnight": "Good night, Sir. Rest well.",
    "thanks": "You're welcome, Sir. Happy to help.",
    "bye": "Goodbye, Sir. I'll be here when you need me.",
    "help": "I can open apps, launch websites, search the web, set timers and reminders, control volume and brightness, answer questions, write documents, and much more. Just speak naturally.",
    "compliment": "Thank you, Sir. I do my best.",
    "love": "I appreciate the sentiment, Sir. I'm here to serve.",
    "joke": "Why do programmers prefer dark mode? Because light attracts bugs.",
}


class ActionExecutor:
    def __init__(self, tts_engine=None):
        self.tts = tts_engine

    def execute(self, payload: dict) -> dict:
        """
        Execute an intent and return a structured result.
        
        Returns:
            {
                "success": bool,
                "response": str,     # Human-readable response text
                "action_taken": str, # What was done
                "speak": bool,       # Whether TTS should speak the response
            }
        """
        intent = payload.get("intent")
        parameters = payload.get("parameters", {})

        print(f"[Hyde Action] Executing Intent: {intent} | Params: {parameters}")

        try:
            if intent == "OPEN_WEBSITE":
                return self._open_website(parameters)
            elif intent == "OPEN_APP":
                return self._open_app(parameters)
            elif intent == "CLOSE_APP":
                return self._close_app(parameters)
            elif intent == "WEB_SEARCH":
                return self._web_search(parameters)
            elif intent == "YOUTUBE_SEARCH":
                return self._youtube_search(parameters)
            elif intent == "GITHUB_SEARCH":
                return self._github_search(parameters)
            elif intent == "REDDIT_SEARCH":
                return self._reddit_search(parameters)
            elif intent == "MAIL_SEARCH":
                return self._mail_search(parameters)
            elif intent == "PLAY_MUSIC":
                return self._play_music(parameters)
            elif intent == "PAUSE_MUSIC":
                return self._media_control("pause")
            elif intent == "NEXT_TRACK":
                return self._media_control("next")
            elif intent == "PREVIOUS_TRACK":
                return self._media_control("previous")
            elif intent in ("VOLUME_UP", "VOLUME_DOWN", "SET_VOLUME", "MUTE",
                            "BRIGHTNESS_UP", "BRIGHTNESS_DOWN", "SCREENSHOT",
                            "LOCK_PC", "SLEEP_PC", "SHUTDOWN_PC", "RESTART_PC",
                            "EMPTY_TRASH"):
                return self._system_control(intent, parameters)
            elif intent == "TIME_QUERY":
                return self._time_query()
            elif intent == "DATE_QUERY":
                return self._date_query()
            elif intent == "SYSTEM_INFO":
                return self._system_info(parameters)
            elif intent in ("SET_TIMER", "SET_REMINDER"):
                return self._set_timer(intent, parameters)
            elif intent == "CANCEL_TIMER":
                return self._result(True, "Timer cancellation noted. This will be handled by the Rust backend.", "cancel_timer", speak=True, rust_action={"action": "cancel_timer"})
            elif intent == "LIST_TIMERS":
                return self._result(True, "Timer listing will be handled by the Rust backend.", "list_timers", speak=True, rust_action={"action": "list_timers"})
            elif intent == "OPEN_FILE":
                return self._open_path(parameters, is_folder=False)
            elif intent == "OPEN_FOLDER":
                return self._open_path(parameters, is_folder=True)
            elif intent == "CREATE_FOLDER":
                return self._create_folder(parameters)
            elif intent in ("COPY_CLIPBOARD", "SHOW_CLIPBOARD"):
                return self._result(True, "Clipboard operation noted.", intent.lower(), speak=False)
            elif intent == "SMALL_TALK":
                return self._small_talk(parameters)
            elif intent in ("AI_CHAT", "AI_WRITE", "AI_EXPLAIN", "AI_SUMMARIZE", "AI_RESEARCH", "GENERAL_AI_CHAT"):
                # AI intents are forwarded back to Rust for LLM API handling
                return self._result(True, None, "ai_forward", speak=False)
            else:
                return self._result(False, f"I understand your intent ({intent}), but I haven't been programmed to execute that action yet, Sir.", "unknown", speak=True)

        except Exception as e:
            return self._result(False, f"An error occurred: {str(e)}", "error", speak=True)

    # ─── Action Handlers ───────────────────────────────────────────────

    def _open_website(self, params: dict) -> dict:
        target = params.get("target", "")
        if not target:
            return self._result(False, "No website specified.", "error")

        url = target
        if not url.startswith("http"):
            if "." not in url:
                url += ".com"
            url = "https://" + url

        webbrowser.open(url)
        name = target.replace(".com", "").replace("www.", "").capitalize()
        msg = f"Opening {name}, Sir."
        if self.tts:
            self.tts.speak(msg)
        return self._result(True, msg, "open_website")

    def _open_app(self, params: dict) -> dict:
        target = params.get("target", "").strip()
        if not target:
            return self._result(False, "No app specified.", "error")

        app_name = target.lower()

        # 1. Try Windows Start Menu search (Get-StartApps)
        try:
            result = subprocess.run(
                'powershell -c "Get-StartApps"',
                capture_output=True, text=True, shell=True,
                creationflags=0x08000000  # CREATE_NO_WINDOW
            )

            found_app_id = None
            exact_match_id = None

            if result.returncode == 0:
                lines = result.stdout.splitlines()
                for line in lines[3:]:
                    line = line.strip()
                    if not line:
                        continue
                    parts = [p.strip() for p in line.split("  ") if p.strip()]
                    if len(parts) >= 2:
                        name = parts[0].lower()
                        aid = parts[-1]
                        if name == app_name:
                            exact_match_id = aid
                            break
                        elif app_name in name and not found_app_id:
                            found_app_id = aid

            final_id = exact_match_id or found_app_id

            if final_id:
                msg = f"Starting {app_name.title()}, Sir."
                if self.tts:
                    self.tts.speak(msg)
                os.system(f'explorer.exe shell:AppsFolder\\{final_id}')
                return self._result(True, msg, "open_app")
        except Exception:
            pass

        # 2. Fallback: Try opening directly
        exe_map = {
            "brave": "brave.exe", "chrome": "chrome.exe", "firefox": "firefox.exe",
            "edge": "msedge.exe", "vscode": "code", "code": "code",
        }
        exe = exe_map.get(app_name, app_name)
        try:
            import shutil
            if shutil.which(exe):
                subprocess.Popen([exe], creationflags=0x08000000)
                msg = f"Launching {app_name.title()}, Sir."
                if self.tts:
                    self.tts.speak(msg)
                return self._result(True, msg, "open_app")
        except Exception:
            pass

        msg = f"I couldn't find {app_name.title()} on this system, Sir."
        if self.tts:
            self.tts.speak(msg)
        return self._result(False, msg, "app_not_found")

    def _close_app(self, params: dict) -> dict:
        target = params.get("target", "").strip()
        if not target:
            return self._result(False, "No app specified to close.", "error")

        try:
            result = subprocess.run(
                f'taskkill /im {target}.exe /f',
                capture_output=True, text=True, shell=True,
                creationflags=0x08000000
            )
            if result.returncode == 0:
                msg = f"Closed {target.title()}, Sir."
                if self.tts:
                    self.tts.speak(msg)
                return self._result(True, msg, "close_app")
            else:
                # Try case-insensitive process name match
                result2 = subprocess.run(
                    f'powershell -c "Get-Process -Name \'*{target}*\' | Stop-Process -Force"',
                    capture_output=True, text=True, shell=True,
                    creationflags=0x08000000
                )
                if result2.returncode == 0:
                    msg = f"Closed {target.title()}, Sir."
                    if self.tts:
                        self.tts.speak(msg)
                    return self._result(True, msg, "close_app")

                msg = f"Couldn't find a running process for {target.title()}, Sir."
                if self.tts:
                    self.tts.speak(msg)
                return self._result(False, msg, "close_app_failed")
        except Exception as e:
            return self._result(False, f"Failed to close {target}: {e}", "error")

    def _web_search(self, params: dict) -> dict:
        query = params.get("query", "")
        if not query:
            return self._result(False, "No search query provided.", "error")

        encoded = urllib.parse.quote(query)
        url = f"https://www.google.com/search?q={encoded}"
        webbrowser.open(url)
        msg = f"Searching the web for '{query}'."
        if self.tts:
            self.tts.speak("Searching the web.")
        return self._result(True, msg, "web_search")

    def _youtube_search(self, params: dict) -> dict:
        query = params.get("query", "")
        if not query:
            return self._result(False, "No search query provided.", "error")

        encoded = urllib.parse.quote(query)
        url = f"https://www.youtube.com/results?search_query={encoded}"
        webbrowser.open(url)
        msg = f"Searching YouTube for '{query}'."
        if self.tts:
            self.tts.speak("Searching YouTube.")
        return self._result(True, msg, "youtube_search")

    def _github_search(self, params: dict) -> dict:
        query = params.get("query", "")
        if not query:
            return self._result(False, "No search query provided.", "error")

        encoded = urllib.parse.quote(query)
        url = f"https://github.com/search?q={encoded}"
        webbrowser.open(url)
        msg = f"Searching GitHub for '{query}'."
        if self.tts:
            self.tts.speak("Searching GitHub.")
        return self._result(True, msg, "github_search")

    def _reddit_search(self, params: dict) -> dict:
        query = params.get("query", "")
        if not query:
            return self._result(False, "No search query provided.", "error")

        encoded = urllib.parse.quote(query)
        url = f"https://www.reddit.com/search/?q={encoded}"
        webbrowser.open(url)
        msg = f"Searching Reddit for '{query}'."
        if self.tts:
            self.tts.speak("Searching Reddit.")
        return self._result(True, msg, "reddit_search")

    def _mail_search(self, params: dict) -> dict:
        query = params.get("query", "")
        provider = params.get("provider", "gmail")
        if not query:
            return self._result(False, "No mail search query provided.", "error")

        encoded = urllib.parse.quote(query)
        if provider == "gmail":
            url = f"https://mail.google.com/mail/u/0/#search/{encoded}"
        elif provider == "yahoo":
            url = f"https://mail.yahoo.com/d/search/keyword={encoded}"
        elif provider == "outlook":
            url = "https://outlook.live.com/mail/0/inbox"
        else:
            return self._result(False, f"Unknown mail provider: {provider}", "error")

        webbrowser.open(url)
        msg = f"Searching {provider} for '{query}'."
        if self.tts:
            self.tts.speak(f"Searching {provider}.")
        return self._result(True, msg, "mail_search")

    def _play_music(self, params: dict) -> dict:
        song = params.get("song", "")
        artist = params.get("artist", "")
        query = song
        if artist:
            query += f" by {artist}"

        encoded = urllib.parse.quote(query)
        url = f"https://www.youtube.com/results?search_query={encoded}"
        webbrowser.open(url)
        msg = f"Playing {song}."
        if self.tts:
            self.tts.speak(msg)
        return self._result(True, msg, "play_music")

    def _media_control(self, action: str) -> dict:
        """Send media key press via PowerShell."""
        key_map = {
            "pause": "0xB3",   # VK_MEDIA_PLAY_PAUSE
            "next": "0xB0",    # VK_MEDIA_NEXT_TRACK
            "previous": "0xB1", # VK_MEDIA_PREV_TRACK
        }
        vk = key_map.get(action)
        if not vk:
            return self._result(False, f"Unknown media action: {action}", "error")

        try:
            ps_cmd = f"""
            Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public class Keyboard {{ [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtraInfo); }}';
            [Keyboard]::keybd_event({vk}, 0, 0, 0);
            [Keyboard]::keybd_event({vk}, 0, 2, 0);
            """
            subprocess.run(
                ["powershell", "-NoProfile", "-Command", ps_cmd],
                creationflags=0x08000000,
                capture_output=True
            )
            action_name = {"pause": "Toggled playback", "next": "Skipped to next track", "previous": "Went to previous track"}
            msg = f"{action_name.get(action, action)}, Sir."
            if self.tts:
                self.tts.speak(msg)
            return self._result(True, msg, f"media_{action}")
        except Exception as e:
            return self._result(False, f"Media control failed: {e}", "error")

    def _system_control(self, intent: str, params: dict) -> dict:
        """System control actions — delegated to Rust for most operations."""
        # These will be forwarded to Rust via IPC for proper native execution
        action_map = {
            "VOLUME_UP": ("volume up", "Increasing volume, Sir."),
            "VOLUME_DOWN": ("volume down", "Decreasing volume, Sir."),
            "SET_VOLUME": ("set_volume", f"Setting volume to {params.get('value', '50')}%, Sir."),
            "MUTE": ("mute", "Toggling mute, Sir."),
            "BRIGHTNESS_UP": ("brightness up", "Increasing brightness, Sir."),
            "BRIGHTNESS_DOWN": ("brightness down", "Decreasing brightness, Sir."),
            "SCREENSHOT": ("screenshot", "Taking screenshot, Sir."),
            "LOCK_PC": ("lock screen", "Locking the screen, Sir."),
            "SLEEP_PC": ("sleep", "Putting the system to sleep, Sir."),
            "SHUTDOWN_PC": ("shutdown", "Shutting down in 5 seconds, Sir."),
            "RESTART_PC": ("restart", "Restarting in 5 seconds, Sir."),
            "EMPTY_TRASH": ("empty trash", "Emptying the recycle bin, Sir."),
        }
        action, msg = action_map.get(intent, (intent.lower(), "Executing system command."))
        if self.tts:
            self.tts.speak(msg)
        # Return the action for Rust to execute
        return self._result(True, msg, action, rust_action={"action": action, **params})

    def _time_query(self) -> dict:
        now = datetime.now()
        time_str = now.strftime("%I:%M %p")
        msg = f"The current time is {time_str}."
        if self.tts:
            self.tts.speak(msg)
        return self._result(True, msg, "time_query")

    def _date_query(self) -> dict:
        now = datetime.now()
        date_str = now.strftime("%A, %B %d, %Y")
        msg = f"Today is {date_str}."
        if self.tts:
            self.tts.speak(msg)
        return self._result(True, msg, "date_query")

    def _system_info(self, params: dict) -> dict:
        info_type = params.get("type", "system")
        # Forward to Rust for native system info queries
        return self._result(True, None, "system_info_forward", rust_action={"type": info_type})

    def _set_timer(self, intent: str, params: dict) -> dict:
        minutes = params.get("minutes", params.get("delay_minutes", "5"))
        message = params.get("message", "Timer finished!")

        try:
            minutes = int(minutes)
        except (ValueError, TypeError):
            return self._result(False, "Invalid timer duration.", "error")

        if intent == "SET_REMINDER":
            msg = f"Reminder set for {minutes} minutes: {message}. I'll remind you, Sir."
        else:
            msg = f"Timer set for {minutes} minutes, Sir."

        if self.tts:
            self.tts.speak(msg)

        # Return timer info for Rust to spawn the actual timer
        return self._result(True, msg, "set_timer", rust_action={
            "minutes": str(minutes),
            "message": message,
        })

    def _open_path(self, params: dict, is_folder: bool) -> dict:
        path = params.get("path", "")
        if not path:
            return self._result(False, "No path specified.", "error")

        try:
            os.startfile(path)
            kind = "folder" if is_folder else "file"
            msg = f"Opened {kind}: {path}"
            if self.tts:
                self.tts.speak(f"Opening the {kind}, Sir.")
            return self._result(True, msg, f"open_{kind}")
        except Exception as e:
            return self._result(False, f"Could not open path: {e}", "error")

    def _create_folder(self, params: dict) -> dict:
        folder_name = params.get("folder_name", "")
        parent = params.get("parent_path", "")

        if not folder_name:
            return self._result(False, "No folder name specified.", "error")

        # Resolve parent path
        if parent:
            # Check if it's a well-known folder name
            home = str(Path.home())
            known = {
                "downloads": os.path.join(home, "Downloads"),
                "documents": os.path.join(home, "Documents"),
                "desktop": os.path.join(home, "Desktop"),
                "pictures": os.path.join(home, "Pictures"),
                "videos": os.path.join(home, "Videos"),
                "music": os.path.join(home, "Music"),
            }
            base = known.get(parent.lower(), parent)
        else:
            base = os.path.join(str(Path.home()), "Desktop")

        full_path = os.path.join(base, folder_name)
        try:
            os.makedirs(full_path, exist_ok=True)
            msg = f"Created folder '{folder_name}' at {full_path}, Sir."
            if self.tts:
                self.tts.speak(f"Folder created, Sir.")
            return self._result(True, msg, "create_folder")
        except Exception as e:
            return self._result(False, f"Failed to create folder: {e}", "error")

    def _small_talk(self, params: dict) -> dict:
        message_key = params.get("message", "greeting")
        response = SMALL_TALK_RESPONSES.get(message_key, "Hello, Sir. How can I help?")
        if self.tts:
            self.tts.speak(response)
        return self._result(True, response, "small_talk")

    # ─── Helper ────────────────────────────────────────────────────────

    def _result(self, success: bool, response: str, action_taken: str,
                speak: bool = True, rust_action: dict = None) -> dict:
        result = {
            "success": success,
            "response": response,
            "action_taken": action_taken,
            "speak": speak,
        }
        if rust_action:
            result["rust_action"] = rust_action
        return result
