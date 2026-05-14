use tracing::info;

use jarvis_common::config::ScreenCaptureConfig;

/// Screen monitor: captures and analyzes the user's screen.
pub struct ScreenMonitor {
    config: ScreenCaptureConfig,
    active: bool,
}

impl ScreenMonitor {
    pub fn new(config: &ScreenCaptureConfig) -> Self {
        Self {
            config: config.clone(),
            active: false,
        }
    }

    pub fn start(&mut self) {
        info!(
            interval = self.config.interval_seconds,
            "Screen monitor started"
        );
        self.active = true;
    }

    pub fn stop(&mut self) {
        self.active = false;
        info!("Screen monitor stopped");
    }
}
