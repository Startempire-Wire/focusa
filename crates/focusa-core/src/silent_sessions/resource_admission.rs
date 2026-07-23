//! Resource admission, enforcement truth, usage, and backpressure policy.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceLimits {
    pub cpu_millis: u64,
    pub memory_bytes: u64,
    pub pids: u32,
    pub file_descriptors: u32,
    pub io_bytes: u64,
    pub disk_bytes: u64,
    pub wall_time_ms: u64,
    pub output_bytes: u64,
    pub tokens: u64,
    pub cost_usd: f64,
    pub turns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdmissionCounts {
    pub global: u32,
    pub user: u32,
    pub project: u32,
    pub provider: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdmissionQuotas {
    pub global: u32,
    pub user: u32,
    pub project: u32,
    pub provider: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnforcementCapabilities {
    pub cpu: bool,
    pub memory: bool,
    pub pids: bool,
    pub file_descriptors: bool,
    pub io: bool,
    pub disk: bool,
    pub priority: bool,
}

impl AdmissionQuotas {
    pub fn authorize(&self, counts: &AdmissionCounts) -> anyhow::Result<()> {
        anyhow::ensure!(
            counts.global < self.global,
            "global concurrency quota exhausted"
        );
        anyhow::ensure!(counts.user < self.user, "user concurrency quota exhausted");
        anyhow::ensure!(
            counts.project < self.project,
            "project concurrency quota exhausted"
        );
        anyhow::ensure!(
            counts.provider < self.provider,
            "provider concurrency quota exhausted"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PressureAction {
    Continue,
    CheckpointAndPause,
    CheckpointAndCancel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceUsage {
    pub memory_bytes: u64,
    pub output_bytes: u64,
    pub tokens: u64,
    pub cost_usd: f64,
    pub turns: u64,
    pub wall_time_ms: u64,
    pub capture_cursor_persisted: bool,
}

impl ResourceUsage {
    pub fn pressure_action(&self, limits: &ResourceLimits) -> PressureAction {
        let hard = self.memory_bytes >= limits.memory_bytes
            || self.tokens >= limits.tokens
            || self.cost_usd >= limits.cost_usd
            || self.wall_time_ms >= limits.wall_time_ms;
        if hard {
            return PressureAction::CheckpointAndCancel;
        }
        let soft = self.output_bytes >= limits.output_bytes || self.turns >= limits.turns;
        if soft {
            PressureAction::CheckpointAndPause
        } else {
            PressureAction::Continue
        }
    }

    pub fn verify_truth_preserved(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.capture_cursor_persisted,
            "backpressure cannot discard canonical output truth"
        );
        Ok(())
    }
}
