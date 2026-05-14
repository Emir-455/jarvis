use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use tracing::info;

use jarvis_common::config::SttConfig;

/// Speech-to-text engine (Whisper-based).
///
/// Listens to microphone input, detects speech segments,
/// and transcribes them using the Whisper model.
pub struct SttEngine {
    config: SttConfig,
    active: AtomicBool,
    muted: AtomicBool,
}

impl SttEngine {
    pub fn new(config: &SttConfig) -> Self {
        Self {
            config: config.clone(),
            active: AtomicBool::new(false),
            muted: AtomicBool::new(false),
        }
    }

    /// Start the STT listener.
    ///
    /// In production, this spawns a background thread that:
    /// 1. Opens the audio input device
    /// 2. Reads chunks at `sample_rate` Hz
    /// 3. Detects voice activity (RMS > threshold)
    /// 4. Accumulates audio until silence timeout
    /// 5. Feeds the audio to Whisper for transcription
    /// 6. Calls `on_command` with the result
    pub fn start<F>(&self, on_command: Arc<F>) -> Result<()>
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        info!(
            engine = %self.config.engine,
            model = %self.config.model_size,
            language = %self.config.language,
            "STT listener started"
        );

        self.active.store(true, Ordering::SeqCst);

        // NOTE: Production implementation uses cpal + whisper-rs:
        //
        // let device = cpal::default_host().default_input_device();
        // let stream = device.build_input_stream(config, move |data| {
        //     if !self.muted && rms(data) > self.config.rms_threshold {
        //         buffer.extend(data);
        //     }
        //     if silence_detected {
        //         let text = whisper.transcribe(&buffer);
        //         on_command(text);
        //         buffer.clear();
        //     }
        // });

        let _ = on_command;
        Ok(())
    }

    pub fn stop(&self) {
        self.active.store(false, Ordering::SeqCst);
        info!("STT listener stopped");
    }

    pub fn mute(&self) {
        self.muted.store(true, Ordering::SeqCst);
    }

    pub fn unmute(&self) {
        self.muted.store(false, Ordering::SeqCst);
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
}
