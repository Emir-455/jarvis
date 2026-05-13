use std::collections::VecDeque;

use anyhow::Result;
use tracing::info;

use jarvis_common::config::VectorCacheConfig;

/// Local vector store for fast semantic search.
///
/// In production this wraps ChromaDB or a Rust-native vector DB.
/// The stub implementation uses simple text matching as a placeholder.
pub struct VectorStore {
    config: VectorCacheConfig,
    entries: VecDeque<String>,
    max_entries: usize,
}

impl VectorStore {
    pub fn new(config: &VectorCacheConfig) -> Self {
        Self {
            config: config.clone(),
            entries: VecDeque::new(),
            max_entries: 5000,
        }
    }

    pub fn init(&mut self) -> Result<()> {
        info!(
            collection = %self.config.collection_name,
            embed_model = %self.config.embed_model,
            "Vector store initialized (stub mode)"
        );
        // Production: load persisted embeddings from disk
        Ok(())
    }

    pub fn add(&mut self, text: &str) -> Result<()> {
        if text.len() < 12 {
            return Ok(());
        }
        self.entries.push_back(text.to_string());
        while self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }
        Ok(())
    }

    /// Simple keyword search (production: cosine similarity on embeddings).
    pub fn search(&self, query: &str, top_k: usize) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let keywords: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scored: Vec<(usize, &String)> = self
            .entries
            .iter()
            .map(|entry| {
                let entry_lower = entry.to_lowercase();
                let score = keywords
                    .iter()
                    .filter(|kw| entry_lower.contains(*kw))
                    .count();
                (score, entry)
            })
            .filter(|(score, _)| *score > 0)
            .collect();

        scored.sort_by_key(|item| std::cmp::Reverse(item.0));
        scored
            .into_iter()
            .take(top_k)
            .map(|(_, entry)| entry.clone())
            .collect()
    }

    pub fn persist(&mut self) -> Result<()> {
        if !self.config.persist_on_shutdown {
            return Ok(());
        }
        info!(entries = self.entries.len(), "Persisting vector store");
        // Production: serialize embeddings to disk
        Ok(())
    }
}
