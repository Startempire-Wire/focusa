//! Typed POSIX process-tree supervision and controlled-stop receipts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlledStopPolicy {
    pub harness_abort_grace_ms: u64,
    pub process_group_grace_ms: u64,
    pub leak_check_attempts: u32,
    pub leak_check_interval_ms: u64,
}

impl Default for ControlledStopPolicy {
    fn default() -> Self {
        Self {
            harness_abort_grace_ms: 2_000,
            process_group_grace_ms: 5_000,
            leak_check_attempts: 20,
            leak_check_interval_ms: 100,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TerminationStage {
    HarnessAbortRequested,
    HarnessGraceExpired,
    ProcessGroupTermRequested,
    ProcessGroupGraceExpired,
    ProcessGroupKillRequested,
    ChildLeakCheckPassed,
    ChildLeakDetected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminationStageRecord {
    pub stage: TerminationStage,
    pub occurred_at: DateTime<Utc>,
    pub process_group_id: i32,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlledStopReceipt {
    pub process_group_id: i32,
    pub stages: Vec<TerminationStageRecord>,
    pub tree_terminated: bool,
    pub child_leak_verified: bool,
}

impl ControlledStopReceipt {
    pub fn record(&mut self, stage: TerminationStage, detail: impl Into<String>) {
        self.stages.push(TerminationStageRecord {
            stage,
            occurred_at: Utc::now(),
            process_group_id: self.process_group_id,
            detail: detail.into(),
        });
    }

    pub fn verify_complete(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.tree_terminated,
            "owned process tree was not terminated"
        );
        anyhow::ensure!(
            self.child_leak_verified,
            "child-leak verification did not pass"
        );
        anyhow::ensure!(
            self.stages
                .iter()
                .any(|record| record.stage == TerminationStage::ProcessGroupTermRequested),
            "controlled stop skipped graceful process-group termination"
        );
        Ok(())
    }
}
