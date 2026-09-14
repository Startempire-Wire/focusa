use std::collections::BTreeMap;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use focusa_license::authority::{
    AuthorityKeySet, AuthorityKeyStatus, AuthorityLeasePayload, AuthorityLeaseStatus,
    AuthorityLeaseVerifier, AuthorityVerificationError, ENVELOPE_SCHEMA, EntitlementState,
    LeaseVerificationContext, SignedEnvelope,
};
use focusa_license::authority_store::{
    AUTHORITY_STATE_SCHEMA, AuthorityStoreError, PersistedAuthorityState,
    parse_production_trust_roots, resolve_authority_state,
};
use focusa_license::{Capability, CapabilityCheck, Tier, resolve_license_guard_from};
use serde::Deserialize;

const FIXTURE: &str = include_str!("fixtures/spec152-authority-golden-vector.json");
const LEASE_DOMAIN: &[u8] = b"FOCUSA-AUTHORITY-LEASE-V1\0";
const KEY_SET_DOMAIN: &[u8] = b"FOCUSA-AUTHORITY-KEY-SET-V1\0";

#[derive(Deserialize)]
struct GoldenVector {
    root_key_id: String,
    root_public_key_b64: String,
    key_set_envelope: SignedEnvelope,
    lease_envelope: SignedEnvelope,
    expected_lease_digest: String,
}

fn at(value: &str) -> DateTime<Utc> {
    value.parse().expect("valid RFC3339 fixture time")
}

fn vector() -> GoldenVector {
    serde_json::from_str(FIXTURE).expect("valid golden vector")
}

fn root_keys(vector: &GoldenVector) -> BTreeMap<String, VerifyingKey> {
    let bytes: [u8; 32] = BASE64
        .decode(&vector.root_public_key_b64)
        .expect("root key base64")
        .try_into()
        .expect("root key length");
    BTreeMap::from([(
        vector.root_key_id.clone(),
        VerifyingKey::from_bytes(&bytes).expect("root public key"),
    )])
}

fn verifier(vector: &GoldenVector) -> AuthorityLeaseVerifier {
    AuthorityLeaseVerifier::from_signed_key_set(
        &vector.key_set_envelope,
        &root_keys(vector),
        at("2026-08-03T00:00:00Z"),
        Some(7),
    )
    .expect("signed key set verifies")
}

fn context(now: &str) -> LeaseVerificationContext {
    LeaseVerificationContext {
        expected_product: "focusa".to_string(),
        expected_node_id: "node-golden-001".to_string(),
        now: at(now),
        minimum_sequence: Some(42),
        expected_previous_digest: Some(
            "sha256:4e738ca5563c06cfd0018299933d58db1dd8bf97f6973dc99bf6cdc64b5550bd".to_string(),
        ),
    }
}

fn lease_signing_key() -> SigningKey {
    SigningKey::from_bytes(&<[u8; 32]>::try_from((32u8..64).collect::<Vec<_>>()).unwrap())
}

fn root_signing_key() -> SigningKey {
    SigningKey::from_bytes(&<[u8; 32]>::try_from((0u8..32).collect::<Vec<_>>()).unwrap())
}

fn resign_lease(mutator: impl FnOnce(&mut AuthorityLeasePayload)) -> SignedEnvelope {
    let vector = vector();
    let bytes = BASE64.decode(vector.lease_envelope.payload_b64).unwrap();
    let mut payload: AuthorityLeasePayload = serde_json::from_slice(&bytes).unwrap();
    mutator(&mut payload);
    let canonical = serde_json::to_vec(&payload).unwrap();
    let signature = lease_signing_key().sign(&[LEASE_DOMAIN, &canonical].concat());
    SignedEnvelope {
        schema: ENVELOPE_SCHEMA.to_string(),
        signer_key_id: "authority-lease-2026-01".to_string(),
        payload_b64: BASE64.encode(canonical),
        signature_b64: BASE64.encode(signature.to_bytes()),
    }
}

fn resign_key_set(mutator: impl FnOnce(&mut AuthorityKeySet)) -> SignedEnvelope {
    let vector = vector();
    let bytes = BASE64.decode(vector.key_set_envelope.payload_b64).unwrap();
    let mut payload: AuthorityKeySet = serde_json::from_slice(&bytes).unwrap();
    mutator(&mut payload);
    let canonical = serde_json::to_vec(&payload).unwrap();
    let signature = root_signing_key().sign(&[KEY_SET_DOMAIN, &canonical].concat());
    SignedEnvelope {
        schema: ENVELOPE_SCHEMA.to_string(),
        signer_key_id: "authority-root-2026-01".to_string(),
        payload_b64: BASE64.encode(canonical),
        signature_b64: BASE64.encode(signature.to_bytes()),
    }
}

#[test]
fn signed_license_type_and_posture_survive_verification() {
    let vector = vector();
    let verifier = verifier(&vector);
    for (code, posture) in [
        ("focusa_operator_lifetime_v1", "paid"),
        ("focusa_uiai_operator_bundle_lifetime_v1", "bundle"),
        ("focusa_evaluation", "evaluation"),
    ] {
        let envelope = resign_lease(|payload| {
            payload.product_code = Some(code.into());
            payload.posture = Some(posture.into());
        });
        let snapshot = verifier
            .verify_lease(&envelope, &context("2026-08-03T00:00:00Z"))
            .unwrap();
        assert_eq!(snapshot.product_code.as_deref(), Some(code));
        assert_eq!(snapshot.posture.as_deref(), Some(posture));

        // Changing only the offer without signing must never promote Evaluation.
        let mut tampered = envelope;
        let bytes = BASE64.decode(&tampered.payload_b64).unwrap();
        let mut payload: AuthorityLeasePayload = serde_json::from_slice(&bytes).unwrap();
        payload.product_code = Some("unsigned_replacement".into());
        tampered.payload_b64 = BASE64.encode(serde_json::to_vec(&payload).unwrap());
        assert!(
            verifier
                .verify_lease(&tampered, &context("2026-08-03T00:00:00Z"))
                .is_err()
        );
    }
}

#[test]
fn signed_full_software_profiles_keep_security_and_resource_boundaries() {
    use focusa_license::{
        CapabilityFamily, operator_includes_software_usage, operator_license_type_grant,
        resolve_premium_family,
    };
    let vector = vector();
    let now = at("2026-08-03T00:00:00Z");
    for (product_code, posture, operator) in [
        ("focusa_uiai_operator_bundle_lifetime_v1", "bundle", true),
        ("focusa_developer", "developer", false),
    ] {
        let envelope = resign_lease(|payload| {
            payload.product_code = Some(product_code.into());
            payload.posture = Some(posture.into());
            payload.features.clear();
            payload.limits.clear();
            payload.expires_at = Utc::now() + chrono::Duration::hours(1);
        });
        let snapshot = verifier(&vector)
            .verify_lease(&envelope, &context("2026-08-03T00:00:00Z"))
            .unwrap();
        assert_eq!(
            operator_license_type_grant(&snapshot, now).is_some(),
            operator
        );
        assert_eq!(
            focusa_license::developer_license_active(&snapshot, now),
            !operator
        );
        for bucket in [
            "workpoints",
            "missions",
            "evidence_records",
            "parallel_agents",
            "silent_session_runs",
            "export_jobs",
            "release_proof_runs",
            "update_runs",
            "unattended_update_runs",
        ] {
            assert!(
                operator_includes_software_usage(&snapshot, bucket, now),
                "{bucket}"
            );
        }
        for bucket in [
            "nodes",
            "node_limit",
            "operator_seats",
            "team_operators",
            "hosted_compute",
            "unknown",
        ] {
            assert!(
                !operator_includes_software_usage(&snapshot, bucket, now),
                "{bucket}"
            );
        }
        assert!(
            resolve_premium_family(
                &snapshot,
                CapabilityFamily::Automation,
                "focusa.agent.silent_sessions",
                now
            )
            .is_feature()
        );
        let guard = focusa_license::LicenseGuard::from_entitlement(snapshot.clone());
        assert!(!guard.check(Capability::CommercialUse).is_denied());
        assert!(guard.check(Capability::HostedMode).is_denied());
        assert!(guard.check(Capability::ProductEmbedding).is_denied());
        assert!(guard.check(Capability::TelemetrySend).is_denied());

        let mut denied = snapshot.clone();
        denied
            .features
            .insert("focusa.agent.silent_sessions".into(), false);
        assert!(
            !resolve_premium_family(
                &denied,
                CapabilityFamily::Automation,
                "focusa.agent.silent_sessions",
                now
            )
            .is_feature()
        );
        for (code, posture) in [
            ("focusa_evaluation", "evaluation"),
            ("focusa_operator_lifetime_v1", "evaluation"),
            ("unknown", "paid"),
            ("focusa_developer", "paid"),
            ("focusa_operator_lifetime_v1", "developer"),
        ] {
            let mut other = snapshot.clone();
            other.product_code = Some(code.into());
            other.posture = Some(posture.into());
            assert!(!operator_includes_software_usage(&other, "workpoints", now));
        }
        let mut expired = snapshot.clone();
        expired.expires_at = Some(now - chrono::Duration::seconds(1));
        assert!(!operator_includes_software_usage(
            &expired,
            "workpoints",
            now
        ));
        let mut revoked = snapshot.clone();
        revoked.state = EntitlementState::RecoveryOnly;
        assert!(!operator_includes_software_usage(
            &revoked,
            "workpoints",
            now
        ));
        let mut wrong_product = snapshot;
        wrong_product.product = "unregistered".into();
        assert!(!operator_includes_software_usage(
            &wrong_product,
            "workpoints",
            now
        ));
    }
}

#[test]
fn legacy_signed_lease_does_not_infer_operator_license_type() {
    let vector = vector();
    let snapshot = verifier(&vector)
        .verify_lease(&vector.lease_envelope, &context("2026-08-03T00:00:00Z"))
        .unwrap();
    assert_eq!(snapshot.product_code, None);
    assert_eq!(snapshot.posture, None);
}

#[test]
fn authority_golden_vector_verifies_byte_for_byte() {
    let vector = vector();
    let snapshot = verifier(&vector)
        .verify_lease(&vector.lease_envelope, &context("2026-08-03T00:00:00Z"))
        .expect("golden lease verifies");
    assert_eq!(snapshot.state, EntitlementState::Active);
    assert_eq!(
        snapshot.lease_digest.as_deref(),
        Some(vector.expected_lease_digest.as_str())
    );
    assert!(snapshot.feature_enabled("agent_runtime"));
    assert!(!snapshot.feature_enabled("release"));
    assert_eq!(snapshot.limit("active_sessions"), Some(4));
}

#[test]
fn php_issued_developer_profile_preserves_identity_and_full_software_access() {
    #[derive(Deserialize)]
    struct DeveloperVector {
        #[serde(flatten)]
        golden: GoldenVector,
        expected_previous_digest: String,
        expected_node_id: String,
        minimum_sequence: u64,
        verification_time: String,
    }
    let fixture: DeveloperVector = serde_json::from_str(include_str!(
        "fixtures/spec152-first-party-developer-vector.json"
    ))
    .unwrap();
    let now = at(&fixture.verification_time);
    let context = LeaseVerificationContext {
        expected_product: "focusa".into(),
        expected_node_id: fixture.expected_node_id,
        now,
        minimum_sequence: Some(fixture.minimum_sequence),
        expected_previous_digest: Some(fixture.expected_previous_digest),
    };
    let snapshot = verifier(&fixture.golden)
        .verify_lease(&fixture.golden.lease_envelope, &context)
        .expect("actual PHP-issued developer envelope verifies in Rust");
    assert_eq!(snapshot.state, EntitlementState::Active);
    assert_eq!(
        snapshot.lease_digest.as_deref(),
        Some(fixture.golden.expected_lease_digest.as_str())
    );
    assert_eq!(snapshot.product_code.as_deref(), Some("focusa_developer"));
    assert_eq!(snapshot.posture.as_deref(), Some("developer"));
    assert!(focusa_license::developer_license_active(&snapshot, now));
    for feature in [
        "packaged_installer",
        "focusa.release.proof",
        "developer_channel",
        "ota_auto_update",
    ] {
        assert!(
            focusa_license::software_feature_enabled(&snapshot, feature, now),
            "{feature}"
        );
    }
    assert_eq!(snapshot.limit("node_limit"), Some(1));
    assert_eq!(snapshot.limit("operator_seats"), Some(1));
}

#[test]
fn forged_or_edited_payload_is_rejected() {
    let vector = vector();
    let mut forged = vector.lease_envelope.clone();
    let mut payload = BASE64.decode(&forged.payload_b64).unwrap();
    let position = payload.iter().position(|byte| *byte == b'4').unwrap();
    payload[position] = b'9';
    forged.payload_b64 = BASE64.encode(payload);
    assert_eq!(
        verifier(&vector).verify_lease(&forged, &context("2026-08-03T00:00:00Z")),
        Err(AuthorityVerificationError::InvalidSignature)
    );
}

#[test]
fn unsigned_developer_profile_is_rejected() {
    let vector = vector();
    let mut forged = vector.lease_envelope.clone();
    let mut payload: serde_json::Value =
        serde_json::from_slice(&BASE64.decode(&forged.payload_b64).unwrap()).unwrap();
    payload["product_code"] = serde_json::json!("focusa_developer");
    payload["posture"] = serde_json::json!("developer");
    forged.payload_b64 = BASE64.encode(serde_json::to_vec(&payload).unwrap());
    assert_eq!(
        verifier(&vector).verify_lease(&forged, &context("2026-08-03T00:00:00Z")),
        Err(AuthorityVerificationError::InvalidSignature)
    );
}

#[test]
fn wrong_product_and_node_are_rejected() {
    let vector = vector();
    let wrong_product = resign_lease(|payload| payload.product = "other-product".to_string());
    assert!(matches!(
        verifier(&vector).verify_lease(&wrong_product, &context("2026-08-03T00:00:00Z")),
        Err(AuthorityVerificationError::WrongProduct { .. })
    ));
    let wrong_node = resign_lease(|payload| payload.node_id = "foreign-node".to_string());
    assert!(matches!(
        verifier(&vector).verify_lease(&wrong_node, &context("2026-08-03T00:00:00Z")),
        Err(AuthorityVerificationError::WrongNode { .. })
    ));
}

#[test]
fn stale_sequence_and_chain_mismatch_are_rejected() {
    let vector = vector();
    let stale = resign_lease(|payload| payload.sequence = 41);
    assert_eq!(
        verifier(&vector).verify_lease(&stale, &context("2026-08-03T00:00:00Z")),
        Err(AuthorityVerificationError::StaleSequence {
            minimum: 42,
            actual: 41
        })
    );
    let wrong_chain =
        resign_lease(|payload| payload.previous_lease_digest = Some("sha256:wrong".to_string()));
    assert_eq!(
        verifier(&vector).verify_lease(&wrong_chain, &context("2026-08-03T00:00:00Z")),
        Err(AuthorityVerificationError::PreviousDigestMismatch)
    );
}

#[test]
fn expired_revoked_and_unknown_key_fail_closed() {
    let vector = vector();
    assert_eq!(
        verifier(&vector).verify_lease(&vector.lease_envelope, &context("2026-10-02T00:00:00Z")),
        Err(AuthorityVerificationError::Expired)
    );
    let revoked = resign_lease(|payload| payload.status = AuthorityLeaseStatus::Revoked);
    assert_eq!(
        verifier(&vector).verify_lease(&revoked, &context("2026-08-03T00:00:00Z")),
        Err(AuthorityVerificationError::RevokedLease)
    );
    let mut unknown = vector.lease_envelope.clone();
    unknown.signer_key_id = "unknown-key".to_string();
    assert!(matches!(
        verifier(&vector).verify_lease(&unknown, &context("2026-08-03T00:00:00Z")),
        Err(AuthorityVerificationError::UnknownKey(_))
    ));
}

#[test]
fn revoked_rotation_key_and_stale_key_set_are_rejected() {
    let vector = vector();
    let revoked = resign_key_set(|set| set.keys[0].status = AuthorityKeyStatus::Revoked);
    assert_eq!(
        AuthorityLeaseVerifier::from_signed_key_set(
            &revoked,
            &root_keys(&vector),
            at("2026-08-03T00:00:00Z"),
            Some(7),
        )
        .unwrap_err(),
        AuthorityVerificationError::EmptyKeySet
    );
    assert_eq!(
        AuthorityLeaseVerifier::from_signed_key_set(
            &vector.key_set_envelope,
            &root_keys(&vector),
            at("2026-08-03T00:00:00Z"),
            Some(8),
        )
        .unwrap_err(),
        AuthorityVerificationError::StaleSequence {
            minimum: 8,
            actual: 7
        }
    );
}

#[test]
fn production_trust_root_parser_rejects_test_and_local_roots() {
    let vector = vector();
    for key_id in [
        "test-root",
        "fixture-authority",
        "local-dev-root",
        "example-root",
    ] {
        let raw = serde_json::json!({ key_id: vector.root_public_key_b64 }).to_string();
        assert!(matches!(
            parse_production_trust_roots(&raw),
            Err(AuthorityStoreError::ForbiddenTrustRoot(_))
        ));
    }
    let production = serde_json::json!({
        "authority-root-2026-01": vector.root_public_key_b64
    })
    .to_string();
    assert_eq!(parse_production_trust_roots(&production).unwrap().len(), 1);
}

#[test]
fn signed_feature_claim_is_the_only_capability_grant() {
    let vector = vector();
    let signed = resign_lease(|payload| {
        payload.features.insert("hosted_mode".to_string(), true);
        payload.features.insert("commercial_use".to_string(), false);
    });
    let snapshot = verifier(&vector)
        .verify_lease(&signed, &context("2026-08-03T00:00:00Z"))
        .unwrap();
    let guard = focusa_license::LicenseGuard::from_entitlement(snapshot);
    assert_eq!(
        guard.check(Capability::HostedMode),
        CapabilityCheck::Permitted
    );
    assert!(guard.check(Capability::CommercialUse).is_denied());
}

#[test]
fn production_guard_ignores_plaintext_legacy_license_and_missing_state_is_unactivated() {
    let directory = std::env::temp_dir().join(format!(
        "focusa-production-guard-{}-{}",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("license.json"),
        r#"{"tier":"enterprise","status":"active","features":["hosted_mode"]}"#,
    )
    .unwrap();
    let guard = resolve_license_guard_from(
        &directory,
        Err(AuthorityStoreError::MissingTrustRoots),
        at("2026-08-03T00:00:00Z"),
    );
    assert_eq!(guard.tier, Tier::Unactivated);
    assert!(matches!(
        guard.check(Capability::HostedMode),
        CapabilityCheck::Denied { .. }
    ));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn durable_entitlement_service_is_fail_closed() {
    let vector = vector();
    let roots = root_keys(&vector);
    let directory = std::env::temp_dir().join(format!(
        "focusa-authority-store-{}-{}",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let state_path = directory.join("authority-lease.json");
    let state = PersistedAuthorityState {
        schema: AUTHORITY_STATE_SCHEMA.to_string(),
        key_set: vector.key_set_envelope,
        lease: vector.lease_envelope,
        key_set_sequence: 7,
        last_validated_at: at("2026-08-03T00:00:00Z"),
        refresh_after: Some(at("2026-08-15T00:00:00Z")),
    };
    std::fs::write(&state_path, serde_json::to_vec(&state).unwrap()).unwrap();
    let active = resolve_authority_state(
        &state_path,
        Ok(roots.clone()),
        &context("2026-08-03T00:00:00Z"),
    );
    assert_eq!(active.state, EntitlementState::Active);

    let missing = resolve_authority_state(
        &directory.join("missing.json"),
        Ok(roots.clone()),
        &context("2026-08-03T00:00:00Z"),
    );
    assert_eq!(missing.state, EntitlementState::Unactivated);

    std::fs::write(&state_path, b"not-json").unwrap();
    let malformed =
        resolve_authority_state(&state_path, Ok(roots), &context("2026-08-03T00:00:00Z"));
    assert_eq!(malformed.state, EntitlementState::RecoveryOnly);
    assert_eq!(
        malformed.recovery_reason.as_deref(),
        Some("authority_state_invalid_json")
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn offline_grace_is_bounded_and_missing_or_invalid_is_not_licensed() {
    let vector = vector();
    let verifier = verifier(&vector);
    let grace = verifier
        .verify_lease(&vector.lease_envelope, &context("2026-09-15T00:00:00Z"))
        .unwrap();
    assert_eq!(grace.state, EntitlementState::OfflineGrace);
    assert!(grace.feature_enabled("mission_canvas"));

    let missing = verifier.resolve(None, &context("2026-08-03T00:00:00Z"));
    assert_eq!(missing.state, EntitlementState::Unactivated);
    assert!(!missing.feature_enabled("agent_runtime"));

    let invalid = resign_lease(|payload| payload.product = "wrong".to_string());
    let recovery = verifier.resolve(Some(&invalid), &context("2026-08-03T00:00:00Z"));
    assert_eq!(recovery.state, EntitlementState::RecoveryOnly);
    assert_eq!(recovery.recovery_reason.as_deref(), Some("wrong_product"));
    assert!(recovery.features.is_empty());
}
