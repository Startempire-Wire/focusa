//! Runtime Constitution route (#256 slice 1): every harness fetches the
//! canonical hash-bound behavioral law from here. No per-harness copies.

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::routing::get;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/runtime-constitution", get(constitution))
}

async fn constitution(State(_state): State<Arc<AppState>>) -> Json<Value> {
    let constitution = focusa_core::runtime_constitution::canonical_constitution();
    Json(json!({
        "status": "ok",
        "constitution": constitution,
    }))
}
