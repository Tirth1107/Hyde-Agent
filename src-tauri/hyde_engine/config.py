import os
from pathlib import Path

# Paths
BASE_DIR = Path(__file__).parent
DATA_DIR = Path.home() / ".hyde-agent" / "engine_data"

# Ensure data directory exists
DATA_DIR.mkdir(parents=True, exist_ok=True)

# Wake Word Settings (Vosk)
VOSK_MODEL_PATH = str(DATA_DIR.parent / "vosk")
WAKE_WORDS = ["hyde", "hide"]

# STT Settings (Faster-Whisper)
WHISPER_MODEL_SIZE = "base.en"
WHISPER_DEVICE = "cpu" # Auto-detect GPU in actual impl, defaulting to CPU for max compatibility
WHISPER_COMPUTE_TYPE = "int8"

# TTS Settings (Piper)
PIPER_MODEL_PATH = str(DATA_DIR / "en_US-lessac-medium.onnx")
PIPER_VOICE_URL = "https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/lessac/medium/en_US-lessac-medium.onnx"
PIPER_JSON_URL = "https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/lessac/medium/en_US-lessac-medium.onnx.json"

# NLU Settings
CONFIDENCE_THRESHOLD = 0.65

# Personality
BOT_NAME = "Hyde"
DEFAULT_REPLIES = {
    "wake": "Yes Sir.",
    "timeout": "Standing by, Sir.",
    "error": "Sir, I didn't understand that. Could you repeat?",
    "confirm": "Task completed.",
}
