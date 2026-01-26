use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::process::Command;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::error::OllamaError;
use crate::models::OllamaConfig;

/// Ollama API client
pub struct OllamaClient {
    client: Client,
    config: OllamaConfig,
}

/// Chat message for Ollama chat API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".to_string(), content: content.into() }
    }
    
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".to_string(), content: content.into() }
    }
}

/// Request body for Ollama chat endpoint
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

/// Response from Ollama chat endpoint (streaming)
#[derive(Debug, Deserialize)]
struct ChatResponse {
    #[serde(default)]
    message: Option<ChatMessageResponse>,
    done: bool,
    #[serde(default)]
    total_duration: Option<u64>,
    #[serde(default)]
    eval_count: Option<u64>,
}

/// Message content in chat response
#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    #[serde(default)]
    role: String,
    #[serde(default)]
    content: String,
}

impl OllamaClient {
    /// Create a new Ollama client with the given configuration
    pub fn new(config: OllamaConfig) -> Result<Self, OllamaError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| OllamaError::RequestFailed(e.to_string()))?;

        Ok(Self { client, config })
    }

    /// Generate a response from Ollama using the chat API
    /// 
    /// - `system_prompt`: Optional system prompt to set model behavior for this request
    /// - `prompt`: The user prompt/message
    /// - `stream_to_stdout`: If true, stream response to stdout as received
    /// 
    /// Returns the complete response text.
    pub async fn generate(
        &self,
        system_prompt: Option<&str>,
        prompt: &str,
        stream_to_stdout: bool,
    ) -> Result<String, OllamaError> {
        let url = format!("{}/api/chat", self.config.url);
        
        // Build messages array with optional system prompt
        let mut messages = Vec::new();
        if let Some(sys) = system_prompt {
            messages.push(ChatMessage::system(sys));
        }
        messages.push(ChatMessage::user(prompt));
        
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            stream: true,
        };

        debug!("Sending chat request to Ollama: {}", url);
        debug!("Using model: {}, system_prompt: {}", 
            self.config.model, 
            system_prompt.map(|s| format!("{}...", &s[..s.len().min(50)])).unwrap_or_else(|| "none".to_string()));

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    OllamaError::ConnectionRefused(format!(
                        "Could not connect to Ollama at {}. Is Ollama running?",
                        self.config.url
                    ))
                } else if e.is_timeout() {
                    OllamaError::Timeout(self.config.timeout_seconds)
                } else {
                    OllamaError::from(e)
                }
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(OllamaError::HttpError { status, message });
        }

        let mut full_response = String::new();
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut generation_done = false;
        let mut token_count = 0usize;
        let mut last_progress_log = std::time::Instant::now();
        let mut last_token_time = std::time::Instant::now();
        let progress_interval = std::time::Duration::from_secs(10);
        let stall_timeout = std::time::Duration::from_secs(120); // 2 minute stall timeout

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| OllamaError::StreamError(e.to_string()))?;
            
            // Check for stall (no tokens for too long)
            if last_token_time.elapsed() > stall_timeout {
                warn!("Generation stalled - no tokens received for {:?}", stall_timeout);
                return Err(OllamaError::Timeout(stall_timeout.as_secs()));
            }
            
            // Ollama sends newline-delimited JSON
            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);
            
            // Process complete lines from the buffer
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].to_string();
                buffer = buffer[newline_pos + 1..].to_string();
                
                if line.is_empty() {
                    continue;
                }

                // Try to parse the JSON line - if it fails, log and continue
                let parsed: ChatResponse = match serde_json::from_str(&line) {
                    Ok(p) => p,
                    Err(e) => {
                        // If we already have content and this is just a parsing error on the final message,
                        // we can safely ignore it
                        if !full_response.is_empty() {
                            debug!("Ignoring parse error on final chunk: {}", e);
                            continue;
                        }
                        return Err(OllamaError::ParseError(format!("Failed to parse: {} - {}", 
                            if line.len() > 200 { &line[..200] } else { &line }, e)));
                    }
                };

                // Extract content from message field (chat API format)
                let content = parsed.message
                    .as_ref()
                    .map(|m| m.content.as_str())
                    .unwrap_or("");
                full_response.push_str(content);
                token_count += 1;
                last_token_time = std::time::Instant::now();

                // Progress logging for non-streaming mode
                if !stream_to_stdout && last_progress_log.elapsed() > progress_interval {
                    info!("Generation in progress: {} tokens, {} chars so far...", 
                        token_count, full_response.len());
                    last_progress_log = std::time::Instant::now();
                }

                if stream_to_stdout {
                    print!("{}", content);
                    io::stdout().flush().ok();
                }

                if parsed.done {
                    generation_done = true;
                    if stream_to_stdout {
                        println!(); // Final newline
                    }
                    if let Some(duration) = parsed.total_duration {
                        debug!("Generation completed in {}ms", duration / 1_000_000);
                    }
                    if let Some(count) = parsed.eval_count {
                        debug!("Tokens generated: {}", count);
                    }
                    break;
                }
            }
            
            // Break out of outer loop if generation is done
            if generation_done {
                break;
            }
        }

        info!("Generated {} characters", full_response.len());
        Ok(full_response)
    }

    /// Check if Ollama is reachable
    pub async fn health_check(&self) -> Result<bool, OllamaError> {
        let url = format!("{}/api/tags", self.config.url);
        
        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    OllamaError::ConnectionRefused(format!(
                        "Could not connect to Ollama at {}",
                        self.config.url
                    ))
                } else {
                    OllamaError::from(e)
                }
            })?;

        Ok(response.status().is_success())
    }

    /// Try to start Ollama if it's not running
    /// Returns true if Ollama was started or is already running
    pub async fn ensure_running(&self) -> Result<bool, OllamaError> {
        // First check if already running
        match self.health_check().await {
            Ok(true) => {
                debug!("Ollama is already running");
                return Ok(true);
            }
            Ok(false) => {
                warn!("Ollama health check returned false");
            }
            Err(OllamaError::ConnectionRefused(_)) => {
                info!("Ollama not running, attempting to start...");
            }
            Err(e) => {
                return Err(e);
            }
        }

        // Try to start Ollama
        if let Err(e) = Self::start_ollama() {
            warn!("Failed to start Ollama: {}", e);
            return Err(OllamaError::ConnectionRefused(format!(
                "Could not start Ollama: {}. Please start it manually with 'ollama serve'",
                e
            )));
        }

        // Wait for Ollama to be ready (up to 30 seconds)
        info!("Waiting for Ollama to start...");
        for i in 0..30 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            
            if let Ok(true) = self.health_check().await {
                info!("Ollama started successfully after {}s", i + 1);
                return Ok(true);
            }
            
            if i % 5 == 4 {
                debug!("Still waiting for Ollama... ({}s)", i + 1);
            }
        }

        Err(OllamaError::ConnectionRefused(
            "Ollama started but did not become ready within 30 seconds".to_string()
        ))
    }

    /// Start Ollama serve in the background
    fn start_ollama() -> Result<(), String> {
        // Check if ollama command exists
        let which_result = Command::new("which")
            .arg("ollama")
            .output();

        match which_result {
            Ok(output) if output.status.success() => {}
            _ => {
                return Err("Ollama not found in PATH. Please install Ollama first.".to_string());
            }
        }

        // Start ollama serve in the background
        // Using different approaches based on platform
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            use std::process::Stdio;

            // Fork a background process
            let result = Command::new("ollama")
                .arg("serve")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .process_group(0) // Detach from parent process group
                .spawn();

            match result {
                Ok(child) => {
                    info!("Started Ollama serve (pid: {})", child.id());
                    Ok(())
                }
                Err(e) => Err(format!("Failed to spawn Ollama: {}", e)),
            }
        }

        #[cfg(not(unix))]
        {
            use std::process::Stdio;

            let result = Command::new("ollama")
                .arg("serve")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();

            match result {
                Ok(child) => {
                    info!("Started Ollama serve (pid: {})", child.id());
                    Ok(())
                }
                Err(e) => Err(format!("Failed to spawn Ollama: {}", e)),
            }
        }
    }

    /// Check if the specified model is available
    pub async fn check_model(&self) -> Result<bool, OllamaError> {
        let url = format!("{}/api/tags", self.config.url);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(false);
        }

        #[derive(Deserialize)]
        struct TagsResponse {
            models: Vec<ModelInfo>,
        }

        #[derive(Deserialize)]
        struct ModelInfo {
            name: String,
        }

        let tags: TagsResponse = response
            .json()
            .await
            .map_err(|e| OllamaError::ParseError(e.to_string()))?;

        let model_name = &self.config.model;
        let found = tags.models.iter().any(|m| {
            m.name == *model_name || m.name.starts_with(&format!("{}:", model_name))
        });

        if !found {
            warn!(
                "Model '{}' not found. Available models: {:?}",
                model_name,
                tags.models.iter().map(|m| &m.name).collect::<Vec<_>>()
            );
        }

        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_constructors() {
        let sys = ChatMessage::system("You are helpful");
        assert_eq!(sys.role, "system");
        assert_eq!(sys.content, "You are helpful");
        
        let user = ChatMessage::user("Hello");
        assert_eq!(user.role, "user");
        assert_eq!(user.content, "Hello");
    }

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatRequest {
            model: "qwen3".to_string(),
            messages: vec![
                ChatMessage::system("Be helpful"),
                ChatMessage::user("Hello"),
            ],
            stream: true,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"qwen3\""));
        assert!(json.contains("\"stream\":true"));
        assert!(json.contains("\"role\":\"system\""));
        assert!(json.contains("\"role\":\"user\""));
    }

    #[test]
    fn test_chat_response_deserialization() {
        let json = r#"{"message":{"role":"assistant","content":"Hello"},"done":false}"#;
        let response: ChatResponse = serde_json::from_str(json).unwrap();
        assert!(!response.done);
        assert_eq!(response.message.as_ref().unwrap().content, "Hello");
        assert_eq!(response.message.as_ref().unwrap().role, "assistant");
    }

    #[test]
    fn test_chat_response_done() {
        let json = r#"{"message":{"role":"assistant","content":"!"},"done":true,"total_duration":1000000000,"eval_count":10}"#;
        let response: ChatResponse = serde_json::from_str(json).unwrap();
        assert!(response.done);
        assert_eq!(response.total_duration, Some(1000000000));
        assert_eq!(response.eval_count, Some(10));
    }

    #[test]
    fn test_chat_response_empty_message() {
        // Final "done" message sometimes has no message content
        let json = r#"{"done":true,"total_duration":1000000000}"#;
        let response: ChatResponse = serde_json::from_str(json).unwrap();
        assert!(response.done);
        assert!(response.message.is_none());
    }
}
