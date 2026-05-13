use std::path::PathBuf;

use anyhow::Result;
use tracing::info;

/// Installer: creates necessary directories and validates environment.
pub struct Installer {
    root: PathBuf,
}

impl Installer {
    pub fn new(root: &str) -> Self {
        Self {
            root: PathBuf::from(root),
        }
    }

    pub fn setup(&self) -> Result<()> {
        let dirs = [
            "data/models",
            "data/downloads",
            "data/biometrics/faces",
            "data/offline_buffer",
            "logs/crashes",
            "marks/mark_i",
            "marks/_staging",
            "marks/_rollback",
            "shield/quarantine",
            "shield/antivirus/rules",
            "memory/vector/chroma",
            "voice/models",
            "config",
        ];

        for dir in &dirs {
            let path = self.root.join(dir);
            std::fs::create_dir_all(&path)?;
        }

        info!(root = %self.root.display(), "Directory structure created");
        Ok(())
    }
}
