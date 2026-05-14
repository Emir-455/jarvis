use std::path::PathBuf;

use anyhow::Result;

pub struct PidFile {
    path: PathBuf,
}

impl PidFile {
    pub fn new(path: &str) -> Self {
        Self {
            path: PathBuf::from(path),
        }
    }

    pub fn write(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, std::process::id().to_string())?;
        Ok(())
    }

    pub fn remove(&self) -> Result<()> {
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }
        Ok(())
    }

    pub fn read(&self) -> Result<u32> {
        let content = std::fs::read_to_string(&self.path)?;
        Ok(content.trim().parse()?)
    }
}
