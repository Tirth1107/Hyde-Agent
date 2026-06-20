
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
    
    if (isUser) {
        avatarDiv.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path><circle cx="12" cy="7" r="4"></circle></svg>';
    } else {
        avatarDiv.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/></svg>';
    }
    
    const bubbleDiv = document.createElement('div');
    bubbleDiv.className = 'bubble';
    
    // Support basic markdown-like formatting for AI responses
    if (!isUser && text) {
        bubbleDiv.innerHTML = formatResponse(text);
    } else {
        bubbleDiv.innerText = text || '';
    }
    
    if (isError) bubbleDiv.classList.add('error-bubble');
    
    msgDiv.appendChild(avatarDiv);
    msgDiv.appendChild(bubbleDiv);
    
    chatHistory.appendChild(msgDiv);
    chatHistory.scrollTop = chatHistory.scrollHeight;
}

function formatResponse(text) {
    // Simple markdown-like formatting
    let html = escapeHtml(text);
    
    // Bold **text**
    html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
    // Inline code `text`
    html = html.replace(/`(.+?)`/g, '<code>$1</code>');
    // Line breaks
    html = html.replace(/\n/g, '<br>');
    // Emoji indicators for action types
    html = html.replace(/^(✅|⏰|🔊|🔇|🔒|💤|⚡|🖥️|📁|📋|🤖|🔍|🎵)/gm, '<span class="action-icon">$1</span>');
    
    return html;
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function showTypingIndicator() {
    const existing = document.getElementById('typing-indicator');
    if (existing) return;
    
    const msgDiv = document.createElement('div');
    msgDiv.className = 'message system-msg';
    msgDiv.id = 'typing-indicator';
    
    const avatarDiv = document.createElement('div');
    avatarDiv.className = 'avatar';
    avatarDiv.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/></svg>';
    
    const bubbleDiv = document.createElement('div');
    bubbleDiv.className = 'bubble typing-bubble';
    bubbleDiv.innerHTML = '<div class="typing-dots"><span></span><span></span><span></span></div>';
    
    msgDiv.appendChild(avatarDiv);
    msgDiv.appendChild(bubbleDiv);
    
    chatHistory.appendChild(msgDiv);
    chatHistory.scrollTop = chatHistory.scrollHeight;
}

function removeTypingIndicator() {
    const existing = document.getElementById('typing-indicator');
    if (existing) existing.remove();
}

async function handleCommand(cmd) {
    if (!cmd.trim()) return;
    appendMessage(cmd, true);
    input.value = '';
    
    showTypingIndicator();
    
    try {
        const result = await invoke('execute_command', { command: cmd });
        removeTypingIndicator();
        appendMessage(result, false);
    } catch (error) {
        removeTypingIndicator();
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
    if (isRecording) {
        isRecording = false;
        micBtn.classList.remove('recording');
        voiceCenter.classList.remove('listening');
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
    startRecording();
});

voiceCenter.addEventListener('click', () => {
    if (currentMode !== 'voice') return;
    if (!isRecording) startRecording();
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

function startRecording() {
    if (currentMode === 'voice') {
        voiceStatus.innerText = "Hyde is always listening. Just say 'Hyde' to wake me!";
        setTimeout(() => {
            if (!isRecording) voiceStatus.innerText = "Listening in background... Say 'Hyde'";
        }, 3000);
    }
}

micBtn.addEventListener('click', () => {
    startRecording();
});

listen('voice-state', (event) => {
    const state = event.payload;
    
    if (state === 'READY') {
        isRecording = true;
        micBtn.classList.add('recording');
        voiceCenter.classList.add('listening');
        if (currentMode === 'voice') voiceStatus.innerText = "Listening... Speak now.";
    } else if (state === 'SPEAKING') {
        if (currentMode === 'voice') voiceStatus.innerText = "Speech detected... Processing...";
    } else if (state === 'TIMEOUT' || state === 'IDLE') {
        isRecording = false;
        micBtn.classList.remove('recording');
        voiceCenter.classList.remove('listening');
        if (currentMode === 'voice') voiceStatus.innerText = "Listening in background... Say 'Hyde'";
    } else if (state === 'ERROR') {
        isRecording = false;
        micBtn.classList.remove('recording');
        voiceCenter.classList.remove('listening');
        if (currentMode === 'voice') voiceStatus.innerText = "Error recognizing speech.";
    } else if (state === 'SUCCESS') {
        if (currentMode === 'voice') voiceStatus.innerText = "✅ Command executed.";
        setTimeout(() => {
            if (currentMode === 'voice') voiceStatus.innerText = "Listening in background... Say 'Hyde'";
        }, 2000);
    } else if (state.startsWith('TEXT:')) {
        const text = state.replace('TEXT:', '');
        input.value = text;
        if (currentMode === 'voice') {
            voiceTranscript.innerText = text;
            setTimeout(() => {
                voiceTranscript.innerText = "";
            }, 3000);
        }
    }
});

// Listen for background events (like timer toast)
listen('show-toast', (event) => {
    appendMessage(`🔔 ${event.payload.message}`, false, event.payload.is_error);
});

// Handle AI commands directly from voice loop
listen('voice-execute', (event) => {
    const text = event.payload;
    if (text) {
        handleCommand(text);
        if (currentMode === 'voice') {
            voiceStatus.innerText = "Executing AI Command...";
        }
    }
});
