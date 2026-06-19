import sys
import os
import json
import queue
import time
import urllib.request
import zipfile
import struct
import shutil

# Add extensive logging
print("PYTHON SCRIPT STARTED")
sys.stdout.flush()

try:
    import sounddevice as sd
    import vosk
    import speech_recognition as sr
except ImportError as e:
    print(f"ERRORS: Missing dependency - {e}")
    print("ERROR")
    sys.exit(1)

# Configuration
import pathlib
HOME_DIR = str(pathlib.Path.home())
AGENT_DIR = os.path.join(HOME_DIR, ".hyde-agent")
MODEL_PATH = os.path.join(AGENT_DIR, "vosk")
MODEL_URL = "https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip"
SAMPLE_RATE = 16000
BLOCK_SIZE = 8000
SILENCE_TIMEOUT = 1.5  # Seconds of silence before stopping
MAX_WAIT_TIME = 10.0   # 10 seconds timeout if no speech detected

def download_model():
    if not os.path.exists(MODEL_PATH) or not os.path.exists(os.path.join(MODEL_PATH, "am")):
        print(f"VOSK MODEL NOT FOUND. DOWNLOADING TO {MODEL_PATH}...")
        sys.stdout.flush()
        
        if not os.path.exists(AGENT_DIR):
            os.makedirs(AGENT_DIR, exist_ok=True)
            
        # Clean up corrupted/dummy folder if it exists
        if os.path.exists(MODEL_PATH):
            shutil.rmtree(MODEL_PATH, ignore_errors=True)
            
        zip_path = os.path.join(AGENT_DIR, "model.zip")
        urllib.request.urlretrieve(MODEL_URL, zip_path)
        print("DOWNLOAD COMPLETE. EXTRACTING...")
        sys.stdout.flush()
        
        with zipfile.ZipFile(zip_path, 'r') as zip_ref:
            zip_ref.extractall(AGENT_DIR)
            
        extracted_folder = os.path.join(AGENT_DIR, "vosk-model-small-en-us-0.15")
        os.rename(extracted_folder, MODEL_PATH)
        os.remove(zip_path)
        print("VOSK MODEL LOADED AND EXTRACTED.")
        sys.stdout.flush()

def main():
    try:
        download_model()
    except Exception as e:
        print(f"ERRORS: Failed to download model: {e}")
        print("ERROR")
        sys.exit(1)
        
    print("VOSK LOADED")
    sys.stdout.flush()

    try:
        model = vosk.Model(MODEL_PATH)
        rec = vosk.KaldiRecognizer(model, SAMPLE_RATE)
    except Exception as e:
        print(f"ERRORS: Failed to initialize Vosk: {e}")
        print("ERROR")
        sys.exit(1)

    q = queue.Queue()

    def callback(indata, frames, time_info, status):
        if status:
            print(f"ERRORS: {status}", file=sys.stderr)
        q.put(bytes(indata))

    print("MIC DETECTED")
    print(f"SAMPLE RATE: {SAMPLE_RATE}")
    sys.stdout.flush()

    try:
        with sd.RawInputStream(samplerate=SAMPLE_RATE, blocksize=BLOCK_SIZE, 
                               device=None, dtype='int16',
                               channels=1, callback=callback):
            print("MIC STARTED")
            print("READY")
            sys.stdout.flush()
            
            speaking = False
            silence_start = None
            wait_start = time.time()
            
            recorded_frames = []
            vosk_text = ""
            
            while True:
                data = q.get()
                print(f"AUDIO RECEIVED")
                print(f"BUFFER SIZE: {len(data)}")
                sys.stdout.flush()
                
                # Accumulate for online backup
                if speaking or len(recorded_frames) > 0:
                    # Optional: collect a bit of pre-speech buffer if wanted, but simpler to just collect while speaking
                    recorded_frames.append(data)
                
                shorts = struct.unpack(f"{len(data)//2}h", data)
                volume = max(abs(s) for s in shorts) if shorts else 0
                
                if volume > 500:
                    if not speaking:
                        print("SPEAKING")
                        sys.stdout.flush()
                        speaking = True
                        recorded_frames.append(data) # ensure first frame is added
                    silence_start = None
                else:
                    if speaking:
                        if silence_start is None:
                            silence_start = time.time()
                        elif time.time() - silence_start > SILENCE_TIMEOUT:
                            break
                    else:
                        if time.time() - wait_start > MAX_WAIT_TIME:
                            print("TIMEOUT")
                            sys.stdout.flush()
                            sys.exit(0)

                if rec.AcceptWaveform(data):
                    res = json.loads(rec.Result())
                    text = res.get("text", "")
                    if text:
                        vosk_text = text
                        break
                else:
                    partial = json.loads(rec.PartialResult())
                    partial_text = partial.get("partial", "")
                    if partial_text:
                        print(f"PARTIAL RESULT: {partial_text}")
                        sys.stdout.flush()

            if not vosk_text:
                res = json.loads(rec.FinalResult())
                vosk_text = res.get("text", "")

            # Attempt Online Speech Recognition
            online_text = None
            audio_data = b"".join(recorded_frames)
            
            if audio_data:
                try:
                    recognizer = sr.Recognizer()
                    audio = sr.AudioData(audio_data, SAMPLE_RATE, 2)
                    online_text = recognizer.recognize_google(audio)
                except Exception as e:
                    print(f"ONLINE SR FAILED: {e}")
                    pass

            # Output the best available result
            final_text = online_text if online_text else vosk_text
            
            print(f"FINAL RESULT: {final_text}")
            if online_text:
                print("SOURCE: ONLINE")
            else:
                print("SOURCE: OFFLINE (VOSK)")
                
            print(f"TEXT:{final_text}")
            sys.stdout.flush()

    except Exception as e:
        print(f"ERRORS: {e}")
        print("ERROR")
        sys.exit(1)

if __name__ == "__main__":
    main()
