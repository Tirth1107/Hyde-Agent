use std::sync::{Arc, Mutex};

pub struct VoiceManager {
    is_listening: Arc<Mutex<bool>>,
}

impl VoiceManager {
    pub fn new() -> Self {
        Self {
            is_listening: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start_listening(&self, _model_path: &str, _callback: impl Fn(String) + Send + 'static) -> Result<(), String> {
        // Vosk requires native C++ libraries (libvosk.lib/dll on Windows).
        // Since we cannot guarantee this environment is set up during compilation,
        // we stub this out for v1.0, or ask the user to manually install libvosk.
        Err("Vosk STT requires libvosk.dll which is not bundled. Voice input disabled.".to_string())
    }

    pub fn stop_listening(&self) {
        let mut is_listening = self.is_listening.lock().unwrap();
        *is_listening = false;
    }
}
