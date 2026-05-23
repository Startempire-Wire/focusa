//! Token CRUD routes — docs/25-capability-permissions.md
//!
//! POST /v1/tokens/create  — Create a new API token
//! POST /v1/tokens/revoke   — Revoke a token
//! GET  /v1/tokens/list     — List active tokens
//! GET  /v1/tokens/audit    — View allow/deny audit log

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use focusa_core::types::*;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

type TokenResult = Result<Json<Value>, (StatusCode, Json<Value>)>;

fn token_failure(
    http_status: StatusCode,
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
) -> (StatusCode, Json<Value>) {
    let error = error.into();
    let why = why.into();
    (
        http_status,
        Json(json!({
            "status": "blocked", "canonical": false, "degraded": true,
            "error": error, "failure_class": failure_class, "why": why,
            "recovery_hint": recovery_hint, "misuse_hint": misuse_hint,
            "next_tools": ["focusa_tool_doctor"],
            "details": {"tool_result_v1": {"ok": false, "status": "blocked", "canonical": false, "degraded": true, "failure_class": failure_class, "summary": why, "retry": {"safe": failure_class != "validation_rejected", "posture": if failure_class == "validation_rejected" { "do_not_retry_unchanged" } else { "safe_retry_after_recovery" }, "reason": failure_class}, "recovery_hint": recovery_hint, "misuse_hint": misuse_hint, "side_effects": [], "evidence_refs": [], "next_tools": ["focusa_tool_doctor"], "error": {"code": failure_class, "message": error}}}
        })),
    )
}

fn token_type_rejected(token_type: &str) -> (StatusCode, Json<Value>) {
    token_failure(
        StatusCode::BAD_REQUEST,
        format!("invalid token_type: {token_type}"),
        "validation_rejected",
        "Token create requires token_type owner, agent, or integration.",
        "Correct token_type before retrying unchanged.",
        "Likely typo, stale client enum, or incompatible token API payload.",
    )
}

fn token_not_found(token_id: uuid::Uuid) -> (StatusCode, Json<Value>) {
    token_failure(
        StatusCode::NOT_FOUND,
        format!("token not found: {token_id}"),
        "not_found",
        "Token revoke did not match an active token.",
        "List tokens and choose an active token_id before retrying revoke.",
        "Likely stale token id, already revoked token, or wrong daemon token store.",
    )
}

/// Audit log entry — records every allow/deny decision per spec §7.
#[derive(Debug, Clone, serde::Serialize)]
#[allow(dead_code)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub token_id: Option<uuid::Uuid>,
    pub agent_id: Option<String>,
    pub permission: String,
    pub outcome: &'static str,
    pub reason: String,
}

/// In-memory audit log (bounded). Production would use SQLite.
#[allow(dead_code)]
pub type AuditLog = Arc<tokio::sync::RwLock<Vec<AuditEntry>>>;

/// Create a shared audit log.
#[allow(dead_code)]
pub fn new_audit_log() -> AuditLog {
    Arc::new(tokio::sync::RwLock::new(Vec::new()))
}

/// Record an audit entry.
#[allow(dead_code)]
pub async fn record_audit(
    log: &AuditLog,
    token_id: Option<uuid::Uuid>,
    permission: &str,
    outcome: &'static str,
    reason: &str,
) {
    let mut entries = log.write().await;
    entries.push(AuditEntry {
        timestamp: chrono::Utc::now(),
        token_id,
        agent_id: None,
        permission: permission.into(),
        outcome,
        reason: reason.into(),
    });
    // Bounded: keep last 1000 entries.
    if entries.len() > 1000 {
        let remove = entries.len() - 1000;
        entries.drain(..remove);
    }
}

/// POST /v1/tokens/create
#[derive(Deserialize)]
struct CreateTokenBody {
    token_type: String,
    scopes: Vec<ScopeBody>,
    #[serde(default)]
    ttl_secs: Option<u64>,
}

#[derive(Deserialize)]
struct ScopeBody {
    domain: String,
    action: String,
}

async fn create_token(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateTokenBody>,
) -> TokenResult {
    let token_type = match body.token_type.as_str() {
        "owner" => ApiTokenType::Owner,
        "agent" => ApiTokenType::Agent,
        "integration" => ApiTokenType::Integration,
        _ => return Err(token_type_rejected(&body.token_type)),
    };

    let scopes: Vec<PermissionScope> = body
        .scopes
        .into_iter()
        .map(|s| PermissionScope {
            domain: s.domain,
            action: s.action,
        })
        .collect();

    let mut store = state.token_store.write().await;
    let token_id = store.create_token(token_type, scopes, body.ttl_secs);

    Ok(Json(json!({
        "status": "created",
        "token_id": token_id,
        "token_type": body.token_type,
    })))
}

/// POST /v1/tokens/revoke
#[derive(Deserialize)]
struct RevokeBody {
    token_id: uuid::Uuid,
}

async fn revoke_token(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RevokeBody>,
) -> TokenResult {
    let mut store = state.token_store.write().await;
    store
        .revoke(body.token_id)
        .map_err(|_| token_not_found(body.token_id))?;

    Ok(Json(json!({
        "status": "revoked",
        "token_id": body.token_id,
    })))
}

/// GET /v1/tokens/list
async fn list_tokens(State(state): State<Arc<AppState>>) -> Json<Value> {
    let store = state.token_store.read().await;
    let active = store.active_tokens();

    let tokens: Vec<Value> = active
        .iter()
        .map(|t| {
            json!({
                "token_id": t.token_id,
                "token_type": t.token_type,
                "scopes": t.scopes,
                "created_at": t.created_at,
                "expires_at": t.expires_at,
            })
        })
        .collect();

    Json(json!({ "tokens": tokens }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/tokens/create", post(create_token))
        .route("/v1/tokens/revoke", post(revoke_token))
        .route("/v1/tokens/list", get(list_tokens))
}
