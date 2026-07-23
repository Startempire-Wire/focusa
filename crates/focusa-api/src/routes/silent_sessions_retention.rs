//! Spec133 retention/export HTTP authority.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::post,
};
use focusa_core::silent_sessions::{
    RunGeneration, SilentSessionAction, SilentSessionId, SilentSessionRunId, export_session_bundle,
    load_retention_operation, load_run, load_session, ordinary_delete_session, purge_session,
    save_retention_operation, set_evidence_hold,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::server::AppState;

use super::{
    silent_sessions::{
        ApiResponse, authorized_projection, disclose_principal_side_effect,
        durable_request_principal, failure, persistence_failure, success_with_principal,
    },
    silent_sessions_contract::{ApiSideEffect, ExactSessionRunTarget, guard_exact_target},
    silent_sessions_retention_export::{export_as_jsonl, export_output},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExportBody {
    run_id: SilentSessionRunId,
    generation: RunGeneration,
    format: String,
    #[serde(default)]
    include_output: bool,
    idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HoldBody {
    run_id: SilentSessionRunId,
    generation: RunGeneration,
    reason: String,
    expires_at: Option<String>,
    idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct DeleteBody {
    run_id: SilentSessionRunId,
    generation: RunGeneration,
    reason: String,
    idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PurgeBody {
    run_id: SilentSessionRunId,
    generation: RunGeneration,
    #[serde(default)]
    commit: bool,
    reason: String,
    idempotency_key: String,
}

fn request_hash<T: Serialize>(action: &str, body: &T) -> String {
    let mut hasher = Sha256::new();
    hasher.update(action.as_bytes());
    hasher.update(serde_json::to_vec(body).unwrap_or_default());
    format!("{:x}", hasher.finalize())
}

fn validate_input(idempotency_key: &str, reason: Option<&str>) -> Result<(), ApiResponse> {
    if idempotency_key.trim().is_empty() || idempotency_key.len() > 200 {
        return Err(failure(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "invalid_idempotency_key",
            "Supply a non-empty idempotency_key no longer than 200 bytes.",
        ));
    }
    if reason.is_some_and(|reason| reason.trim().is_empty() || reason.len() > 500) {
        return Err(failure(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "invalid_reason",
            "Supply a non-empty reason no longer than 500 bytes.",
        ));
    }
    Ok(())
}

fn replay(
    state: &AppState,
    session_id: SilentSessionId,
    action: &str,
    idempotency_key: &str,
    principal_id: &str,
    hash: &str,
) -> Result<Option<Value>, ApiResponse> {
    match load_retention_operation(
        &state.persistence,
        session_id,
        action,
        idempotency_key,
        principal_id,
    ) {
        Ok(Some((existing_hash, response))) if existing_hash == hash => Ok(Some(response)),
        Ok(Some(_)) => Err(failure(
            StatusCode::CONFLICT,
            "idempotency_conflict",
            "idempotency_key_reused",
            "Reuse the original request unchanged or provide a new idempotency_key.",
        )),
        Ok(None) => Ok(None),
        Err(error) => Err(persistence_failure(error)),
    }
}

fn persist(
    state: &AppState,
    session_id: SilentSessionId,
    action: &str,
    idempotency_key: &str,
    principal_id: &str,
    hash: &str,
    response: &Value,
) -> Result<(), ApiResponse> {
    save_retention_operation(
        &state.persistence,
        session_id,
        action,
        idempotency_key,
        principal_id,
        hash,
        response,
    )
    .map_err(persistence_failure)
}

fn authorize(
    state: &Arc<AppState>,
    principal: &crate::middleware::principal::ApiRequestPrincipal,
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
    generation: RunGeneration,
    action: SilentSessionAction,
) -> Result<Value, ApiResponse> {
    let session = load_session(&state.persistence, session_id)
        .map_err(persistence_failure)?
        .ok_or_else(|| {
            failure(
                StatusCode::NOT_FOUND,
                "not_found",
                "not_found",
                "No Silent Session exists for the requested session_id.",
            )
        })?;
    let run = load_run(&state.persistence, run_id)
        .map_err(persistence_failure)?
        .ok_or_else(|| {
            failure(
                StatusCode::NOT_FOUND,
                "not_found",
                "run_not_found",
                "Refresh the session and use its current exact run_id.",
            )
        })?;
    if session.current_run_generation != generation
        || guard_exact_target(
            ExactSessionRunTarget {
                session_id,
                run_id,
                generation,
            },
            &run,
        )
        .is_err()
    {
        return Err(failure(
            StatusCode::CONFLICT,
            "stale_target",
            "stale_target",
            "Refresh the session and retry with its exact current run_id and generation.",
        ));
    }
    let projection = authorized_projection(principal, &session, action).ok_or_else(|| {
        disclose_principal_side_effect(
            failure(
                StatusCode::FORBIDDEN,
                "forbidden",
                "authorization_denied",
                "The authenticated principal is not authorized for this retention action.",
            ),
            principal,
        )
    })?;
    Ok(projection)
}

async fn export(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
    Json(body): Json<ExportBody>,
) -> ApiResponse {
    if !matches!(body.format.as_str(), "json" | "jsonl") {
        return failure(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "unsupported_export_format",
            "Supported formats are json and jsonl.",
        );
    }
    if let Err(response) = validate_input(&body.idempotency_key, None) {
        return response;
    }
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    let hash = request_hash("export", &body);
    if let Some(response) = match replay(
        &state,
        session_id,
        "export",
        &body.idempotency_key,
        &principal.principal.principal_id,
        &hash,
    ) {
        Ok(value) => value,
        Err(response) => return disclose_principal_side_effect(response, &principal),
    } {
        return success_with_principal("replayed", response, &principal);
    }
    if let Err(response) = authorize(
        &state,
        &principal,
        session_id,
        body.run_id,
        body.generation,
        SilentSessionAction::Export,
    ) {
        return response;
    }
    let data = match export_session_bundle(&state.persistence, session_id, body.run_id) {
        Ok(mut data) => {
            if body.include_output {
                data["output"] = match export_output(&state, session_id, body.run_id) {
                    Ok(output) => output,
                    Err(error) => {
                        return disclose_principal_side_effect(
                            persistence_failure(error),
                            &principal,
                        );
                    }
                };
            }
            data["format"] = json!(body.format);
            data["include_output"] = json!(body.include_output);
            if body.format == "jsonl" {
                match export_as_jsonl(&data) {
                    Ok(content) => json!({
                        "schema": "focusa.silent_session_export_jsonl.v1",
                        "format": "jsonl",
                        "content": content,
                    }),
                    Err(error) => {
                        return disclose_principal_side_effect(
                            persistence_failure(error),
                            &principal,
                        );
                    }
                }
            } else {
                data
            }
        }
        Err(error) => {
            return disclose_principal_side_effect(persistence_failure(error), &principal);
        }
    };
    if let Err(response) = persist(
        &state,
        session_id,
        "export",
        &body.idempotency_key,
        &principal.principal.principal_id,
        &hash,
        &data,
    ) {
        return disclose_principal_side_effect(response, &principal);
    }
    success_with_principal("exported", data, &principal)
}

async fn hold(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
    Json(body): Json<HoldBody>,
) -> ApiResponse {
    if body
        .expires_at
        .as_deref()
        .is_some_and(|expires_at| chrono::DateTime::parse_from_rfc3339(expires_at).is_err())
    {
        return failure(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "invalid_hold_expiry",
            "Supply expires_at as an RFC3339 timestamp.",
        );
    }
    retention_mutation(
        state,
        headers,
        session_id,
        body.run_id,
        body.generation,
        "evidence_hold",
        SilentSessionAction::EvidenceHold,
        &body.idempotency_key,
        Some(&body.reason),
        request_hash("evidence_hold", &body),
        |state| {
            set_evidence_hold(
                &state.persistence,
                session_id,
                &body.reason,
                body.expires_at.as_deref(),
            )
            .map(|record| json!(record))
        },
    )
    .await
}

pub(super) async fn ordinary_delete(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
    Json(body): Json<DeleteBody>,
) -> ApiResponse {
    retention_mutation(
        state,
        headers,
        session_id,
        body.run_id,
        body.generation,
        "delete",
        SilentSessionAction::Delete,
        &body.idempotency_key,
        Some(&body.reason),
        request_hash("delete", &body),
        |state| {
            ordinary_delete_session(&state.persistence, session_id, &body.reason)
                .map(|record| json!(record))
        },
    )
    .await
}

async fn purge(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
    Json(body): Json<PurgeBody>,
) -> ApiResponse {
    retention_mutation(
        state,
        headers,
        session_id,
        body.run_id,
        body.generation,
        "purge",
        SilentSessionAction::Purge,
        &body.idempotency_key,
        Some(&body.reason),
        request_hash("purge", &body),
        |state| purge_session(&state.persistence, session_id, body.commit).map(|plan| json!(plan)),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn retention_mutation<F>(
    state: Arc<AppState>,
    headers: HeaderMap,
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
    generation: RunGeneration,
    action_name: &str,
    action: SilentSessionAction,
    idempotency_key: &str,
    reason: Option<&str>,
    hash: String,
    operation: F,
) -> ApiResponse
where
    F: FnOnce(&AppState) -> anyhow::Result<Value>,
{
    if let Err(response) = validate_input(idempotency_key, reason) {
        return response;
    }
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    if let Some(response) = match replay(
        &state,
        session_id,
        action_name,
        idempotency_key,
        &principal.principal.principal_id,
        &hash,
    ) {
        Ok(value) => value,
        Err(response) => return disclose_principal_side_effect(response, &principal),
    } {
        return success_with_principal("replayed", response, &principal);
    }
    if let Err(response) = authorize(&state, &principal, session_id, run_id, generation, action) {
        return response;
    }
    let _guard = state.write_serial_lock.lock().await;
    let data = match operation(&state) {
        Ok(data) => data,
        Err(error) if error.to_string() == "evidence_hold_active" => {
            return disclose_principal_side_effect(
                failure(
                    StatusCode::CONFLICT,
                    "blocked",
                    "evidence_hold_active",
                    "Release the evidence hold through authorized retention policy before purge.",
                ),
                &principal,
            );
        }
        Err(error) => {
            return disclose_principal_side_effect(persistence_failure(error), &principal);
        }
    };
    if let Err(response) = persist(
        &state,
        session_id,
        action_name,
        idempotency_key,
        &principal.principal.principal_id,
        &hash,
        &data,
    ) {
        return disclose_principal_side_effect(response, &principal);
    }
    let mut response = success_with_principal("completed", data, &principal);
    response.1.0.side_effects.push(ApiSideEffect {
        effect: action_name.to_string(),
        status: "committed".into(),
        target_ref: Some(session_id.to_string()),
    });
    response
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/silent-sessions/{session_id}/export", post(export))
        .route("/v1/silent-sessions/{session_id}/evidence-hold", post(hold))
        .route("/v1/silent-sessions/{session_id}/purge", post(purge))
}
