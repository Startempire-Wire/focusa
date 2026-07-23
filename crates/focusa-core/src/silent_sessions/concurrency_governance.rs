//! Writer admission, workspace isolation, scheduling, and governed integration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WriterKind {
    WorkLoop,
    Foreground,
    SilentSession,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WriterIntent {
    pub writer_kind: WriterKind,
    pub owner_id: String,
    pub work_item_id: String,
    pub workspace_ref: String,
    pub path_intents: BTreeSet<String>,
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WriterLease {
    pub lease_id: String,
    pub owner_id: String,
    pub workspace_ref: String,
    pub acquired_at: DateTime<Utc>,
    pub heartbeat_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl WriterLease {
    pub fn authorize(&self, intent: &WriterIntent, now: DateTime<Utc>) -> anyhow::Result<()> {
        anyhow::ensure!(
            now < self.expires_at,
            "writer lease expired; adoption required"
        );
        anyhow::ensure!(
            self.owner_id == intent.owner_id && self.workspace_ref == intent.workspace_ref,
            "exactly one scoped writer is permitted"
        );
        Ok(())
    }
}

pub fn analyze_writer_conflict(
    candidate: &WriterIntent,
    active: &[WriterIntent],
) -> anyhow::Result<()> {
    if candidate.read_only {
        return Ok(());
    }
    for writer in active
        .iter()
        .filter(|writer| !writer.read_only && writer.workspace_ref == candidate.workspace_ref)
    {
        if writer.owner_id != candidate.owner_id
            && !writer.path_intents.is_disjoint(&candidate.path_intents)
        {
            anyhow::bail!("workspace/path writer conflict");
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceStrategy {
    IsolatedWorktree,
    ExclusiveExisting,
    ReadOnlyShared,
    ApprovedShared,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceBinding {
    pub strategy: WorkspaceStrategy,
    pub session_id: String,
    pub owner_id: String,
    pub canonical_path: String,
    pub shared_mode_approval_id: Option<String>,
}

impl WorkspaceBinding {
    pub fn verify(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            !self.session_id.is_empty() && !self.owner_id.is_empty(),
            "session and owner binding required"
        );
        anyhow::ensure!(
            !self.canonical_path.contains(".."),
            "workspace traversal rejected"
        );
        if self.strategy == WorkspaceStrategy::ApprovedShared {
            anyhow::ensure!(
                self.shared_mode_approval_id
                    .as_deref()
                    .is_some_and(|id| !id.is_empty()),
                "shared write mode requires explicit approval"
            );
        }
        Ok(())
    }

    pub fn collision_safe_name(&self) -> String {
        let sanitized: String = self
            .session_id
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        format!("focusa-silent-{sanitized}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduleCandidate {
    pub session_id: String,
    pub priority: i32,
    pub dependencies_ready: bool,
    pub blocked: bool,
    pub lease_admitted: bool,
    pub resource_admitted: bool,
}

pub fn select_work_loop_owned(candidates: &[ScheduleCandidate]) -> Option<&ScheduleCandidate> {
    candidates
        .iter()
        .filter(|c| c.dependencies_ready && !c.blocked && c.lease_admitted && c.resource_admitted)
        .max_by_key(|c| c.priority)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationMethod {
    Merge,
    Rebase,
    CherryPick,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrationEvidence {
    pub tests_ref: String,
    pub checkpoint_ref: String,
    pub diff_ref: String,
    pub commit_ref: String,
    pub preview_ref: String,
    pub authority_ref: String,
    pub conflict_free: bool,
    pub unrelated_dirty_changes_preserved: bool,
    pub destructive_cleanup_requested: bool,
}

impl IntegrationEvidence {
    pub fn authorize(&self, _method: IntegrationMethod) -> anyhow::Result<()> {
        anyhow::ensure!(
            [
                &self.tests_ref,
                &self.checkpoint_ref,
                &self.diff_ref,
                &self.commit_ref,
                &self.preview_ref,
                &self.authority_ref
            ]
            .iter()
            .all(|r| !r.is_empty()),
            "integration proof chain incomplete"
        );
        anyhow::ensure!(self.conflict_free, "integration conflict blocks mutation");
        anyhow::ensure!(
            self.unrelated_dirty_changes_preserved,
            "unrelated dirty changes must be preserved"
        );
        anyhow::ensure!(
            !self.destructive_cleanup_requested,
            "destructive cleanup is outside integration authority"
        );
        Ok(())
    }
}
