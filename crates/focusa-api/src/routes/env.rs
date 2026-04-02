//! Env export routes.
//!
//! GET /v1/env

use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::get};
use serde_json::json;
use std::sync::Arc;

async fn env(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let base = state.config.api_bind.trim();
    let base = format!("http://{}", base);
    let proxy_base = format!("{}/proxy", base);
    Json(json!({
        "proxy_base": proxy_base,
        "messages_base_url": format!("{}/proxy", base),
        "kimi_base_url": format!("{}/proxy", base),
        "kimi_messages_base_url": format!("{}/proxy", base),
        "openai_base_url": format!("{}/proxy/v1", base),
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/env", get(env))
}
