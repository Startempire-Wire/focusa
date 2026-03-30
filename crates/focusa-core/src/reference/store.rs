//! ECS store operations.
//!
//! Storage layout: ~/.focusa/ecs/
//!   objects/  — immutable content-addressed blobs
//!   handles/  — metadata json by id
//!   index.json — small index

use crate::types::*;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use uuid::Uuid;

/// The reference store.
pub struct ReferenceStore {
    pub root: PathBuf,
}

impl ReferenceStore {
    pub fn new(ecs_root: PathBuf) -> anyhow::Result<Self> {
        std::fs::create_dir_all(ecs_root.join("objects"))?;
        std::fs::create_dir_all(ecs_root.join("handles"))?;
        Ok(Self { root: ecs_root })
    }

    /// Store an artifact, returning a HandleRef.
    ///
    /// Process:
    ///   1. Compute sha256
    ///   2. Generate id (UUIDv7)
    ///   3. Write blob file
    ///   4. Write metadata file
    ///   5. Return HandleRef
    pub fn store(
        &self,
        kind: HandleKind,
        label: String,
        content: &[u8],
        session_id: Option<SessionId>,
    ) -> anyhow::Result<HandleRef> {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let sha256 = hex::encode(hasher.finalize());

        let id = Uuid::now_v7();
        let now = Utc::now();

        // Write blob
        let blob_path = self.root.join("objects").join(&sha256);
        if !blob_path.exists() {
            std::fs::write(&blob_path, content)?;
        }

        let handle = HandleRef {
            id,
            kind,
            label,
            size: content.len() as u64,
            sha256,
            created_at: now,
            session_id,
            pinned: false,
        };

        // Write metadata
        let meta_path = self.root.join("handles").join(format!("{}.json", id));
        let meta_json = serde_json::to_string_pretty(&handle)?;
        std::fs::write(&meta_path, meta_json)?;

        Ok(handle)
    }

    /// Resolve a handle — return metadata + content path.
    pub fn resolve(&self, handle_id: HandleId) -> anyhow::Result<(HandleRef, PathBuf)> {
        let meta_path = self
            .root
            .join("handles")
            .join(format!("{}.json", handle_id));
        let meta_str = std::fs::read_to_string(&meta_path)?;
        let handle: HandleRef = serde_json::from_str(&meta_str)?;
        let blob_path = self.root.join("objects").join(&handle.sha256);
        Ok((handle, blob_path))
    }

    /// Check if content exceeds externalization threshold.
    pub fn should_externalize(content: &[u8], config: &FocusaConfig) -> bool {
        content.len() as u64 >= config.ecs_externalize_bytes_threshold
    }
}
