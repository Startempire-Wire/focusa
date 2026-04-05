//! Trust metrics routes.
//!
//! §35.7: Operator correction detection via trust metrics PATCH endpoint.
//!
//! PATCH /v1/trust/metrics — record a trust event (operator correction, etc.)

use crate::routes::permissions::permission_context;
use crate::routes::proxy::create_signal;
use crate::server::AppState;
use axum::extract::{State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Json, Router, routing::patch};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

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
        return Err((StatusCode::FORBIDDEN, Json(json!({"error": "forbidden"}))));
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
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "daemon unavailable"}))))?;

    Ok(Json(json!({"status": "recorded", "event": body.event})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/trust/metrics", patch(record_metric))
}
