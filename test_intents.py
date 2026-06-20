import sys
import json
import os
sys.path.append(os.path.join(os.path.dirname(__file__), "src-tauri"))

from hyde_engine.nlu.intent_parser import IntentParser
from hyde_engine.nlu.context_memory import ContextMemory

def test():
    parser = IntentParser()
    memory = ContextMemory()
    
    test_cases = [
        "turn up the volume",
        "lock my screen",
        "open youtube",
        "play lofi hip hop",
        "who won the super bowl 2024",
        "set a timer for 10 minutes",
        "cancel my timer",
        "show me active timers",
        "explain quantum computing to a 5 year old",
        "write an email to my boss asking for vacation",
        "how are you today",
        "remind me to check the oven in 15 minutes",
        "copy this text: hello world",
        "empty the recycle bin",
        "what time is it",
        "what is the weather like",
    ]
    
    print("=== HYDE INTENT PARSER VALIDATION ===\n")
    
    for tc in test_cases:
        payload = parser.parse(tc)
        intent = payload["intent"]
        conf = payload["confidence"]
        params = payload.get("parameters", {})
        
        print(f'Input: "{tc}"')
        print(f'Intent: {intent} (Conf: {conf:.2f})')
        if params:
            print(f'Params: {json.dumps(params)}')
        print("-" * 40)

if __name__ == "__main__":
    test()
