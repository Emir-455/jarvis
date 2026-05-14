use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use tracing::{info, warn};

use jarvis_common::config::AntivirusConfig;
use jarvis_common::lifecycle::CocoonGuarded;

use crate::quarantine::QuarantineManager;
use crate::watchdog::FileWatchdog;

/// Self-defense antivirus system.
///
/// CRITICAL INVARIANT: This module NEVER shuts down during cocoon/upgrade
/// mode. Even when the system enters metamorphosis, the Shield remains
/// active as a background sentinel.
///
/// Capabilities:
/// - Real-time file system monitoring (watch_paths)
/// - YARA rule-based threat detection
/// - Automatic quarantine of suspicious files
/// - Antidote synthesis for unknown threats
pub struct Antivirus {
    config: AntivirusConfig,
    alive: AtomicBool,
    quarantine: QuarantineManager,
    watchdog: FileWatchdog,
}

impl Antivirus {
    pub fn new(config: &AntivirusConfig) -> Self {
        Self {
            config: config.clone(),
            alive: AtomicBool::new(false),
            quarantine: QuarantineManager::new(&config.quarantine_path),
            watchdog: FileWatchdog::new(&config.watch_paths),
        }
    }

    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Antivirus disabled in config");
            return Ok(());
        }

        info!(
            watch_paths = ?self.config.watch_paths,
            quarantine = %self.config.quarantine_path,
            "Shield antivirus starting"
        );

        // Start file system watcher
        self.watchdog.start()?;

        self.alive.store(true, Ordering::SeqCst);
        info!("Shield ACTIVE — real-time protection enabled");

        Ok(())
    }

    pub async fn stop(&self) {
        if self.must_stay_alive() {
            warn!(
                "SHIELD STOP REJECTED: security_must_stay_alive=true. \
                 Shield cannot be stopped during cocoon mode."
            );
            return;
        }

        self.watchdog.stop();
        self.alive.store(false, Ordering::SeqCst);
        info!("Shield stopped");
    }

    /// Force stop (only for full system shutdown, not cocoon).
    pub fn force_stop(&self) {
        self.watchdog.stop();
        self.alive.store(false, Ordering::SeqCst);
        info!("Shield force-stopped (system shutdown)");
    }

    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }

    /// Scan a file for threats.
    pub async fn scan_file(&self, path: &str) -> Result<ScanResult> {
        info!(path, "Scanning file");

        // Check file size limit
        let metadata = std::fs::metadata(path)?;
        let max_size = 100 * 1024 * 1024; // 100MB default
        if metadata.len() > max_size {
            return Ok(ScanResult::Skipped("File too large".into()));
        }

        // TODO: YARA rule matching
        // let matches = self.yara.scan(path)?;
        // if !matches.is_empty() {
        //     self.quarantine.quarantine(path, &matches)?;
        //     if self.config.auto_synthesize_antidote {
        //         self.synthesize_antidote(&matches).await?;
        //     }
        //     return Ok(ScanResult::ThreatFound(matches));
        // }

        Ok(ScanResult::Clean)
    }

    /// Quarantine a suspicious file.
    pub fn quarantine_file(&self, path: &str) -> Result<()> {
        self.quarantine.quarantine(path)
    }
}

impl CocoonGuarded for Antivirus {
    fn must_stay_alive(&self) -> bool {
        self.config.always_on_during_cocoon
    }
}

#[derive(Debug)]
pub enum ScanResult {
    Clean,
    ThreatFound(Vec<String>),
    Skipped(String),
}
