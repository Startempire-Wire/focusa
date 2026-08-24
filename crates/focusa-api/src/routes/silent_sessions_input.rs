use std::sync::Arc;

use axum::{Json, Router, extract::State, http::HeaderMap, http::StatusCode, routing::post};
use chrono::Utc;
use focusa_core::silent_sessions::{
    ApprovalId, EVENT_SCHEMA_VERSION, RunGeneration, SilentSessionAction, SilentSessionEvent,
    SilentSessionEventId, SilentSessionId, SilentSessionLifecycle, SilentSessionRunId,
    append_reducer_event_and_project, load_config_revision, load_durable_approval, load_run,
    load_session, load_session_events,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{middleware::principal::ApiRequestPrincipal, server::AppState};

use super::{
    silent_sessions::{
        ApiResponse, disclose_principal_side_effect, durable_request_principal,
        ensure_silent_session_temporal_guard, failure, persistence_failure,
        silent_session_temporal_context,
    },
    silent_sessions_approval_payload::{
        DeliveryKind, MAX_TEXT_BYTES, delivery_request_hash_for_approval,
        validate_approval_payload, validate_text,
    },
    silent_sessions_authorize::authorize_mutation,
    silent_sessions_contract::{
        ApiSideEffect, ExactSessionRunTarget, SilentSessionApiEnvelope, guard_exact_target,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeliveryTarget {
    run_id: SilentSessionRunId,
    generation: RunGeneration,
    approval_id: ApprovalId,
    idempotency_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct InputBody {
    #[serde(flatten)]
    target: DeliveryTarget,
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct SteerBody {
    #[serde(flatten)]
    target: DeliveryTarget,
    instruction: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct FollowUpBody {
    #[serde(flatten)]
    target: DeliveryTarget,
    prompt: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct KeysBody {
    #[serde(flatten)]
    target: DeliveryTarget,
    keys: Vec<String>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/silent-sessions/{session_id}/input", post(input))
        .route("/v1/silent-sessions/{session_id}/steer", post(steer))
        .route(
            "/v1/silent-sessions/{session_id}/follow-up",
            post(follow_up),
        )
        .route("/v1/silent-sessions/{session_id}/keys", post(keys))
}

pub(super) async fn input(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(session_id): axum::extract::Path<SilentSessionId>,
    Json(body): Json<InputBody>,
) -> ApiResponse {
    if let Err(response) = validate_text(&body.text, "text") {
        return *response;
    }
    deliver(
        state,
        headers,
        session_id,
        body.target,
        DeliveryKind::Input,
        json!({"text": body.text}),
    )
    .await
}

pub(super) async fn steer(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(session_id): axum::extract::Path<SilentSessionId>,
    Json(body): Json<SteerBody>,
) -> ApiResponse {
    if let Err(response) = validate_text(&body.instruction, "instruction") {
        return *response;
    }
    deliver(
        state,
        headers,
        session_id,
        body.target,
        DeliveryKind::Steer,
        json!({"instruction": body.instruction}),
    )
    .await
}

pub(super) async fn follow_up(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(session_id): axum::extract::Path<SilentSessionId>,
    Json(body): Json<FollowUpBody>,
) -> ApiResponse {
    if let Err(response) = validate_text(&body.prompt, "prompt") {
        return *response;
    }
    deliver(
        state,
        headers,
        session_id,
        body.target,
        DeliveryKind::FollowUp,
        json!({"prompt": body.prompt}),
    )
    .await
}

pub(super) async fn keys(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(session_id): axum::extract::Path<SilentSessionId>,
    Json(body): Json<KeysBody>,
) -> ApiResponse {
    if let Err(response) =
        validate_approval_payload(DeliveryKind::Keys, &json!({"keys": body.keys}))
    {
        return *response;
    }
    deliver(
        state,
        headers,
        session_id,
        body.target,
        DeliveryKind::Keys,
        json!({"keys": body.keys}),
    )
    .await
}

async fn deliver(
    state: Arc<AppState>,
    headers: HeaderMap,
    session_id: SilentSessionId,
    target: DeliveryTarget,
    kind: DeliveryKind,
    payload: Value,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    let mut session = match load_session(&state.persistence, session_id) {
        Ok(Some(session)) => session,
        Ok(None) => return after(not_found("session_id"), &principal),
        Err(error) => return after(persistence_failure(error), &principal),
    };
    let run = match load_run(&state.persistence, target.run_id) {
        Ok(Some(run)) => run,
        Ok(None) => return after(not_found("run_id"), &principal),
        Err(error) => return after(persistence_failure(error), &principal),
    };
    if session.current_run_generation != target.generation {
        return after(stale_target("session generation changed"), &principal);
    }
    if let Err(error) = guard_exact_target(
        ExactSessionRunTarget {
            session_id,
            run_id: target.run_id,
            generation: target.generation,
        },
        &run,
    ) {
        return after(
            stale_target(&format!("exact target rejected: {error:?}")),
            &principal,
        );
    }
    if !kind.accepts(session.lifecycle) {
        return after(invalid_state(session.lifecycle, kind), &principal);
    }
    let request_hash = request_hash(&target, kind, &payload);
    let events = match load_session_events(&state.persistence, session_id) {
        Ok(events) => events,
        Err(error) => return after(persistence_failure(error), &principal),
    };
    if let Some(existing) = events
        .iter()
        .find(|event| event.idempotency_key == target.idempotency_key)
    {
        if existing.payload.get("request_hash").and_then(Value::as_str)
            != Some(request_hash.as_str())
        {
            return after(idempotency_conflict(), &principal);
        }
        return success(
            StatusCode::OK,
            "replayed",
            existing.id.to_string(),
            true,
            &principal,
            &target,
            kind,
            existing
                .payload
                .get("side_effects")
                .and_then(Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            silent_session_temporal_context(&session, Some(&run), None, Some(events.len())),
            None,
        );
    }
    let config = match load_config_revision(&state.persistence, run.config_revision_id) {
        Ok(Some(config)) => config,
        Ok(None) => return after(not_found("config_revision_id"), &principal),
        Err(error) => return after(persistence_failure(error), &principal),
    };
    let approval = match load_durable_approval(&state.persistence, target.approval_id) {
        Ok(Some(approval)) => approval,
        Ok(None) => return after(approval_required(), &principal),
        Err(error) => return after(persistence_failure(error), &principal),
    };
    let side_effects = kind.side_effects(&request_hash);
    if let Err(response) = authorize_mutation(
        &principal,
        &session,
        &run,
        &config,
        SilentSessionAction::SendInput,
        side_effects.clone(),
        Some(approval),
    ) {
        return after(*response, &principal);
    }
    let temporal_guard =
        match ensure_silent_session_temporal_guard(&session, "silent-session:input") {
            Ok(context) => context,
            Err(response) => return after(*response, &principal),
        };
    let temporal_context = silent_session_temporal_context(
        &session,
        Some(&run),
        Some(&config.config),
        Some(events.len()),
    );
    let now = Utc::now();
    session.updated_at = now;
    let previous = events.last();
    let mut event = SilentSessionEvent {
        event_schema_version: EVENT_SCHEMA_VERSION,
        id: SilentSessionEventId::new(),
        silent_session_id: session.id,
        run_id: Some(run.id),
        sequence: previous.map_or(1, |event| event.sequence + 1),
        kind: kind.event_kind().into(),
        payload: json!({
            "delivery_kind": kind,
            "delivery_status": "queued",
            "request_hash": request_hash,
            "content": payload,
            "side_effects": side_effects,
            "governance_required": matches!(kind, DeliveryKind::Steer),
            "approval_id": target.approval_id,
        }),
        idempotency_key: target.idempotency_key.clone(),
        previous_event_hash: previous.map(|event| event.event_hash.clone()),
        event_hash: String::new(),
        occurred_at: now,
    };
    if let Err(error) = append_reducer_event_and_project(&state.persistence, &mut event, &session) {
        return after(persistence_failure(error), &principal);
    }
    success(
        StatusCode::ACCEPTED,
        kind.event_kind(),
        event.id.to_string(),
        false,
        &principal,
        &target,
        kind,
        side_effects,
        temporal_context,
        Some(temporal_guard),
    )
}

#[allow(clippy::too_many_arguments)]
fn success(
    code: StatusCode,
    status: &str,
    event_id: String,
    replayed: bool,
    principal: &ApiRequestPrincipal,
    target: &DeliveryTarget,
    kind: DeliveryKind,
    side_effects: Vec<String>,
    temporal_context: Value,
    temporal_guard: Option<Value>,
) -> ApiResponse {
    let mut envelope = SilentSessionApiEnvelope::canonical(
        status,
        json!({
            "event_id": event_id,
            "run_id": target.run_id,
            "generation": target.generation,
            "delivery_kind": kind,
            "delivery_status": "queued",
            "replayed": replayed,
            "temporal_context":temporal_context,
            "mutation_temporal_guard":temporal_guard,
        }),
    );
    envelope
        .side_effects
        .extend(side_effects.into_iter().map(|effect| ApiSideEffect {
            effect,
            status: "requested".into(),
            target_ref: Some(target.run_id.to_string()),
        }));
    envelope.receipt_refs.push(target.approval_id.to_string());
    after((code, Json(envelope)), principal)
}

fn request_hash(target: &DeliveryTarget, kind: DeliveryKind, payload: &Value) -> String {
    delivery_request_hash_for_approval(
        target.run_id,
        target.generation,
        target.approval_id,
        kind,
        payload,
    )
}

fn validation_failure(hint: &str) -> ApiResponse {
    failure(
        StatusCode::BAD_REQUEST,
        "invalid_request",
        "validation_rejected",
        hint,
    )
}

fn approval_required() -> ApiResponse {
    failure(
        StatusCode::FORBIDDEN,
        "approval_required",
        "approval_not_found",
        "Create a durable approval matching this payload-bound delivery request.",
    )
}

fn idempotency_conflict() -> ApiResponse {
    failure(
        StatusCode::CONFLICT,
        "idempotency_conflict",
        "idempotency_key_reused",
        "Retry with the original request or a new idempotency_key.",
    )
}

fn invalid_state(lifecycle: SilentSessionLifecycle, kind: DeliveryKind) -> ApiResponse {
    failure(
        StatusCode::CONFLICT,
        "invalid_state",
        "invalid_lifecycle_transition",
        &format!(
            "{} cannot be queued while session is {lifecycle:?}",
            kind.as_str()
        ),
    )
}

fn stale_target(reason: &str) -> ApiResponse {
    let mut response = failure(
        StatusCode::CONFLICT,
        "stale_target",
        "exact_target_mismatch",
        reason,
    );
    response.1.0.stale = true;
    response
}

fn not_found(target: &str) -> ApiResponse {
    failure(
        StatusCode::NOT_FOUND,
        "not_found",
        "not_found",
        &format!("No canonical record exists for {target}."),
    )
}

fn after(response: ApiResponse, principal: &ApiRequestPrincipal) -> ApiResponse {
    disclose_principal_side_effect(response, principal)
}

#[cfg(test)]
#[path = "silent_sessions_input_test.rs"]
mod tests;
