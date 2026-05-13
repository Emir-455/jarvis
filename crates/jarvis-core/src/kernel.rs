use std::sync::Arc;

use anyhow::Result;
use tracing::info;

use jarvis_cocoon::manager::CocoonManager;
use jarvis_common::config::JarvisConfig;
use jarvis_common::events::EventBus;
use jarvis_memory::rag::RagEngine;
use jarvis_mind::engine::LlmEngine;
use jarvis_shield::antivirus::Antivirus;
use jarvis_voice::manager::VoiceManager;

use crate::router::CommandRouter;
use crate::state::SystemState;

/// Central kernel that owns all subsystems and orchestrates the boot sequence.
pub struct JarvisKernel {
    pub state: Arc<SystemState>,
    pub event_bus: Arc<EventBus>,
    pub config: JarvisConfig,

    pub llm: Arc<LlmEngine>,
    pub rag: Arc<RagEngine>,
    pub antivirus: Arc<Antivirus>,
    pub voice: Option<Arc<VoiceManager>>,
    pub cocoon: Arc<CocoonManager>,
    pub router: Arc<CommandRouter>,
}

impl JarvisKernel {
    pub fn new(config: JarvisConfig) -> Self {
        let state = Arc::new(SystemState::new(config.system.version_mark));
        let event_bus = Arc::new(EventBus::new(256));

        let llm = Arc::new(LlmEngine::new(&config.llm));
        let rag = Arc::new(RagEngine::new(&config.memory));
        let antivirus = Arc::new(Antivirus::new(&config.security.antivirus));
        let cocoon = Arc::new(CocoonManager::new(&config.cocoon, Arc::clone(&antivirus)));
        let router = Arc::new(CommandRouter::new(Arc::clone(&llm), Arc::clone(&rag)));

        info!(
            mark = state.mark_version.0,
            "Kernel created: {}",
            state.mark_version
        );

        Self {
            state,
            event_bus,
            config,
            llm,
            rag,
            antivirus,
            voice: None,
            cocoon,
            router,
        }
    }

    pub fn attach_voice(&mut self, voice: Arc<VoiceManager>) {
        self.voice = Some(voice);
    }

    /// Full boot sequence matching the Python daemon_run flow.
    pub async fn boot(&mut self) -> Result<()> {
        // e) Antivirus start
        self.antivirus.start().await?;
        info!("Shield ACTIVE");

        // f) Memory connect
        self.rag.connect().await?;
        info!("Memory READY");

        // g) LLM load
        self.llm.load().await?;
        info!(
            "LLM {}",
            if self.llm.is_loaded() { "READY" } else { "OFFLINE" }
        );

        // Mark as booted
        self.state.set_booted();
        info!("Kernel boot complete");

        Ok(())
    }

    /// Graceful shutdown.
    pub async fn shutdown(&self) -> Result<()> {
        self.state.request_shutdown();
        info!("Shutdown initiated");

        if let Some(ref voice) = self.voice {
            voice.stop().await;
        }
        self.antivirus.stop().await;
        self.rag.disconnect().await;

        info!("All subsystems stopped");
        Ok(())
    }
}
