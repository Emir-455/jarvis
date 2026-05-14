use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

/// HTTP client for Ollama local LLM server.
///
/// Ollama runs locally on the user's machine and provides GPU-accelerated
/// LLM inference via a simple HTTP API. This is the easiest way to get
/// interactive JARVIS responses without compiling llama-cpp-rs with CUDA.
///
/// Setup:
/// 1. Install Ollama: https://ollama.com
/// 2. Pull a model: `ollama pull llama3.1`
/// 3. Set `ollama_url` and `ollama_model` in config/llm.yaml
pub struct OllamaClient {
    url: String,
    model: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    options: ChatOptions,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatOptions {
    temperature: f32,
    top_p: f32,
    num_predict: i32,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

impl OllamaClient {
    pub fn new(url: &str, model: &str) -> Self {
        Self {
            url: url.to_string(),
            model: model.to_string(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.url.is_empty() && !self.model.is_empty()
    }

    /// Generate a response using Ollama's chat API with JARVIS personality.
    pub async fn generate(
        &self,
        system_prompt: &str,
        conversation_history: &[String],
        user_input: &str,
        rag_context: &[String],
    ) -> Result<String> {
        let mut messages = Vec::new();

        // System prompt (JARVIS personality)
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: system_prompt.to_string(),
        });

        // Add RAG context if available
        if !rag_context.is_empty() {
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: format!(
                    "Hafıza kayıtları (bağlam):\n{}",
                    rag_context.join("\n")
                ),
            });
        }

        // Add recent conversation history
        for turn in conversation_history {
            if let Some(content) = turn.strip_prefix("Emir: ") {
                messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: content.to_string(),
                });
            } else if let Some(content) = turn.strip_prefix("J.A.R.V.I.S.: ") {
                messages.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: content.to_string(),
                });
            }
        }

        // Current user message
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: user_input.to_string(),
        });

        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            options: ChatOptions {
                temperature: 0.6,
                top_p: 0.9,
                num_predict: 256,
            },
        };

        let api_url = format!("{}/api/chat", self.url.trim_end_matches('/'));

        info!(
            model = %self.model,
            input_len = user_input.len(),
            "Ollama'ya istek gönderiliyor"
        );

        let resp = self
            .client
            .post(&api_url)
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Ollama API hatası: {} — {}", status, body);
        }

        let chat_resp: ChatResponse = resp.json().await?;
        let response = chat_resp.message.content.trim().to_string();

        info!(
            response_len = response.len(),
            "Ollama yanıtı alındı"
        );

        Ok(response)
    }
}
