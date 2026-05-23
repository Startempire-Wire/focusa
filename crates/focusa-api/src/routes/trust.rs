//! Trust metrics routes.
//!
//! §35.7: Operator correction detection via trust metrics PATCH endpoint.
//!
//! PATCH /v1/trust/metrics — record a trust event (operator correction, etc.)

use crate::routes::permissions::permission_context;
use crate::routes::proxy::create_signal;
use crate::server::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::{Json, Router, routing::patch};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

fn trust_failure(
    http_status: StatusCode,
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
    next_tools: Vec<&'static str>,
) -> (StatusCode, Json<serde_json::Value>) {
    let error = error.into();
    let why = why.into();
    let next_tools_value = json!(next_tools);
    (
        http_status,
        Json(json!({
            "status": "blocked",
            "canonical": false,
            "degraded": true,
            "error": error,
            "failure_class": failure_class,
            "why": why,
            "recovery_hint": recovery_hint,
            "misuse_hint": misuse_hint,
            "next_tools": next_tools_value.clone(),
            "details": {
                "tool_result_v1": {
                    "ok": false,
                    "status": "blocked",
                    "canonical": false,
                    "degraded": true,
                    "failure_class": failure_class,
                    "summary": why,
                    "retry": {"safe": true, "posture": "safe_retry", "reason": failure_class},
                    "recovery_hint": recovery_hint,
                    "misuse_hint": misuse_hint,
                    "side_effects": [],
                    "evidence_refs": [],
                    "next_tools": next_tools_value,
                    "error": {"code": failure_class, "message": error}
                }
            }
        })),
    )
}

fn trust_forbidden() -> (StatusCode, Json<serde_json::Value>) {
    trust_failure(
        StatusCode::FORBIDDEN,
        "forbidden",
        "permission_denied",
        "Trust metrics write requires read:* or admin:* permission.",
        "Use an authorized local/admin context before retrying trust metric mutation.",
        "Likely missing auth token, wrong permission scope, or remote caller without write authorization.",
        vec!["focusa_tool_doctor"],
    )
}

fn trust_dispatch_failed(error: impl std::fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    trust_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("daemon unavailable: {error}"),
        "daemon_unavailable",
        "Trust metric signal could not be dispatched to daemon command channel.",
        "Check daemon health and retry after command channel recovery is clear.",
        "Likely daemon command channel closed, runtime shutdown, or writer/transport ownership issue.",
        vec!["focusa_tool_doctor", "focusa_work_loop_status"],
    )
}

/// Record a trust event (operator correction, model failure, etc.).
///
/// Emitted by: operator correction handler in Pi extension (§35.7).
#[derive(Deserialize)]
struct TrustMetricsBody {
    event: String,
    #[serde(default)]
    detail: Option<String>,
}

/// PATCH /v1/trust/metrics — record a trust event.
async fn record_metric(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<TrustMetricsBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let token_enabled =
        state.config.auth_token.is_some() || std::env::var("FOCUSA_AUTH_TOKEN").is_ok();
    let permissions = permission_context(&headers, token_enabled);
    if !permissions.allows("read:*") && !permissions.allows("admin:*") {
        return Err(trust_forbidden());
    }

    // §35.7: Feed operator corrections into Intuition Engine.
    // Trust events decrease autonomy score, triggering more conservative behavior.
    let summary = format!(
        "Trust event: {}",
        body.detail.as_deref().unwrap_or(&body.event)
    );

    let signal = create_signal(focusa_core::types::SignalKind::Warning, summary);
    state
        .command_tx
        .send(focusa_core::types::Action::IngestSignal { signal })
        .await
        .map_err(trust_dispatch_failed)?;

    Ok(Json(json!({"status": "recorded", "event": body.event})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/trust/metrics", patch(record_metric))
}
