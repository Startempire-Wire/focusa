use super::*;
use crate::install_lifecycle::{
    LifecycleDataClass, LifecycleEntitlementBinding, LifecycleEntitlementDecision,
    LifecycleEntitlementReceiptClass, LifecycleEntitlementState, MaintenanceAction,
    PreservationDisposition, PreservationItem,
};
use std::collections::BTreeSet;

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

fn authority_time(value: &str) -> chrono::DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339(value)
        .expect("valid authority time")
        .with_timezone(&Utc)
}

fn authority_digest(byte: char) -> String {
    format!("sha256:{}", byte.to_string().repeat(64))
}

fn entitlement(state: LifecycleEntitlementState) -> LifecycleEntitlementDecision {
    LifecycleEntitlementDecision {
        binding: LifecycleEntitlementBinding {
            schema_version: "focusa.lifecycle_entitlement_binding.v1".into(),
            state,
            lease_id: "lease:paid:001".into(),
            lease_sequence: 9,
            lease_payload_digest: authority_digest('a'),
            product_grants_digest: authority_digest('b'),
            feature_grants_digest: authority_digest('c'),
            node_id: "node:001".into(),
            license_class: "paid".into(),
            refresh_after: authority_time("2026-08-06T00:00:00Z"),
            offline_valid_until: authority_time("2026-08-08T00:00:00Z"),
            expires_at: Some(authority_time("2030-08-08T00:00:00Z")),
            authority_key_id: "authority-lease-2026-01".into(),
            signature_verified: true,
        },
        granted_products: BTreeSet::from(["focusa".into()]),
        granted_features: BTreeSet::from([
            "focusa.install.channel.stable".into(),
            "focusa.repair.execute".into(),
            "focusa.update.apply".into(),
            "focusa.update.unattended".into(),
        ]),
        remaining_limits: std::collections::BTreeMap::new(),
        evidence_refs: vec!["evidence:signed-entitlement".into()],
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
        dry_run: false,
        recovery_safe: false,
        unattended: false,
        selected_product: "focusa".into(),
        selected_channel: "stable".into(),
        required_features: BTreeSet::new(),
        entitlement: Some(entitlement(LifecycleEntitlementState::ActivePaid)),
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
    unsafe {
        std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
    }
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
    assert_eq!(
        receipt.entitlement_receipt_class,
        LifecycleEntitlementReceiptClass::PaidReady
    );
    assert_eq!(receipt.entitlement_binding.unwrap().lease_sequence, 9);
}

#[test]
fn entitlement_transition_table_blocks_mutation_without_signed_grants() {
    unsafe {
        std::env::set_var("FOCUSA_ACTIVATION_BYPASS_DISABLE", "1");
    }
    let now = authority_time("2026-08-07T00:00:00Z");

    let mut install = request(LifecycleOperation::Install);
    install.entitlement = None;
    assert_eq!(
        validate_request_at(&install, now),
        Err(LifecycleOrchestratorError::EntitlementRequired)
    );
    install.dry_run = true;
    assert_eq!(validate_request_at(&install, now), Ok(()));

    let mut update = request(LifecycleOperation::Update);
    update.unattended = true;
    update
        .entitlement
        .as_mut()
        .unwrap()
        .granted_features
        .remove("focusa.update.unattended");
    assert_eq!(
        validate_request_at(&update, now),
        Err(LifecycleOrchestratorError::FeatureGrantRequired)
    );

    let mut revoked = request(LifecycleOperation::Install);
    revoked.entitlement.as_mut().unwrap().binding.state = LifecycleEntitlementState::Revoked;
    assert_eq!(
        validate_request_at(&revoked, now),
        Err(LifecycleOrchestratorError::EntitlementBlocked)
    );

    let mut recovery_repair = request(LifecycleOperation::Repair);
    recovery_repair.entitlement = None;
    recovery_repair.recovery_safe = true;
    assert_eq!(validate_request_at(&recovery_repair, now), Ok(()));

    let mut uninstall = request(LifecycleOperation::Uninstall);
    uninstall.entitlement = None;
    assert_eq!(validate_request_at(&uninstall, now), Ok(()));

    let mut purge = request(LifecycleOperation::Purge);
    purge.entitlement = None;
    purge.purge_confirmed_separately = true;
    assert_eq!(validate_request_at(&purge, now), Ok(()));
}

#[test]
fn rollback_does_not_restore_or_require_stale_entitlement_authority() {
    let now = authority_time("2026-08-07T00:00:00Z");
    let mut rollback = request(LifecycleOperation::Rollback);
    rollback.entitlement.as_mut().unwrap().binding.state = LifecycleEntitlementState::Revoked;
    assert_eq!(validate_request_at(&rollback, now), Ok(()));
    assert_eq!(
        rollback.entitlement.unwrap().binding.receipt_class(),
        LifecycleEntitlementReceiptClass::BlockedEntitlement
    );
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
