use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use tracing::{error, info, warn};

use jarvis_common::config::CocoonConfig;
use jarvis_shield::antivirus::Antivirus;

use crate::patch::PatchManager;
use crate::version::VersionManager;

/// Metamorphosis Mode — Self-upgrade orchestrator.
///
/// Like a caterpillar entering its cocoon:
/// - Small updates → hot patch (no cocoon, module reload)
/// - Large updates → COCOON: non-essential modules sleep, SHIELD STAYS ACTIVE
/// - After cocoon → Mark version +1 (Mark I → Mark II → ...)
/// - Failed patches → automatic rollback to previous state
///
/// CRITICAL: Shield/Antivirus module NEVER stops during upgrade.
pub struct CocoonManager {
    config: CocoonConfig,
    antivirus: Arc<Antivirus>,
    patch_mgr: PatchManager,
    upgrade_in_progress: AtomicBool,
    staging_dir: PathBuf,
    rollback_dir: PathBuf,
    mark_threshold: u64,
    version_mgr: parking_lot::Mutex<VersionManager>,
}

impl CocoonManager {
    pub fn new(config: &CocoonConfig, antivirus: Arc<Antivirus>) -> Self {
        Self {
            config: config.clone(),
            antivirus,
            patch_mgr: PatchManager::new(config),
            upgrade_in_progress: AtomicBool::new(false),
            staging_dir: PathBuf::from(&config.staging_dir),
            rollback_dir: PathBuf::from(&config.rollback_dir),
            mark_threshold: config.mark_threshold_loc_changed,
            version_mgr: parking_lot::Mutex::new(VersionManager::new(config)),
        }
    }

    pub async fn init(&self) -> Result<()> {
        std::fs::create_dir_all(&self.staging_dir)?;
        std::fs::create_dir_all(&self.rollback_dir)?;
        self.version_mgr.lock().init()?;

        let mark = self.version_mgr.lock().current_mark();
        info!(
            mark,
            staging = %self.staging_dir.display(),
            "CocoonManager ready"
        );
        Ok(())
    }

    /// Poll staging directory for pending patches.
    pub async fn poll(&self) -> Result<()> {
        if self.upgrade_in_progress.load(Ordering::SeqCst) {
            return Ok(());
        }

        let patches = self.find_pending_patches();
        if patches.is_empty() {
            return Ok(());
        }

        for patch_dir in patches {
            if !self.config.auto_upgrade_enabled {
                info!(path = %patch_dir.display(), "Auto-upgrade disabled, patch waiting");
                continue;
            }
            let _ = self.trigger_upgrade(&patch_dir).await;
        }

        Ok(())
    }

    fn find_pending_patches(&self) -> Vec<PathBuf> {
        if !self.staging_dir.exists() {
            return Vec::new();
        }

        let mut patches = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.staging_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && !path.file_name().unwrap_or_default().to_string_lossy().starts_with('.') {
                    patches.push(path);
                }
            }
        }

        patches.sort();
        patches
    }

    /// Trigger an upgrade. LOC determines hot-patch vs cocoon mode.
    pub async fn trigger_upgrade(&self, patch_dir: &Path) -> Result<bool> {
        self.upgrade_in_progress.store(true, Ordering::SeqCst);

        let result = self.do_upgrade(patch_dir).await;

        self.upgrade_in_progress.store(false, Ordering::SeqCst);
        result
    }

    async fn do_upgrade(&self, patch_dir: &Path) -> Result<bool> {
        let info = self.patch_mgr.get_patch_info(patch_dir)?;
        let major = info.loc_changed >= self.mark_threshold;

        info!(
            loc = info.loc_changed,
            files = info.files_affected,
            major,
            path = %patch_dir.display(),
            "=== UPGRADE DETECTED ==="
        );

        // Validate patch
        if let Err(e) = self.patch_mgr.validate(patch_dir) {
            error!("Patch validation failed: {e}");
            return Ok(false);
        }

        let success = if major {
            self.cocoon_upgrade(patch_dir).await?
        } else {
            self.hot_patch(patch_dir).await?
        };

        if success {
            info!("=== UPGRADE COMPLETE ===");
        } else {
            warn!("=== UPGRADE FAILED — rolling back ===");
        }

        Ok(success)
    }

    /// Full cocoon upgrade (major version bump).
    async fn cocoon_upgrade(&self, patch_dir: &Path) -> Result<bool> {
        info!("Entering COCOON mode...");

        // CRITICAL: Verify Shield is alive before proceeding
        assert!(
            self.antivirus.is_alive(),
            "SHIELD must be alive before entering cocoon"
        );

        // 1. Create rollback snapshot
        self.patch_mgr.create_snapshot(&self.rollback_dir)?;

        // 2. Enter cocoon (non-essential modules would sleep here)
        info!("Cocoon phase: non-essential modules sleeping");

        // 3. Verify Shield is STILL alive (double-check)
        if !self.antivirus.is_alive() {
            error!("SHIELD went down during cocoon prep — ABORTING");
            return Ok(false);
        }

        // 4. Apply patch
        match self.patch_mgr.apply(patch_dir) {
            Ok(()) => {
                // 5. Bump Mark version
                let new_mark = self.version_mgr.lock().bump_mark()?;
                info!(new_mark, "Mark version upgraded");

                // 6. Exit cocoon
                info!("Exiting cocoon mode — all modules waking up");
                Ok(true)
            }
            Err(e) => {
                error!("Patch apply failed: {e} — rolling back");
                self.patch_mgr.rollback(&self.rollback_dir)?;
                Ok(false)
            }
        }
    }

    /// Hot patch (minor update, no cocoon).
    async fn hot_patch(&self, patch_dir: &Path) -> Result<bool> {
        info!("Applying hot patch...");

        self.patch_mgr.create_snapshot(&self.rollback_dir)?;

        match self.patch_mgr.apply(patch_dir) {
            Ok(()) => {
                info!("Hot patch applied successfully");
                Ok(true)
            }
            Err(e) => {
                error!("Hot patch failed: {e} — rolling back");
                self.patch_mgr.rollback(&self.rollback_dir)?;
                Ok(false)
            }
        }
    }
}
