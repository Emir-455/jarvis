use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Utc;
use tracing::{info, warn};

/// Manages the quarantine area for suspicious files.
pub struct QuarantineManager {
    quarantine_path: PathBuf,
}

impl QuarantineManager {
    pub fn new(path: &str) -> Self {
        Self {
            quarantine_path: PathBuf::from(path),
        }
    }

    pub fn init(&self) -> Result<()> {
        std::fs::create_dir_all(&self.quarantine_path)?;
        Ok(())
    }

    /// Move a suspicious file to quarantine.
    pub fn quarantine(&self, file_path: &str) -> Result<()> {
        let source = Path::new(file_path);
        if !source.exists() {
            warn!(path = file_path, "File not found for quarantine");
            return Ok(());
        }

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = source
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let quarantined_name = format!("{timestamp}_{filename}.quarantined");
        let dest = self.quarantine_path.join(&quarantined_name);

        std::fs::create_dir_all(&self.quarantine_path)?;
        std::fs::rename(source, &dest)?;

        info!(
            source = file_path,
            dest = %dest.display(),
            "File quarantined"
        );
        Ok(())
    }

    /// List quarantined files.
    pub fn list(&self) -> Result<Vec<PathBuf>> {
        if !self.quarantine_path.exists() {
            return Ok(Vec::new());
        }

        let entries: Vec<PathBuf> = std::fs::read_dir(&self.quarantine_path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();

        Ok(entries)
    }
}
