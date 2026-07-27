//! Append-only checkpoints for interruption-safe Master Release Cycle resume.

use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, ensure};
use serde::{Deserialize, Serialize};

use crate::release_cycle::ReleaseCandidate;
use crate::release_orchestrator::{ReleaseExecutionPlan, ReleaseStageReceipt};

pub const RELEASE_CHECKPOINT_SCHEMA: &str = "focusa.release_checkpoint.v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseRunCheckpoint {
    pub schema: String,
    pub sequence: u64,
    pub status: String,
    pub observed_at: String,
    pub candidate: ReleaseCandidate,
    pub plan: ReleaseExecutionPlan,
    pub receipts: Vec<ReleaseStageReceipt>,
    pub blocked_reasons: Vec<String>,
}

impl ReleaseRunCheckpoint {
    pub fn validate(&self) -> anyhow::Result<()> {
        ensure!(
            self.schema == RELEASE_CHECKPOINT_SCHEMA,
            "unsupported release checkpoint schema"
        );
        ensure!(
            !self.status.trim().is_empty(),
            "checkpoint status is required"
        );
        ensure!(
            !self.observed_at.trim().is_empty(),
            "checkpoint timestamp is required"
        );
        self.candidate.validate_identity()?;
        ensure!(
            self.plan.candidate_id == self.candidate.candidate_id,
            "checkpoint plan candidate mismatch"
        );
        ensure!(
            self.plan.exact_sha == self.candidate.exact_sha,
            "checkpoint plan SHA mismatch"
        );
        ensure!(
            self.receipts
                .iter()
                .all(|receipt| receipt.evidence.exact_sha == self.candidate.exact_sha),
            "checkpoint receipt SHA mismatch"
        );
        Ok(())
    }
}

pub trait ReleaseCheckpointSink: Send + Sync {
    fn next_sequence(&self) -> anyhow::Result<u64> {
        Ok(0)
    }
    fn append(&self, checkpoint: &ReleaseRunCheckpoint) -> anyhow::Result<()>;
}

pub struct NoopReleaseCheckpointSink;

impl ReleaseCheckpointSink for NoopReleaseCheckpointSink {
    fn append(&self, _checkpoint: &ReleaseRunCheckpoint) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct JsonlReleaseRunLedger {
    path: PathBuf,
    project_root: String,
    candidate_id: String,
    exact_sha: String,
}

impl JsonlReleaseRunLedger {
    pub fn new(
        path: impl Into<PathBuf>,
        project_root: impl Into<String>,
        candidate_id: impl Into<String>,
        exact_sha: impl Into<String>,
    ) -> anyhow::Result<Self> {
        let path = path.into();
        let project_root = project_root.into();
        let candidate_id = candidate_id.into();
        let exact_sha = exact_sha.into();
        ensure!(path.is_absolute(), "release ledger path must be absolute");
        ensure!(
            !project_root.trim().is_empty(),
            "ledger project_root is required"
        );
        ensure!(
            !candidate_id.trim().is_empty(),
            "ledger candidate_id is required"
        );
        ensure!(!exact_sha.trim().is_empty(), "ledger exact_sha is required");
        Ok(Self {
            path,
            project_root,
            candidate_id,
            exact_sha,
        })
    }

    pub fn latest(&self) -> anyhow::Result<Option<ReleaseRunCheckpoint>> {
        read_checkpoints(&self.path).map(|items| items.into_iter().last())
    }

    fn validate_scope(&self, checkpoint: &ReleaseRunCheckpoint) -> anyhow::Result<()> {
        checkpoint.validate()?;
        ensure!(
            checkpoint.candidate.project_root == self.project_root,
            "release ledger project mismatch"
        );
        ensure!(
            checkpoint.candidate.candidate_id == self.candidate_id,
            "release ledger candidate mismatch"
        );
        ensure!(
            checkpoint.candidate.exact_sha == self.exact_sha,
            "release ledger SHA mismatch"
        );
        Ok(())
    }
}

impl ReleaseCheckpointSink for JsonlReleaseRunLedger {
    fn next_sequence(&self) -> anyhow::Result<u64> {
        Ok(self.latest()?.map_or(0, |item| item.sequence + 1))
    }

    fn append(&self, checkpoint: &ReleaseRunCheckpoint) -> anyhow::Result<()> {
        self.validate_scope(checkpoint)?;
        let latest = self.latest()?;
        let expected = latest.as_ref().map_or(0, |item| item.sequence + 1);
        ensure!(
            checkpoint.sequence == expected,
            "release checkpoint sequence mismatch"
        );
        if let Some(previous) = latest {
            ensure!(
                checkpoint.candidate.stage >= previous.candidate.stage,
                "release checkpoint stage regressed"
            );
        }
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .with_context(|| format!("open release ledger {}", self.path.display()))?;
        serde_json::to_writer(&mut file, checkpoint)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        Ok(())
    }
}

fn read_checkpoints(path: &Path) -> anyhow::Result<Vec<ReleaseRunCheckpoint>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut output = Vec::new();
    for (index, line) in BufReader::new(std::fs::File::open(path)?)
        .lines()
        .enumerate()
    {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let checkpoint: ReleaseRunCheckpoint = serde_json::from_str(&line)
            .with_context(|| format!("invalid release checkpoint line {}", index + 1))?;
        checkpoint.validate()?;
        output.push(checkpoint);
    }
    Ok(output)
}

#[cfg(test)]
#[path = "release_ledger_test.rs"]
mod tests;
