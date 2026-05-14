pub mod pid;
pub mod signals;

use std::path::Path;

use anyhow::Result;
use tracing::{info, error};

use pid::PidFile;

/// Daemon lifecycle management.
pub struct Daemon {
    pid_file: PidFile,
}

impl Daemon {
    pub fn new(pid_path: &str) -> Self {
        Self {
            pid_file: PidFile::new(pid_path),
        }
    }

    pub fn start(&self) -> Result<()> {
        self.pid_file.write()?;
        info!("Daemon started (PID: {})", std::process::id());
        Ok(())
    }

    pub fn stop(&self) {
        if let Err(e) = self.pid_file.remove() {
            error!("Failed to remove PID file: {e}");
        }
        info!("Daemon stopped");
    }

    pub fn is_running(pid_path: &str) -> bool {
        Path::new(pid_path).exists()
    }
}
