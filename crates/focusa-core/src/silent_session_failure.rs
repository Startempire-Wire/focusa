//! Closed Spec 133 failure taxonomy and recovery envelope.

use crate::silent_session::SilentSessionLifecycleState;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const SILENT_SESSION_FAILURE_SCHEMA: &str = "focusa.silent_session_failure.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SilentSessionFailureClass {
    ScopeMismatch,
    ProjectIdentityUnverified,
    ContinuityMissing,
    WorkpointUnavailable,
    WriterConflict,
    WorkspaceConflict,
    AuthorizationRequired,
    PermissionDenied,
    ApprovalExpired,
    ContextAuthorityBlocked,
    ConfigInvalid,
    ConfigLocked,
    ModelNotFound,
    ModelEntitlementUnverified,
    ModelMismatch,
    FallbackDisallowed,
    HarnessUnsupported,
    BackendUnsupported,
    CapabilityMissing,
    RunnerUnavailable,
    RunnerLost,
    ProcessSpawnFailed,
    ProcessControlFailed,
    ProcessExited,
    ChildLeakDetected,
    TransportDegraded,
    TransportLost,
    WaitingInput,
    ProviderFailure,
    RetryExhausted,
    ResourceAdmissionDenied,
    ResourceLimitExceeded,
    OutputStoragePressure,
    StreamCorruption,
    CheckpointFailed,
    EvidenceMissing,
    VerificationFailed,
    CompletionEvidenceMissing,
    ReceiptCommitFailed,
    OrphanAdoptionRejected,
    ProtocolIncompatible,
    RetentionBlockedByHold,
}

pub const ALL_SILENT_SESSION_FAILURE_CLASSES: [SilentSessionFailureClass; 42] = [
    SilentSessionFailureClass::ScopeMismatch,
    SilentSessionFailureClass::ProjectIdentityUnverified,
    SilentSessionFailureClass::ContinuityMissing,
    SilentSessionFailureClass::WorkpointUnavailable,
    SilentSessionFailureClass::WriterConflict,
    SilentSessionFailureClass::WorkspaceConflict,
    SilentSessionFailureClass::AuthorizationRequired,
    SilentSessionFailureClass::PermissionDenied,
    SilentSessionFailureClass::ApprovalExpired,
    SilentSessionFailureClass::ContextAuthorityBlocked,
    SilentSessionFailureClass::ConfigInvalid,
    SilentSessionFailureClass::ConfigLocked,
    SilentSessionFailureClass::ModelNotFound,
    SilentSessionFailureClass::ModelEntitlementUnverified,
    SilentSessionFailureClass::ModelMismatch,
    SilentSessionFailureClass::FallbackDisallowed,
    SilentSessionFailureClass::HarnessUnsupported,
    SilentSessionFailureClass::BackendUnsupported,
    SilentSessionFailureClass::CapabilityMissing,
    SilentSessionFailureClass::RunnerUnavailable,
    SilentSessionFailureClass::RunnerLost,
    SilentSessionFailureClass::ProcessSpawnFailed,
    SilentSessionFailureClass::ProcessControlFailed,
    SilentSessionFailureClass::ProcessExited,
    SilentSessionFailureClass::ChildLeakDetected,
    SilentSessionFailureClass::TransportDegraded,
    SilentSessionFailureClass::TransportLost,
    SilentSessionFailureClass::WaitingInput,
    SilentSessionFailureClass::ProviderFailure,
    SilentSessionFailureClass::RetryExhausted,
    SilentSessionFailureClass::ResourceAdmissionDenied,
    SilentSessionFailureClass::ResourceLimitExceeded,
    SilentSessionFailureClass::OutputStoragePressure,
    SilentSessionFailureClass::StreamCorruption,
    SilentSessionFailureClass::CheckpointFailed,
    SilentSessionFailureClass::EvidenceMissing,
    SilentSessionFailureClass::VerificationFailed,
    SilentSessionFailureClass::CompletionEvidenceMissing,
    SilentSessionFailureClass::ReceiptCommitFailed,
    SilentSessionFailureClass::OrphanAdoptionRejected,
    SilentSessionFailureClass::ProtocolIncompatible,
    SilentSessionFailureClass::RetentionBlockedByHold,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalFailurePosture {
    Intact,
    Degraded,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeFailurePosture {
    Healthy,
    Degraded,
    Blocked,
    WaitingInput,
    Paused,
    Stopped,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureRetryPosture {
    SafeAfterRecovery,
    SafeWithFreshApproval,
    SafeWithNewRunGeneration,
    WaitForOperator,
    Exhausted,
    NotRetryable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureSideEffect {
    pub kind: String,
    pub artifact_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentSessionFailureEnvelope {
    pub schema: String,
    pub failure_class: SilentSessionFailureClass,
    pub why: String,
    pub current_lifecycle: SilentSessionLifecycleState,
    pub canonical_posture: CanonicalFailurePosture,
    pub runtime_posture: RuntimeFailurePosture,
    pub retry_posture: FailureRetryPosture,
    pub side_effects: Vec<FailureSideEffect>,
    pub exact_recovery_tools: Vec<String>,
    pub operator_action_required: bool,
}

impl SilentSessionFailureEnvelope {
    pub fn new(
        failure_class: SilentSessionFailureClass,
        why: impl Into<String>,
        current_lifecycle: SilentSessionLifecycleState,
        side_effects: Vec<FailureSideEffect>,
    ) -> Result<Self, FailureEnvelopeError> {
        let why = why.into();
        if why.trim().is_empty() {
            return Err(FailureEnvelopeError::WhyMissing);
        }
        let policy = failure_policy(failure_class);
        let envelope = Self {
            schema: SILENT_SESSION_FAILURE_SCHEMA.into(),
            failure_class,
            why,
            current_lifecycle,
            canonical_posture: policy.canonical_posture,
            runtime_posture: policy.runtime_posture,
            retry_posture: policy.retry_posture,
            side_effects,
            exact_recovery_tools: policy
                .recovery_tools
                .iter()
                .map(|value| (*value).to_owned())
                .collect(),
            operator_action_required: policy.operator_action_required,
        };
        envelope.validate()?;
        Ok(envelope)
    }

    pub fn validate(&self) -> Result<(), FailureEnvelopeError> {
        if self.schema != SILENT_SESSION_FAILURE_SCHEMA {
            return Err(FailureEnvelopeError::InvalidSchema);
        }
        if self.why.trim().is_empty() {
            return Err(FailureEnvelopeError::WhyMissing);
        }
        if self.exact_recovery_tools.is_empty()
            || self
                .exact_recovery_tools
                .iter()
                .any(|tool| tool.trim().is_empty())
        {
            return Err(FailureEnvelopeError::RecoveryToolsMissing);
        }
        if self
            .side_effects
            .iter()
            .any(|effect| effect.kind.trim().is_empty())
        {
            return Err(FailureEnvelopeError::InvalidSideEffect);
        }
        Ok(())
    }
}

struct FailurePolicy {
    canonical_posture: CanonicalFailurePosture,
    runtime_posture: RuntimeFailurePosture,
    retry_posture: FailureRetryPosture,
    recovery_tools: &'static [&'static str],
    operator_action_required: bool,
}

fn failure_policy(class: SilentSessionFailureClass) -> FailurePolicy {
    use SilentSessionFailureClass as F;
    match class {
        F::ScopeMismatch | F::ProjectIdentityUnverified => policy(
            CanonicalFailurePosture::Blocked,
            RuntimeFailurePosture::Blocked,
            FailureRetryPosture::SafeAfterRecovery,
            &["focusa_project_verify", "focusa_project_identity"],
            false,
        ),
        F::ContinuityMissing | F::WorkpointUnavailable => policy(
            CanonicalFailurePosture::Blocked,
            RuntimeFailurePosture::Blocked,
            FailureRetryPosture::SafeAfterRecovery,
            &["focusa_workpoint_resume", "focusa_workpoint_checkpoint"],
            false,
        ),
        F::WriterConflict | F::WorkspaceConflict => policy(
            CanonicalFailurePosture::Blocked,
            RuntimeFailurePosture::Blocked,
            FailureRetryPosture::WaitForOperator,
            &["focusa_work_loop_writer_status", "focusa silent status"],
            true,
        ),
        F::AuthorizationRequired | F::PermissionDenied | F::ApprovalExpired => policy(
            CanonicalFailurePosture::Blocked,
            RuntimeFailurePosture::Blocked,
            FailureRetryPosture::SafeWithFreshApproval,
            &["focusa silent approvals", "focusa silent status"],
            true,
        ),
        F::ContextAuthorityBlocked => policy(
            CanonicalFailurePosture::Blocked,
            RuntimeFailurePosture::Blocked,
            FailureRetryPosture::SafeAfterRecovery,
            &["focusa_context_cognition", "focusa_workpoint_resume"],
            false,
        ),
        F::ConfigInvalid | F::ConfigLocked => policy(
            CanonicalFailurePosture::Blocked,
            RuntimeFailurePosture::Blocked,
            FailureRetryPosture::WaitForOperator,
            &["focusa silent config validate", "focusa silent config diff"],
            true,
        ),
        F::ModelNotFound
        | F::ModelEntitlementUnverified
        | F::ModelMismatch
        | F::FallbackDisallowed => policy(
            CanonicalFailurePosture::Intact,
            RuntimeFailurePosture::Blocked,
            FailureRetryPosture::WaitForOperator,
            &["focusa silent models", "focusa silent config diff"],
            true,
        ),
        F::HarnessUnsupported | F::BackendUnsupported | F::CapabilityMissing => policy(
            CanonicalFailurePosture::Intact,
            RuntimeFailurePosture::Unavailable,
            FailureRetryPosture::NotRetryable,
            &["focusa silent capabilities", "focusa silent status"],
            true,
        ),
        F::RunnerUnavailable | F::RunnerLost => policy(
            CanonicalFailurePosture::Intact,
            RuntimeFailurePosture::Unavailable,
            FailureRetryPosture::SafeAfterRecovery,
            &["focusa silent runners", "focusa silent recover"],
            false,
        ),
        F::ProcessSpawnFailed | F::ProcessControlFailed | F::ChildLeakDetected => policy(
            CanonicalFailurePosture::Intact,
            RuntimeFailurePosture::Stopped,
            FailureRetryPosture::SafeWithNewRunGeneration,
            &["focusa silent inspect", "focusa silent recover"],
            true,
        ),
        F::ProcessExited => policy(
            CanonicalFailurePosture::Intact,
            RuntimeFailurePosture::Stopped,
            FailureRetryPosture::SafeWithNewRunGeneration,
            &["focusa silent inspect", "focusa silent restart"],
            false,
        ),
        F::TransportDegraded | F::TransportLost => policy(
            CanonicalFailurePosture::Intact,
            RuntimeFailurePosture::Degraded,
            FailureRetryPosture::SafeAfterRecovery,
            &["focusa silent reconnect", "focusa silent status"],
            false,
        ),
        F::WaitingInput => policy(
            CanonicalFailurePosture::Intact,
            RuntimeFailurePosture::WaitingInput,
            FailureRetryPosture::WaitForOperator,
            &["focusa silent watch", "focusa silent send"],
            true,
        ),
        F::ProviderFailure => policy(
            CanonicalFailurePosture::Intact,
            RuntimeFailurePosture::Degraded,
            FailureRetryPosture::SafeAfterRecovery,
            &["focusa silent inspect", "focusa silent retry"],
            false,
        ),
        F::RetryExhausted => policy(
            CanonicalFailurePosture::Intact,
            RuntimeFailurePosture::Blocked,
            FailureRetryPosture::Exhausted,
            &["focusa silent inspect", "focusa silent config diff"],
            true,
        ),
        F::ResourceAdmissionDenied => policy(
            CanonicalFailurePosture::Intact,
            RuntimeFailurePosture::Blocked,
            FailureRetryPosture::SafeAfterRecovery,
            &["focusa_resource_mode", "focusa silent resources"],
            false,
        ),
        F::ResourceLimitExceeded | F::OutputStoragePressure => policy(
            CanonicalFailurePosture::Intact,
            RuntimeFailurePosture::Paused,
            FailureRetryPosture::WaitForOperator,
            &["focusa silent checkpoint", "focusa silent resources"],
            true,
        ),
        F::StreamCorruption | F::CheckpointFailed => policy(
            CanonicalFailurePosture::Degraded,
            RuntimeFailurePosture::Degraded,
            FailureRetryPosture::SafeAfterRecovery,
            &["focusa silent recover", "focusa silent inspect"],
            true,
        ),
        F::EvidenceMissing | F::VerificationFailed | F::CompletionEvidenceMissing => policy(
            CanonicalFailurePosture::Blocked,
            RuntimeFailurePosture::Blocked,
            FailureRetryPosture::SafeAfterRecovery,
            &["focusa_evidence_capture", "focusa silent verify"],
            false,
        ),
        F::ReceiptCommitFailed => policy(
            CanonicalFailurePosture::Blocked,
            RuntimeFailurePosture::Degraded,
            FailureRetryPosture::SafeAfterRecovery,
            &["focusa silent receipts", "focusa silent verify"],
            true,
        ),
        F::OrphanAdoptionRejected => policy(
            CanonicalFailurePosture::Intact,
            RuntimeFailurePosture::Stopped,
            FailureRetryPosture::SafeWithNewRunGeneration,
            &["focusa silent recover", "focusa silent inspect"],
            true,
        ),
        F::ProtocolIncompatible => policy(
            CanonicalFailurePosture::Intact,
            RuntimeFailurePosture::Unavailable,
            FailureRetryPosture::NotRetryable,
            &["focusa silent capabilities", "focusa silent migrations"],
            true,
        ),
        F::RetentionBlockedByHold => policy(
            CanonicalFailurePosture::Blocked,
            RuntimeFailurePosture::Healthy,
            FailureRetryPosture::WaitForOperator,
            &["focusa silent holds", "focusa silent retention"],
            true,
        ),
    }
}

const fn policy(
    canonical_posture: CanonicalFailurePosture,
    runtime_posture: RuntimeFailurePosture,
    retry_posture: FailureRetryPosture,
    recovery_tools: &'static [&'static str],
    operator_action_required: bool,
) -> FailurePolicy {
    FailurePolicy {
        canonical_posture,
        runtime_posture,
        retry_posture,
        recovery_tools,
        operator_action_required,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FailureEnvelopeError {
    #[error("silent-session failure envelope schema is invalid")]
    InvalidSchema,
    #[error("silent-session failure envelope requires a concrete why")]
    WhyMissing,
    #[error("silent-session failure envelope requires exact recovery tools")]
    RecoveryToolsMissing,
    #[error("silent-session failure side effects require a typed kind")]
    InvalidSideEffect,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn every_required_failure_class_has_a_complete_validated_envelope() {
        assert_eq!(ALL_SILENT_SESSION_FAILURE_CLASSES.len(), 42);
        assert_eq!(
            ALL_SILENT_SESSION_FAILURE_CLASSES
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .len(),
            42
        );
        for class in ALL_SILENT_SESSION_FAILURE_CLASSES {
            let envelope = SilentSessionFailureEnvelope::new(
                class,
                format!("verified cause for {class:?}"),
                SilentSessionLifecycleState::Running,
                vec![FailureSideEffect {
                    kind: "event_appended".into(),
                    artifact_ref: Some("event:test".into()),
                }],
            )
            .unwrap();
            envelope.validate().unwrap();
            let encoded = serde_json::to_value(&envelope).unwrap();
            assert_eq!(encoded["schema"], SILENT_SESSION_FAILURE_SCHEMA);
            assert!(encoded["failure_class"].as_str().is_some());
            assert!(!envelope.exact_recovery_tools.is_empty());
        }
    }

    #[test]
    fn retry_exhaustion_waiting_input_and_unknown_side_effects_are_truthful() {
        let exhausted = SilentSessionFailureEnvelope::new(
            SilentSessionFailureClass::RetryExhausted,
            "runner reconnect budget exhausted",
            SilentSessionLifecycleState::Blocked,
            vec![],
        )
        .unwrap();
        assert_eq!(exhausted.retry_posture, FailureRetryPosture::Exhausted);
        assert!(exhausted.operator_action_required);

        let waiting = SilentSessionFailureEnvelope::new(
            SilentSessionFailureClass::WaitingInput,
            "provider requested operator confirmation",
            SilentSessionLifecycleState::WaitingInput,
            vec![],
        )
        .unwrap();
        assert_eq!(waiting.runtime_posture, RuntimeFailurePosture::WaitingInput);
        assert_eq!(waiting.retry_posture, FailureRetryPosture::WaitForOperator);

        assert_eq!(
            SilentSessionFailureEnvelope::new(
                SilentSessionFailureClass::ProviderFailure,
                "provider failed",
                SilentSessionLifecycleState::Running,
                vec![FailureSideEffect {
                    kind: "".into(),
                    artifact_ref: None,
                }],
            ),
            Err(FailureEnvelopeError::InvalidSideEffect)
        );
    }
}
