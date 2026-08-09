use super::*;
use crate::install_lifecycle::{
    AdapterEntitlementPosture, CompleteVersionSet, LifecycleAcceptanceReceipt,
    LifecycleEntitlementBinding, LifecycleEntitlementReceiptClass, LifecycleEntitlementState,
    LifecycleOperation, LifecycleScope, LifecycleState,
};
use chrono::{DateTime, Utc};
use std::collections::BTreeSet;

fn time(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .expect("valid receipt fixture time")
        .with_timezone(&Utc)
}

fn digest(byte: char) -> String {
    format!("sha256:{}", byte.to_string().repeat(64))
}

fn acceptance(class: LifecycleEntitlementReceiptClass) -> LifecycleAcceptanceReceipt {
    let binding = LifecycleEntitlementBinding {
        schema_version: "focusa.lifecycle_entitlement_binding.v1".into(),
        state: LifecycleEntitlementState::ActiveEvaluation,
        lease_id: "lease:evaluation:001".into(),
        lease_sequence: 11,
        lease_payload_digest: digest('a'),
        product_grants_digest: digest('b'),
        feature_grants_digest: digest('c'),
        node_id: "node:evaluation:001".into(),
        license_class: "evaluation".into(),
        refresh_after: time("2026-08-05T13:00:00Z"),
        offline_valid_until: time("2026-08-06T12:00:00Z"),
        expires_at: Some(time("2026-08-12T12:00:00Z")),
        authority_key_id: "authority-lease-2026-01".into(),
        signature_verified: true,
    };
    LifecycleAcceptanceReceipt {
        transaction_id: "transaction:001".into(),
        operation: LifecycleOperation::Install,
        scope: LifecycleScope {
            host_id: "host:001".into(),
            project_root: Some("/project".into()),
            continuity_id: Some("continuity:001".into()),
        },
        final_state: LifecycleState::Accepted,
        journal_head_hash: digest('d'),
        version_set: CompleteVersionSet {
            cli_version: "0.9.144".into(),
            daemon_version: "0.9.144".into(),
            api_version: "0.9.144".into(),
            pi_extension_version: "0.9.144".into(),
            schema_version: "150a.1".into(),
        },
        daemon_service_healthy: true,
        project_verified: true,
        bootstrap_committed: true,
        genesis_committed: true,
        first_workpoint_id: Some("workpoint:001".into()),
        preserved_data_classes: BTreeSet::from(["FocusaState".into(), "ProjectFiles".into()]),
        entitlement_receipt_class: class,
        entitlement_binding: Some(binding),
        entitlement_evidence_refs: vec!["evidence:signed-evaluation-lease".into()],
        closure_allowed: true,
    }
}

fn adapter() -> AdapterEntitlementPosture {
    AdapterEntitlementPosture {
        schema_version: "focusa.adapter_entitlement_posture.v1".into(),
        product: "uiai-engine".into(),
        lease_id: "lease:uiai:001".into(),
        lease_sequence: 3,
        product_granted: true,
        required_features_granted: true,
        parent_lease_digest: digest('a'),
        child_token_id: "child-token:001".into(),
        child_token_audience: Some("uiai-engine:node:evaluation:001".into()),
        child_token_expires_at: Some(time("2026-08-05T12:15:00Z")),
        entitlement_digest: digest('f'),
        account_id: None,
        edd_customer_id: None,
    }
}

#[test]
fn versioned_receipt_round_trips_without_account_or_credential_fields() {
    let receipt = LifecycleReceiptV1::from_acceptance(
        "receipt:001",
        &acceptance(LifecycleEntitlementReceiptClass::EvaluationReady),
        time("2026-08-05T12:10:00Z"),
        digest('e'),
        Some(digest('f')),
        vec![adapter()],
        None,
    )
    .expect("valid reconciled receipt");
    assert!(receipt.product_ready());
    receipt.verify(RECEIPT_GENESIS_HASH).unwrap();

    let value = serde_json::to_value(&receipt).expect("serialize receipt");
    let object = value.as_object().expect("receipt object");
    for forbidden in ["email", "account", "token", "credential", "private_key"] {
        assert!(!object.contains_key(forbidden));
    }
    let decoded: LifecycleReceiptV1 = serde_json::from_value(value).expect("deserialize receipt");
    assert_eq!(decoded, receipt);
}

#[test]
fn receipt_chain_is_tamper_evident_and_replay_is_idempotent() {
    let first = LifecycleReceiptV1::from_acceptance(
        "receipt:001",
        &acceptance(LifecycleEntitlementReceiptClass::EvaluationReady),
        time("2026-08-05T12:10:00Z"),
        digest('e'),
        None,
        vec![],
        None,
    )
    .unwrap();
    let second = LifecycleReceiptV1::from_acceptance(
        "receipt:002",
        &acceptance(LifecycleEntitlementReceiptClass::EvaluationReady),
        time("2026-08-05T12:11:00Z"),
        digest('f'),
        None,
        vec![],
        Some(first.receipt_hash.clone()),
    )
    .unwrap();
    let mut receipts = Vec::new();
    assert_eq!(
        append_lifecycle_receipt(&mut receipts, first.clone()),
        Ok(LifecycleReceiptAppendOutcome::Appended)
    );
    assert_eq!(
        append_lifecycle_receipt(&mut receipts, first),
        Ok(LifecycleReceiptAppendOutcome::IdempotentReplay)
    );
    append_lifecycle_receipt(&mut receipts, second).unwrap();
    verify_lifecycle_receipt_chain(&receipts).unwrap();

    receipts[0].installer_artifact_digest = digest('0');
    assert_eq!(
        verify_lifecycle_receipt_chain(&receipts),
        Err(LifecycleReceiptError::IntegrityFailure)
    );
}

#[test]
fn blocked_or_interrupted_receipt_cannot_claim_product_ready() {
    let mut blocked = acceptance(LifecycleEntitlementReceiptClass::BlockedEntitlement);
    blocked.final_state = LifecycleState::BlockedLicense;
    blocked.closure_allowed = false;
    blocked.entitlement_binding = None;
    blocked.entitlement_evidence_refs.clear();
    let receipt = LifecycleReceiptV1::from_acceptance(
        "receipt:blocked",
        &blocked,
        time("2026-08-05T12:10:00Z"),
        digest('e'),
        None,
        vec![],
        None,
    )
    .unwrap();
    assert!(!receipt.product_ready());

    let mut forged_ready = receipt;
    forged_ready.entitlement_receipt_class = LifecycleEntitlementReceiptClass::PaidReady;
    assert_eq!(
        forged_ready.verify(RECEIPT_GENESIS_HASH),
        Err(LifecycleReceiptError::UnverifiedProductReady)
    );
}

#[test]
fn adapter_posture_must_match_parent_authority_digest() {
    let mut mismatch = adapter();
    mismatch.parent_lease_digest = digest('0');
    assert_eq!(
        LifecycleReceiptV1::from_acceptance(
            "receipt:mismatch",
            &acceptance(LifecycleEntitlementReceiptClass::EvaluationReady),
            time("2026-08-05T12:10:00Z"),
            digest('e'),
            None,
            vec![mismatch],
            None,
        ),
        Err(LifecycleReceiptError::AdapterMismatch)
    );
}

#[test]
fn receipt_presenter_posture_uses_shared_presenter_vocabulary() {
    // Product-ready receipts render as the shared `activated` state with
    // node management and refresh actions — the same posture the menubar,
    // TUI, and daemon REST surface expose for an entitled registration.
    let ready = LifecycleReceiptV1::from_acceptance(
        "receipt:ready",
        &acceptance(LifecycleEntitlementReceiptClass::PaidReady),
        time("2026-08-05T12:10:00Z"),
        digest('e'),
        None,
        vec![],
        None,
    )
    .unwrap();
    let posture = ready.presenter_posture();
    assert_eq!(posture.presenter_state, "activated");
    assert_eq!(posture.next_action, "activated");
    assert!(posture.product_ready);
    assert!(posture.terminal);
    assert!(
        posture
            .allowed_actions
            .contains(&"manage_nodes".to_string())
    );
    assert!(
        posture
            .allowed_actions
            .contains(&"refresh_lease".to_string())
    );

    // RecoveryReady renders recovery_only with the shared recovery actions.
    let recovery = LifecycleReceiptV1::from_acceptance(
        "receipt:recovery",
        &acceptance(LifecycleEntitlementReceiptClass::RecoveryReady),
        time("2026-08-05T12:10:00Z"),
        digest('e'),
        None,
        vec![],
        None,
    )
    .unwrap();
    let recovery_posture = recovery.presenter_posture();
    assert_eq!(recovery_posture.presenter_state, "recovery_only");
    assert_eq!(recovery_posture.next_action, "recovery");
    assert!(
        recovery_posture
            .allowed_actions
            .contains(&"repair".to_string())
    );
    assert!(
        recovery_posture
            .allowed_actions
            .contains(&"uninstall".to_string())
    );
    assert!(!recovery_posture.product_ready);

    // BlockedEntitlement renders denied with activation-or-manage guidance;
    // it never renders as usable.
    let mut blocked = acceptance(LifecycleEntitlementReceiptClass::BlockedEntitlement);
    blocked.final_state = LifecycleState::BlockedLicense;
    blocked.closure_allowed = false;
    blocked.entitlement_binding = None;
    blocked.entitlement_evidence_refs.clear();
    let blocked_receipt = LifecycleReceiptV1::from_acceptance(
        "receipt:blocked",
        &blocked,
        time("2026-08-05T12:10:00Z"),
        digest('e'),
        None,
        vec![],
        None,
    )
    .unwrap();
    let denied = blocked_receipt.presenter_posture();
    assert_eq!(denied.presenter_state, "denied");
    assert_eq!(denied.next_action, "activate_or_manage_entitlement");
    assert!(denied.allowed_actions.contains(&"recovery".to_string()));

    // A product-ready class without a verified signature fails closed to
    // recovery_only and never renders as activated. (Construction already
    // refuses such receipts via UnverifiedProductReady; the posture still
    // fails closed when projection runs on any such record.)
    let mut forged = LifecycleReceiptV1::from_acceptance(
        "receipt:forged",
        &acceptance(LifecycleEntitlementReceiptClass::PaidReady),
        time("2026-08-05T12:10:00Z"),
        digest('e'),
        None,
        vec![],
        None,
    )
    .unwrap();
    forged.signature_verified = false;
    let forged_posture = forged.presenter_posture();
    assert_eq!(forged_posture.presenter_state, "recovery_only");
    assert!(!forged_posture.product_ready);

    // The posture is presenter-safe: no raw email, key, credential, or card
    // data field by construction.
    let body = serde_json::to_string(&posture).unwrap();
    assert!(!body.contains("email"));
    assert!(!body.contains("credential"));
    assert!(!body.contains("card"));
}
