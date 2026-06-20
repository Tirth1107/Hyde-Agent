"""
Hyde Neural Engine — Intent Classifier

Classifies natural language input into structured intents with confidence scores.
This is the single source of truth for ALL input classification (chat + voice).

Classification Pipeline:
  1. Normalize (strip filler words, punctuation)
  2. Exact match (registered keywords/aliases) → confidence 1.0
  3. Pattern rules (30+ regex categories) → confidence 0.70–0.98
  4. Contextual reclassification (e.g., "open youtube" → OPEN_WEBSITE not OPEN_APP)
  5. Fallback to GENERAL_AI_CHAT (NOT web search) → confidence 0.50
"""

import re
import json

# ─── Known Websites (for OPEN_APP → OPEN_WEBSITE disambiguation) ───────────
KNOWN_WEBSITES = {
    "youtube", "google", "facebook", "reddit", "twitter", "x", "instagram",
    "whatsapp", "chatgpt", "claude", "netflix", "amazon", "linkedin", "twitch",
    "pinterest", "stackoverflow", "wikipedia", "ebay", "figma", "notion",
    "canva", "medium", "quora", "imgur", "bing", "yahoo", "duckduckgo",
    "weather", "maps", "calendar", "drive", "photos", "translate", "news",
    "meet", "tiktok", "snapchat", "vimeo", "soundcloud", "hulu", "github",
    "gmail", "outlook", "spotify web", "discord web", "zoom web",
}

# ─── Known Apps (for disambiguation) ───────────────────────────────────────
KNOWN_APPS = {
    "brave", "chrome", "firefox", "edge", "notepad", "vscode", "terminal",
    "explorer", "calculator", "spotify", "discord", "steam", "paint",
    "word", "excel", "powerpoint", "telegram", "slack", "obs", "vlc", "zoom",
    "task manager", "settings", "code",
}


class IntentParser:
    def __init__(self):
        # ─── Intent Pattern Rules ──────────────────────────────────────
        # Each intent maps to a list of (compiled_regex, entity_extractor_fn)
        # Entity extractor receives match groups and returns a dict
        self._build_rules()

        # Noise / filler words to strip from input
        self.noise_patterns = [
            r"^(?:can you please|could you please|would you please|could you kindly)\s+",
            r"^(?:can you|could you|would you|would you mind)\s+",
            r"^(?:please|kindly|just|go ahead and)\s+",
            r"^(?:i want to|i'd like to|i need you to|i need to)\s+",
            r"^(?:do me a favor and|for me|let's)\s+",
            r"^(?:hey hyde|hi hyde|okay hyde|hyde)\s+",
        ]

    def _build_rules(self):
        """Build all intent pattern rules."""
        self.rules = []

        # ── OPEN WEBSITE ───────────────────────────────────────────────
        self._add_rule("OPEN_WEBSITE", [
            (r"(?:open|launch|go to|take me to|navigate to|visit)\s+(?:the\s+)?(?:website\s+)?([a-zA-Z0-9\-\.]+(?:\.(?:com|org|net|io|dev|app|ai|co|me|xyz|edu|gov)))", lambda m: {"target": m.group(1).strip()}),
            (r"(?:open|launch|go to|take me to|navigate to|visit)\s+(?:the\s+)?(?:website\s+)?(https?://\S+)", lambda m: {"target": m.group(1).strip()}),
        ], base_confidence=0.95)

        # ── OPEN APP ──────────────────────────────────────────────────
        self._add_rule("OPEN_APP", [
            (r"(?:open|launch|start|run)\s+(?:the\s+)?(?:app(?:lication)?\s+)?(.+)", lambda m: {"target": m.group(1).strip()}),
        ], base_confidence=0.85)

        # ── CLOSE APP ─────────────────────────────────────────────────
        self._add_rule("CLOSE_APP", [
            (r"(?:close|kill|quit|stop|exit|end|terminate)\s+(?:the\s+)?(?:app(?:lication)?\s+)?(.+)", lambda m: {"target": m.group(1).strip()}),
        ], base_confidence=0.90)

        # ── YOUTUBE SEARCH ────────────────────────────────────────────
        self._add_rule("YOUTUBE_SEARCH", [
            (r"(?:search|find|look for|look up)\s+(.+?)\s+on\s+youtube", lambda m: {"query": m.group(1).strip()}),
            (r"(?:search|find|look for)\s+youtube\s+(?:for\s+)?(.+)", lambda m: {"query": m.group(1).strip()}),
            (r"youtube\s+search\s+(?:for\s+)?(.+)", lambda m: {"query": m.group(1).strip()}),
            (r"play\s+(.+?)\s+on\s+youtube", lambda m: {"query": m.group(1).strip()}),
        ], base_confidence=0.92)

        # ── GITHUB SEARCH ─────────────────────────────────────────────
        self._add_rule("GITHUB_SEARCH", [
            (r"(?:search|find|look for|look up)\s+(.+?)\s+on\s+github", lambda m: {"query": m.group(1).strip()}),
            (r"(?:search|find)\s+github\s+(?:for\s+)?(.+)", lambda m: {"query": m.group(1).strip()}),
        ], base_confidence=0.92)

        # ── REDDIT SEARCH ─────────────────────────────────────────────
        self._add_rule("REDDIT_SEARCH", [
            (r"(?:search|find|look for|look up)\s+(.+?)\s+on\s+reddit", lambda m: {"query": m.group(1).strip()}),
            (r"(?:search|find)\s+reddit\s+(?:for\s+)?(.+)", lambda m: {"query": m.group(1).strip()}),
        ], base_confidence=0.92)

        # ── WEB SEARCH ────────────────────────────────────────────────
        self._add_rule("WEB_SEARCH", [
            (r"(?:search|google|look up|find|search for|search the web for)\s+(.+)", lambda m: {"query": m.group(1).strip()}),
            (r"what is\s+(.+)", lambda m: {"query": m.group(1).strip()}),
        ], base_confidence=0.82)

        # ── MAIL SEARCH ───────────────────────────────────────────────
        self._add_rule("MAIL_SEARCH", [
            (r"(?:find|search|show|check)(?:\s+me)?(?:\s+the)?\s+(?:mail|mails|email|emails)\s+(?:from|by)\s+(.+)", lambda m: {"query": f"from:{m.group(1).strip()}", "provider": "gmail"}),
            (r"(?:find|search|show|check)(?:\s+me)?(?:\s+the)?\s+(?:mail|mails|email|emails)\s+(?:about|for|regarding|of|with)\s+(.+)", lambda m: {"query": m.group(1).strip(), "provider": "gmail"}),
        ], base_confidence=0.90)

        # ── SET REMINDER ──────────────────────────────────────────────
        self._add_rule("SET_REMINDER", [
            (r"remind\s+me\s+to\s+(.+?)\s+in\s+(\d+)\s+(minute|minutes|min|mins|hour|hours|hr|hrs|second|seconds|sec|secs)", self._extract_reminder),
            (r"remind\s+me\s+(?:about\s+)?(.+?)\s+in\s+(\d+)\s+(minute|minutes|min|mins|hour|hours|hr|hrs|second|seconds|sec|secs)", self._extract_reminder),
            (r"remind\s+me\s+to\s+(.+?)\s+(?:after|in)\s+(?:half\s+an?\s+hour|30\s+min)", lambda m: {"message": m.group(1).strip(), "delay_minutes": 30}),
        ], base_confidence=0.96)
        
        self._add_rule("SET_REMINDER_DEFAULT", [
            (r"remind\s+me\s+to\s+(.+)", lambda m: {"message": m.group(1).strip(), "delay_minutes": 5}),
        ], base_confidence=0.90)

        # ── SET TIMER ─────────────────────────────────────────────────
        self._add_rule("SET_TIMER", [
            (r"(?:set\s+(?:a\s+)?timer|timer|countdown)\s+(?:for\s+)?(\d+)\s+(minute|minutes|min|mins|hour|hours|hr|hrs|second|seconds|sec|secs)", self._extract_timer),
            (r"(?:set\s+(?:a\s+)?timer|timer|countdown)\s+(?:for\s+)?(\d+)", lambda m: {"minutes": m.group(1).strip()}),
            (r"wake\s+me\s+(?:up\s+)?in\s+(\d+)\s+(minute|minutes|min|mins|hour|hours)", self._extract_timer),
        ], base_confidence=0.93)

        # ── CANCEL TIMER ──────────────────────────────────────────────
        self._add_rule("CANCEL_TIMER", [
            (r"(?:cancel|stop|remove|delete|clear)\s+(?:the\s+|my\s+|a\s+)?(?:timer|reminder|alarm)", lambda m: {}),
            (r"(?:cancel|stop|remove|delete|clear)\s+(?:all\s+)?(?:timers|reminders|alarms)", lambda m: {"all": True}),
        ], base_confidence=0.92)

        # ── LIST TIMERS ───────────────────────────────────────────────
        self._add_rule("LIST_TIMERS", [
            (r"(?:list|show|display|what are)\s+(?:my\s+|the\s+|active\s+)?(?:timers|reminders|alarms)", lambda m: {}),
            (r"(?:any|do i have)\s+(?:active\s+)?(?:timers|reminders)", lambda m: {}),
            (r"show\s+me\s+(?:my\s+|the\s+|active\s+)?(?:timers|reminders|alarms)", lambda m: {}),
        ], base_confidence=0.90)

        # ── FILE OPERATIONS ───────────────────────────────────────────
        self._add_rule("OPEN_FILE", [
            (r"open\s+(?:the\s+)?file\s+(.+)", lambda m: {"path": m.group(1).strip()}),
        ], base_confidence=0.90)

        self._add_rule("OPEN_FOLDER", [
            (r"open\s+(?:the\s+)?folder\s+(.+)", lambda m: {"path": m.group(1).strip()}),
            (r"open\s+(?:the\s+)?(.+?)\s+folder", lambda m: {"path": m.group(1).strip()}),
            (r"(?:go to|navigate to|show)\s+(?:the\s+)?(.+?)\s+(?:folder|directory)", lambda m: {"path": m.group(1).strip()}),
        ], base_confidence=0.88)

        self._add_rule("CREATE_FOLDER", [
            (r"(?:create|make|new)\s+(?:a\s+)?folder\s+(?:called|named)?\s*(.+?)(?:\s+in\s+(.+))?$", self._extract_create_folder),
            (r"(?:create|make|new)\s+(?:a\s+)?(?:directory|dir)\s+(?:called|named)?\s*(.+?)(?:\s+in\s+(.+))?$", self._extract_create_folder),
        ], base_confidence=0.90)

        # ── CLIPBOARD ─────────────────────────────────────────────────
        self._add_rule("COPY_CLIPBOARD", [
            (r"copy\s+(.+?)(?:\s+to\s+(?:the\s+)?clipboard)?$", lambda m: {"text": m.group(1).strip()}),
            (r"(?:put|save)\s+(.+?)\s+(?:in|to|on)\s+(?:the\s+)?clipboard", lambda m: {"text": m.group(1).strip()}),
        ], base_confidence=0.88)

        self._add_rule("SHOW_CLIPBOARD", [
            (r"(?:show|display|what'?s?\s+(?:in|on))\s+(?:the\s+|my\s+)?clipboard", lambda m: {}),
            (r"(?:clipboard|clip\s+history|paste\s+history)", lambda m: {}),
            (r"what\s+did\s+i\s+copy", lambda m: {}),
        ], base_confidence=0.90)

        # ── SYSTEM CONTROLS ───────────────────────────────────────────
        self._add_rule("VOLUME_UP", [
            (r"(?:volume\s+up|louder|increase\s+(?:the\s+)?volume|turn\s+up\s+(?:the\s+)?(?:volume|sound))", lambda m: {}),
            (r"(?:raise|bump\s+up)\s+(?:the\s+)?(?:volume|sound)", lambda m: {}),
        ], base_confidence=0.95)

        self._add_rule("VOLUME_DOWN", [
            (r"(?:volume\s+down|quieter|decrease\s+(?:the\s+)?volume|turn\s+down\s+(?:the\s+)?(?:volume|sound))", lambda m: {}),
            (r"(?:lower|reduce)\s+(?:the\s+)?(?:volume|sound)", lambda m: {}),
        ], base_confidence=0.95)

        self._add_rule("SET_VOLUME", [
            (r"(?:set\s+)?volume\s+(?:to\s+)?(\d+)(?:\s*%)?", lambda m: {"value": m.group(1).strip()}),
        ], base_confidence=0.93)

        self._add_rule("MUTE", [
            (r"(?:mute|silence|unmute|toggle\s+mute)\s*(?:the\s+)?(?:sound|audio|volume)?", lambda m: {}),
            (r"turn\s+(?:off|on)\s+(?:the\s+)?(?:sound|audio)", lambda m: {}),
        ], base_confidence=0.95)

        self._add_rule("BRIGHTNESS_UP", [
            (r"(?:brightness\s+up|brighter|increase\s+(?:the\s+)?brightness)", lambda m: {}),
        ], base_confidence=0.93)

        self._add_rule("BRIGHTNESS_DOWN", [
            (r"(?:brightness\s+down|dimmer|decrease\s+(?:the\s+)?brightness)", lambda m: {}),
        ], base_confidence=0.93)

        self._add_rule("SCREENSHOT", [
            (r"(?:take\s+(?:a\s+)?)?(?:screenshot|screen\s*shot|screen\s+capture|print\s+screen|capture\s+screen)", lambda m: {}),
        ], base_confidence=0.95)

        self._add_rule("LOCK_PC", [
            (r"(?:lock)\s+(?:the\s+|my\s+)?(?:pc|computer|system|screen|workstation|device)", lambda m: {}),
            (r"lock\s+(?:the\s+|my\s+)?screen", lambda m: {}),
        ], base_confidence=0.95)

        self._add_rule("SLEEP_PC", [
            (r"(?:sleep|hibernate|suspend)\s*(?:the\s+)?(?:pc|computer|system)?", lambda m: {}),
            (r"put\s+(?:the\s+)?(?:pc|computer|system)\s+to\s+sleep", lambda m: {}),
        ], base_confidence=0.93)

        self._add_rule("SHUTDOWN_PC", [
            (r"(?:shutdown|shut\s+down|turn\s+off|power\s+off)\s*(?:the\s+)?(?:pc|computer|system)?", lambda m: {}),
        ], base_confidence=0.93)

        self._add_rule("RESTART_PC", [
            (r"(?:restart|reboot)\s*(?:the\s+)?(?:pc|computer|system)?", lambda m: {}),
        ], base_confidence=0.93)

        self._add_rule("EMPTY_TRASH", [
            (r"(?:empty|clear|clean)\s+(?:the\s+|my\s+)?(?:trash|recycle\s+bin|recycling\s+bin|bin)", lambda m: {}),
        ], base_confidence=0.93)

        # ── WEATHER ───────────────────────────────────────────────────
        self._add_rule("WEATHER_QUERY", [
            (r"(?:what'?s?\s+(?:is\s+)?the\s+weather|how'?s?\s+the\s+weather|weather\s+like|is\s+it\s+raining|temperature)", lambda m: {}),
        ], base_confidence=0.95)

        # ── SYSTEM INFO ───────────────────────────────────────────────
        self._add_rule("TIME_QUERY", [
            (r"(?:what(?:'?s|\s+is)\s+)?(?:the\s+)?(?:current\s+)?time", lambda m: {}),
            (r"what\s+time\s+is\s+it", lambda m: {}),
            (r"tell\s+me\s+the\s+time", lambda m: {}),
        ], base_confidence=0.95)

        self._add_rule("DATE_QUERY", [
            (r"(?:what(?:'?s|\s+is)\s+)?(?:the\s+|today'?s?\s+)?date", lambda m: {}),
            (r"what\s+(?:day|date)\s+is\s+(?:it\s+)?today", lambda m: {}),
        ], base_confidence=0.95)

        self._add_rule("SYSTEM_INFO", [
            (r"(?:system\s+info|system\s+information|specs|specifications)", lambda m: {"type": "system"}),
            (r"(?:battery|power\s+level|charge\s+level|am\s+i\s+charging)", lambda m: {"type": "battery"}),
            (r"(?:how'?s?\s+(?:my\s+)?battery)", lambda m: {"type": "battery"}),
            (r"(?:ram|memory\s+usage|memory|how\s+much\s+(?:ram|memory))", lambda m: {"type": "ram"}),
            (r"(?:cpu|processor|cpu\s+usage)", lambda m: {"type": "cpu"}),
            (r"(?:ip\s+address|my\s+ip|local\s+ip|what'?s?\s+my\s+ip)", lambda m: {"type": "ip_address"}),
        ], base_confidence=0.93)

        # ── MEDIA CONTROLS ────────────────────────────────────────────
        self._add_rule("PLAY_MUSIC", [
            (r"play\s+(.+?)(?:\s+by\s+(.+))?$", self._extract_music),
        ], base_confidence=0.85)

        self._add_rule("PAUSE_MUSIC", [
            (r"(?:pause|stop)\s+(?:the\s+)?(?:music|song|track|audio|playback)", lambda m: {}),
        ], base_confidence=0.93)

        self._add_rule("NEXT_TRACK", [
            (r"(?:next|skip)\s+(?:the\s+)?(?:song|track|music)", lambda m: {}),
            (r"skip\s+(?:this\s+)?(?:song|track)?", lambda m: {}),
        ], base_confidence=0.93)

        self._add_rule("PREVIOUS_TRACK", [
            (r"(?:previous|last|go\s+back)\s+(?:the\s+)?(?:song|track|music)", lambda m: {}),
            (r"(?:go\s+back|play\s+(?:the\s+)?previous)", lambda m: {}),
        ], base_confidence=0.93)

        # ── AI INTENTS ────────────────────────────────────────────────
        self._add_rule("AI_WRITE", [
            (r"(?:write|draft|compose|create)\s+(?:me\s+)?(?:a\s+)?(.+?)\s*(?:letter|email|essay|report|proposal|document|message|story|poem|script|article|blog\s*post|bio|cover\s+letter|resignation\s+letter)", self._extract_ai_write),
            (r"(?:write|draft|compose|create)\s+(?:me\s+)?(?:a\s+)?(.+)", lambda m: {"query": m.group(1).strip(), "ai_type": "write"}),
        ], base_confidence=0.88)

        self._add_rule("AI_EXPLAIN", [
            (r"(?:explain|describe|what\s+is|what\s+are|tell\s+me\s+about|how\s+does|how\s+do|how\s+to)\s+(.+)", lambda m: {"query": m.group(1).strip(), "ai_type": "explain"}),
        ], base_confidence=0.80)

        self._add_rule("AI_SUMMARIZE", [
            (r"(?:summarize|summarise|sum\s+up|give\s+me\s+a\s+summary\s+of|tldr|tl;dr)\s+(.+)", lambda m: {"query": m.group(1).strip(), "ai_type": "summarize"}),
        ], base_confidence=0.90)

        # ── SMALL TALK ────────────────────────────────────────────────
        self._add_rule("SMALL_TALK", [
            (r"^(?:hi|hello|hey|hallo|hiya|heya|yo|sup|howdy)$", lambda m: {"message": "greeting"}),
            (r"^(?:how\s+are\s+you|how'?s?\s+it\s+going|how\s+are\s+you\s+doing|how\s+do\s+you\s+do)$", lambda m: {"message": "how_are_you"}),
            (r"^(?:who\s+are\s+you|what\s+are\s+you|tell\s+me\s+about\s+yourself|introduce\s+yourself)$", lambda m: {"message": "who_are_you"}),
            (r"^(?:good\s+morning|morning)$", lambda m: {"message": "good_morning"}),
            (r"^(?:good\s+afternoon|afternoon)$", lambda m: {"message": "good_afternoon"}),
            (r"^(?:good\s+evening|evening)$", lambda m: {"message": "good_evening"}),
            (r"^(?:good\s*night|good\s+night|nite)$", lambda m: {"message": "goodnight"}),
            (r"^(?:thank\s*you|thanks|thank\s+you\s+(?:so\s+much|very\s+much)|thx|ty)$", lambda m: {"message": "thanks"}),
            (r"^(?:bye|goodbye|see\s+you|see\s+ya|later|peace|cya|farewell)$", lambda m: {"message": "bye"}),
            (r"^(?:what\s+can\s+you\s+do|help|commands|capabilities|what\s+do\s+you\s+do)$", lambda m: {"message": "help"}),
            (r"^(?:you'?re?\s+(?:great|awesome|the\s+best|amazing)|good\s+(?:job|work)|well\s+done|nice)$", lambda m: {"message": "compliment"}),
            (r"^(?:i\s+love\s+you|love\s+you)$", lambda m: {"message": "love"}),
            (r"^(?:tell\s+me\s+a\s+joke|joke|make\s+me\s+laugh)$", lambda m: {"message": "joke"}),
        ], base_confidence=0.95)

    def _add_rule(self, intent: str, patterns: list, base_confidence: float):
        """Register an intent with its patterns."""
        for pattern, extractor in patterns:
            compiled = re.compile(pattern, re.IGNORECASE)
            self.rules.append((intent, compiled, extractor, base_confidence))

    # ─── Entity Extractors ─────────────────────────────────────────────

    def _extract_reminder(self, m):
        message = m.group(1).strip()
        amount = int(m.group(2))
        unit = m.group(3).lower()
        minutes = self._normalize_time_to_minutes(amount, unit)
        return {"message": message, "delay_minutes": minutes}

    def _extract_timer(self, m):
        amount = int(m.group(1))
        unit = m.group(2).lower()
        minutes = self._normalize_time_to_minutes(amount, unit)
        return {"minutes": str(minutes)}

    def _extract_create_folder(self, m):
        result = {"folder_name": m.group(1).strip()}
        if m.group(2):
            result["parent_path"] = m.group(2).strip()
        return result

    def _extract_music(self, m):
        result = {"song": m.group(1).strip()}
        if m.group(2):
            result["artist"] = m.group(2).strip()
        return result

    def _extract_ai_write(self, m):
        full_text = m.group(0).strip()
        return {"query": full_text, "ai_type": "write"}

    def _normalize_time_to_minutes(self, amount: int, unit: str) -> int:
        unit = unit.lower().rstrip("s")
        if unit in ("hour", "hr"):
            return amount * 60
        elif unit in ("second", "sec"):
            return max(1, amount // 60)  # Minimum 1 minute
        return amount  # Already minutes

    # ─── Normalization ─────────────────────────────────────────────────

    def _normalize(self, text: str) -> str:
        text = text.strip().lower()
        # Remove trailing punctuation
        text = re.sub(r'[.!?,;:]+$', '', text)
        # Remove internal punctuation except apostrophes and hyphens
        text = re.sub(r"[^\w\s'\-]", '', text)
        # Strip filler / polite words iteratively
        for pattern in self.noise_patterns:
            while True:
                new_text = re.sub(pattern, "", text, flags=re.IGNORECASE).strip()
                if new_text == text:
                    break
                text = new_text
        # Collapse whitespace
        text = re.sub(r'\s+', ' ', text).strip()
        return text

    # ─── Main Classification ───────────────────────────────────────────

    def parse(self, text: str) -> dict:
        """
        Classify input text into an intent with confidence and parameters.

        Returns:
            {
                "intent": str,
                "confidence": float,
                "parameters": dict,
                "original_text": str
            }
        """
        original = text
        normalized = self._normalize(text)

        if not normalized:
            return {
                "intent": "EMPTY",
                "confidence": 0.0,
                "parameters": {},
                "original_text": original,
            }

        best_match = None
        best_confidence = 0.0

        for intent, pattern, extractor, base_conf in self.rules:
            match = pattern.search(normalized)
            if match:
                # Calculate confidence based on coverage ratio
                span_ratio = (match.end() - match.start()) / max(len(normalized), 1)
                confidence = base_conf * (0.7 + span_ratio * 0.3)
                confidence = min(confidence, 1.0)
                confidence = round(confidence, 2)

                if confidence > best_confidence:
                    try:
                        parameters = extractor(match)
                    except Exception:
                        parameters = {}

                    best_match = {
                        "intent": intent,
                        "confidence": confidence,
                        "parameters": parameters,
                        "original_text": original,
                    }
                    best_confidence = confidence

        if best_match:
            # ── Post-classification disambiguation ─────────────────
            best_match = self._disambiguate(best_match, normalized)
            return best_match

        # ── Fallback: GENERAL_AI_CHAT (NOT web search!) ────────────
        return {
            "intent": "GENERAL_AI_CHAT",
            "confidence": 0.50,
            "parameters": {"query": original},
            "original_text": original,
        }

    def _disambiguate(self, result: dict, normalized: str) -> dict:
        """
        Resolve ambiguities between similar intents.
        e.g., "open youtube" should be OPEN_WEBSITE, not OPEN_APP.
        """
        intent = result["intent"]
        target = result["parameters"].get("target", "").lower().strip()

        # OPEN_APP → OPEN_WEBSITE if target is a known website
        if intent == "OPEN_APP" and target in KNOWN_WEBSITES:
            result["intent"] = "OPEN_WEBSITE"
            result["confidence"] = min(result["confidence"] + 0.05, 1.0)

        # OPEN_WEBSITE → OPEN_APP if target is a known local app
        if intent == "OPEN_WEBSITE" and target in KNOWN_APPS and target not in KNOWN_WEBSITES:
            result["intent"] = "OPEN_APP"

        # AI_EXPLAIN should not match simple "what is <website>" queries
        if intent == "AI_EXPLAIN" and target in KNOWN_WEBSITES:
            result["intent"] = "OPEN_WEBSITE"
            result["parameters"] = {"target": target}

        # "play X" should be PLAY_MUSIC not YOUTUBE_SEARCH unless "on youtube" is in the text
        if intent == "PLAY_MUSIC":
            song = result["parameters"].get("song", "")
            # If the song name is a known website/app, it's probably "play spotify"
            if song.lower() in KNOWN_APPS or song.lower() in KNOWN_WEBSITES:
                result["intent"] = "OPEN_APP"
                result["parameters"] = {"target": song}

        return result
