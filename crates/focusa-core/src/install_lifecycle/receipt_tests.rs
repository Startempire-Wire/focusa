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
        state: LifecycleEntitlementState::ActiveVerifiedLimited,
        lease_id: "lease:limited:001".into(),
        lease_sequence: 11,
        lease_payload_digest: digest('a'),
        product_grants_digest: digest('b'),
        feature_grants_digest: digest('c'),
        node_id: "node:limited:001".into(),
        license_class: "verified_limited".into(),
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
        entitlement_evidence_refs: vec!["evidence:signed-limited-access-lease".into()],
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
        child_token_audience: Some("uiai-engine:node:limited:001".into()),
        child_token_expires_at: Some(time("2026-08-05T12:15:00Z")),
        entitlement_digest: digest('f'),
        account_id: None,
        edd_customer_id: None,
    }
}

fn binding_with(
    state: LifecycleEntitlementState,
    license_class: &str,
) -> LifecycleEntitlementBinding {
    LifecycleEntitlementBinding {
        schema_version: "focusa.lifecycle_entitlement_binding.v1".into(),
        state,
        lease_id: "lease:focusa:042".into(),
        lease_sequence: 42,
        lease_payload_digest: digest('a'),
        product_grants_digest: digest('b'),
        feature_grants_digest: digest('c'),
        node_id: "node:focusa:042".into(),
        license_class: license_class.into(),
        refresh_after: time("2026-08-05T13:00:00Z"),
        offline_valid_until: time("2026-08-06T12:00:00Z"),
        expires_at: Some(time("2026-08-12T12:00:00Z")),
        authority_key_id: "authority-lease-2026-01".into(),
        signature_verified: true,
    }
}

#[test]
fn versioned_receipt_round_trips_without_account_or_credential_fields() {
    let receipt = LifecycleReceiptV1::from_acceptance(
        "receipt:001",
        &acceptance(LifecycleEntitlementReceiptClass::LimitedAccessReady),
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
        &acceptance(LifecycleEntitlementReceiptClass::LimitedAccessReady),
        time("2026-08-05T12:10:00Z"),
        digest('e'),
        None,
        vec![],
        None,
    )
    .unwrap();
    let second = LifecycleReceiptV1::from_acceptance(
        "receipt:002",
        &acceptance(LifecycleEntitlementReceiptClass::LimitedAccessReady),
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
            &acceptance(LifecycleEntitlementReceiptClass::LimitedAccessReady),
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

#[test]
fn receipt_records_canonical_simple_policy_binding() {
    let mut paid = acceptance(LifecycleEntitlementReceiptClass::PaidReady);
    paid.entitlement_binding = Some(binding_with(
        LifecycleEntitlementState::ActivePaid,
        "focusa",
    ));
    let receipt = LifecycleReceiptV1::from_acceptance(
        "receipt:policy-paid",
        &paid,
        time("2026-08-05T12:10:00Z"),
        digest('e'),
        None,
        vec![],
        None,
    )
    .unwrap();
    let binding = &receipt.policy_binding;
    assert_eq!(binding.schema_version, "focusa.lifecycle_policy_binding.v1");
    assert_eq!(
        binding.policy_digest,
        focusa_license::embedded_entitlement_policy_registry()
            .expect("embedded registry")
            .digest()
    );
    assert_eq!(binding.capability_family, "base_focusa");
    assert_eq!(binding.entitlement_state, "active_paid");
    assert_eq!(binding.lease_sequence, 42);
    assert!(!binding.recovery_posture);
    assert!(binding.product_ready);
    assert!(receipt.product_ready());
    assert_eq!(receipt.reconcile_policy(), Ok(()));
    assert_eq!(receipt.presenter_posture().presenter_state, "activated");
}

#[test]
fn receipt_recovery_posture_records_recovery_family_state_and_sequence() {
    // Recovery-ready receipt still records the canonical policy digest and
    // the lease sequence from the binding, but projects recovery posture and
    // never claims product readiness.
    let mut recovery = acceptance(LifecycleEntitlementReceiptClass::RecoveryReady);
    recovery.entitlement_binding = Some(binding_with(
        LifecycleEntitlementState::OfflineGrace,
        "focusa_standard",
    ));
    let receipt = LifecycleReceiptV1::from_acceptance(
        "receipt:policy-recovery",
        &recovery,
        time("2026-08-05T12:10:00Z"),
        digest('e'),
        None,
        vec![],
        None,
    )
    .unwrap();
    let binding = &receipt.policy_binding;
    assert_eq!(binding.capability_family, "account_recovery");
    assert_eq!(binding.entitlement_state, "offline_grace");
    assert_eq!(binding.lease_sequence, 42);
    assert!(binding.recovery_posture);
    assert!(!binding.product_ready);
    assert_eq!(receipt.reconcile_policy(), Ok(()));
    assert_eq!(receipt.presenter_posture().presenter_state, "recovery_only");

    // Blocked receipts record the recovery family with no binding claims.
    let mut blocked = acceptance(LifecycleEntitlementReceiptClass::BlockedEntitlement);
    blocked.final_state = LifecycleState::BlockedLicense;
    blocked.closure_allowed = false;
    blocked.entitlement_binding = None;
    blocked.entitlement_evidence_refs.clear();
    let blocked_receipt = LifecycleReceiptV1::from_acceptance(
        "receipt:policy-blocked",
        &blocked,
        time("2026-08-05T12:10:00Z"),
        digest('e'),
        None,
        vec![],
        None,
    )
    .unwrap();
    let blocked_binding = &blocked_receipt.policy_binding;
    assert_eq!(blocked_binding.capability_family, "account_recovery");
    assert_eq!(blocked_binding.entitlement_state, "none");
    assert_eq!(blocked_binding.lease_sequence, 0);
    assert!(blocked_binding.recovery_posture);
    assert!(!blocked_binding.product_ready);
    assert_eq!(blocked_receipt.reconcile_policy(), Ok(()));
    assert_eq!(
        blocked_receipt.presenter_posture().presenter_state,
        "denied"
    );
}

#[test]
fn receipt_tampered_policy_binding_fails_reconciliation() {
    let mut paid = acceptance(LifecycleEntitlementReceiptClass::PaidReady);
    paid.entitlement_binding = Some(binding_with(
        LifecycleEntitlementState::ActivePaid,
        "focusa",
    ));
    let mut receipt = LifecycleReceiptV1::from_acceptance(
        "receipt:policy-tamper",
        &paid,
        time("2026-08-05T12:10:00Z"),
        digest('e'),
        None,
        vec![],
        None,
    )
    .unwrap();

    // A drifted or forged policy digest fails reconciliation even when it is
    // a well-formed digest.
    receipt.policy_binding.policy_digest = digest('0');
    assert_eq!(
        receipt.reconcile_policy(),
        Err(LifecycleReceiptError::PolicyReconciliation)
    );
    assert_eq!(
        receipt.verify(RECEIPT_GENESIS_HASH),
        Err(LifecycleReceiptError::IntegrityFailure)
    );

    // A caller-invented family that is a canonical label elsewhere still
    // fails reconciliation because the canonical policy recomputes base_focusa
    // for an accepted paid install.
    let mut swapped = LifecycleReceiptV1::from_acceptance(
        "receipt:policy-swap",
        &paid,
        time("2026-08-05T12:10:00Z"),
        digest('e'),
        None,
        vec![],
        None,
    )
    .unwrap();
    swapped.policy_binding.capability_family = "automation".into();
    assert_eq!(
        swapped.reconcile_policy(),
        Err(LifecycleReceiptError::PolicyReconciliation)
    );

    // A recovery/product inconsistency is structurally rejected.
    let mut inconsistent = LifecycleReceiptV1::from_acceptance(
        "receipt:policy-inconsistent",
        &paid,
        time("2026-08-05T12:10:00Z"),
        digest('e'),
        None,
        vec![],
        None,
    )
    .unwrap();
    inconsistent.policy_binding.recovery_posture = true;
    inconsistent.policy_binding.product_ready = true;
    assert_eq!(
        inconsistent.reconcile_policy(),
        Err(LifecycleReceiptError::PolicyReconciliation)
    );
}

#[test]
fn receipt_policy_binding_never_records_raw_key_material() {
    let mut paid = acceptance(LifecycleEntitlementReceiptClass::PaidReady);
    paid.entitlement_binding = Some(binding_with(
        LifecycleEntitlementState::ActivePaid,
        "focusa",
    ));
    let receipt = LifecycleReceiptV1::from_acceptance(
        "receipt:policy-raw",
        &paid,
        time("2026-08-05T12:10:00Z"),
        digest('e'),
        None,
        vec![],
        None,
    )
    .unwrap();
    let body = serde_json::to_string(&receipt.policy_binding).unwrap();
    for fragment in [
        "license_key",
        "private_key",
        "secret_key",
        "signing_key",
        "credential",
        "email",
        "card",
        "token_id",
        "raw",
    ] {
        assert!(!body.contains(fragment), "raw material leaked: {fragment}");
    }
    assert_eq!(receipt.reconcile_policy(), Ok(()));
}
