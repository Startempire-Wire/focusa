use super::*;
use crate::install_lifecycle::{
    LifecycleDataClass, MaintenanceAction, PreservationDisposition, PreservationItem,
};

fn preservation() -> PreservationDeclaration {
    let classes = [
        LifecycleDataClass::ManagedBinaries,
        LifecycleDataClass::Services,
        LifecycleDataClass::Integrations,
        LifecycleDataClass::FocusaState,
        LifecycleDataClass::LogsCaches,
        LifecycleDataClass::LicenseState,
        LifecycleDataClass::ProviderHarnessState,
        LifecycleDataClass::ProjectFiles,
        LifecycleDataClass::ProjectTaskData,
        LifecycleDataClass::OperatorAuthoredInstructions,
    ];
    PreservationDeclaration {
        action: MaintenanceAction::Rerun,
        items: classes
            .into_iter()
            .map(|data_class| PreservationItem {
                data_class,
                disposition: PreservationDisposition::Preserve,
                owner_authorized: true,
                evidence_refs: vec!["evidence:preservation".into()],
            })
            .collect(),
        destructive_purge_confirmed: false,
    }
}

fn request(operation: LifecycleOperation) -> LifecycleOperationRequest {
    LifecycleOperationRequest {
        transaction_id: "transaction-1".into(),
        operation,
        scope: LifecycleScope {
            host_id: "host-1".into(),
            project_root: Some("/project".into()),
            continuity_id: Some("continuity-1".into()),
        },
        idempotency_key: "idem-1".into(),
        selected_version: "1.0.0".into(),
        artifact_signature_verified: true,
        preservation: preservation(),
        purge_confirmed_separately: false,
    }
}

fn versions() -> CompleteVersionSet {
    CompleteVersionSet {
        cli_version: "1.0.0".into(),
        daemon_version: "1.0.0".into(),
        api_version: "1.0.0".into(),
        pi_extension_version: "1.0.0".into(),
        schema_version: "150.1".into(),
    }
}

#[test]
fn journal_resumes_idempotently_and_detects_tampering() {
    let request = request(LifecycleOperation::Install);
    let mut journal = Vec::new();
    append_lifecycle_transition(
        &mut journal,
        &request,
        LifecycleState::Uninspected,
        LifecycleState::Preflighted,
        "preflight",
        vec!["evidence:preflight".into()],
    )
    .unwrap();
    assert_eq!(
        resume_state(&request, &journal).unwrap(),
        LifecycleState::Preflighted
    );
    journal[0].action = "tampered".into();
    assert_eq!(
        verify_journal(&journal),
        Err(LifecycleOrchestratorError::JournalIntegrityFailure)
    );
}

#[test]
fn update_requires_signed_artifact_and_purge_has_separate_confirmation() {
    let mut update = request(LifecycleOperation::Update);
    update.artifact_signature_verified = false;
    assert_eq!(
        append_lifecycle_transition(
            &mut Vec::new(),
            &update,
            LifecycleState::Uninspected,
            LifecycleState::Preflighted,
            "update",
            vec![]
        ),
        Err(LifecycleOrchestratorError::ArtifactTrustRequired)
    );
    let purge = request(LifecycleOperation::Purge);
    assert_eq!(
        resume_state(&purge, &[]),
        Err(LifecycleOrchestratorError::PurgeConfirmationRequired)
    );
}

#[test]
fn final_acceptance_requires_coherent_versions_service_project_and_first_workpoint() {
    let request = request(LifecycleOperation::Install);
    let mut journal = Vec::new();
    append_lifecycle_transition(
        &mut journal,
        &request,
        LifecycleState::ExperienceSelected,
        LifecycleState::Accepted,
        "accept",
        vec!["evidence:e2e".into()],
    )
    .unwrap();
    assert_eq!(
        finalize_lifecycle(&request, &journal, versions(), true, true, true, true, None),
        Err(LifecycleOrchestratorError::FirstWorkpointNotAccepted)
    );
    let receipt = finalize_lifecycle(
        &request,
        &journal,
        versions(),
        true,
        true,
        true,
        true,
        Some("workpoint-1".into()),
    )
    .unwrap();
    assert!(receipt.closure_allowed);
    assert_eq!(receipt.preserved_data_classes.len(), 10);
}

#[test]
fn rerun_preserves_transaction_state_and_incoherent_version_set_blocks() {
    let request = request(LifecycleOperation::Rerun);
    let mut journal = Vec::new();
    append_lifecycle_transition(
        &mut journal,
        &request,
        LifecycleState::Uninspected,
        LifecycleState::Preflighted,
        "rerun",
        vec!["evidence:rerun".into()],
    )
    .unwrap();
    let mut bad = versions();
    bad.daemon_version = "0.9.0".into();
    assert_eq!(
        finalize_lifecycle(
            &request,
            &journal,
            bad,
            true,
            true,
            true,
            true,
            Some("workpoint-1".into())
        ),
        Err(LifecycleOrchestratorError::VersionSetIncoherent)
    );
}
