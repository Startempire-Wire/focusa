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

fn profile_conformance(scope: &ScopeRef) -> Value {
    if scope.scope_kind == ScopeKind::Host {
        return json!({"status":"verified_not_applicable","reason":"project_profile_matrix_not_applicable_to_host_scope"});
    }
    let path = scope
        .root_path
        .join("docs/contracts/spec138-profile-activation-and-conformance-matrix.v1.yaml");
    std::fs::read_to_string(path)
        .ok()
        .and_then(|body| serde_json::from_str(&body).ok())
        .unwrap_or_else(|| {
            json!({
                "schema":"focusa.spec138_profile_activation_conformance.v1",
                "runtime_status":"degraded",
                "full_conformance_status":"unknown",
                "warnings":["Spec138 profile artifact unavailable; full conformance is blocked."]
            })
        })
}

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
    let conformance = profile_conformance(&query.scope.root_scope);
    let durable =
        PersistentPredictionAuthorityLedger::for_scope(query.scope, Some(&state.config.data_dir))
            .map_err(storage_failure)?;
    let events = durable.read_all().map_err(storage_failure)?;
    let projection = durable.projection().map_err(storage_failure)?;
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
    let conformance = profile_conformance(&scope.root_scope);
    let durable =
        PersistentPredictionAuthorityLedger::for_scope(scope, Some(&state.config.data_dir))
            .map_err(storage_failure)?;
    let events = durable.read_all().map_err(storage_failure)?;
    let projection = durable.projection().map_err(storage_failure)?;
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
