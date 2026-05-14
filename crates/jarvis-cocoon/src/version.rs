use std::path::PathBuf;

use anyhow::Result;
use tracing::info;

use jarvis_common::config::CocoonConfig;
use jarvis_common::types::MarkVersion;

/// Manages Mark version tracking (Mark I → Mark II → ...).
pub struct VersionManager {
    marks_root: PathBuf,
    current_mark: u32,
}

impl VersionManager {
    pub fn new(config: &CocoonConfig) -> Self {
        // Extract mark number from current_mark_dir or default to 1
        let mark = Self::parse_mark_from_dir(&config.current_mark_dir);

        Self {
            marks_root: PathBuf::from(&config.marks_root),
            current_mark: mark,
        }
    }

    pub fn init(&self) -> Result<()> {
        std::fs::create_dir_all(&self.marks_root)?;
        let mark_dir = self.mark_dir(self.current_mark);
        std::fs::create_dir_all(&mark_dir)?;
        Ok(())
    }

    pub fn current_mark(&self) -> u32 {
        self.current_mark
    }

    pub fn current_version(&self) -> MarkVersion {
        MarkVersion(self.current_mark)
    }

    /// Bump to the next Mark version.
    pub fn bump_mark(&mut self) -> Result<u32> {
        let old = self.current_mark;
        self.current_mark += 1;
        let new_dir = self.mark_dir(self.current_mark);
        std::fs::create_dir_all(&new_dir)?;

        info!(
            old_mark = old,
            new_mark = self.current_mark,
            "Mark version upgraded: {} → {}",
            MarkVersion(old),
            MarkVersion(self.current_mark)
        );

        Ok(self.current_mark)
    }

    fn mark_dir(&self, mark: u32) -> PathBuf {
        let roman = MarkVersion(mark).roman().to_lowercase();
        self.marks_root.join(format!("mark_{roman}"))
    }

    fn parse_mark_from_dir(dir: &str) -> u32 {
        // Try to extract from "mark_I", "mark_II", etc.
        let lower = dir.to_lowercase();
        if lower.contains("mark_i") && !lower.contains("mark_ii") {
            1
        } else if lower.contains("mark_ii") && !lower.contains("mark_iii") {
            2
        } else if lower.contains("mark_iii") {
            3
        } else {
            1
        }
    }
}
