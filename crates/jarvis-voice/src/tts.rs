use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use tracing::{info, warn};

use jarvis_common::config::TtsConfig;

/// Text-to-speech engine — Iron Man 2 J.A.R.V.I.S. voice.
///
/// Primary engine: XTTS v2 with the jarvis_ironman2 voice profile.
/// Fallback: pyttsx3 (system TTS) if XTTS unavailable.
pub struct TtsEngine {
    config: TtsConfig,
    ready: AtomicBool,
}

impl TtsEngine {
    pub fn new(config: &TtsConfig) -> Self {
        Self {
            config: config.clone(),
            ready: AtomicBool::new(false),
        }
    }

    /// Warm up the TTS engine (load voice model).
    pub async fn warmup(&self) -> Result<()> {
        info!(
            engine = %self.config.engine,
            voice_profile = %self.config.voice_profile,
            "Warming up TTS engine"
        );

        if self.config.voice_profile.is_empty() {
            warn!("No voice profile configured — TTS in simulation mode");
        }

        // NOTE: Production loads XTTS v2 model with the Iron Man 2 JARVIS
        // voice reference file (jarvis_ironman2.wav / .m4a).
        //
        // let model = XttsModel::load("tts_models/multilingual/multi-dataset/xtts_v2");
        // model.set_speaker_wav(&self.config.voice_profile);

        self.ready.store(true, Ordering::SeqCst);
        info!("TTS engine ready");
        Ok(())
    }

    /// Speak the given text using the JARVIS voice.
    pub async fn speak(&self, text: &str) -> Result<()> {
        if !self.is_ready() {
            warn!("TTS not ready, skipping speech");
            return Ok(());
        }

        info!(text_len = text.len(), "TTS speaking");

        // Production:
        // 1. Generate WAV via XTTS v2 with JARVIS voice
        // 2. Play through audio output device
        // 3. Wait for playback to finish

        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::SeqCst)
    }
}
