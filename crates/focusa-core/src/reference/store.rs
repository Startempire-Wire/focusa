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
    #[allow(clippy::too_many_arguments)]
    pub fn store(
        &self,
        kind: HandleKind,
        label: String,
        content: &[u8],
        session_id: Option<SessionId>,
        handle_id: Option<HandleId>,
        project_root: Option<String>,
        continuity_id: Option<String>,
    ) -> anyhow::Result<HandleRef> {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let sha256 = hex::encode(hasher.finalize());

        let id = handle_id.unwrap_or_else(Uuid::now_v7);
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
            project_root,
            continuity_id,
            pinned: false,
            trajectory: None,
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

    /// Resolve a handle only when its stored scope matches the expected project/workstream.
    ///
    /// Legacy unscoped handles remain readable through `resolve`; scoped callers use this
    /// method so an id from another project or continuity cannot hydrate as current proof.
    pub fn resolve_scoped(
        &self,
        handle_id: HandleId,
        expected_project_root: Option<&str>,
        expected_continuity_id: Option<&str>,
    ) -> anyhow::Result<(HandleRef, PathBuf)> {
        let (handle, blob_path) = self.resolve(handle_id)?;
        if let Some(expected) = clean_scope(expected_project_root) {
            let actual = clean_scope(handle.project_root.as_deref());
            anyhow::ensure!(
                actual.as_deref() == Some(expected.as_str()),
                "reference handle project_root scope mismatch"
            );
        }
        if let Some(expected) = clean_scope(expected_continuity_id) {
            let actual = clean_scope(handle.continuity_id.as_deref());
            anyhow::ensure!(
                actual.as_deref() == Some(expected.as_str()),
                "reference handle continuity_id scope mismatch"
            );
        }
        Ok((handle, blob_path))
    }

    /// Check if content exceeds externalization threshold.
    pub fn should_externalize(content: &[u8], config: &FocusaConfig) -> bool {
        content.len() as u64 >= config.ecs_externalize_bytes_threshold
    }
}

fn clean_scope(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim().trim_end_matches('/').to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store() -> ReferenceStore {
        let root = std::env::temp_dir().join(format!("focusa-reference-store-test-{}", Uuid::now_v7()));
        ReferenceStore::new(root).unwrap()
    }

    #[test]
    fn resolve_scoped_accepts_matching_project_and_continuity() {
        let store = test_store();
        let handle = store
            .store(
                HandleKind::Text,
                "match".to_string(),
                b"content",
                None,
                None,
                Some("/tmp/project/".to_string()),
                Some("cont-a".to_string()),
            )
            .unwrap();
        let (resolved, path) = store
            .resolve_scoped(handle.id, Some("/tmp/project"), Some("cont-a"))
            .unwrap();
        assert_eq!(resolved.id, handle.id);
        assert!(path.exists());
    }

    #[test]
    fn resolve_scoped_rejects_cross_project_or_workstream() {
        let store = test_store();
        let handle = store
            .store(
                HandleKind::Text,
                "mismatch".to_string(),
                b"content",
                None,
                None,
                Some("/tmp/project-a".to_string()),
                Some("cont-a".to_string()),
            )
            .unwrap();
        let project_err = store
            .resolve_scoped(handle.id, Some("/tmp/project-b"), Some("cont-a"))
            .unwrap_err()
            .to_string();
        assert!(project_err.contains("project_root scope mismatch"));
        let cont_err = store
            .resolve_scoped(handle.id, Some("/tmp/project-a"), Some("cont-b"))
            .unwrap_err()
            .to_string();
        assert!(cont_err.contains("continuity_id scope mismatch"));
    }

    #[test]
    fn legacy_unscoped_handle_remains_readable_but_not_scoped() {
        let store = test_store();
        let handle = store
            .store(HandleKind::Text, "legacy".to_string(), b"content", None, None, None, None)
            .unwrap();
        assert!(store.resolve(handle.id).is_ok());
        assert!(store
            .resolve_scoped(handle.id, Some("/tmp/project"), Some("cont-a"))
            .is_err());
    }
}
