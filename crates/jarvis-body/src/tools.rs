use std::process::Command;

use anyhow::Result;
use tracing::info;

/// System tools: file operations, process management, and OS interaction.
pub struct SystemTools;

impl SystemTools {
    /// Execute a shell command and return its output.
    pub fn exec(cmd: &str) -> Result<String> {
        info!(cmd, "Executing system command");

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", cmd]).output()?
        } else {
            Command::new("sh").args(["-c", cmd]).output()?
        };

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
