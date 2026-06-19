import sounddevice as sd
import numpy as np
import speech_recognition as sr
import scipy.io.wavfile as wav
import sys
import warnings
import os

warnings.filterwarnings("ignore")

fs = 16000
chunk_duration = 0.5  # 500ms chunks
threshold = 0.015     # volume threshold (1.5%)

def main():
    recorded_chunks = []
    speaking = False
    silence_chunks = 0
    max_silence = 3 # 1.5 seconds of silence stops recording
    max_wait_chunks = 20 # 10 seconds timeout if no speech detected
    wait_chunks = 0
    
    # Pre-warm the sound device
    try:
        sd.rec(int(0.1 * fs), samplerate=fs, channels=1, dtype='float32')
        sd.wait()
    except Exception as e:
        sys.exit(1)
        
    print("READY")
    sys.stdout.flush()
    
    while True:
        chunk = sd.rec(int(chunk_duration * fs), samplerate=fs, channels=1, dtype='float32')
        sd.wait()
        
        volume = np.max(np.abs(chunk))
        
        if volume > threshold:
            if not speaking:
                speaking = True
                print("SPEAKING")
                sys.stdout.flush()
            recorded_chunks.append(chunk)
            silence_chunks = 0
        else:
            if speaking:
                recorded_chunks.append(chunk)
                silence_chunks += 1
                if silence_chunks >= max_silence:
                    break
            else:
                wait_chunks += 1
                if wait_chunks >= max_wait_chunks:
                    print("TIMEOUT")
                    sys.exit(0)
                    
    if not recorded_chunks:
        sys.exit(0)
        
    full_audio = np.concatenate(recorded_chunks, axis=0)
    full_audio_int16 = np.int16(full_audio * 32767)
    
    wav.write('temp.wav', fs, full_audio_int16)
    
    r = sr.Recognizer()
    try:
        with sr.AudioFile('temp.wav') as source:
            audio = r.record(source)
            text = r.recognize_google(audio)
            print(f"TEXT:{text}")
            sys.stdout.flush()
    except Exception:
        print("ERROR")
        pass
        
    try:
        os.remove('temp.wav')
    except:
        pass

if __name__ == "__main__":
    main()
