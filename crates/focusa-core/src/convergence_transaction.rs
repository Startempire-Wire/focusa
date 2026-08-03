use crate::installation_convergence::{
    ConvergenceActionKind, ConvergencePlan, InstallationPlatform, ManagedSurface,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceApplyState {
    Pending,
    Staged,
    Applied,
    HealthVerified,
    RolledBack,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceTransactionState {
    pub action: ConvergenceActionKind,
    pub state: SurfaceApplyState,
    pub prior_artifact_ref: Option<String>,
    pub staged_artifact_ref: Option<String>,
    pub installed_artifact_digest: Option<String>,
    pub health_evidence_ref: Option<String>,
    pub failure_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergenceTransaction {
    pub schema: String,
    pub transaction_id: String,
    pub installation_id: String,
    pub desired_generation: u64,
    pub platform: InstallationPlatform,
    pub manifest_digest: String,
    pub surfaces: BTreeMap<ManagedSurface, SurfaceTransactionState>,
    pub rollback_required: bool,
    pub committed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergenceCommitReceipt {
    pub transaction_id: String,
    pub installation_id: String,
    pub desired_generation: u64,
    pub manifest_digest: String,
    pub surface_evidence: BTreeMap<ManagedSurface, String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConvergenceTransactionError {
    #[error("convergence transaction identity is incomplete")]
    MissingIdentity,
    #[error("surface is not part of the convergence plan")]
    UnknownSurface,
    #[error("surface transition is invalid")]
    InvalidTransition,
    #[error("artifact reference or digest is missing")]
    MissingArtifactEvidence,
    #[error("health evidence is missing")]
    MissingHealthEvidence,
    #[error("transaction requires rollback")]
    RollbackRequired,
    #[error("transaction is not fully health verified")]
    NotHealthVerified,
    #[error("transaction is already committed")]
    AlreadyCommitted,
}

impl ConvergenceTransaction {
    pub fn prepare(
        transaction_id: &str,
        plan: &ConvergencePlan,
    ) -> Result<Self, ConvergenceTransactionError> {
        if transaction_id.trim().is_empty()
            || plan.installation_id.trim().is_empty()
            || plan.artifact_manifest_digest.trim().is_empty()
        {
            return Err(ConvergenceTransactionError::MissingIdentity);
        }
        let surfaces = plan
            .actions
            .iter()
            .map(|action| {
                (
                    action.surface,
                    SurfaceTransactionState {
                        action: action.action,
                        state: if action.action == ConvergenceActionKind::NoOp {
                            SurfaceApplyState::HealthVerified
                        } else {
                            SurfaceApplyState::Pending
                        },
                        prior_artifact_ref: None,
                        staged_artifact_ref: None,
                        installed_artifact_digest: None,
                        health_evidence_ref: (action.action == ConvergenceActionKind::NoOp)
                            .then(|| "evidence:already-converged".into()),
                        failure_class: None,
                    },
                )
            })
            .collect();
        Ok(Self {
            schema: "focusa.convergence_transaction.v1".into(),
            transaction_id: transaction_id.into(),
            installation_id: plan.installation_id.clone(),
            desired_generation: plan.desired_generation,
            platform: plan.platform,
            manifest_digest: plan.artifact_manifest_digest.clone(),
            surfaces,
            rollback_required: false,
            committed: false,
        })
    }

    pub fn stage(
        &mut self,
        surface: ManagedSurface,
        prior_artifact_ref: Option<String>,
        staged_artifact_ref: String,
    ) -> Result<(), ConvergenceTransactionError> {
        self.ensure_mutable()?;
        if staged_artifact_ref.trim().is_empty() {
            return Err(ConvergenceTransactionError::MissingArtifactEvidence);
        }
        let state = self.surface_mut(surface)?;
        if state.state != SurfaceApplyState::Pending {
            return Err(ConvergenceTransactionError::InvalidTransition);
        }
        state.prior_artifact_ref = prior_artifact_ref;
        state.staged_artifact_ref = Some(staged_artifact_ref);
        state.state = SurfaceApplyState::Staged;
        Ok(())
    }

    pub fn mark_applied(
        &mut self,
        surface: ManagedSurface,
        installed_artifact_digest: String,
    ) -> Result<(), ConvergenceTransactionError> {
        self.ensure_mutable()?;
        if installed_artifact_digest.trim().is_empty() {
            return Err(ConvergenceTransactionError::MissingArtifactEvidence);
        }
        let state = self.surface_mut(surface)?;
        if state.state != SurfaceApplyState::Staged {
            return Err(ConvergenceTransactionError::InvalidTransition);
        }
        state.installed_artifact_digest = Some(installed_artifact_digest);
        state.state = SurfaceApplyState::Applied;
        Ok(())
    }

    pub fn verify_health(
        &mut self,
        surface: ManagedSurface,
        evidence_ref: String,
    ) -> Result<(), ConvergenceTransactionError> {
        self.ensure_mutable()?;
        if evidence_ref.trim().is_empty() {
            return Err(ConvergenceTransactionError::MissingHealthEvidence);
        }
        let state = self.surface_mut(surface)?;
        if state.state != SurfaceApplyState::Applied {
            return Err(ConvergenceTransactionError::InvalidTransition);
        }
        state.health_evidence_ref = Some(evidence_ref);
        state.state = SurfaceApplyState::HealthVerified;
        Ok(())
    }

    pub fn mark_failed(
        &mut self,
        surface: ManagedSurface,
        failure_class: String,
    ) -> Result<(), ConvergenceTransactionError> {
        self.ensure_mutable()?;
        let state = self.surface_mut(surface)?;
        if matches!(
            state.state,
            SurfaceApplyState::HealthVerified | SurfaceApplyState::RolledBack
        ) {
            return Err(ConvergenceTransactionError::InvalidTransition);
        }
        state.state = SurfaceApplyState::Failed;
        state.failure_class = Some(failure_class);
        self.rollback_required = self.surfaces.values().any(|candidate| {
            matches!(
                candidate.state,
                SurfaceApplyState::Staged | SurfaceApplyState::Applied | SurfaceApplyState::Failed
            )
        });
        Ok(())
    }

    pub fn mark_rolled_back(
        &mut self,
        surface: ManagedSurface,
        rollback_evidence_ref: String,
    ) -> Result<(), ConvergenceTransactionError> {
        self.ensure_mutable()?;
        if rollback_evidence_ref.trim().is_empty() {
            return Err(ConvergenceTransactionError::MissingArtifactEvidence);
        }
        let state = self.surface_mut(surface)?;
        if !matches!(
            state.state,
            SurfaceApplyState::Staged | SurfaceApplyState::Applied | SurfaceApplyState::Failed
        ) {
            return Err(ConvergenceTransactionError::InvalidTransition);
        }
        state.state = SurfaceApplyState::RolledBack;
        state.health_evidence_ref = Some(rollback_evidence_ref);
        self.rollback_required = self.surfaces.values().any(|candidate| {
            matches!(
                candidate.state,
                SurfaceApplyState::Staged | SurfaceApplyState::Applied | SurfaceApplyState::Failed
            )
        });
        Ok(())
    }

    pub fn commit(&mut self) -> Result<ConvergenceCommitReceipt, ConvergenceTransactionError> {
        self.ensure_mutable()?;
        if self.rollback_required {
            return Err(ConvergenceTransactionError::RollbackRequired);
        }
        if self
            .surfaces
            .values()
            .any(|surface| surface.state != SurfaceApplyState::HealthVerified)
        {
            return Err(ConvergenceTransactionError::NotHealthVerified);
        }
        self.committed = true;
        Ok(ConvergenceCommitReceipt {
            transaction_id: self.transaction_id.clone(),
            installation_id: self.installation_id.clone(),
            desired_generation: self.desired_generation,
            manifest_digest: self.manifest_digest.clone(),
            surface_evidence: self
                .surfaces
                .iter()
                .map(|(surface, state)| {
                    (
                        *surface,
                        state
                            .health_evidence_ref
                            .clone()
                            .expect("health-verified surfaces have evidence"),
                    )
                })
                .collect(),
        })
    }

    fn ensure_mutable(&self) -> Result<(), ConvergenceTransactionError> {
        if self.committed {
            Err(ConvergenceTransactionError::AlreadyCommitted)
        } else {
            Ok(())
        }
    }

    fn surface_mut(
        &mut self,
        surface: ManagedSurface,
    ) -> Result<&mut SurfaceTransactionState, ConvergenceTransactionError> {
        self.surfaces
            .get_mut(&surface)
            .ok_or(ConvergenceTransactionError::UnknownSurface)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::installation_convergence::SurfaceConvergenceAction;

    fn plan() -> ConvergencePlan {
        ConvergencePlan {
            schema: "focusa.installation_convergence_plan.v1".into(),
            installation_id: "install-1".into(),
            enrollment_generation: 1,
            desired_generation: 2,
            platform: InstallationPlatform::LinuxGnuX64,
            channel: "stable".into(),
            artifact_manifest_digest: "sha256:manifest".into(),
            actions: vec![SurfaceConvergenceAction {
                surface: ManagedSurface::Daemon,
                action: ConvergenceActionKind::Update,
                from_version: Some("0.9.143".into()),
                to_version: "0.9.144".into(),
                reason: "version_drift".into(),
            }],
            operator_approval_ref: "approval:1".into(),
        }
    }

    #[test]
    fn transaction_commits_only_after_artifact_and_health_evidence() {
        let mut transaction = ConvergenceTransaction::prepare("tx-1", &plan()).unwrap();
        transaction
            .stage(
                ManagedSurface::Daemon,
                Some("rollback:daemon-143".into()),
                "stage:daemon-144".into(),
            )
            .unwrap();
        transaction
            .mark_applied(ManagedSurface::Daemon, "sha256:daemon-144".into())
            .unwrap();
        assert_eq!(
            transaction.commit(),
            Err(ConvergenceTransactionError::NotHealthVerified)
        );
        transaction
            .verify_health(ManagedSurface::Daemon, "health:daemon-144".into())
            .unwrap();
        let receipt = transaction.commit().unwrap();
        assert_eq!(
            receipt.surface_evidence.get(&ManagedSurface::Daemon),
            Some(&"health:daemon-144".into())
        );
    }

    #[test]
    fn partial_failure_requires_rollback_before_any_commit() {
        let mut transaction = ConvergenceTransaction::prepare("tx-1", &plan()).unwrap();
        transaction
            .stage(
                ManagedSurface::Daemon,
                Some("rollback:daemon-143".into()),
                "stage:daemon-144".into(),
            )
            .unwrap();
        transaction
            .mark_failed(ManagedSurface::Daemon, "service_health_failed".into())
            .unwrap();
        assert_eq!(
            transaction.commit(),
            Err(ConvergenceTransactionError::RollbackRequired)
        );
        transaction
            .mark_rolled_back(ManagedSurface::Daemon, "rollback:verified".into())
            .unwrap();
        assert_eq!(
            transaction.commit(),
            Err(ConvergenceTransactionError::NotHealthVerified)
        );
    }
}
