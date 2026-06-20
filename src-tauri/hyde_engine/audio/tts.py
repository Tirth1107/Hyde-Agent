import os
import json
import wave
import urllib.request
import sounddevice as sd
import numpy as np
import subprocess
from ..config import PIPER_MODEL_PATH, PIPER_VOICE_URL, PIPER_JSON_URL, DATA_DIR, BOT_NAME, DEFAULT_REPLIES

class TextToSpeech:
    def __init__(self):
        self._ensure_piper_models()
        print(f"[{BOT_NAME}] TTS Initialized.")

    def _ensure_piper_models(self):
        json_path = PIPER_MODEL_PATH + ".json"
        
        if not os.path.exists(PIPER_MODEL_PATH):
            print("[Hyde] Downloading Piper TTS model (this only happens once)...")
            urllib.request.urlretrieve(PIPER_VOICE_URL, PIPER_MODEL_PATH)
            
        if not os.path.exists(json_path):
            print("[Hyde] Downloading Piper TTS config...")
            urllib.request.urlretrieve(PIPER_JSON_URL, json_path)

    def speak(self, text: str):
        if not text:
            return
            
        print(f"[{BOT_NAME}]: {text}")
        
        # In a real production build, we would load the ONNX session directly via onnxruntime.
        # For this prototype, if piper is installed globally (via `pip install piper-tts`), 
        # we can stream it directly to sounddevice or use the piper python module.
        # We will use the piper module here if available.
        try:
            from piper import PiperVoice
            voice = PiperVoice.load(PIPER_MODEL_PATH)
            
            # Synthesize directly into a stream or file
            # For simplicity, we write to a temp wav and play it.
            # In a highly optimized version, we'd stream the generator output to sounddevice
            wav_file = str(DATA_DIR / "temp_reply.wav")
            with wave.open(wav_file, "wb") as wav:
                wav.setnchannels(1)
                wav.setsampwidth(2)
                wav.setframerate(voice.config.sample_rate)
                voice.synthesize(text, wav)
                
            self._play_wav(wav_file)
            os.remove(wav_file)
            
        except ImportError:
            print("[WARN] piper-tts not installed. Unable to speak audio. Please `pip install piper-tts`")

    def _play_wav(self, file_path):
        import scipy.io.wavfile as wavfile
        try:
            rate, data = wavfile.read(file_path)
            sd.play(data, rate)
            sd.wait()
        except Exception as e:
            print(f"[ERROR] Failed to play TTS output: {e}")

    def reply_wake(self):
        self.speak(DEFAULT_REPLIES["wake"])

    def reply_timeout(self):
        self.speak(DEFAULT_REPLIES["timeout"])

    def reply_error(self):
        self.speak(DEFAULT_REPLIES["error"])

    def reply_confirm(self):
        self.speak(DEFAULT_REPLIES["confirm"])
