"""
Hyde Neural Engine — Main Entry Point

Runs two concurrent loops:
  1. Voice Loop: Wake word → STT → NLU → Execute → TTS (existing)
  2. Chat Loop: stdin JSON-RPC → NLU → Execute → stdout JSON-RPC (new)

The Chat Loop allows the Rust frontend to send typed commands through
stdin and receive structured responses through stdout, sharing the same
NLU pipeline and context memory as the voice loop.

Protocol:
  Rust → Python (stdin, one JSON per line):
    {"id": "req_001", "method": "classify", "params": {"text": "open youtube"}}

  Python → Rust (stdout, one JSON per line):
    {"id": "req_001", "result": {...}}

  Voice events (stdout, plain text — unchanged):
    [STATE: READY]
    [STATE: SPEAKING]
    [User]: <text>
    [STATE: SUCCESS]
"""

import sys
import json
import time
import threading
import traceback
import winsound

from .audio.wake_word import WakeWordDetector
from .audio.stt import SpeechToText
from .audio.tts import TextToSpeech
from .nlu.intent_parser import IntentParser
from .nlu.context_memory import ContextMemory
from .actions.executor import ActionExecutor
from .config import CONFIDENCE_THRESHOLD


def _send_json(obj: dict):
    """Send a JSON object to stdout (for Rust to read)."""
    line = json.dumps(obj, ensure_ascii=False)
    print(f"[JSON]{line}", flush=True)


def _send_state(state: str):
    """Send a voice state marker to stdout."""
    print(f"[STATE: {state}]", flush=True)


def chat_loop(intent_parser: IntentParser, memory: ContextMemory, executor: ActionExecutor):
    """
    Read JSON-RPC requests from stdin, classify intent, execute, respond.
    Runs in a background thread so it doesn't block the voice loop.
    """
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue

        try:
            request = json.loads(line)
        except json.JSONDecodeError:
            continue

        req_id = request.get("id", "unknown")
        method = request.get("method", "")
        params = request.get("params", {})

        if method == "classify":
            text = params.get("text", "")
            source = params.get("source", "chat")

            if not text.strip():
                _send_json({"id": req_id, "result": {
                    "intent": "EMPTY",
                    "confidence": 0.0,
                    "response": "",
                    "success": False,
                }})
                continue

            # Phase 1: Intent Classification
            payload = intent_parser.parse(text)

            # Phase 2: Context Injection
            payload = memory.inject_context(payload)

            # Phase 3: Confidence Check
            if payload["confidence"] < CONFIDENCE_THRESHOLD:
                _send_json({"id": req_id, "result": {
                    "intent": payload["intent"],
                    "confidence": payload["confidence"],
                    "response": "Sir, I didn't fully understand that. Could you rephrase?",
                    "success": False,
                    "parameters": payload.get("parameters", {}),
                }})
                continue

            # Phase 4: Execute
            exec_result = executor.execute(payload)

            # Phase 5: Update Context
            memory.add_context(payload)

            # Phase 6: Respond
            response_data = {
                "intent": payload["intent"],
                "confidence": payload["confidence"],
                "parameters": payload.get("parameters", {}),
                "success": exec_result.get("success", False),
                "response": exec_result.get("response", ""),
                "action_taken": exec_result.get("action_taken", ""),
                "original_text": text,
            }

            # If there's a Rust action to forward (e.g., system controls, timers)
            if "rust_action" in exec_result:
                response_data["rust_action"] = exec_result["rust_action"]

            # If intent requires AI (LLM), signal Rust to handle it
            if payload["intent"] in ("AI_CHAT", "AI_GENERATION"):
                response_data["requires_ai"] = True
                response_data["ai_type"] = payload.get("parameters", {}).get("ai_type", "chat")
                response_data["ai_query"] = payload.get("parameters", {}).get("query", text)

            _send_json({"id": req_id, "result": response_data})

        elif method == "context":
            # Return current active context for UI display
            active = memory.get_active_context()
            _send_json({"id": req_id, "result": {"context": active}})

        elif method == "ping":
            _send_json({"id": req_id, "result": {"status": "alive"}})

        else:
            _send_json({"id": req_id, "error": f"Unknown method: {method}"})


def voice_loop(wake_detector: WakeWordDetector, stt: SpeechToText, tts: TextToSpeech,
               intent_parser: IntentParser, memory: ContextMemory, executor: ActionExecutor):
    """
    Continuous voice loop: wake word → listen → classify → execute → speak.
    """
    try:
        while True:
            # Phase 1: Wait for Wake Word
            _send_state("IDLE")
            if wake_detector.listen_for_wake_word():
                _send_state("READY")
                # Instantaneous beep feedback instead of slow TTS
                winsound.Beep(800, 150)

                # Phase 2: Speech to Text (8s timeout)
                text = stt.listen_and_transcribe(timeout=8.0)

                if not text:
                    _send_state("TIMEOUT")
                    tts.reply_timeout()
                    continue

                print(f"[User]: {text}", flush=True)

                # Phase 3: Intent Classification
                payload = intent_parser.parse(text)
                print(f"[NLU] Intent: {payload['intent']} | Confidence: {payload['confidence']}")

                # Phase 4: Confidence Check
                if payload["confidence"] < CONFIDENCE_THRESHOLD:
                    _send_state("ERROR")
                    tts.reply_error()
                    continue

                # Phase 5: Context Injection
                payload = memory.inject_context(payload)
                memory.add_context(payload)

                # Phase 6: Execute
                exec_result = executor.execute(payload)

                if exec_result.get("success"):
                    _send_state("SUCCESS")
                else:
                    _send_state("ERROR")
                    error_msg = exec_result.get("response", "Something went wrong.")
                    if exec_result.get("speak", True) and error_msg:
                        tts.speak(error_msg)

    except KeyboardInterrupt:
        print("\n[Hyde Engine] Voice loop shutting down...")
    except Exception as e:
        print(f"[ERROR] Voice loop crashed: {e}")
        traceback.print_exc()


def main():
    print("[Hyde Engine] Initializing...", flush=True)

    # Initialize components
    wake_detector = WakeWordDetector()
    stt = SpeechToText()
    tts = TextToSpeech()
    intent_parser = IntentParser()
    memory = ContextMemory()
    executor = ActionExecutor(tts_engine=tts)

    print("\n=================================")
    print("   HYDE NEURAL ENGINE ONLINE     ")
    print("=================================\n", flush=True)

    # Start Chat Loop in background thread (stdin JSON-RPC)
    chat_thread = threading.Thread(
        target=chat_loop,
        args=(intent_parser, memory, executor),
        daemon=True,
    )
    chat_thread.start()

    # Send ready signal
    _send_json({"id": "boot", "result": {"status": "ready"}})

    # Start Voice Loop in main thread
    try:
        voice_loop(wake_detector, stt, tts, intent_parser, memory, executor)
    except Exception as e:
        print(f"[Hyde Engine] Fatal error: {e}", flush=True)
        traceback.print_exc()
    finally:
        wake_detector.close()
        print("[Hyde Engine] Shutdown complete.", flush=True)


if __name__ == "__main__":
    main()
