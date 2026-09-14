use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{ChannelSelection, InstallLifecycleValidationError, LifecycleState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreflightFindingDisposition {
    Required,
    Optional,
    AlreadySatisfied,
    OperatorChoice,
    Unsupported,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreflightSubject {
    HostTarget,
    UserHomeBoundary,
    Binary,
    Daemon,
    Service,
    Extension,
    Skill,
    FocusaState,
    Dependency,
    Network,
    ArtifactMetadata,
    License,
    PiCapability,
    UiaiCapability,
    MenubarCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicensePosture {
    VerifiedLimitedAccess,
    Commercial,
    AuthorizedDevelopment,
    ActivationRequired,
    Blocked,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightFinding {
    pub finding_id: String,
    pub subject: PreflightSubject,
    pub disposition: PreflightFindingDisposition,
    pub summary: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightReport {
    pub host_id: String,
    pub os: String,
    pub architecture: String,
    pub user_home_boundary: String,
    pub shell: String,
    pub tty_present: bool,
    pub supported_target: Option<String>,
    pub existing_version_set: Vec<ComponentVersion>,
    pub writable_user_targets: Vec<String>,
    pub network_available: bool,
    pub offline_allowed: bool,
    pub artifact_metadata_reachable: bool,
    pub license_posture: LicensePosture,
    pub explicit_project_path: Option<String>,
    pub inspected_project_path: Option<String>,
    #[serde(default)]
    pub findings: Vec<PreflightFinding>,
    pub inspected_at: DateTime<Utc>,
}

impl PreflightReport {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        if self.host_id.trim().is_empty() {
            return Err(InstallLifecycleValidationError::EmptyHostId);
        }
        if self.os.trim().is_empty() || self.architecture.trim().is_empty() {
            return Err(InstallLifecycleValidationError::PlatformEvidenceRequired);
        }
        if let Some(inspected) = self.inspected_project_path.as_deref() {
            if self.explicit_project_path.as_deref() != Some(inspected) {
                return Err(InstallLifecycleValidationError::ProjectInspectionWithoutExactScope);
            }
        }
        if self.supported_target.is_none()
            && !self.findings.iter().any(|finding| {
                finding.disposition == PreflightFindingDisposition::Unsupported
                    || finding.disposition == PreflightFindingDisposition::Blocked
            })
        {
            return Err(InstallLifecycleValidationError::UnsupportedTargetMustBlock);
        }
        if self.findings.iter().any(|finding| {
            finding.finding_id.trim().is_empty() || finding.summary.trim().is_empty()
        }) {
            return Err(InstallLifecycleValidationError::IncompletePreflightFinding);
        }
        Ok(())
    }

    pub fn mutation_ready(&self) -> bool {
        self.validate().is_ok()
            && self.supported_target.is_some()
            && !self.findings.iter().any(|finding| {
                matches!(
                    finding.disposition,
                    PreflightFindingDisposition::Unsupported | PreflightFindingDisposition::Blocked
                )
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentVersion {
    pub component: String,
    pub version: String,
    pub compatible: bool,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactTrustEvidence {
    pub declared_version: String,
    pub declared_channel: ChannelSelection,
    pub target: String,
    pub metadata_complete: bool,
    #[serde(default)]
    pub checksum_refs: Vec<String>,
    #[serde(default)]
    pub signature_refs: Vec<String>,
    #[serde(default)]
    pub provenance_refs: Vec<String>,
    pub staged_extraction_verified: bool,
}

impl ArtifactTrustEvidence {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        if self.is_complete() {
            Ok(())
        } else {
            Err(InstallLifecycleValidationError::ArtifactTrustIncomplete)
        }
    }

    pub(super) fn is_complete(&self) -> bool {
        !self.declared_version.trim().is_empty()
            && !self.target.trim().is_empty()
            && self.metadata_complete
            && !self.checksum_refs.is_empty()
            && !self.signature_refs.is_empty()
            && !self.provenance_refs.is_empty()
            && self.staged_extraction_verified
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryClass {
    UnsupportedHost,
    ArtifactIncomplete,
    TrustFailure,
    PermissionBoundary,
    DaemonDegraded,
    IntegrationIncompatible,
    ScopeMismatch,
    ConfirmationMissing,
    ProviderUnavailable,
    ProjectConflict,
    UpdatePartial,
    UninstallAmbiguous,
    UnknownCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryInstructions {
    pub primary_class: RecoveryClass,
    pub summary: String,
    #[serde(default)]
    pub operator_actions: Vec<String>,
    pub resume_from_state: LifecycleState,
    pub inspect_before_retry: bool,
    pub requires_confirmation: bool,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

impl RecoveryInstructions {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        if self.summary.trim().is_empty() || self.operator_actions.is_empty() {
            return Err(InstallLifecycleValidationError::RecoveryRequiresGuidance);
        }
        if self.primary_class == RecoveryClass::UnknownCompletion && !self.inspect_before_retry {
            return Err(InstallLifecycleValidationError::UnknownCompletionRequiresInspection);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackBoundary {
    pub replacement_planned: bool,
    #[serde(default)]
    pub prior_version_set: Vec<ComponentVersion>,
    #[serde(default)]
    pub rollback_artifact_refs: Vec<String>,
    #[serde(default)]
    pub rollback_trust_refs: Vec<String>,
    pub atomic_activation: bool,
    pub preserves_user_data: bool,
    pub preserves_project_data: bool,
}

impl RollbackBoundary {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        if self.replacement_planned
            && (self.prior_version_set.is_empty()
                || self.rollback_artifact_refs.is_empty()
                || self.rollback_trust_refs.is_empty()
                || !self.preserves_user_data
                || !self.preserves_project_data)
        {
            return Err(InstallLifecycleValidationError::RollbackBoundaryIncomplete);
        }
        Ok(())
    }

    pub fn rollback_available(&self) -> bool {
        !self.replacement_planned || self.validate().is_ok()
    }
}
