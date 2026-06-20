import os
import json
import time
import struct
import sounddevice as sd
import vosk
from ..config import VOSK_MODEL_PATH, WAKE_WORDS

class WakeWordDetector:
    def __init__(self):
        # Prevent Vosk from spamming the console
        vosk.SetLogLevel(-1)
        
        if not os.path.exists(VOSK_MODEL_PATH) or not os.path.exists(os.path.join(VOSK_MODEL_PATH, "am")):
            print(f"[WARN] Vosk model not found at {VOSK_MODEL_PATH}. Wake word will fail.")
            self.model = None
            return

        try:
            print("[Hyde] Loading Vosk Wake Word Model...")
            self.model = vosk.Model(VOSK_MODEL_PATH)
            
            # Constrain the grammar to only our wake words + unknown
            # This makes inference lightning fast and use 0% CPU
            grammar = json.dumps(WAKE_WORDS + ["[unk]"])
            self.recognizer = vosk.KaldiRecognizer(self.model, 16000, grammar)
            
        except Exception as e:
            print(f"[ERROR] Failed to initialize Vosk Wake Word: {e}")
            self.model = None

    def listen_for_wake_word(self):
        if not self.model:
            time.sleep(1) # Prevent tight loop if uninitialized
            return False

        print(f"[Hyde] Listening for wake word: {WAKE_WORDS}...")
        
        try:
            # 8000 blocksize is 0.5s of audio at 16khz
            with sd.RawInputStream(samplerate=16000, blocksize=8000, dtype='int16', channels=1) as stream:
                while True:
                    data, overflowed = stream.read(8000)
                    
                    if self.recognizer.AcceptWaveform(bytes(data)):
                        result = json.loads(self.recognizer.Result())
                        text = result.get("text", "")
                        
                        for word in WAKE_WORDS:
                            if word in text:
                                return True
                    else:
                        partial = json.loads(self.recognizer.PartialResult())
                        partial_text = partial.get("partial", "")
                        
                        for word in WAKE_WORDS:
                            if word in partial_text:
                                return True
                                
        except Exception as e:
            print(f"[ERROR] Wake word stream error: {e}")
            time.sleep(1)
            return False

    def close(self):
        # Python's GC will clean up Vosk naturally
        pass
