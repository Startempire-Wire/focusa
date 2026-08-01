use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{
    ArtifactTrustEvidence, CanvasSelection, ComponentVersion, GitSelection,
    InstallLifecycleValidationError, InstructionSelection, IntegrationSelection, LifecycleScope,
    LifecycleSelections, LifecycleState, LifecycleTransactionKind, MaintenanceAction,
    PreflightReport, PreservationDeclaration, ProjectSelection, RecoveryInstructions,
    RollbackBoundary, TaskProviderSelection,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionProgress {
    Pending,
    InProgress,
    Blocked,
    Partial,
    Complete,
    RolledBack,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdempotencyRecord {
    pub key: String,
    pub intent_digest: String,
    pub replay_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayDisposition {
    Start,
    Resume,
    ReturnStoredReceipt,
    InspectBeforeResume,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedLifecycleState {
    pub transaction_id: String,
    pub transaction_kind: LifecycleTransactionKind,
    pub scope: LifecycleScope,
    pub idempotency: IdempotencyRecord,
    pub current_state: LifecycleState,
    pub progress: TransactionProgress,
    pub last_completed_action: Option<String>,
    #[serde(default)]
    pub transition_refs: Vec<String>,
    pub stored_receipt_ref: Option<String>,
    pub completion_known: bool,
    pub recovery: Option<RecoveryInstructions>,
    pub rollback: RollbackBoundary,
    pub updated_at: DateTime<Utc>,
}

impl PersistedLifecycleState {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        if self.transaction_id.trim().is_empty() {
            return Err(InstallLifecycleValidationError::EmptyTransactionId);
        }
        if self.scope.host_id.trim().is_empty() {
            return Err(InstallLifecycleValidationError::EmptyHostId);
        }
        if self.idempotency.key.trim().is_empty()
            || self.idempotency.intent_digest.trim().is_empty()
        {
            return Err(InstallLifecycleValidationError::IdempotencyKeyRequired);
        }
        self.rollback.validate()?;
        if self.current_state.is_recovery() {
            self.recovery
                .as_ref()
                .ok_or(InstallLifecycleValidationError::RecoveryRequiresGuidance)?
                .validate()?;
        }
        if self.progress == TransactionProgress::Complete
            && (!self.completion_known || self.stored_receipt_ref.is_none())
        {
            return Err(InstallLifecycleValidationError::CompletedTransactionRequiresReceipt);
        }
        Ok(())
    }

    pub fn replay(
        &self,
        key: &str,
        intent_digest: &str,
    ) -> Result<ReplayDisposition, InstallLifecycleValidationError> {
        if key != self.idempotency.key || intent_digest != self.idempotency.intent_digest {
            return Err(InstallLifecycleValidationError::IdempotencyConflict);
        }
        if !self.completion_known {
            return Ok(ReplayDisposition::InspectBeforeResume);
        }
        if self.progress == TransactionProgress::Complete {
            return if self.stored_receipt_ref.is_some() {
                Ok(ReplayDisposition::ReturnStoredReceipt)
            } else {
                Err(InstallLifecycleValidationError::CompletedTransactionRequiresReceipt)
            };
        }
        Ok(ReplayDisposition::Resume)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostInstallIntent {
    pub selections: LifecycleSelections,
    pub preflight: PreflightReport,
    pub artifact: ArtifactTrustEvidence,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectOnboardingIntent {
    pub selections: LifecycleSelections,
    pub exact_scope: LifecycleScope,
    pub bootstrap_preview_ref: String,
    pub mutation_confirmation_ref: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleMaintenanceIntent {
    pub action: MaintenanceAction,
    pub selections: LifecycleSelections,
    pub preservation: PreservationDeclaration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostInstallTransaction {
    pub intent: HostInstallIntent,
    pub persisted: PersistedLifecycleState,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectOnboardingTransaction {
    pub intent: ProjectOnboardingIntent,
    pub persisted: PersistedLifecycleState,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleMaintenanceTransaction {
    pub intent: LifecycleMaintenanceIntent,
    pub persisted: PersistedLifecycleState,
}

impl HostInstallTransaction {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        self.persisted.validate()?;
        if self.persisted.transaction_kind != LifecycleTransactionKind::HostInstall {
            return Err(InstallLifecycleValidationError::TransactionKindMismatch);
        }
        if self.persisted.scope.project_root.is_some()
            || self.persisted.scope.continuity_id.is_some()
        {
            return Err(InstallLifecycleValidationError::ProjectScopeForbiddenForHostInstall);
        }
        if self.intent.selections.project != ProjectSelection::Skip {
            return Err(InstallLifecycleValidationError::ProjectSelectionForbiddenForHostInstall);
        }
        self.intent.selections.validate()?;
        self.intent.preflight.validate()?;
        if self.intent.preflight.host_id != self.persisted.scope.host_id {
            return Err(InstallLifecycleValidationError::ScopeMismatch);
        }
        if self.intent.artifact.target != self.intent.selections.target
            || self.intent.artifact.declared_channel != self.intent.selections.channel
            || self.intent.preflight.supported_target.as_deref()
                != Some(self.intent.selections.target.as_str())
        {
            return Err(InstallLifecycleValidationError::ArtifactSelectionMismatch);
        }
        self.intent.artifact.validate()?;
        Ok(())
    }
}
impl ProjectOnboardingTransaction {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        self.persisted.validate()?;
        if self.persisted.transaction_kind != LifecycleTransactionKind::ProjectOnboarding {
            return Err(InstallLifecycleValidationError::TransactionKindMismatch);
        }
        self.intent.selections.validate()?;
        if self.persisted.scope != self.intent.exact_scope
            || self.intent.exact_scope.project_root.is_none()
            || self.intent.exact_scope.continuity_id.is_none()
        {
            return Err(InstallLifecycleValidationError::ProjectScopeRequiredForOnboarding);
        }
        let selected_path = self.intent.selections.project.explicit_path();
        if selected_path != self.intent.exact_scope.project_root.as_deref() {
            return Err(InstallLifecycleValidationError::ScopeMismatch);
        }
        if self.intent.bootstrap_preview_ref.trim().is_empty() {
            return Err(InstallLifecycleValidationError::BootstrapPreviewRequired);
        }
        if self.persisted.current_state == LifecycleState::ProjectBootstrapped
            && self.intent.mutation_confirmation_ref.is_none()
        {
            return Err(InstallLifecycleValidationError::MutationConfirmationRequired);
        }
        Ok(())
    }
}
impl LifecycleMaintenanceTransaction {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        self.persisted.validate()?;
        if self.persisted.transaction_kind != LifecycleTransactionKind::LifecycleMaintenance {
            return Err(InstallLifecycleValidationError::TransactionKindMismatch);
        }
        self.intent.selections.validate()?;
        self.intent.preservation.validate()?;
        if self.intent.action != self.intent.preservation.action {
            return Err(InstallLifecycleValidationError::MaintenanceActionMismatch);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceHealthEvidence {
    pub required: bool,
    pub healthy: bool,
    pub posture: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformEvidence {
    pub target: String,
    pub mechanism: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationOutcome {
    pub integration: IntegrationSelection,
    pub status: String,
    pub required: bool,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionProof {
    pub artifact_trust: ArtifactTrustEvidence,
    pub version_set: Vec<ComponentVersion>,
    pub daemon_healthy: bool,
    #[serde(default)]
    pub daemon_health_refs: Vec<String>,
    pub service: ServiceHealthEvidence,
    pub expected_scope: LifecycleScope,
    pub observed_scope: LifecycleScope,
    #[serde(default)]
    pub scope_evidence_refs: Vec<String>,
    pub mutation_required_confirmation: bool,
    pub mutation_confirmation_ref: Option<String>,
    pub workpoint_required: bool,
    pub workpoint_ref: Option<String>,
    pub secret_values_detected: bool,
    #[serde(default)]
    pub secret_handling_refs: Vec<String>,
    pub rollback: RollbackBoundary,
    pub preservation: PreservationDeclaration,
    #[serde(default)]
    pub platform_evidence: Vec<PlatformEvidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FalseCompletionReason {
    ArtifactTrustAbsent,
    VersionSetIncompatible,
    DaemonUnhealthy,
    RequiredServiceUnhealthy,
    ExactScopeUnproven,
    MutationConfirmationMissing,
    WorkpointMissing,
    SecretLeakDetected,
    PreservationAmbiguous,
    RollbackUnavailable,
    PlatformEvidenceMissing,
    ReceiptPayloadIncomplete,
}

impl CompletionProof {
    pub fn false_completion_reasons(&self) -> Vec<FalseCompletionReason> {
        let mut reasons = Vec::new();
        if !self.artifact_trust.is_complete() {
            reasons.push(FalseCompletionReason::ArtifactTrustAbsent);
        }
        if self.version_set.is_empty()
            || self
                .version_set
                .iter()
                .any(|item| !item.compatible || item.evidence_refs.is_empty())
        {
            reasons.push(FalseCompletionReason::VersionSetIncompatible);
        }
        if !self.daemon_healthy || self.daemon_health_refs.is_empty() {
            reasons.push(FalseCompletionReason::DaemonUnhealthy);
        }
        if self.service.required && (!self.service.healthy || self.service.evidence_refs.is_empty())
        {
            reasons.push(FalseCompletionReason::RequiredServiceUnhealthy);
        }
        if self.expected_scope != self.observed_scope || self.scope_evidence_refs.is_empty() {
            reasons.push(FalseCompletionReason::ExactScopeUnproven);
        }
        if self.mutation_required_confirmation
            && self
                .mutation_confirmation_ref
                .as_deref()
                .is_none_or(str::is_empty)
        {
            reasons.push(FalseCompletionReason::MutationConfirmationMissing);
        }
        if self.workpoint_required && self.workpoint_ref.as_deref().is_none_or(str::is_empty) {
            reasons.push(FalseCompletionReason::WorkpointMissing);
        }
        if self.secret_values_detected || self.secret_handling_refs.is_empty() {
            reasons.push(FalseCompletionReason::SecretLeakDetected);
        }
        if self.preservation.validate().is_err() {
            reasons.push(FalseCompletionReason::PreservationAmbiguous);
        }
        if !self.rollback.rollback_available() {
            reasons.push(FalseCompletionReason::RollbackUnavailable);
        }
        if self.platform_evidence.is_empty()
            || self.platform_evidence.iter().any(|item| {
                item.target.trim().is_empty()
                    || item.mechanism.trim().is_empty()
                    || item.evidence_refs.is_empty()
            })
        {
            reasons.push(FalseCompletionReason::PlatformEvidenceMissing);
        }
        reasons
    }

    pub fn validate(&self) -> Result<(), Vec<FalseCompletionReason>> {
        let reasons = self.false_completion_reasons();
        if reasons.is_empty() {
            Ok(())
        } else {
            Err(reasons)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostInstallReceiptPayload {
    pub target: String,
    pub version_set: Vec<ComponentVersion>,
    pub artifact_trust_refs: Vec<String>,
    pub daemon_health_refs: Vec<String>,
    pub service_posture: String,
    pub integration_outcomes: Vec<IntegrationOutcome>,
    pub preservation: PreservationDeclaration,
    pub recovery: Option<RecoveryInstructions>,
    pub rollback: RollbackBoundary,
    pub update_action: Option<MaintenanceAction>,
    pub uninstall_action: Option<MaintenanceAction>,
    pub completion: CompletionProof,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectOnboardingReceiptPayload {
    pub exact_scope: LifecycleScope,
    pub bootstrap_refs: Vec<String>,
    pub git: GitSelection,
    pub task_provider: TaskProviderSelection,
    pub instructions: InstructionSelection,
    pub genesis_status: String,
    pub hlt_status: String,
    pub workpoint_ref: String,
    pub canvas: CanvasSelection,
    pub deferred_optional_work: Vec<String>,
    pub completion: CompletionProof,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleMaintenanceReceiptPayload {
    pub action: MaintenanceAction,
    pub preservation: PreservationDeclaration,
    pub rollback: RollbackBoundary,
    pub recovery: Option<RecoveryInstructions>,
    pub completion: CompletionProof,
}

fn finish_receipt_validation(
    mut reasons: Vec<FalseCompletionReason>,
) -> Result<(), Vec<FalseCompletionReason>> {
    reasons.sort_by_key(|reason| *reason as u8);
    reasons.dedup();
    if reasons.is_empty() {
        Ok(())
    } else {
        Err(reasons)
    }
}

impl HostInstallReceiptPayload {
    pub fn validate(&self) -> Result<(), Vec<FalseCompletionReason>> {
        let mut reasons = self.completion.false_completion_reasons();
        if self.target.trim().is_empty()
            || self.target != self.completion.artifact_trust.target
            || self.artifact_trust_refs.is_empty()
        {
            reasons.push(FalseCompletionReason::ArtifactTrustAbsent);
        }
        if self.version_set.is_empty() || self.version_set != self.completion.version_set {
            reasons.push(FalseCompletionReason::VersionSetIncompatible);
        }
        if self.daemon_health_refs.is_empty()
            || self.daemon_health_refs != self.completion.daemon_health_refs
        {
            reasons.push(FalseCompletionReason::DaemonUnhealthy);
        }
        if self.service_posture.trim().is_empty() {
            reasons.push(FalseCompletionReason::RequiredServiceUnhealthy);
        }
        if self.preservation != self.completion.preservation {
            reasons.push(FalseCompletionReason::PreservationAmbiguous);
        }
        if self.rollback != self.completion.rollback {
            reasons.push(FalseCompletionReason::RollbackUnavailable);
        }
        finish_receipt_validation(reasons)
    }
}
impl ProjectOnboardingReceiptPayload {
    pub fn validate(&self) -> Result<(), Vec<FalseCompletionReason>> {
        let mut reasons = self.completion.false_completion_reasons();
        if self.exact_scope != self.completion.expected_scope
            || self.exact_scope.project_root.is_none()
            || self.exact_scope.continuity_id.is_none()
        {
            reasons.push(FalseCompletionReason::ExactScopeUnproven);
        }
        if self.workpoint_ref.trim().is_empty()
            || self.completion.workpoint_ref.as_deref() != Some(self.workpoint_ref.as_str())
        {
            reasons.push(FalseCompletionReason::WorkpointMissing);
        }
        if self.bootstrap_refs.is_empty()
            || self.genesis_status.trim().is_empty()
            || self.hlt_status.trim().is_empty()
        {
            reasons.push(FalseCompletionReason::ReceiptPayloadIncomplete);
        }
        finish_receipt_validation(reasons)
    }
}
impl LifecycleMaintenanceReceiptPayload {
    pub fn validate(&self) -> Result<(), Vec<FalseCompletionReason>> {
        let mut reasons = self.completion.false_completion_reasons();
        if self.preservation.validate().is_err()
            || self.preservation != self.completion.preservation
        {
            reasons.push(FalseCompletionReason::PreservationAmbiguous);
        }
        if !self.rollback.rollback_available() || self.rollback != self.completion.rollback {
            reasons.push(FalseCompletionReason::RollbackUnavailable);
        }
        if self.action != self.preservation.action {
            reasons.push(FalseCompletionReason::ReceiptPayloadIncomplete);
        }
        finish_receipt_validation(reasons)
    }
}
