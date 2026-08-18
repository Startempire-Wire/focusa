use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use focusa_core::{
    prediction_authority::ScopedAuthorityEvent,
    prediction_authority_storage::{PersistentPredictionAuthorityLedger, PredictionStorageError},
    scoped_state::{ScopeKind, ScopeRef, WorkstreamKey},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct AppendBody {
    scope: WorkstreamKey,
    event: ScopedAuthorityEvent,
}

#[derive(Debug, Deserialize)]
struct ProjectionQuery {
    scope: WorkstreamKey,
}

#[derive(Debug, Deserialize)]
struct ProjectionGetQuery {
    scope_kind: ScopeKind,
    scope_id: String,
    root_path: std::path::PathBuf,
    canonical_name: String,
    fingerprint: String,
    continuity_id: String,
}

impl ProjectionGetQuery {
    fn into_scope(self) -> Result<WorkstreamKey, String> {
        WorkstreamKey::new(
            ScopeRef {
                scope_kind: self.scope_kind,
                scope_id: self.scope_id,
                root_path: self.root_path,
                canonical_name: self.canonical_name,
                fingerprint: self.fingerprint,
            },
            self.continuity_id,
        )
        .and_then(|scope| {
            scope.validate()?;
            Ok(scope)
        })
        .map_err(|error| error.to_string())
    }
}

const SPEC138_PROFILE_CONFORMANCE: &str = include_str!(
    "../../../../docs/contracts/spec138-profile-activation-and-conformance-matrix.v1.yaml"
);

fn profile_conformance(scope: &ScopeRef, event_count: usize) -> Value {
    if scope.scope_kind == ScopeKind::Host {
        return json!({"status":"verified_not_applicable","reason":"project_profile_matrix_not_applicable_to_host_scope"});
    }
    let mut artifact: Value =
        serde_json::from_str(SPEC138_PROFILE_CONFORMANCE).unwrap_or_else(|error| {
            json!({
                "schema":"focusa.spec138_profile_activation_conformance.v1",
                "runtime_status":"degraded",
                "full_conformance_status":"blocked",
                "warnings":[format!("embedded Spec138 profile artifact is invalid: {error}")]
            })
        });
    let incomplete = artifact
        .get("profiles")
        .and_then(Value::as_array)
        .map(|profiles| {
            profiles.is_empty()
                || profiles.iter().any(|profile| {
                    !matches!(
                        profile.get("status").and_then(Value::as_str),
                        Some("verified_complete" | "verified_not_applicable")
                    )
                })
        })
        .unwrap_or(true);
    artifact["artifact_source"] = json!("embedded_release");
    artifact["scope_activation_status"] = if event_count == 0 {
        json!("available_unproven_for_scope")
    } else {
        json!("observed")
    };
    if incomplete {
        artifact["runtime_status"] = json!("degraded");
        artifact["full_conformance_status"] = json!("blocked");
    }
    if !artifact.get("warnings").is_some_and(Value::is_array) {
        artifact["warnings"] = json!([]);
    }
    let warnings = artifact
        .get_mut("warnings")
        .and_then(Value::as_array_mut)
        .expect("Spec138 warnings must be an array");
    if incomplete {
        warnings.push(json!(
            "Spec138 contains incomplete profile records; full conformance remains blocked."
        ));
    }
    if event_count == 0 {
        warnings.push(json!(
            "No scoped epistemic events exist; profile availability is not live activation proof."
        ));
    }
    artifact
}

#[cfg(test)]
mod conformance_tests {
    use super::profile_conformance;
    use focusa_core::scoped_state::{ScopeKind, ScopeRef};
    use std::path::PathBuf;

    #[test]
    fn conformance_is_release_embedded_and_empty_scope_is_not_activation_proof() {
        let scope = ScopeRef {
            scope_kind: ScopeKind::Project,
            scope_id: "project:test".into(),
            root_path: PathBuf::from("/tmp/focusa-test-project"),
            canonical_name: "test".into(),
            fingerprint: "fingerprint:test".into(),
        };
        let artifact = profile_conformance(&scope, 0);
        assert_eq!(artifact["artifact_source"], "embedded_release");
        assert_eq!(
            artifact["scope_activation_status"],
            "available_unproven_for_scope"
        );
        assert!(
            artifact["warnings"]
                .as_array()
                .is_some_and(|warnings| warnings.iter().any(|warning| warning
                    .as_str()
                    .is_some_and(|text| text.contains("not live activation proof"))))
        );
    }
}

#[allow(clippy::items_after_test_module)]
fn scope_matches(scope: &WorkstreamKey, authority: &WorkstreamKey) -> bool {
    scope == authority
}

async fn append_event(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AppendBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if !scope_matches(&body.scope, &body.event.scope) {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"status":"blocked","error":"scope_mismatch"})),
        ));
    }
    let durable = PersistentPredictionAuthorityLedger::for_scope(
        body.event.scope.clone(),
        Some(&state.config.data_dir),
    )
    .map_err(storage_failure)?;
    durable
        .append_batch(vec![body.event.clone()])
        .map_err(storage_failure)?;
    let crdt_warning = state
        .prediction_authority_store
        .upsert(body.scope, body.event.event_id.clone(), body.event.clone())
        .await
        .err()
        .map(|error| error.to_string());
    Ok(Json(json!({
        "status":if crdt_warning.is_some(){"completed_degraded"}else{"completed"},
        "canonical":true,
        "durability":"atomic_fsync_causal_jsonl",
        "event_id":body.event.event_id,
        "sequence":body.event.sequence,
        "receipt_ref":body.event.receipt_ref,
        "warnings":crdt_warning.into_iter().collect::<Vec<_>>()
    })))
}

async fn projection(
    State(state): State<Arc<AppState>>,
    Json(query): Json<ProjectionQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    query.scope.validate().map_err(|error| (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(json!({"status":"blocked","error":"typed_scope_invalid","reason":error.to_string()})),
    ))?;
    let root_scope = query.scope.root_scope.clone();
    let durable =
        PersistentPredictionAuthorityLedger::for_scope(query.scope, Some(&state.config.data_dir))
            .map_err(storage_failure)?;
    let events = durable.read_all().map_err(storage_failure)?;
    let projection = durable.projection().map_err(storage_failure)?;
    let conformance = profile_conformance(&root_scope, events.len());
    Ok(Json(json!({
        "status":"completed",
        "canonical":true,
        "durability":"atomic_fsync_causal_jsonl",
        "projection":projection,
        "profile_conformance":conformance,
        "event_count":events.len(),
        "legacy_event_count":events.iter().filter(|row|row.schema_version==0).count()
    })))
}

async fn projection_get(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProjectionGetQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let scope = query.into_scope().map_err(|reason| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"status":"blocked","error":"typed_scope_required","reason":reason})),
        )
    })?;
    let root_scope = scope.root_scope.clone();
    let durable =
        PersistentPredictionAuthorityLedger::for_scope(scope, Some(&state.config.data_dir))
            .map_err(storage_failure)?;
    let events = durable.read_all().map_err(storage_failure)?;
    let projection = durable.projection().map_err(storage_failure)?;
    let conformance = profile_conformance(&root_scope, events.len());
    Ok(Json(json!({
        "status":"completed","canonical":true,
        "durability":"atomic_fsync_causal_jsonl",
        "projection":projection,"profile_conformance":conformance,
        "event_count":events.len(),
        "legacy_event_count":events.iter().filter(|row|row.schema_version==0).count()
    })))
}

fn storage_failure(error: PredictionStorageError) -> (StatusCode, Json<Value>) {
    (
        StatusCode::CONFLICT,
        Json(json!({
            "status":"blocked",
            "failure_class":"prediction_authority_storage",
            "error":format!("{error:?}"),
            "recovery":["verify project scope","inspect prediction authority ledger","retry unchanged only after corruption or sequence mismatch is resolved"]
        })),
    )
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/prediction-authority/events", post(append_event))
        .route(
            "/v1/prediction-authority/projection",
            get(projection_get).post(projection),
        )
}
