use std::sync::Arc;

use anyhow::Result;
use tracing::info;

use jarvis_mind::engine::LlmEngine;
use jarvis_memory::rag::RagEngine;

/// Intent categories recognized by J.A.R.V.I.S.
#[derive(Debug)]
enum Intent {
    /// System commands: status, diagnostics, health
    SystemQuery,
    /// Security: threat scan, quarantine, shield status
    Security,
    /// Code & development: write, analyze, optimize
    Development,
    /// General conversation: JARVIS handles with personality
    Conversation,
}

/// Routes incoming commands (voice or WebSocket) to the appropriate handler.
///
/// Unlike a chatbot router, J.A.R.V.I.S. classifies intent first:
/// 1. Trim & sanitize input
/// 2. Check direct intents (fast path — no LLM needed)
/// 3. Classify intent category
/// 4. Retrieve relevant RAG context
/// 5. Build personality-aware prompt
/// 6. Generate response with JARVIS character
/// 7. Store interaction in memory
pub struct CommandRouter {
    llm: Arc<LlmEngine>,
    rag: Arc<RagEngine>,
}

impl CommandRouter {
    pub fn new(llm: Arc<LlmEngine>, rag: Arc<RagEngine>) -> Self {
        Self { llm, rag }
    }

    /// Process a user command with intent classification.
    pub async fn handle(&self, input: &str) -> Result<String> {
        let input = input.trim();
        if input.is_empty() {
            return Ok("Efendim, bir komut algılayamadım.".to_string());
        }

        // Fast path: check direct intents on raw input BEFORE RAG augmentation
        if let Some(response) = self.llm.handle_direct_intent(input) {
            let intent = classify_intent(input);
            info!(
                input_len = input.len(),
                intent = ?intent,
                "Doğrudan intent yanıtı"
            );
            let _ = self.rag.store(input, &response).await;
            return Ok(response);
        }

        let intent = classify_intent(input);
        info!(
            input_len = input.len(),
            intent = ?intent,
            "Komut işleniyor"
        );

        // RAG context retrieval
        let context = self.rag.query(input, 6).await.unwrap_or_default();

        // Build prompt with context
        let prompt = if context.is_empty() {
            input.to_string()
        } else {
            format!(
                "Hafıza kayıtları:\n{}\n\nEmir'in komutu: {}",
                context.join("\n"),
                input
            )
        };

        // Generate response via LLM (with JARVIS personality)
        let response = self.llm.generate(&prompt).await?;

        // Store interaction in memory for future context
        let _ = self.rag.store(input, &response).await;

        Ok(response)
    }
}

/// Classify user input into intent categories.
fn classify_intent(input: &str) -> Intent {
    let lower = input.to_lowercase();

    let security_keywords = [
        "güvenlik", "tehdit", "virüs", "tarama", "scan", "shield",
        "karantina", "quarantine", "antivirüs", "saldırı", "threat",
    ];

    let system_keywords = [
        "durum", "status", "sistem", "cpu", "ram", "gpu", "disk",
        "performans", "sağlık", "health", "uptime", "sıcaklık",
    ];

    let dev_keywords = [
        "kod", "code", "yaz", "write", "analiz", "analyze", "optimize",
        "debug", "derle", "build", "compile", "test", "deploy",
        "roblox", "luau", "unity", "godot", "unreal", "react",
        "program", "fonksiyon", "function", "class", "struct",
    ];

    if security_keywords.iter().any(|k| lower.contains(k)) {
        Intent::Security
    } else if system_keywords.iter().any(|k| lower.contains(k)) {
        Intent::SystemQuery
    } else if dev_keywords.iter().any(|k| lower.contains(k)) {
        Intent::Development
    } else {
        Intent::Conversation
    }
}
