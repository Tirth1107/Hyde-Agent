const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;

const input = document.getElementById('command-input');
const sendBtn = document.getElementById('send-btn');
const micBtn = document.getElementById('mic-btn');
const chatHistory = document.getElementById('chat-history');
const closeBtn = document.getElementById('close-btn');
const settingsBtn = document.getElementById('settings-btn');

const appWindow = getCurrentWindow();

closeBtn.addEventListener('click', () => {
    appWindow.hide();
});

settingsBtn.addEventListener('click', () => {
    // Open settings window from rust
    invoke('execute_command', { command: 'settings' }).catch(err => {
        console.error(err);
        appendMessage("Could not open settings.", false, true);
    });
});

function appendMessage(text, isUser = false, isError = false) {
    const msgDiv = document.createElement('div');
    msgDiv.className = `message ${isUser ? 'user-msg' : 'system-msg'}`;
    
    const avatarDiv = document.createElement('div');
    avatarDiv.className = 'avatar';
    avatarDiv.innerText = isUser ? 'YOU' : 'AI';
    
    const bubbleDiv = document.createElement('div');
    bubbleDiv.className = 'bubble';
    bubbleDiv.innerText = text;
    
    if (isError) bubbleDiv.style.color = '#fb7185';
    
    msgDiv.appendChild(avatarDiv);
    msgDiv.appendChild(bubbleDiv);
    
    chatHistory.appendChild(msgDiv);
    chatHistory.scrollTop = chatHistory.scrollHeight;
}

async function handleCommand(cmd) {
    if (!cmd.trim()) return;
    appendMessage(cmd, true);
    input.value = '';
    
    try {
        const result = await invoke('execute_command', { command: cmd });
        appendMessage(result, false);
    } catch (error) {
        appendMessage(error, false, true);
    }
}

const suggestionsBox = document.getElementById('suggestions');
const modeChatBtn = document.getElementById('mode-chat');
const modeVoiceBtn = document.getElementById('mode-voice');
const chatView = document.getElementById('chat-view');
const voiceView = document.getElementById('voice-view');
const voiceStatus = document.getElementById('voice-status');
const voiceTranscript = document.getElementById('voice-transcript');
const voiceCenter = document.querySelector('.voice-center');

let currentMode = 'chat'; // 'chat' or 'voice'

modeChatBtn.addEventListener('click', () => {
    currentMode = 'chat';
    modeChatBtn.classList.add('active');
    modeVoiceBtn.classList.remove('active');
    chatView.classList.add('active');
    chatView.classList.remove('hidden');
    voiceView.classList.remove('active');
    voiceView.classList.add('hidden');
    if (shouldListenContinuously) {
        shouldListenContinuously = false;
    }
});

modeVoiceBtn.addEventListener('click', () => {
    currentMode = 'voice';
    modeVoiceBtn.classList.add('active');
    modeChatBtn.classList.remove('active');
    voiceView.classList.add('active');
    voiceView.classList.remove('hidden');
    chatView.classList.remove('active');
    chatView.classList.add('hidden');
    
    // Auto-start native voice recognition
    shouldListenContinuously = true;
    listenLoop();
});

voiceCenter.addEventListener('click', () => {
    if (currentMode !== 'voice') return;
    
    if (isRecording || shouldListenContinuously) {
        shouldListenContinuously = false;
        voiceStatus.innerText = "Paused. Tap to listen.";
    } else {
        shouldListenContinuously = true;
        listenLoop();
    }
});

sendBtn.addEventListener('click', () => {
    handleCommand(input.value);
    suggestionsBox.classList.add('hidden');
});

input.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
        handleCommand(input.value);
        suggestionsBox.classList.add('hidden');
    }
});

input.addEventListener('input', async (e) => {
    const val = e.target.value.trim();
    if (!val) {
        suggestionsBox.classList.add('hidden');
        return;
    }
    
    try {
        const suggs = await invoke('get_suggestions', { query: val });
        if (suggs.length > 0) {
            suggestionsBox.innerHTML = '';
            suggs.forEach(s => {
                const div = document.createElement('div');
                div.className = 'suggestion-item';
                div.innerText = s;
                div.onclick = () => {
                    input.value = s;
                    handleCommand(s);
                    suggestionsBox.classList.add('hidden');
                };
                suggestionsBox.appendChild(div);
            });
            suggestionsBox.classList.remove('hidden');
        } else {
            suggestionsBox.classList.add('hidden');
        }
    } catch (err) {
        console.error(err);
    }
});

let isRecording = false;
let shouldListenContinuously = false;

async function listenLoop() {
    if (!shouldListenContinuously) return;
    isRecording = true;
    micBtn.classList.add('recording');
    micBtn.innerHTML = '🔴';
    
    if (currentMode === 'voice') {
        voiceCenter.classList.add('listening');
        voiceStatus.innerText = "Listening...";
    }
    
    try {
        const text = await invoke('start_native_listening');
        if (text) {
            input.value = text;
            if (currentMode === 'voice') {
                voiceTranscript.innerText = text;
                voiceStatus.innerText = "Processing...";
            }
            await handleCommand(text);
            if (currentMode === 'voice') {
                if (shouldListenContinuously) voiceStatus.innerText = "Listening...";
                voiceTranscript.innerText = "";
            }
        }
    } catch (e) {
        console.error("Native voice error:", e);
    }
    
    isRecording = false;
    micBtn.classList.remove('recording');
    micBtn.innerHTML = '🎙️';
    voiceCenter.classList.remove('listening');
    
    if (shouldListenContinuously) {
        setTimeout(listenLoop, 100);
    } else {
        if (currentMode === 'voice') {
             voiceStatus.innerText = "Tap to start listening...";
        }
    }
}

micBtn.addEventListener('click', () => {
    if (isRecording || shouldListenContinuously) {
        shouldListenContinuously = false;
    } else {
        shouldListenContinuously = true;
        listenLoop();
    }
});

// Listen for background events (like timer toast)
listen('show-toast', (event) => {
    appendMessage(`🔔 Notification: ${event.payload.message}`, false, event.payload.is_error);
});
