use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use focusa_core::{
    prediction_authority::{EpistemicScope, ScopedAuthorityEvent},
    prediction_authority_storage::{PersistentPredictionAuthorityLedger, PredictionStorageError},
    scoped_state::WorkstreamKey,
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
    project_root: String,
    continuity_id: String,
}

fn profile_conformance(project_root: &str) -> Value {
    let path = std::path::Path::new(project_root)
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

fn scope_matches(scope: &WorkstreamKey, authority: &EpistemicScope) -> bool {
    scope.root_scope.root_path.to_string_lossy() == authority.project_root
        && scope.continuity_id == authority.continuity_id
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
    let durable = PersistentPredictionAuthorityLedger::for_project(body.event.scope.clone())
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
    Json(query): Json<ProjectionQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let scope = EpistemicScope {
        project_root: query
            .scope
            .root_scope
            .root_path
            .to_string_lossy()
            .into_owned(),
        continuity_id: query.scope.continuity_id,
    };
    let conformance = profile_conformance(&scope.project_root);
    let durable =
        PersistentPredictionAuthorityLedger::for_project(scope).map_err(storage_failure)?;
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
    Query(query): Query<ProjectionGetQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let scope = EpistemicScope {
        project_root: query.project_root,
        continuity_id: query.continuity_id,
    };
    let conformance = profile_conformance(&scope.project_root);
    let durable =
        PersistentPredictionAuthorityLedger::for_project(scope).map_err(storage_failure)?;
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
