use std::sync::Arc;

use anyhow::Result;
use tracing::info;

use jarvis_mind::engine::LlmEngine;
use jarvis_memory::rag::RagEngine;

/// Routes incoming commands (voice or WebSocket) to the appropriate handler.
pub struct CommandRouter {
    llm: Arc<LlmEngine>,
    rag: Arc<RagEngine>,
}

impl CommandRouter {
    pub fn new(llm: Arc<LlmEngine>, rag: Arc<RagEngine>) -> Self {
        Self { llm, rag }
    }

    /// Process a user command: RAG context retrieval → LLM inference → response.
    pub async fn handle(&self, input: &str) -> Result<String> {
        info!(input_len = input.len(), "Processing command");

        // 1. Retrieve relevant context from RAG memory
        let context = self.rag.query(input, 6).await.unwrap_or_default();

        // 2. Build prompt with context
        let prompt = if context.is_empty() {
            input.to_string()
        } else {
            format!(
                "Bağlam bilgisi:\n{}\n\nKullanıcı komutu: {}",
                context.join("\n"),
                input
            )
        };

        // 3. Generate response via LLM
        let response = self.llm.generate(&prompt).await?;

        // 4. Store interaction in memory
        let _ = self
            .rag
            .store(input, &response)
            .await;

        Ok(response)
    }
}
