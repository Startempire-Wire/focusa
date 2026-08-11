use chrono::Utc;
use ed25519_dalek::SigningKey;
use focusa_core::temporal::{TemporalEventKind, TemporalLedger, TemporalScope, project_temporal};
use serde_json::json;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::{
    temporal::{TemporalScopeDimensions, idempotent_replay_matches},
    temporal_canonical_mutation::{CanonicalMutationRequest, exact_scope, metadata_event},
    temporal_canonical_read::SPEC131_OWNED_ROUTES,
};

fn request(root: String, evidence_refs: Vec<String>, confirm: bool) -> CanonicalMutationRequest {
    CanonicalMutationRequest {
        project_root: root,
        continuity_id: "continuity:test".into(),
        dimensions: TemporalScopeDimensions::default(),
        idempotency_key: "replay-key".into(),
        confirm,
        evidence_refs,
        claim: None,
        guard: None,
        progress_signal: None,
        entity_id: Some("entity:test".into()),
        expected_revision: None,
        reason_code: Some("operator_request".into()),
        metadata: BTreeMap::new(),
    }
}

#[test]
fn canonical_mutations_fail_closed_without_evidence_or_confirmation() {
    let root = format!("/tmp/focusa-temporal-api-negative-{}", Uuid::now_v7());
    std::fs::create_dir_all(&root).unwrap();
    let missing_evidence = exact_scope(&request(root.clone(), vec![], true)).unwrap_err();
    assert_eq!(
        missing_evidence.0,
        axum::http::StatusCode::PRECONDITION_FAILED
    );
    let missing_confirmation =
        exact_scope(&request(root.clone(), vec!["evidence:test".into()], false)).unwrap_err();
    assert_eq!(
        missing_confirmation.0,
        axum::http::StatusCode::PRECONDITION_REQUIRED
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn signed_mutation_replay_is_idempotent_and_updates_projection_source() {
    let root = format!("/tmp/focusa-temporal-api-replay-{}", Uuid::now_v7());
    std::fs::create_dir_all(&root).unwrap();
    let scope = TemporalScope::project(root.clone(), "continuity:test");
    let ledger = TemporalLedger::for_project(scope.clone()).unwrap();
    let event = metadata_event(
        TemporalEventKind::CancellationRequested,
        scope.clone(),
        "replay-key",
        "cancellation",
        json!({"cancellation_id":"cancel:test"}),
        &["evidence:test".into()],
        Some(&"operator_request".into()),
    );
    let key = SigningKey::from_bytes(&[7_u8; 32]);
    let first = ledger
        .append_signed_batch("replay-key", vec![event.clone()], "test-key", &key)
        .unwrap();
    let replay = ledger
        .append_signed_batch("replay-key", vec![event], "test-key", &key)
        .unwrap();
    assert_eq!(first, replay);
    assert!(idempotent_replay_matches(
        &first,
        std::slice::from_ref(&first[0])
    ));
    let different = metadata_event(
        TemporalEventKind::ProgressObserved,
        scope.clone(),
        "replay-key",
        "progress_signal",
        json!({"signal_id":"different"}),
        &["evidence:test".into()],
        Some(&"operator_request".into()),
    );
    assert!(!idempotent_replay_matches(&first, &[different]));
    assert!(first[0].signature.is_some());
    let events = ledger.read_all().unwrap();
    assert_eq!(events.len(), 1);
    let projection = project_temporal(scope, &events, Utc::now());
    assert_eq!(projection.observed_duration_count, 0);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn canonical_route_inventory_covers_spec137_without_stealing_spec131_ownership() {
    let router_source = format!(
        "{}\n{}",
        include_str!("temporal.rs"),
        include_str!("temporal_clients.rs")
    );
    for route in [
        "/v1/time/now",
        "/v1/time/awareness",
        "/v1/time/status",
        "/v1/time/trust",
        "/v1/time/samples",
        "/v1/time/capabilities",
        "/v1/time/stream",
        "/v1/deadline/set",
        "/v1/deadline/revise",
        "/v1/deadline/clear",
        "/v1/deadlines",
        "/v1/deadline/{id}",
        "/v1/deadline/resolve-civil",
        "/v1/deadline/conflicts",
        "/v1/deadline/propagate",
        "/v1/temporal/guard/issue",
        "/v1/temporal/guard/validate",
        "/v1/temporal/guard/revoke",
        "/v1/cancellation/request",
        "/v1/cancellation/{id}",
        "/v1/estimate/request",
        "/v1/estimate/validate",
        "/v1/estimate/evaluate",
        "/v1/estimate/{id}",
        "/v1/estimate/history",
        "/v1/response/temporal-claims/validate",
        "/v1/progress/record",
        "/v1/progress/status",
        "/v1/no-progress/incidents",
        "/v1/lost-time/incidents",
        "/v1/opportunities",
        "/v1/temporal/preflight",
    ] {
        assert!(
            router_source.contains(route),
            "missing canonical route {route}"
        );
    }
    assert_eq!(SPEC131_OWNED_ROUTES.len(), 10);
    assert!(
        SPEC131_OWNED_ROUTES
            .iter()
            .all(|route| !router_source.contains(route))
    );
}
