use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleTransactionKind {
    HostInstall,
    ProjectOnboarding,
    LifecycleMaintenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleState {
    Uninspected,
    Preflighted,
    ArtifactSelected,
    ArtifactVerified,
    HostInstalled,
    DaemonReady,
    HostAccepted,
    IntegrationsSelected,
    IntegrationsVerified,
    ProjectSelected,
    ProjectVerified,
    ProjectBootstrapPreviewed,
    ProjectBootstrapped,
    GenesisStarted,
    GenesisCommitted,
    FirstWorkpointReady,
    ExperienceSelected,
    Accepted,
    BlockedUnsupportedHost,
    BlockedLicense,
    BlockedArtifactTrust,
    BlockedPermission,
    BlockedScope,
    BlockedConfirmation,
    BlockedProviderHandoff,
    BlockedDependency,
    PartialHostInstall,
    PartialIntegration,
    PartialProjectBootstrap,
    DegradedDaemon,
    RollbackRequired,
    OperatorActionRequired,
}

impl LifecycleState {
    pub fn is_recovery(self) -> bool {
        matches!(
            self,
            Self::BlockedUnsupportedHost
                | Self::BlockedLicense
                | Self::BlockedArtifactTrust
                | Self::BlockedPermission
                | Self::BlockedScope
                | Self::BlockedConfirmation
                | Self::BlockedProviderHandoff
                | Self::BlockedDependency
                | Self::PartialHostInstall
                | Self::PartialIntegration
                | Self::PartialProjectBootstrap
                | Self::DegradedDaemon
                | Self::RollbackRequired
                | Self::OperatorActionRequired
        )
    }
}

const PRIMARY_STATES: [LifecycleState; 18] = [
    LifecycleState::Uninspected,
    LifecycleState::Preflighted,
    LifecycleState::ArtifactSelected,
    LifecycleState::ArtifactVerified,
    LifecycleState::HostInstalled,
    LifecycleState::DaemonReady,
    LifecycleState::HostAccepted,
    LifecycleState::IntegrationsSelected,
    LifecycleState::IntegrationsVerified,
    LifecycleState::ProjectSelected,
    LifecycleState::ProjectVerified,
    LifecycleState::ProjectBootstrapPreviewed,
    LifecycleState::ProjectBootstrapped,
    LifecycleState::GenesisStarted,
    LifecycleState::GenesisCommitted,
    LifecycleState::FirstWorkpointReady,
    LifecycleState::ExperienceSelected,
    LifecycleState::Accepted,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceAction {
    Inspect,
    Rerun,
    Repair,
    Update,
    Rollback,
    Uninstall,
    Purge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleScope {
    pub host_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuity_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleTransition {
    pub transaction_id: String,
    pub transaction_kind: LifecycleTransactionKind,
    pub scope: LifecycleScope,
    pub prior_state: LifecycleState,
    pub new_state: LifecycleState,
    pub action: String,
    pub status: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleReceipt {
    pub receipt_id: String,
    pub transaction_id: String,
    pub transaction_kind: LifecycleTransactionKind,
    pub scope: LifecycleScope,
    pub final_state: LifecycleState,
    pub accepted: bool,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub transition_refs: Vec<String>,
    pub issued_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleValidationError {
    EmptyTransactionId,
    EmptyHostId,
    ProjectScopeForbiddenForHostInstall,
    ProjectScopeRequiredForOnboarding,
    InvalidPrimaryTransition,
    RecoveryRequiresGuidance,
    AcceptedRequiresEvidence,
}

fn primary_index(state: LifecycleState) -> Option<usize> {
    PRIMARY_STATES
        .iter()
        .position(|candidate| *candidate == state)
}

impl LifecycleTransition {
    pub fn validate(&self) -> Result<(), LifecycleValidationError> {
        if self.transaction_id.trim().is_empty() {
            return Err(LifecycleValidationError::EmptyTransactionId);
        }
        if self.scope.host_id.trim().is_empty() {
            return Err(LifecycleValidationError::EmptyHostId);
        }
        match self.transaction_kind {
            LifecycleTransactionKind::HostInstall if self.scope.project_root.is_some() => {
                return Err(LifecycleValidationError::ProjectScopeForbiddenForHostInstall);
            }
            LifecycleTransactionKind::ProjectOnboarding
                if self.scope.project_root.is_none() || self.scope.continuity_id.is_none() =>
            {
                return Err(LifecycleValidationError::ProjectScopeRequiredForOnboarding);
            }
            _ => {}
        }
        if self.new_state.is_recovery() {
            if self
                .recovery
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                return Err(LifecycleValidationError::RecoveryRequiresGuidance);
            }
            return Ok(());
        }
        let Some(prior) = primary_index(self.prior_state) else {
            return Err(LifecycleValidationError::InvalidPrimaryTransition);
        };
        let Some(next) = primary_index(self.new_state) else {
            return Err(LifecycleValidationError::InvalidPrimaryTransition);
        };
        if next != prior + 1 {
            return Err(LifecycleValidationError::InvalidPrimaryTransition);
        }
        if self.new_state == LifecycleState::Accepted && self.evidence_refs.is_empty() {
            return Err(LifecycleValidationError::AcceptedRequiresEvidence);
        }
        Ok(())
    }
}

impl LifecycleReceipt {
    pub fn validate(&self) -> Result<(), LifecycleValidationError> {
        if self.transaction_id.trim().is_empty() {
            return Err(LifecycleValidationError::EmptyTransactionId);
        }
        if self.scope.host_id.trim().is_empty() {
            return Err(LifecycleValidationError::EmptyHostId);
        }
        if self.accepted
            && (self.final_state != LifecycleState::Accepted || self.evidence_refs.is_empty())
        {
            return Err(LifecycleValidationError::AcceptedRequiresEvidence);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> LifecycleScope {
        LifecycleScope {
            host_id: "host:test".into(),
            project_root: None,
            continuity_id: None,
        }
    }

    #[test]
    fn primary_transition_is_strictly_sequential() {
        let transition = LifecycleTransition {
            transaction_id: "tx:1".into(),
            transaction_kind: LifecycleTransactionKind::HostInstall,
            scope: scope(),
            prior_state: LifecycleState::Uninspected,
            new_state: LifecycleState::Preflighted,
            action: "inspect".into(),
            status: "completed".into(),
            evidence_refs: vec![],
            recovery: None,
            occurred_at: Utc::now(),
        };
        assert_eq!(transition.validate(), Ok(()));
        let skipped = LifecycleTransition {
            new_state: LifecycleState::ArtifactVerified,
            ..transition
        };
        assert_eq!(
            skipped.validate(),
            Err(LifecycleValidationError::InvalidPrimaryTransition)
        );
    }

    #[test]
    fn host_install_cannot_mutate_project_scope() {
        let transition = LifecycleTransition {
            transaction_id: "tx:host".into(),
            transaction_kind: LifecycleTransactionKind::HostInstall,
            scope: LifecycleScope {
                host_id: "host:test".into(),
                project_root: Some("/project".into()),
                continuity_id: Some("continuity".into()),
            },
            prior_state: LifecycleState::Uninspected,
            new_state: LifecycleState::Preflighted,
            action: "inspect".into(),
            status: "completed".into(),
            evidence_refs: vec![],
            recovery: None,
            occurred_at: Utc::now(),
        };
        assert_eq!(
            transition.validate(),
            Err(LifecycleValidationError::ProjectScopeForbiddenForHostInstall)
        );
    }

    #[test]
    fn recovery_and_acceptance_fail_closed() {
        let recovery = LifecycleTransition {
            transaction_id: "tx:recovery".into(),
            transaction_kind: LifecycleTransactionKind::LifecycleMaintenance,
            scope: scope(),
            prior_state: LifecycleState::DaemonReady,
            new_state: LifecycleState::DegradedDaemon,
            action: "health".into(),
            status: "blocked".into(),
            evidence_refs: vec![],
            recovery: None,
            occurred_at: Utc::now(),
        };
        assert_eq!(
            recovery.validate(),
            Err(LifecycleValidationError::RecoveryRequiresGuidance)
        );
        let acceptance = LifecycleReceipt {
            receipt_id: "receipt:1".into(),
            transaction_id: "tx:1".into(),
            transaction_kind: LifecycleTransactionKind::LifecycleMaintenance,
            scope: scope(),
            final_state: LifecycleState::Accepted,
            accepted: true,
            evidence_refs: vec![],
            transition_refs: vec![],
            issued_at: Utc::now(),
        };
        assert_eq!(
            acceptance.validate(),
            Err(LifecycleValidationError::AcceptedRequiresEvidence)
        );
    }
}
