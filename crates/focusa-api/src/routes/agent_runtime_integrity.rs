//! Spec140A foundational instruction-integrity and canonical-amendment API.

use crate::{routes::permissions::permission_context, server::AppState};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::{
    agent_runtime_constitution::RuntimeConstitutionEvent,
    agent_runtime_instruction_integrity::{
        CanonicalInstructionAmendment, HeadlessCapabilityParity, InstructionIntegrityRequest,
        evaluate_instruction_integrity, validate_amendment_activation, validate_amendment_proposal,
        validate_headless_parity,
    },
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

const SCHEMA: &str = "focusa.agent_runtime_instruction_integrity.v1";
type ApiError = (StatusCode, Json<Value>);

#[derive(Debug, Deserialize)]
struct IntegrityEnvelope {
    constitution_id: String,
    idempotency_key: String,
    request: InstructionIntegrityRequest,
}

#[derive(Debug, Deserialize)]
struct AmendmentEnvelope {
    constitution_id: String,
    idempotency_key: String,
    amendment: CanonicalInstructionAmendment,
}

#[derive(Debug, Deserialize)]
struct HeadlessEnvelope {
    constitution_id: String,
    idempotency_key: String,
    parity: HeadlessCapabilityParity,
}

async fn evaluate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<IntegrityEnvelope>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:write")?;
    let result = evaluate_instruction_integrity(&body.request, Utc::now());
    let event = RuntimeConstitutionEvent::InstructionIntegrityEvaluated(result.clone());
    let stored = state
        .persistence
        .append_runtime_constitution_event(
            &Uuid::now_v7().to_string(),
            &body.constitution_id,
            &body.idempotency_key,
            &event,
        )
        .map_err(internal)?;
    Ok(Json(json!({
        "schema":SCHEMA,"status":"completed","canonical":true,
        "result":result,"event_hash":stored.event_hash
    })))
}

async fn propose_amendment(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<AmendmentEnvelope>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:write")?;
    validate_amendment_proposal(&body.amendment).map_err(invalid)?;
    let event =
        RuntimeConstitutionEvent::CanonicalInstructionAmendmentProposed(body.amendment.clone());
    let stored = state
        .persistence
        .append_runtime_constitution_event(
            &Uuid::now_v7().to_string(),
            &body.constitution_id,
            &body.idempotency_key,
            &event,
        )
        .map_err(internal)?;
    Ok(Json(json!({
        "schema":SCHEMA,"status":"proposed","canonical":true,
        "amendment":body.amendment,"event_hash":stored.event_hash
    })))
}

async fn activate_amendment(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<AmendmentEnvelope>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:control")?;
    validate_amendment_activation(&body.amendment).map_err(invalid)?;
    let event =
        RuntimeConstitutionEvent::CanonicalInstructionAmendmentActivated(body.amendment.clone());
    let stored = state
        .persistence
        .append_runtime_constitution_event(
            &Uuid::now_v7().to_string(),
            &body.constitution_id,
            &body.idempotency_key,
            &event,
        )
        .map_err(internal)?;
    Ok(Json(json!({
        "schema":SCHEMA,"status":"activated","canonical":true,
        "amendment":body.amendment,"event_hash":stored.event_hash
    })))
}

async fn verify_headless(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<HeadlessEnvelope>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:write")?;
    validate_headless_parity(&body.parity).map_err(invalid)?;
    let event = RuntimeConstitutionEvent::HeadlessCapabilityParityVerified(body.parity.clone());
    let stored = state
        .persistence
        .append_runtime_constitution_event(
            &Uuid::now_v7().to_string(),
            &body.constitution_id,
            &body.idempotency_key,
            &event,
        )
        .map_err(internal)?;
    Ok(Json(json!({
        "schema":SCHEMA,"status":"verified_complete","canonical":true,
        "headless_parity":body.parity,"event_hash":stored.event_hash
    })))
}

async fn status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    Ok(Json(json!({
        "schema":SCHEMA,"status":"available","canonical":true,
        "mission_canvas_required":false,
        "dynamic_authority_outage_posture":"fail_closed_for_durable_or_consequential_actions",
        "amendment_activation_requires":"second_operator_approval_plus_complete_documentation_sweep"
    })))
}

fn require(headers: &HeaderMap, state: &AppState, permission: &str) -> Result<(), ApiError> {
    if permission_context(headers, state.config.auth_token.is_some()).allows(permission) {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error":"permission_denied","required":permission})),
        ))
    }
}

fn invalid(error: impl std::fmt::Debug) -> ApiError {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(json!({"status":"blocked","error":format!("{error:?}")})),
    )
}

fn internal(error: impl std::fmt::Display) -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"status":"blocked","error":error.to_string()})),
    )
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/v1/agent-runtime/instruction-integrity/status",
            get(status),
        )
        .route(
            "/v1/agent-runtime/instruction-integrity/evaluate",
            post(evaluate),
        )
        .route(
            "/v1/agent-runtime/amendments/propose",
            post(propose_amendment),
        )
        .route(
            "/v1/agent-runtime/amendments/activate",
            post(activate_amendment),
        )
        .route("/v1/agent-runtime/headless/verify", post(verify_headless))
}
