//! ECS store operations.
//!
//! Storage layout: ~/.focusa/ecs/
//!   objects/ — immutable content-addressed blobs
//!   handles/ — canonical complete metadata by id
//!
//! `FocusaState.reference_index` is the bounded hot projection; this filesystem
//! store remains the lossless exact-id authority for cold metadata and content.

use crate::durable_fs::{atomic_replace, sync_directory};
use crate::types::*;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::Write;
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
        let meta_path = self.root.join("handles").join(format!("{}.json", id));
        anyhow::ensure!(
            !meta_path.exists(),
            "artifact {id} is already registered; immutable handle ids cannot be reused"
        );

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

        self.write_metadata(&handle, false)?;

        Ok(handle)
    }

    /// Atomically persist mutable handle metadata such as its trajectory binding.
    /// Artifact content remains immutable and content-addressed.
    pub fn persist_metadata(&self, handle: &HandleRef) -> anyhow::Result<()> {
        self.write_metadata(handle, true)
    }

    fn write_metadata(&self, handle: &HandleRef, replace: bool) -> anyhow::Result<()> {
        let handles_dir = self.root.join("handles");
        let meta_path = handles_dir.join(format!("{}.json", handle.id));
        let staged_path =
            handles_dir.join(format!(".{}.{}.metadata.tmp", handle.id, Uuid::now_v7()));
        if replace {
            anyhow::ensure!(
                meta_path.is_file(),
                "cannot update missing artifact metadata for {}",
                handle.id
            );
        }
        let bytes = serde_json::to_vec_pretty(handle)?;
        let result = (|| -> anyhow::Result<()> {
            let mut staged = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&staged_path)?;
            staged.write_all(&bytes)?;
            staged.sync_all()?;
            if replace {
                atomic_replace(&staged_path, &meta_path)?;
            } else {
                std::fs::hard_link(&staged_path, &meta_path).map_err(|error| {
                    anyhow::anyhow!(
                        "artifact {} is already registered or metadata publication failed: {error}",
                        handle.id
                    )
                })?;
                if let Err(error) = std::fs::remove_file(&staged_path) {
                    tracing::warn!(
                        path = %staged_path.display(),
                        "artifact metadata committed but staging hard link cleanup failed: {error}"
                    );
                }
            }
            sync_directory(&handles_dir)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&staged_path);
        }
        result
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
        let root =
            std::env::temp_dir().join(format!("focusa-reference-store-test-{}", Uuid::now_v7()));
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
    fn duplicate_explicit_handle_id_is_rejected_before_metadata_replacement() {
        let store = test_store();
        let id = Uuid::now_v7();
        let first = store
            .store(
                HandleKind::Text,
                "first".to_string(),
                b"first-content",
                None,
                Some(id),
                None,
                None,
            )
            .unwrap();
        let error = store
            .store(
                HandleKind::Text,
                "replacement".to_string(),
                b"replacement-content",
                None,
                Some(id),
                None,
                None,
            )
            .unwrap_err()
            .to_string();
        assert!(error.contains("immutable handle ids cannot be reused"));
        assert_eq!(store.resolve(id).unwrap().0.label, first.label);
    }

    #[test]
    fn trajectory_binding_is_durable_in_handle_metadata() {
        let store = test_store();
        let mut handle = store
            .store(
                HandleKind::Text,
                "trajectory-bound".to_string(),
                b"content",
                None,
                None,
                Some("/tmp/project".to_string()),
                Some("cont-a".to_string()),
            )
            .unwrap();
        handle.trajectory = Some(TrajectoryLadderContext {
            trajectory_id: Some("trajectory-a".to_string()),
            project_root: Some("/tmp/project".to_string()),
            continuity_id: Some("cont-a".to_string()),
            ..TrajectoryLadderContext::default()
        });
        store.persist_metadata(&handle).unwrap();

        let resolved = store.resolve(handle.id).unwrap().0;
        assert_eq!(
            resolved
                .trajectory
                .and_then(|trajectory| trajectory.trajectory_id),
            Some("trajectory-a".to_string())
        );
    }

    #[test]
    fn legacy_unscoped_handle_remains_readable_but_not_scoped() {
        let store = test_store();
        let handle = store
            .store(
                HandleKind::Text,
                "legacy".to_string(),
                b"content",
                None,
                None,
                None,
                None,
            )
            .unwrap();
        assert!(store.resolve(handle.id).is_ok());
        assert!(
            store
                .resolve_scoped(handle.id, Some("/tmp/project"), Some("cont-a"))
                .is_err()
        );
    }
}
