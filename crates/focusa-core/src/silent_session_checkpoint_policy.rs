//! Checkpoint cadence without transcript, log, or Focus State impersonation.

use crate::silent_session::WorkpointBinding;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCheckpointPolicy {
    pub interval_seconds: u64,
    pub semantic_event_interval: u64,
}

impl RuntimeCheckpointPolicy {
    pub fn validate(&self) -> Result<(), CheckpointPolicyError> {
        if self.interval_seconds == 0 || self.semantic_event_interval == 0 {
            return Err(CheckpointPolicyError::InvalidCadence);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryCheckpointBoundary {
    BeforeEscalation,
    AfterEscalation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleCheckpointBoundary {
    BeforePause,
    BeforeProcessRestart,
    BeforeDaemonUpgrade,
    RunnerDisconnect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeCheckpointReason {
    TimeInterval,
    SemanticEventInterval,
    DurableToolChange,
    BeforeRetryEscalation,
    AfterRetryEscalation,
    BeforePause,
    BeforeProcessRestart,
    BeforeDaemonUpgrade,
    RunnerDisconnect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeCheckpointObservation {
    pub now: DateTime<Utc>,
    pub last_checkpoint_at: DateTime<Utc>,
    pub semantic_events_since_checkpoint: u64,
    pub tool_completed: bool,
    pub durable_project_change: bool,
    pub retry_boundary: Option<RetryCheckpointBoundary>,
    pub lifecycle_boundary: Option<LifecycleCheckpointBoundary>,
    pub stream_cursor: String,
    pub resource_counters: BTreeMap<String, u64>,
    pub retry_state: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeCheckpointDirective {
    pub reasons: BTreeSet<RuntimeCheckpointReason>,
    pub checkpoint_at: DateTime<Utc>,
    pub stream_cursor: String,
    pub resource_counters: BTreeMap<String, u64>,
    pub retry_state: Value,
    pub sink: CheckpointSink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointSink {
    SilentSessionRuntimeStore,
    CanonicalWorkpointPath,
}

pub fn evaluate_runtime_checkpoint(
    policy: &RuntimeCheckpointPolicy,
    observation: RuntimeCheckpointObservation,
) -> Result<Option<RuntimeCheckpointDirective>, CheckpointPolicyError> {
    policy.validate()?;
    if observation.now < observation.last_checkpoint_at {
        return Err(CheckpointPolicyError::ClockRegression);
    }
    let mut reasons = BTreeSet::new();
    if observation.now - observation.last_checkpoint_at
        >= Duration::seconds(policy.interval_seconds as i64)
    {
        reasons.insert(RuntimeCheckpointReason::TimeInterval);
    }
    if observation.semantic_events_since_checkpoint >= policy.semantic_event_interval {
        reasons.insert(RuntimeCheckpointReason::SemanticEventInterval);
    }
    if observation.tool_completed && observation.durable_project_change {
        reasons.insert(RuntimeCheckpointReason::DurableToolChange);
    }
    match observation.retry_boundary {
        Some(RetryCheckpointBoundary::BeforeEscalation) => {
            reasons.insert(RuntimeCheckpointReason::BeforeRetryEscalation);
        }
        Some(RetryCheckpointBoundary::AfterEscalation) => {
            reasons.insert(RuntimeCheckpointReason::AfterRetryEscalation);
        }
        None => {}
    }
    match observation.lifecycle_boundary {
        Some(LifecycleCheckpointBoundary::BeforePause) => {
            reasons.insert(RuntimeCheckpointReason::BeforePause);
        }
        Some(LifecycleCheckpointBoundary::BeforeProcessRestart) => {
            reasons.insert(RuntimeCheckpointReason::BeforeProcessRestart);
        }
        Some(LifecycleCheckpointBoundary::BeforeDaemonUpgrade) => {
            reasons.insert(RuntimeCheckpointReason::BeforeDaemonUpgrade);
        }
        Some(LifecycleCheckpointBoundary::RunnerDisconnect) => {
            reasons.insert(RuntimeCheckpointReason::RunnerDisconnect);
        }
        None => {}
    }
    if reasons.is_empty() {
        return Ok(None);
    }
    if observation.stream_cursor.trim().is_empty()
        || observation.resource_counters.is_empty()
        || observation.retry_state.is_null()
    {
        return Err(CheckpointPolicyError::RuntimeStateMissing);
    }
    Ok(Some(RuntimeCheckpointDirective {
        reasons,
        checkpoint_at: observation.now,
        stream_cursor: observation.stream_cursor,
        resource_counters: observation.resource_counters,
        retry_state: observation.retry_state,
        sink: CheckpointSink::SilentSessionRuntimeStore,
    }))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeaningSnapshot {
    pub workpoint_ref: WorkpointBinding,
    pub mission: String,
    pub action_intent: String,
    pub active_objects: BTreeSet<String>,
    pub blockers: BTreeSet<String>,
    pub verified_evidence: BTreeSet<String>,
    pub next_slice: String,
    pub work_item_ref: Option<String>,
    pub operator_direction_ref: String,
    pub model_binding_ref: String,
    pub completion_evaluation_started: bool,
}

impl MeaningSnapshot {
    fn validate(&self) -> Result<(), CheckpointPolicyError> {
        if self.workpoint_ref.workpoint_id.trim().is_empty()
            || self.mission.trim().is_empty()
            || self.action_intent.trim().is_empty()
            || self.next_slice.trim().is_empty()
            || self.operator_direction_ref.trim().is_empty()
            || self.model_binding_ref.trim().is_empty()
            || self
                .active_objects
                .iter()
                .any(|value| value.trim().is_empty())
            || self.blockers.iter().any(|value| value.trim().is_empty())
            || self
                .verified_evidence
                .iter()
                .any(|value| value.trim().is_empty())
            || self
                .work_item_ref
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
        {
            return Err(CheckpointPolicyError::MeaningStateMissing);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkpointCheckpointReason {
    InitialCanonicalBinding,
    MissionOrActionIntentChanged,
    ActiveObjectSetChanged,
    BlockersChanged,
    EvidenceChanged,
    NextSliceChanged,
    WorkItemAdvanced,
    OperatorSteeringChangedDirection,
    ModelSwitched,
    CompletionEvaluationBegan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkpointCheckpointDirective {
    pub workpoint_ref: WorkpointBinding,
    pub reasons: BTreeSet<WorkpointCheckpointReason>,
    pub snapshot: MeaningSnapshot,
    pub sink: CheckpointSink,
}

pub fn evaluate_workpoint_checkpoint(
    previous: Option<&MeaningSnapshot>,
    current: MeaningSnapshot,
) -> Result<WorkpointCheckpointDirective, CheckpointPolicyError> {
    current.validate()?;
    let mut reasons = BTreeSet::new();
    let Some(previous) = previous else {
        reasons.insert(WorkpointCheckpointReason::InitialCanonicalBinding);
        return Ok(WorkpointCheckpointDirective {
            workpoint_ref: current.workpoint_ref.clone(),
            reasons,
            snapshot: current,
            sink: CheckpointSink::CanonicalWorkpointPath,
        });
    };
    previous.validate()?;
    if previous.workpoint_ref != current.workpoint_ref {
        return Err(CheckpointPolicyError::WorkpointAuthorityChanged);
    }
    if previous.mission != current.mission || previous.action_intent != current.action_intent {
        reasons.insert(WorkpointCheckpointReason::MissionOrActionIntentChanged);
    }
    if previous.active_objects != current.active_objects {
        reasons.insert(WorkpointCheckpointReason::ActiveObjectSetChanged);
    }
    if previous.blockers != current.blockers {
        reasons.insert(WorkpointCheckpointReason::BlockersChanged);
    }
    if previous.verified_evidence != current.verified_evidence {
        reasons.insert(WorkpointCheckpointReason::EvidenceChanged);
    }
    if previous.next_slice != current.next_slice {
        reasons.insert(WorkpointCheckpointReason::NextSliceChanged);
    }
    if previous.work_item_ref != current.work_item_ref {
        reasons.insert(WorkpointCheckpointReason::WorkItemAdvanced);
    }
    if previous.operator_direction_ref != current.operator_direction_ref {
        reasons.insert(WorkpointCheckpointReason::OperatorSteeringChangedDirection);
    }
    if previous.model_binding_ref != current.model_binding_ref {
        reasons.insert(WorkpointCheckpointReason::ModelSwitched);
    }
    if !previous.completion_evaluation_started && current.completion_evaluation_started {
        reasons.insert(WorkpointCheckpointReason::CompletionEvaluationBegan);
    }
    if reasons.is_empty() {
        return Err(CheckpointPolicyError::MeaningUnchanged);
    }
    Ok(WorkpointCheckpointDirective {
        workpoint_ref: current.workpoint_ref.clone(),
        reasons,
        snapshot: current,
        sink: CheckpointSink::CanonicalWorkpointPath,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CheckpointPolicyError {
    #[error("runtime checkpoint cadence must be non-zero")]
    InvalidCadence,
    #[error("runtime checkpoint clock regressed")]
    ClockRegression,
    #[error("runtime cursor, resource counters, or retry state is missing")]
    RuntimeStateMissing,
    #[error("canonical meaning state is incomplete")]
    MeaningStateMissing,
    #[error("Workpoint authority cannot change inside checkpoint policy")]
    WorkpointAuthorityChanged,
    #[error("canonical Workpoint checkpoint is forbidden when meaning is unchanged")]
    MeaningUnchanged,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn runtime(now: DateTime<Utc>) -> RuntimeCheckpointObservation {
        RuntimeCheckpointObservation {
            now,
            last_checkpoint_at: now - Duration::seconds(61),
            semantic_events_since_checkpoint: 11,
            tool_completed: true,
            durable_project_change: true,
            retry_boundary: Some(RetryCheckpointBoundary::BeforeEscalation),
            lifecycle_boundary: Some(LifecycleCheckpointBoundary::BeforePause),
            stream_cursor: "stream:42".into(),
            resource_counters: BTreeMap::from([("rss_bytes".into(), 42)]),
            retry_state: json!({"transport": 2}),
        }
    }

    fn meaning() -> MeaningSnapshot {
        MeaningSnapshot {
            workpoint_ref: WorkpointBinding {
                workpoint_id: "workpoint:canonical".into(),
                revision: Some(7),
            },
            mission: "finish checkpoint policy".into(),
            action_intent: "implement".into(),
            active_objects: BTreeSet::from(["spec:133".into()]),
            blockers: BTreeSet::new(),
            verified_evidence: BTreeSet::from(["evidence:prior".into()]),
            next_slice: "implement completion bundle".into(),
            work_item_ref: Some("focusa-a6yq6.7.3".into()),
            operator_direction_ref: "steering:1".into(),
            model_binding_ref: "model:one".into(),
            completion_evaluation_started: false,
        }
    }

    #[test]
    fn runtime_policy_combines_all_due_reasons_into_one_runtime_store_checkpoint() {
        let now = Utc::now();
        let directive = evaluate_runtime_checkpoint(
            &RuntimeCheckpointPolicy {
                interval_seconds: 60,
                semantic_event_interval: 10,
            },
            runtime(now),
        )
        .unwrap()
        .unwrap();
        assert_eq!(directive.sink, CheckpointSink::SilentSessionRuntimeStore);
        assert!(
            directive
                .reasons
                .contains(&RuntimeCheckpointReason::TimeInterval)
        );
        assert!(
            directive
                .reasons
                .contains(&RuntimeCheckpointReason::SemanticEventInterval)
        );
        assert!(
            directive
                .reasons
                .contains(&RuntimeCheckpointReason::DurableToolChange)
        );
        assert!(
            directive
                .reasons
                .contains(&RuntimeCheckpointReason::BeforeRetryEscalation)
        );
        assert!(
            directive
                .reasons
                .contains(&RuntimeCheckpointReason::BeforePause)
        );
    }

    #[test]
    fn all_lifecycle_and_retry_boundaries_trigger_runtime_only_checkpoints() {
        let now = Utc::now();
        for retry in [
            RetryCheckpointBoundary::BeforeEscalation,
            RetryCheckpointBoundary::AfterEscalation,
        ] {
            let mut observation = runtime(now);
            observation.last_checkpoint_at = now;
            observation.semantic_events_since_checkpoint = 0;
            observation.tool_completed = false;
            observation.retry_boundary = Some(retry);
            observation.lifecycle_boundary = None;
            assert_eq!(
                evaluate_runtime_checkpoint(
                    &RuntimeCheckpointPolicy {
                        interval_seconds: 60,
                        semantic_event_interval: 10,
                    },
                    observation,
                )
                .unwrap()
                .unwrap()
                .sink,
                CheckpointSink::SilentSessionRuntimeStore
            );
        }
        for lifecycle in [
            LifecycleCheckpointBoundary::BeforePause,
            LifecycleCheckpointBoundary::BeforeProcessRestart,
            LifecycleCheckpointBoundary::BeforeDaemonUpgrade,
            LifecycleCheckpointBoundary::RunnerDisconnect,
        ] {
            let mut observation = runtime(now);
            observation.last_checkpoint_at = now;
            observation.semantic_events_since_checkpoint = 0;
            observation.tool_completed = false;
            observation.retry_boundary = None;
            observation.lifecycle_boundary = Some(lifecycle);
            assert!(
                evaluate_runtime_checkpoint(
                    &RuntimeCheckpointPolicy {
                        interval_seconds: 60,
                        semantic_event_interval: 10,
                    },
                    observation,
                )
                .unwrap()
                .is_some()
            );
        }
    }

    #[test]
    fn workpoint_checkpoint_is_forbidden_when_meaning_is_unchanged() {
        let current = meaning();
        assert_eq!(
            evaluate_workpoint_checkpoint(Some(&current), current.clone()),
            Err(CheckpointPolicyError::MeaningUnchanged)
        );
    }

    #[test]
    fn every_canonical_trigger_is_detected_without_minting_new_authority() {
        let previous = meaning();
        let mut current = previous.clone();
        current.mission = "changed mission".into();
        current.active_objects.insert("file:new".into());
        current.blockers.insert("blocker:new".into());
        current.verified_evidence.insert("evidence:new".into());
        current.next_slice = "changed next".into();
        current.work_item_ref = Some("focusa-a6yq6.7.4".into());
        current.operator_direction_ref = "steering:2".into();
        current.model_binding_ref = "model:two".into();
        current.completion_evaluation_started = true;
        let directive = evaluate_workpoint_checkpoint(Some(&previous), current).unwrap();
        assert_eq!(directive.sink, CheckpointSink::CanonicalWorkpointPath);
        assert_eq!(directive.workpoint_ref, previous.workpoint_ref);
        assert_eq!(directive.reasons.len(), 9);
    }

    #[test]
    fn set_order_changes_do_not_create_checkpoint_spam() {
        let previous = meaning();
        let current = MeaningSnapshot {
            active_objects: previous.active_objects.iter().cloned().collect(),
            blockers: previous.blockers.iter().cloned().collect(),
            verified_evidence: previous.verified_evidence.iter().cloned().collect(),
            ..previous.clone()
        };
        assert_eq!(
            evaluate_workpoint_checkpoint(Some(&previous), current),
            Err(CheckpointPolicyError::MeaningUnchanged)
        );
    }
}
