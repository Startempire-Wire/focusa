//! Deterministic machine-reboot recovery decisions for Silent Sessions.
//!
//! A reboot never preserves process authority. This planner consumes durable
//! session/run/checkpoint facts and may authorize only a new run generation.

use crate::silent_session::{
    SilentSessionCheckpointId, SilentSessionId, SilentSessionLifecycleState, SilentSessionRunId,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const REBOOT_RECOVERY_DECISION_SCHEMA: &str = "focusa.reboot_recovery_decision.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelaunchPolicy {
    Never,
    CheckpointedBounded,
    OperatorAcknowledged,
}

impl RelaunchPolicy {
    pub fn parse(value: &str) -> Result<Self, RebootRecoveryError> {
        match value.trim() {
            "never" => Ok(Self::Never),
            "bounded" | "checkpointed" | "checkpointed_bounded" => Ok(Self::CheckpointedBounded),
            "operator" | "operator_ack" | "operator_acknowledged" => Ok(Self::OperatorAcknowledged),
            other => Err(RebootRecoveryError::UnknownRelaunchPolicy(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RebootRecoveryStatus {
    TerminalNoRelaunch,
    BlockedMissingCheckpoints,
    BlockedByPolicy,
    BlockedRetryBudget,
    RequiresOperatorAcknowledgment,
    RelaunchPermitted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebootRecoveryRequest {
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub lifecycle_state: SilentSessionLifecycleState,
    pub previous_boot_id: String,
    pub current_boot_id: String,
    pub runtime_checkpoint_refs: Vec<SilentSessionCheckpointId>,
    pub workpoint_checkpoint_refs: Vec<String>,
    pub restart_policy: String,
    pub max_process_restarts: u32,
    pub operator_acknowledgment_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebootRecoveryDecision {
    pub schema: String,
    pub session_id: SilentSessionId,
    pub previous_run_id: SilentSessionRunId,
    pub previous_generation: u64,
    pub previous_boot_id: String,
    pub current_boot_id: String,
    pub original_process_survived: bool,
    pub recovered_lifecycle_state: SilentSessionLifecycleState,
    pub status: RebootRecoveryStatus,
    pub latest_runtime_checkpoint_ref: Option<String>,
    pub latest_workpoint_checkpoint_ref: Option<String>,
    pub operator_acknowledgment_ref: Option<String>,
    pub new_run_id: Option<SilentSessionRunId>,
    pub new_generation: Option<u64>,
}

impl RebootRecoveryRequest {
    pub fn evaluate(self) -> Result<RebootRecoveryDecision, RebootRecoveryError> {
        if !self.session_id.is_uuid_v7() || !self.run_id.is_uuid_v7() || self.generation == 0 {
            return Err(RebootRecoveryError::InvalidRunIdentity);
        }
        if self.previous_boot_id.trim().is_empty()
            || self.current_boot_id.trim().is_empty()
            || self.previous_boot_id == self.current_boot_id
        {
            return Err(RebootRecoveryError::BootIdentityNotChanged);
        }
        let policy = RelaunchPolicy::parse(&self.restart_policy)?;
        let latest_runtime_checkpoint_ref =
            self.runtime_checkpoint_refs.last().map(ToString::to_string);
        let latest_workpoint_checkpoint_ref = self
            .workpoint_checkpoint_refs
            .iter()
            .rev()
            .find(|value| !value.trim().is_empty())
            .cloned();
        let terminal = matches!(
            self.lifecycle_state,
            SilentSessionLifecycleState::Completed
                | SilentSessionLifecycleState::Failed
                | SilentSessionLifecycleState::Cancelled
        );
        let recovered_lifecycle_state = if terminal {
            self.lifecycle_state.clone()
        } else {
            SilentSessionLifecycleState::Orphaned
        };

        let retries_used = self.generation.saturating_sub(1);
        let has_checkpoints =
            latest_runtime_checkpoint_ref.is_some() && latest_workpoint_checkpoint_ref.is_some();
        let acknowledgment = self
            .operator_acknowledgment_ref
            .filter(|value| !value.trim().is_empty());
        let status = if terminal {
            RebootRecoveryStatus::TerminalNoRelaunch
        } else if !has_checkpoints {
            RebootRecoveryStatus::BlockedMissingCheckpoints
        } else if policy == RelaunchPolicy::Never {
            RebootRecoveryStatus::BlockedByPolicy
        } else if retries_used >= u64::from(self.max_process_restarts) {
            RebootRecoveryStatus::BlockedRetryBudget
        } else if policy == RelaunchPolicy::OperatorAcknowledged && acknowledgment.is_none() {
            RebootRecoveryStatus::RequiresOperatorAcknowledgment
        } else {
            RebootRecoveryStatus::RelaunchPermitted
        };
        let relaunch = status == RebootRecoveryStatus::RelaunchPermitted;
        let new_generation = if relaunch {
            Some(
                self.generation
                    .checked_add(1)
                    .ok_or(RebootRecoveryError::GenerationExhausted)?,
            )
        } else {
            None
        };

        Ok(RebootRecoveryDecision {
            schema: REBOOT_RECOVERY_DECISION_SCHEMA.into(),
            session_id: self.session_id,
            previous_run_id: self.run_id,
            previous_generation: self.generation,
            previous_boot_id: self.previous_boot_id,
            current_boot_id: self.current_boot_id,
            original_process_survived: false,
            recovered_lifecycle_state,
            status,
            latest_runtime_checkpoint_ref,
            latest_workpoint_checkpoint_ref,
            operator_acknowledgment_ref: acknowledgment,
            new_run_id: relaunch.then(SilentSessionRunId::new),
            new_generation,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RebootRecoveryError {
    #[error("reboot recovery requires UUIDv7 session/run ids and a positive generation")]
    InvalidRunIdentity,
    #[error("reboot recovery requires distinct nonempty boot identities")]
    BootIdentityNotChanged,
    #[error("unknown reboot relaunch policy: {0}")]
    UnknownRelaunchPolicy(String),
    #[error("run generation is exhausted")]
    GenerationExhausted,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> RebootRecoveryRequest {
        RebootRecoveryRequest {
            session_id: SilentSessionId::new(),
            run_id: SilentSessionRunId::new(),
            generation: 1,
            lifecycle_state: SilentSessionLifecycleState::Running,
            previous_boot_id: "boot:before".into(),
            current_boot_id: "boot:after".into(),
            runtime_checkpoint_refs: vec![SilentSessionCheckpointId::new()],
            workpoint_checkpoint_refs: vec!["workpoint:checkpoint:7".into()],
            restart_policy: "bounded".into(),
            max_process_restarts: 2,
            operator_acknowledgment_ref: None,
        }
    }

    #[test]
    fn reboot_never_claims_process_survival_and_creates_only_a_new_generation() {
        let request = request();
        let old_run_id = request.run_id;
        let decision = request.evaluate().unwrap();
        assert_eq!(decision.status, RebootRecoveryStatus::RelaunchPermitted);
        assert!(!decision.original_process_survived);
        assert_eq!(
            decision.recovered_lifecycle_state,
            SilentSessionLifecycleState::Orphaned
        );
        assert_eq!(decision.new_generation, Some(2));
        assert!(decision.new_run_id.is_some_and(|id| id != old_run_id));
        assert!(decision.latest_runtime_checkpoint_ref.is_some());
        assert_eq!(
            decision.latest_workpoint_checkpoint_ref.as_deref(),
            Some("workpoint:checkpoint:7")
        );
    }

    #[test]
    fn checkpoint_policy_budget_and_operator_gates_fail_closed() {
        let mut missing = request();
        missing.runtime_checkpoint_refs.clear();
        assert_eq!(
            missing.evaluate().unwrap().status,
            RebootRecoveryStatus::BlockedMissingCheckpoints
        );

        let mut never = request();
        never.restart_policy = "never".into();
        assert_eq!(
            never.evaluate().unwrap().status,
            RebootRecoveryStatus::BlockedByPolicy
        );

        let mut exhausted = request();
        exhausted.generation = 3;
        assert_eq!(
            exhausted.evaluate().unwrap().status,
            RebootRecoveryStatus::BlockedRetryBudget
        );

        let mut acknowledged = request();
        acknowledged.restart_policy = "operator_acknowledged".into();
        assert_eq!(
            acknowledged.clone().evaluate().unwrap().status,
            RebootRecoveryStatus::RequiresOperatorAcknowledgment
        );
        acknowledged.operator_acknowledgment_ref = Some("approval:reboot:1".into());
        assert_eq!(
            acknowledged.evaluate().unwrap().status,
            RebootRecoveryStatus::RelaunchPermitted
        );
    }

    #[test]
    fn terminal_runs_and_unchanged_boot_identity_never_relaunch() {
        let mut terminal = request();
        terminal.lifecycle_state = SilentSessionLifecycleState::Completed;
        let decision = terminal.evaluate().unwrap();
        assert_eq!(decision.status, RebootRecoveryStatus::TerminalNoRelaunch);
        assert_eq!(
            decision.recovered_lifecycle_state,
            SilentSessionLifecycleState::Completed
        );
        assert!(decision.new_run_id.is_none());

        let mut same_boot = request();
        same_boot.current_boot_id = same_boot.previous_boot_id.clone();
        assert_eq!(
            same_boot.evaluate(),
            Err(RebootRecoveryError::BootIdentityNotChanged)
        );
    }
}
