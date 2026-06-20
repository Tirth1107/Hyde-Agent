"""
Hyde Neural Engine — Context Memory

Maintains short-term conversational context to enable contextual follow-up commands.

Context Types:
  - Platform context: "Open YouTube" → "Search AI tutorials" → search YouTube
  - Location context: "Open Downloads" → "Create folder Hyde" → create in Downloads
  - Topic context: "Explain quantum computing" → "Tell me more" → continue quantum topic
  - App context: "Open Spotify" → "Play lofi" → play on Spotify
"""

import time


# Well-known folder names mapped to paths
KNOWN_FOLDERS = {
    "downloads": "Downloads",
    "documents": "Documents",
    "desktop": "Desktop",
    "pictures": "Pictures",
    "videos": "Videos",
    "music": "Music",
}

# Platform names that support search
SEARCHABLE_PLATFORMS = {
    "youtube": "YOUTUBE_SEARCH",
    "github": "GITHUB_SEARCH",
    "reddit": "REDDIT_SEARCH",
    "google": "WEB_SEARCH",
    "gmail": "MAIL_SEARCH",
}


class ContextMemory:
    def __init__(self):
        self.history = []
        self.context = {
            "platform": None,      # Last opened website/platform
            "location": None,      # Last opened folder/directory
            "topic": None,         # Last conversational topic
            "last_app": None,      # Last opened app
        }
        self.context_ttl = {
            "platform": 30,        # 30 seconds
            "location": 60,        # 60 seconds
            "topic": 120,          # 2 minutes
            "last_app": 30,        # 30 seconds
        }

    def add_context(self, intent_payload: dict):
        """Record a command and update context state."""
        now = time.time()
        intent = intent_payload.get("intent")
        params = intent_payload.get("parameters", {})

        # Update history
        self.history.append({
            "timestamp": now,
            "payload": intent_payload
        })
        # Keep only last 10 commands
        if len(self.history) > 10:
            self.history.pop(0)

        # Update context based on intent
        if intent == "OPEN_WEBSITE":
            target = params.get("target", "").lower()
            self.context["platform"] = {
                "value": target,
                "expires": now + self.context_ttl["platform"]
            }

        elif intent == "OPEN_APP":
            target = params.get("target", "").lower()
            self.context["last_app"] = {
                "value": target,
                "expires": now + self.context_ttl["last_app"]
            }

        elif intent in ("OPEN_FOLDER",):
            path = params.get("path", "")
            if path:
                self.context["location"] = {
                    "value": path,
                    "expires": now + self.context_ttl["location"]
                }

        elif intent in ("AI_EXPLAIN", "AI_CHAT", "GENERAL_AI_CHAT", "AI_WRITE", "AI_SUMMARIZE"):
            query = params.get("query", "")
            if query:
                self.context["topic"] = {
                    "value": query,
                    "expires": now + self.context_ttl["topic"]
                }

    def inject_context(self, current_payload: dict) -> dict:
        """
        Enhance the current intent based on active context.
        This is where follow-up commands get their intelligence.
        """
        self._cleanup()
        intent = current_payload.get("intent")
        params = current_payload.get("parameters", {})

        # ── Rule 1: Search inherits platform ──────────────────────────
        # "Open YouTube" → "Search AI tutorials" → search YouTube
        if intent == "WEB_SEARCH":
            platform_ctx = self._get_context("platform")
            if platform_ctx:
                platform = platform_ctx.lower()
                if platform in SEARCHABLE_PLATFORMS:
                    current_payload["intent"] = SEARCHABLE_PLATFORMS[platform]
                    current_payload["parameters"]["platform"] = platform
                    # Boost confidence since context confirms intent
                    current_payload["confidence"] = min(
                        current_payload.get("confidence", 0.8) + 0.05, 1.0
                    )

        # ── Rule 2: Folder operations inherit location ────────────────
        # "Open Downloads" → "Create folder Hyde" → create in Downloads
        if intent == "CREATE_FOLDER":
            location_ctx = self._get_context("location")
            if location_ctx and "parent_path" not in params:
                current_payload["parameters"]["parent_path"] = location_ctx

        # ── Rule 3: Music inherits app context ────────────────────────
        # "Open Spotify" → "Play lofi" → play on Spotify
        if intent == "PLAY_MUSIC":
            app_ctx = self._get_context("last_app")
            if app_ctx and app_ctx in ("spotify",):
                current_payload["parameters"]["platform"] = app_ctx

        # ── Rule 4: "Tell me more" / follow-up inherits topic ─────────
        if intent == "GENERAL_AI_CHAT":
            topic_ctx = self._get_context("topic")
            query = params.get("query", "").lower()
            follow_up_signals = [
                "tell me more", "go on", "continue", "more details",
                "elaborate", "expand on that", "what else",
                "and then", "more about", "keep going",
            ]
            if topic_ctx and any(signal in query for signal in follow_up_signals):
                current_payload["parameters"]["query"] = f"Continue explaining: {topic_ctx}. User asked: {query}"
                current_payload["intent"] = "AI_EXPLAIN"
                current_payload["confidence"] = 0.85

        return current_payload

    def get_last_intent(self):
        self._cleanup()
        if self.history:
            return self.history[-1]["payload"]
        return None

    def get_active_context(self) -> dict:
        """Return currently active context for UI display."""
        self._cleanup()
        active = {}
        for key, ctx in self.context.items():
            if ctx and time.time() < ctx["expires"]:
                active[key] = ctx["value"]
        return active

    def _get_context(self, key: str):
        ctx = self.context.get(key)
        if ctx and time.time() < ctx["expires"]:
            return ctx["value"]
        return None

    def _cleanup(self):
        """Remove expired context and old history."""
        now = time.time()
        for key in list(self.context.keys()):
            ctx = self.context[key]
            if ctx and now >= ctx["expires"]:
                self.context[key] = None

        # Also clean history (keep last 120 seconds)
        self.history = [
            item for item in self.history
            if now - item["timestamp"] <= 120
        ]
