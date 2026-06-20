import time
import numpy as np
import sounddevice as sd
import speech_recognition as sr
from faster_whisper import WhisperModel
from ..config import WHISPER_MODEL_SIZE, WHISPER_DEVICE, WHISPER_COMPUTE_TYPE

class SpeechToText:
    def __init__(self):
        print("[Hyde] Loading Faster-Whisper Model...")
        self.model = WhisperModel(
            WHISPER_MODEL_SIZE,
            device=WHISPER_DEVICE,
            compute_type=WHISPER_COMPUTE_TYPE
        )
        self.sample_rate = 16000
        self.recognizer = sr.Recognizer()

    def listen_and_transcribe(self, timeout=8):
        """
        Listens to the microphone for up to `timeout` seconds of silence.
        Uses Google STT for high accuracy, falls back to offline Whisper.
        """
        print("[Hyde] Listening for command...")
        
        # Audio parameters
        block_duration = 0.05  # 50ms
        block_size = int(self.sample_rate * block_duration)
        
        audio_buffer = []
        silence_start = None
        speaking = False
        wait_start = time.time()
        
        try:
            # We must use int16 so speech_recognition can parse the bytes directly
            with sd.RawInputStream(samplerate=self.sample_rate, channels=1, blocksize=block_size, dtype='int16') as stream:
                while True:
                    data, overflowed = stream.read(block_size)
                    audio_buffer.append(bytes(data))
                    
                    # Convert to numpy for VAD
                    # data is bytes, convert to int16 array
                    audio_np = np.frombuffer(data, dtype=np.int16)
                    # Basic VAD (Volume Thresholding)
                    volume = np.max(np.abs(audio_np))
                    
                    if volume > 500:  # Int16 max is 32767, so 500 is a decent threshold
                        if not speaking:
                            speaking = True
                            print("[STATE: SPEAKING]", flush=True)
                        silence_start = None
                    else:
                        if speaking:
                            if silence_start is None:
                                silence_start = time.time()
                            elif time.time() - silence_start > 1.5:  # 1.5s of silence after speaking
                                break
                        else:
                            if time.time() - wait_start > timeout:
                                return None  # Timeout reached, no speech detected

        except Exception as e:
            print(f"[ERROR] Audio capture failed: {e}")
            return None

        print("[Hyde] Transcribing...")
        
        raw_audio = b"".join(audio_buffer)
        
        # 1. Try Online Google STT for highest accuracy
        try:
            audio_data = sr.AudioData(raw_audio, self.sample_rate, 2) # 2 bytes per sample for int16
            text = self.recognizer.recognize_google(audio_data)
            print(f"[Hyde] (Online SR) Heard: {text}")
            return text
        except sr.UnknownValueError:
            pass # Google couldn't understand, let's try offline just in case
        except sr.RequestError:
            print("[WARN] Google STT unreachable. Falling back to offline Whisper.")
        except Exception as e:
            print(f"[WARN] Online STT error: {e}")

        # 2. Fallback to Offline Faster-Whisper
        print("[Hyde] Using offline Whisper fallback...")
        try:
            # Whisper expects float32 [-1.0, 1.0]
            float_audio = np.frombuffer(raw_audio, dtype=np.int16).astype(np.float32) / 32768.0
            segments, info = self.model.transcribe(float_audio, beam_size=5)
            text = "".join([segment.text for segment in segments]).strip()
            return text
        except Exception as e:
            print(f"[ERROR] Whisper transcription failed: {e}")
            return None
