use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use tracing::info;

use jarvis_common::config::CocoonConfig;

pub struct PatchInfo {
    pub loc_changed: u64,
    pub files_affected: u64,
}

/// Manages patch application, validation, snapshot, and rollback.
pub struct PatchManager {
    project_root: PathBuf,
    snapshot_dirs: Vec<String>,
}

impl PatchManager {
    pub fn new(config: &CocoonConfig) -> Self {
        Self {
            project_root: PathBuf::from(&config.project_root),
            snapshot_dirs: vec![
                "core".into(),
                "cocoon".into(),
                "memory".into(),
                "shield".into(),
                "dev_tools".into(),
                "media".into(),
                "protocols".into(),
                "config".into(),
            ],
        }
    }

    /// Analyze a patch directory to determine scope.
    pub fn get_patch_info(&self, patch_dir: &Path) -> Result<PatchInfo> {
        let mut loc = 0u64;
        let mut files = 0u64;

        if patch_dir.is_dir() {
            for entry in walkdir(patch_dir) {
                if entry.is_file() {
                    files += 1;
                    if let Ok(content) = std::fs::read_to_string(&entry) {
                        loc += content.lines().count() as u64;
                    }
                }
            }
        }

        Ok(PatchInfo {
            loc_changed: loc,
            files_affected: files,
        })
    }

    /// Validate patch structure before application.
    pub fn validate(&self, patch_dir: &Path) -> Result<()> {
        if !patch_dir.exists() {
            bail!("Patch directory does not exist: {}", patch_dir.display());
        }
        if !patch_dir.is_dir() {
            bail!("Patch path is not a directory: {}", patch_dir.display());
        }
        Ok(())
    }

    /// Create a snapshot of current state for rollback.
    pub fn create_snapshot(&self, rollback_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(rollback_dir)?;

        for dir_name in &self.snapshot_dirs {
            let source = self.project_root.join(dir_name);
            if source.exists() {
                let dest = rollback_dir.join(dir_name);
                copy_dir_recursive(&source, &dest)?;
            }
        }

        info!(
            dest = %rollback_dir.display(),
            "Rollback snapshot created"
        );
        Ok(())
    }

    /// Apply a patch by copying files from patch dir to project root.
    pub fn apply(&self, patch_dir: &Path) -> Result<()> {
        for entry in walkdir(patch_dir) {
            if entry.is_file() {
                let relative = entry
                    .strip_prefix(patch_dir)
                    .unwrap_or(&entry);
                let dest = self.project_root.join(relative);

                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(&entry, &dest)?;
            }
        }

        info!(
            patch = %patch_dir.display(),
            "Patch applied"
        );
        Ok(())
    }

    /// Rollback to the snapshot.
    pub fn rollback(&self, rollback_dir: &Path) -> Result<()> {
        for dir_name in &self.snapshot_dirs {
            let source = rollback_dir.join(dir_name);
            if source.exists() {
                let dest = self.project_root.join(dir_name);
                if dest.exists() {
                    std::fs::remove_dir_all(&dest)?;
                }
                copy_dir_recursive(&source, &dest)?;
            }
        }

        info!("Rollback complete");
        Ok(())
    }
}

fn walkdir(path: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                results.extend(walkdir(&path));
            } else {
                results.push(path);
            }
        }
    }
    results
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
