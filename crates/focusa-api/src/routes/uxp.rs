//! UXP / UFI routes.

use crate::server::AppState;
use axum::extract::{Query, State};
use axum::{Json, Router, routing::get};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /v1/uxp — UXP profile.
async fn uxp_profile(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    Json(serde_json::to_value(&s.uxp).unwrap_or(json!({})))
}

#[derive(Debug, Deserialize)]
struct UfiQuery {
    #[serde(default = "default_ufi_limit")]
    limit: usize,
}

fn default_ufi_limit() -> usize {
    20
}

/// GET /v1/ufi — bounded, content-free UFI state.
async fn ufi_state(
    State(state): State<Arc<AppState>>,
    Query(query): Query<UfiQuery>,
) -> Json<Value> {
    let s = state.focusa.read().await;
    let limit = query.limit.clamp(1, 100);
    let start = s.ufi.signals.len().saturating_sub(limit);
    Json(json!({
        "schema": "focusa.ufi_bounded_view.v1",
        "aggregate": s.ufi.aggregate,
        "signal_count": s.ufi.signals.len(),
        "signals": &s.ufi.signals[start..],
        "privacy": {
            "raw_user_content_retained": false,
            "surveillance_authority": false,
            "bounded_limit": limit,
            "signal_shape": "type_timestamp_optional_session_weight_only"
        }
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/uxp", get(uxp_profile))
        .route("/v1/ufi", get(ufi_state))
}
