# Hyde Agent V3 AI Capabilities

Hyde V3 implements a **Dual Mode AI Architecture**, providing flexibility, privacy, and power depending on your workflow.

## The Dual Mode Architecture

### Mode 1: Browser AI Mode (Default)
**Use Case:** Out-of-the-box experience. No API keys required. Free to use.

When you ask Hyde an AI-related question (e.g., "tell me about react" or "write a resignation letter"), Hyde will automatically:
1. Detect your configured AI Provider (default: Gemini).
2. Construct a URL deep-link containing your prompt.
3. Automatically copy your prompt to your clipboard (as a fallback).
4. Launch your default browser and open the AI Provider's chat interface.

**Supported Browser Mode Providers:**
* **ChatGPT**: `chatgpt.com`
* **Claude**: `claude.ai`
* **Gemini**: `gemini.google.com`
* **Perplexity**: `perplexity.ai`
* **Grok**: `grok.com`

*Note: If the text does not automatically populate in the provider's text box, simply press `Ctrl+V` to paste the prompt Hyde saved to your clipboard.*

---

### Mode 2: API Mode (Advanced)
**Use Case:** Integrated, seamless desktop experience. Requires API keys.

If you configure an API Key for your provider in Hyde's Settings, Hyde bypasses the browser entirely. It streams the AI's response directly into the Hyde Agent UI using the official REST APIs.

**Supported API Mode Providers:**
* **OpenAI (ChatGPT)** (`gpt-4o`)
* **Anthropic (Claude)** (`claude-3-5-sonnet`)
* **Google (Gemini)** (`gemini-1.5-flash`)
* **Perplexity** (`llama-3-sonar-large-32k-online`)
* **Ollama** (Local `llama3` execution - No key required)

## Absolute Fallback Routing
If Hyde receives a completely unrecognized command, it no longer performs a blind Web Search. Instead, the input is treated as `AI_CHAT` and routed to your AI Provider.

**Example Flow:**
`"Why is the sky blue?"` -> (Not an explicit OS command) -> `AI_CHAT` -> Opens Browser AI Mode (or queries API).
