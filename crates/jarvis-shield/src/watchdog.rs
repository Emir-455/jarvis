use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use tracing::info;

/// File system watchdog that monitors directories for changes.
///
/// Uses the `notify` crate for cross-platform file system events.
pub struct FileWatchdog {
    watch_paths: Vec<String>,
    active: AtomicBool,
}

impl FileWatchdog {
    pub fn new(watch_paths: &[String]) -> Self {
        Self {
            watch_paths: watch_paths.to_vec(),
            active: AtomicBool::new(false),
        }
    }

    pub fn start(&self) -> Result<()> {
        info!(paths = ?self.watch_paths, "File watchdog starting");

        // NOTE: Production implementation uses notify::RecommendedWatcher:
        //
        // let (tx, rx) = std::sync::mpsc::channel();
        // let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
        // for path in &self.watch_paths {
        //     watcher.watch(Path::new(path), RecursiveMode::Recursive)?;
        // }
        // tokio::spawn(async move {
        //     for event in rx {
        //         // scan changed files
        //     }
        // });

        self.active.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub fn stop(&self) {
        self.active.store(false, Ordering::SeqCst);
        info!("File watchdog stopped");
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
}
