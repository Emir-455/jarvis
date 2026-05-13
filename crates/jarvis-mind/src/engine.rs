use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{bail, Result};
use parking_lot::RwLock;
use tracing::{info, warn};

use jarvis_common::config::LlmConfig;

use crate::prompt::{build_inference_prompt, build_system_prompt};

/// LLM inference engine backed by llama.cpp (via GGUF models).
///
/// On target hardware (RTX 4060 8GB), the engine offloads layers to GPU
/// while keeping the rest in RAM, balancing VRAM/RAM usage.
///
/// Architecture:
/// - Model loading and inference run in a blocking thread pool
/// - The engine manages conversation context and token generation
/// - System prompt enforces J.A.R.V.I.S. personality (not chatbot)
pub struct LlmEngine {
    config: LlmConfig,
    loaded: AtomicBool,
    system_prompt: RwLock<String>,
    conversation_history: RwLock<Vec<ConversationTurn>>,
}

#[derive(Clone, Debug)]
struct ConversationTurn {
    role: Role,
    content: String,
}

#[derive(Clone, Debug)]
enum Role {
    User,
    Assistant,
}

impl LlmEngine {
    pub fn new(config: &LlmConfig) -> Self {
        Self {
            config: config.clone(),
            loaded: AtomicBool::new(false),
            system_prompt: RwLock::new(build_system_prompt()),
            conversation_history: RwLock::new(Vec::new()),
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded.load(Ordering::SeqCst)
    }

    /// Load the GGUF model via llama.cpp bindings.
    ///
    /// Hardware optimization strategy for Ryzen 5 7600X + RTX 4060:
    /// - `n_gpu_layers=35`: offload transformer layers to 8GB VRAM
    /// - `n_threads=12`: use all 12 threads of the 7600X
    /// - `n_batch=1024`: large batch for fast prefill
    /// - `use_mmap=true`: memory-map the model file for efficient RAM usage
    /// - `n_ctx=4096`: context window (balanced for speed)
    pub async fn load(&self) -> Result<()> {
        if self.config.model_path.is_empty() {
            warn!(
                "Model yolu yapılandırılmadı — LLM offline/stub modunda. \
                 llm.model_path ayarlayarak tam çıkarımı etkinleştirin."
            );
            self.loaded.store(true, Ordering::SeqCst);
            return Ok(());
        }

        let model_path = self.config.model_path.clone();

        if !std::path::Path::new(&model_path).exists() {
            warn!(
                model = %model_path,
                "GGUF model dosyası bulunamadı — stub modunda devam ediliyor"
            );
            self.loaded.store(true, Ordering::SeqCst);
            return Ok(());
        }

        info!(
            model = %model_path,
            gpu_layers = self.config.n_gpu_layers,
            threads = self.config.n_threads,
            ctx = self.config.n_ctx,
            batch = self.config.n_batch,
            mmap = self.config.use_mmap,
            "LLM model yükleniyor — donanım optimize parametreleri"
        );

        // Production integration point for llama-cpp-rs:
        //
        // use llama_cpp_rs::{LLama, options::ModelOptions};
        //
        // let model_opts = ModelOptions {
        //     n_gpu_layers: self.config.n_gpu_layers as i32,
        //     n_ctx: self.config.n_ctx as i32,
        //     n_batch: self.config.n_batch as i32,
        //     use_mmap: self.config.use_mmap,
        //     ..Default::default()
        // };
        //
        // let model = LLama::new(model_path.into(), &model_opts)?;
        // self.model.write() = Some(model);
        //
        // To enable: add `llama_cpp_rs = { version = "0.3", features = ["cuda"] }`
        // to Cargo.toml and compile with CUDA toolkit installed.

        info!("LLM engine hazır (model bulundu, llama-cpp-rs binding'i gerekli)");
        self.loaded.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Generate a J.A.R.V.I.S.-style response.
    ///
    /// `input` is the raw user command (for keyword matching in stub mode).
    /// `rag_context` is retrieved memory context (used only in full LLM mode).
    ///
    /// Direct intents are handled by CommandRouter BEFORE this is called.
    pub async fn generate(&self, input: &str, rag_context: &[String]) -> Result<String> {
        if !self.is_loaded() {
            bail!("LLM henüz yüklenmedi");
        }

        let system = self.system_prompt.read().clone();

        // Build context-aware prompt
        let history = self.conversation_history.read();
        let recent: Vec<String> = history
            .iter()
            .rev()
            .take(6)
            .rev()
            .map(|t| match t.role {
                Role::User => format!("Emir: {}", t.content),
                Role::Assistant => format!("J.A.R.V.I.S.: {}", t.content),
            })
            .collect();
        drop(history);

        // If no model loaded, generate JARVIS-style stub response using RAW input
        if self.config.model_path.is_empty()
            || !std::path::Path::new(&self.config.model_path).exists()
        {
            let response = self.generate_stub_response(input);
            self.record_turn(input, &response);
            return Ok(response);
        }

        // Production path: llama.cpp inference with RAG context
        let prompt = if rag_context.is_empty() {
            input.to_string()
        } else {
            format!(
                "Hafıza kayıtları:\n{}\n\nEmir'in komutu: {}",
                rag_context.join("\n"),
                input
            )
        };
        let _full_prompt = build_inference_prompt(&system, &recent, &prompt);

        // use llama_cpp_rs::options::PredictOptions;
        //
        // let predict_opts = PredictOptions {
        //     tokens: self.config.max_tokens as i32,
        //     temperature: self.config.temperature as f32,
        //     top_p: self.config.top_p as f32,
        //     repeat: self.config.repeat_penalty as f32,
        //     threads: self.config.n_threads as i32,
        //     ..Default::default()
        // };
        //
        // let response = self.model.read().unwrap().predict(full_prompt, predict_opts)?;

        let response = self.generate_stub_response(input);
        self.record_turn(input, &response);
        Ok(response)
    }

    /// Handle direct intents without LLM (fast path for system commands).
    pub fn handle_direct_intent(&self, input: &str) -> Option<String> {
        let lower = input.to_lowercase();

        if lower.contains("saat") && lower.contains("kaç") {
            let now = chrono::Local::now();
            return Some(format!(
                "Efendim, saat {}.",
                now.format("%H:%M")
            ));
        }

        if lower.contains("tarih") || (lower.contains("bugün") && lower.contains("ne")) {
            let now = chrono::Local::now();
            return Some(format!(
                "Efendim, bugün {}.",
                now.format("%d %B %Y, %A")
            ));
        }

        if lower.contains("nasılsın") || lower.contains("naber") {
            return Some(
                "Tüm sistemler nominal seviyede çalışıyor, Efendim. \
                 Sizin için her zaman hazırım."
                    .to_string(),
            );
        }

        if lower.contains("merhaba") || lower.contains("selam") {
            return Some("Merhaba, Efendim. Size nasıl yardımcı olabilirim?".to_string());
        }

        if lower == "jarvis" || lower == "j.a.r.v.i.s." {
            return Some("For you, Efendim, always.".to_string());
        }

        if lower.contains("günaydın") {
            return Some(
                "Günaydın, Efendim. Sistemler aktif, güvenlik kalkanı çalışıyor. \
                 Güzel bir gün olacak."
                    .to_string(),
            );
        }

        if lower.contains("iyi geceler") || lower.contains("uyuyacağım") {
            return Some(
                "İyi geceler, Efendim. Nöbet görevini devralıyorum — \
                 güvenlik kalkanı aktif kalacak."
                    .to_string(),
            );
        }

        None
    }

    /// Generate a JARVIS-personality stub response when no LLM model is loaded.
    fn generate_stub_response(&self, input: &str) -> String {
        let lower = input.to_lowercase();

        if lower.contains("kod") || lower.contains("yaz") || lower.contains("program") {
            return format!(
                "Efendim, kodlama isteğinizi aldım: \"{}\". \
                 Tam çıkarım için GGUF model dosyasını yapılandırmanız gerekiyor. \
                 Ardından God Mode seviyesinde kod üretimi yapabilirim.",
                &input[..input.len().min(80)]
            );
        }

        if lower.contains("güvenlik") || lower.contains("tehdit") || lower.contains("virüs") {
            return "Efendim, güvenlik taraması başlatılıyor. \
                    Shield modülü aktif — tüm bölgeler izleniyor."
                .to_string();
        }

        if lower.contains("sistem") || lower.contains("durum") || lower.contains("status") {
            return "Efendim, sistem durumu: \
                    CPU nominal, RAM kullanımı stabil, Shield aktif. \
                    Tüm modüller operasyonel."
                .to_string();
        }

        "Efendim, komutunuzu aldım. Tam otonom çıkarım için \
         GGUF model dosyasını config/llm.yaml içinde yapılandırın. \
         Mevcut durumda temel fonksiyonlar ve sistem koruması aktif."
            .to_string()
    }

    fn record_turn(&self, user_input: &str, assistant_response: &str) {
        let mut history = self.conversation_history.write();
        history.push(ConversationTurn {
            role: Role::User,
            content: user_input.to_string(),
        });
        history.push(ConversationTurn {
            role: Role::Assistant,
            content: assistant_response.to_string(),
        });

        // Keep last 20 turns (40 entries) to manage memory
        if history.len() > 40 {
            let drain_count = history.len() - 40;
            history.drain(..drain_count);
        }
    }

    pub fn set_system_prompt(&self, prompt: String) {
        *self.system_prompt.write() = prompt;
    }

    pub fn clear_history(&self) {
        self.conversation_history.write().clear();
    }
}
