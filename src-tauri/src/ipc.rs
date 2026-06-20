//! Hyde Neural Engine IPC — Bidirectional JSON-RPC communication with the Python engine.
//! 
//! The Rust side sends typed/chat commands to the Python engine via stdin,
//! and receives structured intent responses via stdout.
//! Voice events and JSON-RPC responses share stdout but are differentiated
//! by the `[JSON]` prefix.

use std::io::Write;
use std::process::{Child, ChildStdin, Stdio};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A request sent from Rust to Python via stdin
#[derive(Serialize)]
pub struct IpcRequest {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

/// The result field of a JSON-RPC response from Python
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IpcResult {
    pub intent: Option<String>,
    pub confidence: Option<f64>,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    pub success: Option<bool>,
    pub response: Option<String>,
    pub action_taken: Option<String>,
    pub requires_ai: Option<bool>,
    pub ai_type: Option<String>,
    pub ai_query: Option<String>,
    pub rust_action: Option<HashMap<String, String>>,
    pub status: Option<String>,
    pub context: Option<HashMap<String, String>>,
}

/// A full JSON-RPC response from Python
#[derive(Debug, Deserialize)]
pub struct IpcResponse {
    pub id: String,
    pub result: Option<IpcResult>,
    pub error: Option<String>,
}

/// Manages bidirectional communication with the Python Hyde Engine
pub struct EngineIpc {
    stdin: Arc<Mutex<ChildStdin>>,
    pending: Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<IpcResult>>>>,
    request_counter: Arc<Mutex<u64>>,
}

impl EngineIpc {
    /// Start the Python engine and set up IPC channels.
    /// Returns (EngineIpc, Child) — caller must handle stdout reader separately.
    pub fn spawn() -> Result<(Self, Child), String> {
        #[cfg(target_os = "windows")]
        use std::os::windows::process::CommandExt;

        let mut child = std::process::Command::new("python")
            .arg("-u")
            .arg("-m")
            .arg("hyde_engine.main")
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .map_err(|e| format!("Failed to spawn Hyde Engine: {}", e))?;

        let stdin = child.stdin.take()
            .ok_or("Failed to capture engine stdin")?;

        let ipc = EngineIpc {
            stdin: Arc::new(Mutex::new(stdin)),
            pending: Arc::new(Mutex::new(HashMap::new())),
            request_counter: Arc::new(Mutex::new(0)),
        };

        Ok((ipc, child))
    }

    /// Send a classification request and wait for the response.
    pub async fn classify(&self, text: &str) -> Result<IpcResult, String> {
        let id = self.next_id();
        let request = IpcRequest {
            id: id.clone(),
            method: "classify".to_string(),
            params: serde_json::json!({
                "text": text,
                "source": "chat"
            }),
        };

        self.send_and_wait(id, request).await
    }

    /// Ping the engine to check if it's alive.
    pub async fn ping(&self) -> Result<IpcResult, String> {
        let id = self.next_id();
        let request = IpcRequest {
            id: id.clone(),
            method: "ping".to_string(),
            params: serde_json::json!({}),
        };

        self.send_and_wait(id, request).await
    }

    /// Send a request and register a oneshot channel to receive the response.
    async fn send_and_wait(&self, id: String, request: IpcRequest) -> Result<IpcResult, String> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        // Register the pending request
        {
            let mut pending = self.pending.lock().map_err(|e| e.to_string())?;
            pending.insert(id.clone(), tx);
        }

        // Send the request via stdin
        {
            let json_line = serde_json::to_string(&request)
                .map_err(|e| format!("JSON serialize error: {}", e))?;
            let mut stdin = self.stdin.lock().map_err(|e| e.to_string())?;
            writeln!(stdin, "{}", json_line)
                .map_err(|e| format!("Failed to write to engine stdin: {}", e))?;
            stdin.flush()
                .map_err(|e| format!("Failed to flush engine stdin: {}", e))?;
        }

        // Wait for the response (with timeout)
        match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(_)) => Err("IPC channel closed unexpectedly".to_string()),
            Err(_) => {
                // Clean up the pending request
                let mut pending = self.pending.lock().map_err(|e| e.to_string())?;
                pending.remove(&id);
                Err("IPC request timed out (10s)".to_string())
            }
        }
    }

    /// Called by the stdout reader thread when a JSON response is received.
    pub fn handle_response(&self, response: IpcResponse) {
        if let Some(result) = response.result {
            let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(tx) = pending.remove(&response.id) {
                let _ = tx.send(result);
            }
        } else if let Some(error) = response.error {
            let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(tx) = pending.remove(&response.id) {
                let _ = tx.send(IpcResult {
                    intent: None,
                    confidence: None,
                    parameters: None,
                    success: Some(false),
                    response: Some(error),
                    action_taken: None,
                    requires_ai: None,
                    ai_type: None,
                    ai_query: None,
                    rust_action: None,
                    status: None,
                    context: None,
                });
            }
        }
    }

    fn next_id(&self) -> String {
        let mut counter = self.request_counter.lock().unwrap();
        *counter += 1;
        format!("req_{:06}", *counter)
    }
}
