use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use tokio::sync::RwLock;
use tracing::{info, warn};

use jarvis_common::config::MemoryConfig;

use crate::drive::DriveClient;
use crate::vector::VectorStore;

/// Hybrid RAG engine: local vector cache + Google Drive long-term memory.
///
/// Memory hierarchy:
/// 1. VectorStore (local) — fast semantic search via embeddings
/// 2. DriveClient (remote) — 5TB Google Drive as persistent brain
/// 3. Offline buffer — JSONL buffer for when Drive is unreachable
pub struct RagEngine {
    config: MemoryConfig,
    vector: RwLock<VectorStore>,
    drive: RwLock<DriveClient>,
    connected: AtomicBool,
}

impl RagEngine {
    pub fn new(config: &MemoryConfig) -> Self {
        Self {
            config: config.clone(),
            vector: RwLock::new(VectorStore::new(&config.vector_cache)),
            drive: RwLock::new(DriveClient::new(&config.drive)),
            connected: AtomicBool::new(false),
        }
    }

    pub async fn connect(&self) -> Result<()> {
        info!("Connecting memory systems...");

        self.vector.write().await.init()?;

        match self.drive.write().await.connect().await {
            Ok(()) => {
                info!(
                    capacity_tb = self.config.drive.capacity_tb,
                    "Google Drive connected"
                );
            }
            Err(e) => {
                warn!(
                    "Google Drive unavailable: {e}. Operating in offline mode \
                     with local vector cache only."
                );
            }
        }

        self.connected.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub async fn disconnect(&self) {
        if let Err(e) = self.vector.write().await.persist() {
            warn!("Failed to persist vector store: {e}");
        }
        self.drive.write().await.disconnect();
        self.connected.store(false, Ordering::SeqCst);
        info!("Memory systems disconnected");
    }

    /// Query the RAG system for relevant context.
    pub async fn query(&self, input: &str, top_k: usize) -> Result<Vec<String>> {
        let local_results = self.vector.read().await.search(input, top_k);

        if !local_results.is_empty() {
            return Ok(local_results);
        }

        let drive = self.drive.read().await;
        if drive.is_connected() {
            return drive.search(input, top_k).await;
        }

        Ok(Vec::new())
    }

    /// Store an interaction in memory.
    pub async fn store(&self, query: &str, response: &str) -> Result<()> {
        let entry = format!("Q: {query}\nA: {response}");

        self.vector.write().await.add(&entry)?;

        let drive = self.drive.read().await;
        if drive.is_connected() {
            let _ = drive.append_memory(&entry).await;
        } else {
            drop(drive);
            self.buffer_offline(&entry)?;
        }

        Ok(())
    }

    fn buffer_offline(&self, entry: &str) -> Result<()> {
        let path = &self.config.drive.offline_buffer_path;
        if path.is_empty() {
            return Ok(());
        }

        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        let record = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "entry": entry,
        });
        writeln!(file, "{}", serde_json::to_string(&record)?)?;
        Ok(())
    }
}
