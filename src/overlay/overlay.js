import { pipeline, env } from './transformers.min.js';
env.allowLocalModels = false;
env.useBrowserCache = true;

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
    if (isRecording) {
        mediaRecorder.stop();
        isRecording = false;
        micBtn.classList.remove('recording');
        micBtn.innerHTML = '🎙️';
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

let transcriber = null;
let isRecording = false;
let mediaRecorder = null;
let audioChunks = [];

async function loadWhisper() {
    if (transcriber) return;
    voiceStatus.innerText = "Loading Offline Voice Model (first time only)...";
    try {
        transcriber = await pipeline('automatic-speech-recognition', 'Xenova/whisper-tiny.en');
        voiceStatus.innerText = "Tap to start listening...";
    } catch (e) {
        voiceStatus.innerText = "Error loading model. Check console.";
        console.error(e);
    }
}

async function startRecording() {
    await loadWhisper();
    try {
        const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
        mediaRecorder = new MediaRecorder(stream);
        audioChunks = [];
        
        mediaRecorder.ondataavailable = e => {
            if (e.data.size > 0) audioChunks.push(e.data);
        };
        
        mediaRecorder.onstop = async () => {
            const mimeType = audioChunks.length > 0 ? audioChunks[0].type : 'audio/webm';
            const audioBlob = new Blob(audioChunks, { type: mimeType });
            await processAudio(audioBlob);
            stream.getTracks().forEach(track => track.stop());
        };
        
        mediaRecorder.start(250); // 250ms chunks to prevent missing duration bugs
        isRecording = true;
        micBtn.classList.add('recording');
        micBtn.innerHTML = '🔴';
        if (currentMode === 'voice') {
            voiceCenter.classList.add('listening');
            voiceStatus.innerText = "Listening... Tap to stop and process.";
        }
    } catch (err) {
        console.error("Mic access error:", err);
        voiceStatus.innerText = "Microphone access denied. Check Windows Privacy Settings.";
    }
}

async function processAudio(blob) {
    if (currentMode === 'voice') {
        voiceStatus.innerText = "Processing...";
    }
    
    try {
        // Decode Blob to Float32Array
        const arrayBuffer = await blob.arrayBuffer();
        const audioContext = new (window.AudioContext || window.webkitAudioContext)({ sampleRate: 16000 });
        const audioBuffer = await audioContext.decodeAudioData(arrayBuffer);
        const audioData = audioBuffer.getChannelData(0);
        
        if (currentMode === 'voice') {
            voiceStatus.innerText = `Processing ${audioBuffer.duration.toFixed(1)}s audio...`;
        }

        const output = await transcriber(audioData, {
            language: 'english',
            task: 'transcribe'
        });
        let text = output.text.trim();
        
        // Filter out common hallucinations if audio is mostly silence
        const lowerText = text.toLowerCase().replace(/[^a-z]/g, '');
        if (lowerText === 'you' || lowerText === 'thankyou' || lowerText === 'thanksforwatching') {
            text = '';
        }
        
        if (text) {
            input.value = text;
            if (currentMode === 'voice') {
                voiceTranscript.innerText = text;
            }
            await handleCommand(text);
            if (currentMode === 'voice') {
                setTimeout(() => {
                    voiceTranscript.innerText = "";
                }, 3000);
            }
        }
    } catch(err) {
        console.error("Whisper decoding error:", err);
        if (currentMode === 'voice') voiceStatus.innerText = "Error decoding audio.";
    }
    
    if (currentMode === 'voice') {
        setTimeout(() => {
            if (!isRecording) voiceStatus.innerText = "Tap to start listening...";
        }, 1500);
    }
}

micBtn.addEventListener('click', () => {
    if (isRecording) {
        mediaRecorder.stop();
        isRecording = false;
        micBtn.classList.remove('recording');
        micBtn.innerHTML = '🎙️';
        voiceCenter.classList.remove('listening');
    } else {
        startRecording();
    }
});

// Listen for background events (like timer toast)
listen('show-toast', (event) => {
    appendMessage(`🔔 Notification: ${event.payload.message}`, false, event.payload.is_error);
});
