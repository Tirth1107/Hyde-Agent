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
        recognition.stop();
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
    
    // Auto-start voice recognition if available
    if (SpeechRecognition) {
        shouldListenContinuously = true;
        try { recognition.start(); } catch(e) {}
    } else {
        voiceStatus.innerText = "Voice not supported. Please download Vosk in Settings.";
    }
});

voiceCenter.addEventListener('click', () => {
    if (currentMode !== 'voice' || !SpeechRecognition) return;
    
    if (isRecording || shouldListenContinuously) {
        shouldListenContinuously = false;
        recognition.stop();
        voiceStatus.innerText = "Paused. Tap to listen.";
    } else {
        shouldListenContinuously = true;
        voiceStatus.innerText = "Listening...";
        try { recognition.start(); } catch(e) {}
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

// Voice Detection (Web Speech API)
const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
if (SpeechRecognition) {
    const recognition = new SpeechRecognition();
    recognition.continuous = false;
    recognition.lang = 'en-US';
    recognition.interimResults = true;
    
    let isRecording = false;
    let shouldListenContinuously = false;
    
    micBtn.addEventListener('click', () => {
        if (isRecording || shouldListenContinuously) {
            shouldListenContinuously = false;
            recognition.stop();
        } else {
            shouldListenContinuously = true;
            recognition.start();
        }
    });
    
    recognition.onstart = () => {
        isRecording = true;
        micBtn.classList.add('recording');
        micBtn.innerHTML = '🔴';
        
        if (currentMode === 'voice') {
            voiceCenter.classList.add('listening');
            voiceStatus.innerText = "Listening...";
        }
    };
    
    recognition.onresult = (event) => {
        let interimTranscript = '';
        for (let i = event.resultIndex; i < event.results.length; i++) {
            if (event.results[i].isFinal) {
                const finalTranscript = event.results[i][0].transcript;
                input.value = finalTranscript;
                if (currentMode === 'voice') {
                    voiceTranscript.innerText = finalTranscript;
                    voiceStatus.innerText = "Processing...";
                }
                handleCommand(finalTranscript).then(() => {
                    if (currentMode === 'voice') {
                        setTimeout(() => {
                            if (isRecording) voiceStatus.innerText = "Listening...";
                            voiceTranscript.innerText = "";
                        }, 2000);
                    }
                });
            } else {
                interimTranscript += event.results[i][0].transcript;
            }
        }
        if (interimTranscript !== '') {
            input.value = interimTranscript;
            if (currentMode === 'voice') {
                voiceTranscript.innerText = interimTranscript;
            }
        }
    };
    
    recognition.onerror = (event) => {
        console.error("Speech recognition error", event.error);
        if (event.error !== 'no-speech') {
            appendMessage(`Voice error: ${event.error}`, false, true);
        }
    };
    
    recognition.onend = () => {
        isRecording = false;
        micBtn.classList.remove('recording');
        micBtn.innerHTML = '🎙️';
        voiceCenter.classList.remove('listening');
        if (currentMode === 'voice' && voiceStatus.innerText === "Listening...") {
             voiceStatus.innerText = "Tap to start listening...";
        }
        
        if (shouldListenContinuously) {
            try {
                recognition.start();
            } catch(e) {
                console.error("Failed to restart continuous listening", e);
            }
        }
    };
} else {
    micBtn.title = "Voice recognition not supported in this environment";
    micBtn.style.opacity = 0.5;
    micBtn.addEventListener('click', () => {
        appendMessage("Voice recognition is not supported in this environment.", false, true);
    });
}

// Listen for background events (like timer toast)
listen('show-toast', (event) => {
    appendMessage(`🔔 Notification: ${event.payload.message}`, false, event.payload.is_error);
});
