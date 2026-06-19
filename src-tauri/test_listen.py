import speech_recognition as sr
import sys

r = sr.Recognizer()
r.pause_threshold = 1.0  # seconds of non-speaking audio before a phrase is considered complete

try:
    with sr.Microphone() as source:
        # Adjust for ambient noise
        r.adjust_for_ambient_noise(source, duration=0.5)
        
        # Listen until silence is detected
        audio = r.listen(source, timeout=10, phrase_time_limit=15)
        
    text = r.recognize_google(audio)
    print(text)
    sys.stdout.flush()
except Exception as e:
    print(f"ERROR: {str(e)}", file=sys.stderr)
    sys.exit(1)
