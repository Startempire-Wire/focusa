//! Complete deterministic Spec133 failure and recovery envelope.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SilentFailureClass {
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalRuntimePosture {
    CanonicalOnly,
    RuntimeConfirmed,
    RuntimeUnknown,
    RuntimeAbsent,
    Diverged,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SafeRetryPosture {
    SameIdempotencyKey,
    RefreshExactTarget,
    AfterRecovery,
    NeverAutomatic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SilentFailureEnvelope {
    pub failure_class: SilentFailureClass,
    pub why: String,
    pub current_lifecycle: String,
    pub canonical_runtime_posture: CanonicalRuntimePosture,
    pub safe_retry_posture: SafeRetryPosture,
    pub side_effects_performed: Vec<String>,
    pub exact_recovery_tools: Vec<String>,
    pub operator_action_required: bool,
}

impl SilentFailureEnvelope {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(!self.why.trim().is_empty(), "failure why is required");
        anyhow::ensure!(
            !self.current_lifecycle.trim().is_empty(),
            "current lifecycle is required"
        );
        anyhow::ensure!(
            !self.exact_recovery_tools.is_empty(),
            "exact recovery tools are required"
        );
        anyhow::ensure!(
            self.exact_recovery_tools
                .iter()
                .all(|tool| tool.starts_with("focusa_")),
            "recovery tools must be exact Focusa tool names"
        );
        Ok(())
    }
}
