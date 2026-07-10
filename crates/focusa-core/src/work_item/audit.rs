//! Closure audit appender (Spec 116 §11 + §18).
//!
// Every lifecycle stage, every verifier call, every override writes a
//! row to `~/.focusa/state/closure-audit.jsonl`. The log is
//! append-only and human-readable. Reviewers replay it to see what
//! the agent did, when, and with what evidence.

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::work_item::types::{ClaimStatus, ClosureClaim, LifecycleStage, WorkItemProvider};

/// One append-only audit row.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClosureAuditEvent {
    /// Schema tag.
    pub schema: String, // "focusa.closure_audit.v1"
    /// Wall clock timestamp.
    pub ts: DateTime<Utc>,
    /// Which stage the event belongs to.
    pub stage: LifecycleStage,
    /// Claim id (if known at the time of the event).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_id: Option<String>,
    /// Provider the claim targets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<WorkItemProvider>,
    /// Provider item id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_item_id: Option<String>,
    /// Status of the claim after this stage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_status: Option<ClaimStatus>,
    /// Actor id (operator email or agent name).
    pub actor: String,
    /// Free-form detail string.
    pub detail: String,
    /// Optional structured result (e.g. verifier result).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    /// Optional evidence_url pointer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_url: Option<String>,
}

impl ClosureAuditEvent {
    pub fn new(stage: LifecycleStage, actor: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            schema: "focusa.closure_audit.v1".into(),
            ts: Utc::now(),
            stage,
            claim_id: None,
            provider: None,
            provider_item_id: None,
            claim_status: None,
            actor: actor.into(),
            detail: detail.into(),
            result: None,
            evidence_url: None,
        }
    }

    pub fn with_claim(mut self, claim: &ClosureClaim) -> Self {
        self.claim_id = Some(claim.claim_id.clone());
        self.provider = Some(claim.work_item.provider);
        self.provider_item_id = Some(claim.work_item.provider_item_id.clone());
        self.claim_status = Some(claim.status);
        self
    }
}

/// Append-only log. Thread-safe.
#[derive(Clone)]
pub struct ClosureAuditLog {
    inner: Arc<Mutex<ClosureAuditLogInner>>,
}

struct ClosureAuditLogInner {
    path: PathBuf,
    file: Option<std::fs::File>,
}

impl ClosureAuditLog {
    /// Open or create the audit log at the given path. The path is
    /// created with `chmod 600` so only the operator can read it.
    pub fn open(path: impl Into<PathBuf>) -> std::io::Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut p = std::fs::metadata(parent)?.permissions();
                p.set_mode(0o700);
                std::fs::set_permissions(parent, p)?;
            }
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut p = file.metadata()?.permissions();
            p.set_mode(0o600);
            std::fs::set_permissions(&path, p)?;
        }
        Ok(Self {
            inner: Arc::new(Mutex::new(ClosureAuditLogInner {
                path,
                file: Some(file),
            })),
        })
    }

    /// Default path: `~/.focusa/state/closure-audit.jsonl`.
    pub fn default_path() -> PathBuf {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/root"));
        home.join(".focusa")
            .join("state")
            .join("closure-audit.jsonl")
    }

    /// Open the default log.
    pub fn open_default() -> Self {
        Self::open(Self::default_path()).unwrap_or_else(|e| {
            eprintln!("[closure-audit] open failed: {e}; using /dev/null");
            Self {
                inner: Arc::new(Mutex::new(ClosureAuditLogInner {
                    path: PathBuf::from("/dev/null"),
                    file: None,
                })),
            }
        })
    }

    /// Append a single event. Returns the file path the event went to,
    /// or `None` if the log is closed.
    pub fn append(&self, event: ClosureAuditEvent) -> std::io::Result<Option<PathBuf>> {
        let mut guard = self.inner.lock();
        #[allow(clippy::redundant_closure)]
        let line = serde_json::to_string(&event).map_err(|e| {
            std::io::Error::other(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })?;
        if let Some(f) = guard.file.as_mut() {
            writeln!(f, "{line}")?;
            f.flush()?;
            Ok(Some(guard.path.clone()))
        } else {
            Ok(None)
        }
    }

    /// Replay the log into a Vec (for the closure doctor / audit
    /// inspector).
    pub fn replay(path: &Path) -> std::io::Result<Vec<ClosureAuditEvent>> {
        let Ok(contents) = std::fs::read_to_string(path) else {
            return Ok(Vec::new());
        };
        let mut out = Vec::new();
        for line in contents.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(ev) = serde_json::from_str::<ClosureAuditEvent>(line) {
                out.push(ev);
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_event_roundtrip_json() {
        let ev = ClosureAuditEvent::new(
            LifecycleStage::Validate,
            "verious.smith@philoveracity.com",
            "starting validate",
        )
        .with_claim(&ClosureClaim {
            schema: "focusa.closure_claim.v1".into(),
            claim_id: "claim_test_audit".into(),
            idempotency_key: "idem_test_audit".into(),
            work_item: crate::work_item::types::WorkItemRef {
                provider: WorkItemProvider::Bd,
                provider_item_id: "focusa-test".into(),
                project_root: "/tmp".into(),
                external_url: None,
            },
            project_root: "/tmp".into(),
            continuity_id: "focusa-cont-test".into(),
            workpoint_id: None,
            actor_id: "verious.smith@philoveracity.com".into(),
            agent_session_id: None,
            closure_summary: "audit roundtrip".into(),
            closure_kind: crate::work_item::types::ClosureKind::Code,
            code_refs: vec![],
            spec_refs: vec![],
            proof_refs: vec![],
            deploy_refs: vec![],
            artifact_refs: vec![],
            policy: "default".into(),
            created_at: Utc::now(),
            expires_at: Utc::now(),
            status: ClaimStatus::Valid,
            override_reason: None,
            machine_id: None,
        });
        let s = serde_json::to_string(&ev).unwrap();
        let back: ClosureAuditEvent = serde_json::from_str(&s).unwrap();
        assert_eq!(back.detail, "starting validate");
        assert_eq!(back.claim_id.as_deref(), Some("claim_test_audit"));
        assert_eq!(back.provider, Some(WorkItemProvider::Bd));
    }

    #[test]
    fn audit_log_append_and_replay_roundtrip() {
        let dir = std::env::temp_dir().join("focusa-audit-tests");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join(format!(
            "audit-{}-{}.jsonl",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let log = ClosureAuditLog::open(&p).unwrap();
        log.append(ClosureAuditEvent::new(
            LifecycleStage::Prepare,
            "agent",
            "started prepare",
        ))
        .unwrap();
        log.append(ClosureAuditEvent::new(
            LifecycleStage::Validate,
            "agent",
            "started validate",
        ))
        .unwrap();
        drop(log);
        let events = ClosureAuditLog::replay(&p).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].stage, LifecycleStage::Prepare);
        assert_eq!(events[1].stage, LifecycleStage::Validate);
        let _ = std::fs::remove_file(p);
    }
}
