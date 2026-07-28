use crate::server::AppState;
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use focusa_core::{
    prediction_authority::{EpistemicScope, ScopedAuthorityEvent},
    prediction_authority_ledger::PredictionAuthorityLedger,
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
    if state
        .prediction_authority_store
        .get(&body.scope, &body.event.event_id)
        .await
        .map_err(internal)?
        .is_some()
    {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"status":"blocked","error":"append_only_event_exists"})),
        ));
    }
    let records = state
        .prediction_authority_store
        .recent(&body.scope, 100_000)
        .await
        .map_err(internal)?;
    let mut ledger = PredictionAuthorityLedger::default();
    for record in records {
        ledger.append(record.value).map_err(invalid)?;
    }
    ledger.append(body.event.clone()).map_err(invalid)?;
    state
        .prediction_authority_store
        .upsert(body.scope, body.event.event_id.clone(), body.event.clone())
        .await
        .map_err(internal)?;
    Ok(Json(json!({
        "status":"completed",
        "event_id":body.event.event_id,
        "sequence":body.event.sequence,
        "receipt_ref":body.event.receipt_ref
    })))
}

async fn projection(
    State(state): State<Arc<AppState>>,
    Json(query): Json<ProjectionQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let records = state
        .prediction_authority_store
        .recent(&query.scope, 100_000)
        .await
        .map_err(internal)?;
    let mut ledger = PredictionAuthorityLedger::default();
    for record in records {
        ledger.append(record.value).map_err(invalid)?;
    }
    let scope = EpistemicScope {
        project_root: query
            .scope
            .root_scope
            .root_path
            .to_string_lossy()
            .into_owned(),
        continuity_id: query.scope.continuity_id,
    };
    Ok(Json(json!({
        "status":"completed",
        "projection":ledger.project(&scope),
        "event_count":ledger.events().len()
    })))
}

fn invalid(error: String) -> (StatusCode, Json<Value>) {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(json!({"status":"blocked","error":error})),
    )
}

fn internal(error: anyhow::Error) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"status":"failed","error":error.to_string()})),
    )
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/prediction-authority/events", post(append_event))
        .route("/v1/prediction-authority/projection", post(projection))
}
