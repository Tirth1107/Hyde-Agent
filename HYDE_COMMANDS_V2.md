# Hyde Agent V3 Command Inventory

The Hyde V3 architecture introduces the Dual-Mode AI engine and sets AI logic as the absolute fallback.

## System Commands (Native Execution)
These commands trigger high-speed local OS controls.

* "Volume up/down/mute"
* "Set volume to 50%"
* "Lock my PC"
* "Sleep the computer"
* "Shutdown the PC"
* "Restart my device"
* "Take a screenshot"
* "Empty the recycle bin"
* "What is the time?"
* "What is today's date?"
* "How much RAM am I using?"
* "What is my battery level?"

## Media Controls
* "Play [Song Name] by [Artist]"
* "Pause music"
* "Next track"
* "Previous track"

## Application & Web
* "Open Chrome"
* "Launch Discord"
* "Open google.com" (Domain TLD supported: .com, .in, .net, .org, .io, .dev, .ai, .app, .co)
* "Open thenn.in"

## Scheduling & Productivity
* "Set a timer for 10 minutes"
* "Remind me to drink water in 30 minutes"
* "What are my active timers?"
* "Cancel my timer"

## Search & Navigation
* "Search for rust tutorials on Youtube"
* "Look up Tauri on Github"
* "Find mechanical keyboards on Reddit"
* "Search mail for invoices"
* "Google how to bake a cake"

## AI & Generation (Dual Mode)
These bypass the OS executor and query the configured LLM (ChatGPT, Claude, Gemini, Ollama, Perplexity, Grok). If no API key is provided, Hyde gracefully opens the AI in your default browser and copies the prompt to your clipboard.

* "Tell me a joke"
* "Explain quantum computing"
* "Write me a resignation letter"
* "Summarize this article"
* "Research the history of the Roman Empire"

## Unrecognized Commands
*Any command that is completely unrecognized by Hyde will automatically fall back to the AI Engine (`AI_CHAT`), rather than performing a blind web search.*
