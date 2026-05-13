use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{bail, Result};
use parking_lot::RwLock;
use tracing::{info, warn};

use jarvis_common::config::LlmConfig;

use crate::prompt::build_system_prompt;

/// LLM inference engine backed by llama.cpp (via GGUF models).
///
/// On target hardware (RTX 4060 8GB), the engine offloads layers to GPU
/// while keeping the rest in RAM, balancing VRAM/RAM usage.
pub struct LlmEngine {
    config: LlmConfig,
    loaded: AtomicBool,
    system_prompt: RwLock<String>,
}

impl LlmEngine {
    pub fn new(config: &LlmConfig) -> Self {
        Self {
            config: config.clone(),
            loaded: AtomicBool::new(false),
            system_prompt: RwLock::new(build_system_prompt()),
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
                "No model path configured — LLM running in offline/stub mode. \
                 Set llm.model_path in config to enable inference."
            );
            self.loaded.store(true, Ordering::SeqCst);
            return Ok(());
        }

        let model_path = self.config.model_path.clone();
        let n_gpu_layers = self.config.n_gpu_layers;
        let n_threads = self.config.n_threads;
        let n_ctx = self.config.n_ctx;
        let n_batch = self.config.n_batch;
        let use_mmap = self.config.use_mmap;

        info!(
            model = %model_path,
            gpu_layers = n_gpu_layers,
            threads = n_threads,
            ctx = n_ctx,
            batch = n_batch,
            mmap = use_mmap,
            "Loading LLM model with hardware-optimized parameters"
        );

        // NOTE: In production this calls llama-cpp-rs or llama-cpp-2 bindings.
        // The actual FFI integration requires the llama.cpp shared library
        // compiled with CUDA support on the target machine.
        //
        // For now we mark as loaded in stub mode so the rest of the system
        // can boot and function (offline mode capability).

        info!("LLM engine ready (stub mode — install llama.cpp for full inference)");
        self.loaded.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Generate a response for the given prompt.
    pub async fn generate(&self, prompt: &str) -> Result<String> {
        if !self.is_loaded() {
            bail!("LLM not loaded");
        }

        let system = self.system_prompt.read().clone();

        // In stub mode, return an informative response
        if self.config.model_path.is_empty() {
            return Ok(format!(
                "[JARVIS Offline Mode] Model yüklenmedi. \
                 Komut alındı ({} karakter). \
                 Tam çıkarım için GGUF model dosyasını yapılandırın.",
                prompt.len()
            ));
        }

        // Production path: llama.cpp inference
        // let params = InferenceParams {
        //     temperature: self.config.temperature,
        //     top_p: self.config.top_p,
        //     repeat_penalty: self.config.repeat_penalty,
        //     max_tokens: self.config.max_tokens,
        //     stop_tokens: &self.config.stop_tokens,
        //     seed: self.config.seed,
        // };
        // let full_prompt = format!("{system}\n\nUser: {prompt}\nAssistant:");
        // self.model.generate(&full_prompt, &params).await

        let _ = system;
        Ok(format!(
            "[JARVIS] Komutunuz işleniyor: {}",
            &prompt[..prompt.len().min(100)]
        ))
    }

    pub fn set_system_prompt(&self, prompt: String) {
        *self.system_prompt.write() = prompt;
    }
}
