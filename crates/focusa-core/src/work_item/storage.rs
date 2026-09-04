//! Closure claim storage (Spec 116 §11 + §18).
//!
//! Each claim is serialized to `~/.focusa/state/closure-claims/<claim_id>.json`.
//! The lifecycle writes a fresh file on every stage transition; older
//! revisions are kept alongside (rotated to `<claim_id>.<n>.json`)
//! so a reviewer can replay what changed when.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::work_item::types::ClosureClaim;

/// Storage errors.
#[derive(Debug)]
pub enum ClaimStorageError {
    Io(std::io::Error),
    Serde(serde_json::Error),
    NotFound(String),
    Conflict(String),
}

impl std::fmt::Display for ClaimStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io: {e}"),
            Self::Serde(e) => write!(f, "json: {e}"),
            Self::NotFound(id) => write!(f, "claim not found: {id}"),
            Self::Conflict(id) => write!(f, "claim conflict: {id}"),
        }
    }
}

impl std::error::Error for ClaimStorageError {}

impl From<std::io::Error> for ClaimStorageError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for ClaimStorageError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}

pub type ClaimStorageResult<T> = Result<T, ClaimStorageError>;

/// Read/write a closure claim to/from a JSON file.
#[derive(Clone, Debug)]
pub struct ClaimStorage {
    root: PathBuf,
}

impl ClaimStorage {
    /// Construct storage rooted at the given directory.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Default path: `~/.focusa/state/closure-claims/`.
    pub fn default_root() -> PathBuf {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/root"));
        home.join(".focusa").join("state").join("closure-claims")
    }

    /// Open the default storage.
    pub fn open_default() -> Self {
        Self::new(Self::default_root())
    }

    fn path(&self, claim_id: &str) -> PathBuf {
        self.root.join(format!("{claim_id}.json"))
    }

    /// Save the claim. Rotates any prior file at the same path to
    /// `<claim_id>.<n>.json` so the history is preserved.
    pub fn save(&self, claim: &ClosureClaim) -> ClaimStorageResult<PathBuf> {
        std::fs::create_dir_all(&self.root)?;
        let path = self.path(&claim.claim_id);
        if path.exists() {
            // Rotate the prior file: find the next free rotation
            // suffix, move the old file aside, write the new one.
            let mut n = 1u32;
            let rotated;
            loop {
                let candidate = self.root.join(format!("{}.{n}.json", claim.claim_id));
                if !candidate.exists() {
                    rotated = candidate;
                    break;
                }
                n += 1;
            }
            std::fs::rename(&path, &rotated)?;
        }
        let json = serde_json::to_string_pretty(claim)?;
        std::fs::write(&path, json)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut p = std::fs::metadata(&path)?.permissions();
            p.set_mode(0o600);
            std::fs::set_permissions(&path, p)?;
        }
        Ok(path)
    }

    /// Load a claim by id.
    pub fn load(&self, claim_id: &str) -> ClaimStorageResult<ClosureClaim> {
        let path = self.path(claim_id);
        if !path.exists() {
            return Err(ClaimStorageError::NotFound(claim_id.into()));
        }
        let s = std::fs::read_to_string(&path)?;
        let claim: ClosureClaim = serde_json::from_str(&s)?;
        Ok(claim)
    }

    /// Find the canonical claim for an idempotency key, ignoring rotated snapshots.
    pub fn find_by_idempotency_key(
        &self,
        idempotency_key: &str,
    ) -> ClaimStorageResult<Option<ClosureClaim>> {
        for claim_id in self.list()? {
            if claim_id.contains('.') {
                continue;
            }
            let claim = self.load(&claim_id)?;
            if claim.idempotency_key == idempotency_key {
                return Ok(Some(claim));
            }
        }
        Ok(None)
    }

    /// List claim ids currently on disk.
    pub fn list(&self) -> ClaimStorageResult<Vec<String>> {
        let mut out = Vec::new();
        if !self.root.exists() {
            return Ok(out);
        }
        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                    out.push(stem.to_string());
                }
            }
        }
        out.sort();
        Ok(out)
    }

    /// Delete a claim (admin only; rarely used).
    pub fn delete(&self, claim_id: &str) -> ClaimStorageResult<()> {
        let path = self.path(claim_id);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}

/// A claim snapshot in time. Used by the audit replay surface.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClaimSnapshot {
    pub claim_id: String,
    pub status: String,
    pub path: String,
    pub bytes: u64,
    pub mtime_unix: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::work_item::types::{
        ClaimStatus, ClosureClaim, ClosureKind, WorkItemProvider, WorkItemRef,
    };
    use chrono::Utc;
    use std::path::PathBuf;

    fn sample_claim(id: &str) -> ClosureClaim {
        ClosureClaim {
            schema: "focusa.closure_claim.v1".into(),
            claim_id: id.into(),
            idempotency_key: format!("idem_{id}"),
            work_item: WorkItemRef {
                provider: WorkItemProvider::Bd,
                provider_item_id: "focusa-test".into(),
                project_root: PathBuf::from("/tmp/p"),
                external_url: None,
            },
            project_root: PathBuf::from("/tmp/p"),
            continuity_id: "focusa-cont-test".into(),
            workpoint_id: None,
            actor_id: "verious.smith@philoveracity.com".into(),
            agent_session_id: None,
            closure_summary: "test".into(),
            closure_kind: ClosureKind::Code,
            code_refs: vec![],
            spec_refs: vec![],
            proof_refs: vec![],
            deploy_refs: vec![],
            artifact_refs: vec![],
            policy: "release_proof".into(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            status: ClaimStatus::Draft,
            override_reason: None,
            machine_id: None,
        }
    }

    #[test]
    fn save_load_roundtrip() {
        let dir = std::env::temp_dir().join("focusa-claim-store-tests");
        std::fs::create_dir_all(&dir).unwrap();
        let store = ClaimStorage::new(&dir);
        let c1 = sample_claim("claim_roundtrip");
        let path = store.save(&c1).unwrap();
        assert!(path.exists());
        let back = store.load("claim_roundtrip").unwrap();
        assert_eq!(back.claim_id, "claim_roundtrip");
        assert_eq!(back.closure_summary, "test");
        store.delete("claim_roundtrip").unwrap();
    }

    #[test]
    fn save_rotates_prior_file() {
        let dir = std::env::temp_dir().join("focusa-claim-rotate-tests");
        std::fs::create_dir_all(&dir).unwrap();
        let store = ClaimStorage::new(&dir);
        let mut c1 = sample_claim("claim_rotate");
        c1.closure_summary = "first".into();
        store.save(&c1).unwrap();
        let mut c2 = sample_claim("claim_rotate");
        c2.closure_summary = "second".into();
        store.save(&c2).unwrap();
        let entries: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert!(entries.iter().any(|n| n == "claim_rotate.json"));
        assert!(entries.iter().any(|n| n.starts_with("claim_rotate.")));
        let back = store.load("claim_rotate").unwrap();
        assert_eq!(back.closure_summary, "second");
        // Clean up
        for e in entries {
            let _ = std::fs::remove_file(dir.join(&e));
        }
    }

    #[test]
    fn list_returns_ids() {
        let dir =
            std::env::temp_dir().join(format!("focusa-claim-list-tests-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&dir).unwrap();
        let store = ClaimStorage::new(&dir);
        store.save(&sample_claim("c1")).unwrap();
        store.save(&sample_claim("c2")).unwrap();
        let mut ids = store.list().unwrap();
        ids.sort();
        assert_eq!(ids, vec!["c1", "c2"]);
        store.delete("c1").unwrap();
        store.delete("c2").unwrap();
        std::fs::remove_dir(&dir).unwrap();
    }
}
