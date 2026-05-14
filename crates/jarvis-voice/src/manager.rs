use std::sync::Arc;

use anyhow::Result;
use tracing::info;

use jarvis_common::config::VoiceConfig;

use crate::stt::SttEngine;
use crate::tts::TtsEngine;
use crate::wake_word::WakeWordDetector;

/// Voice manager: coordinates STT, TTS, and wake word detection.
pub struct VoiceManager {
    pub stt: SttEngine,
    pub tts: TtsEngine,
    pub wake_word: WakeWordDetector,
    enabled: bool,
}

impl VoiceManager {
    pub fn new(config: &VoiceConfig) -> Self {
        Self {
            stt: SttEngine::new(&config.stt),
            tts: TtsEngine::new(&config.tts),
            wake_word: WakeWordDetector::new(&config.wake_word),
            enabled: config.enabled,
        }
    }

    pub async fn start<F>(&self, on_command: Arc<F>) -> Result<()>
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        if !self.enabled {
            info!("Voice system disabled in config");
            return Ok(());
        }

        self.tts.warmup().await?;
        self.stt.start(on_command)?;

        info!("Voice manager started");
        Ok(())
    }

    pub async fn stop(&self) {
        self.stt.stop();
        info!("Voice manager stopped");
    }

    pub async fn speak(&self, text: &str) -> Result<()> {
        self.tts.speak(text).await
    }

    pub fn is_ready(&self) -> bool {
        self.tts.is_ready() && self.stt.is_active()
    }
}
