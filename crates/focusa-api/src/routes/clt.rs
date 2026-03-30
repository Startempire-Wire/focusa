//! CLT routes — Context Lineage Tree inspection.

use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /v1/clt/nodes — all CLT nodes.
async fn nodes(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    Json(json!({
        "nodes": s.clt.nodes,
        "head_id": s.clt.head_id,
        "total": s.clt.nodes.len(),
    }))
}

/// GET /v1/clt/path — lineage path from head to root.
async fn path(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    let path = focusa_core::clt::lineage_path(&s.clt);
    let ids: Vec<&str> = path.iter().map(|n| n.node_id.as_str()).collect();
    Json(json!({
        "path": ids,
        "depth": path.len(),
    }))
}

/// GET /v1/clt/stats — node counts by type.
async fn stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    let (interactions, summaries, markers) = focusa_core::clt::node_counts(&s.clt);
    Json(json!({
        "interactions": interactions,
        "summaries": summaries,
        "branch_markers": markers,
        "total": s.clt.nodes.len(),
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/clt/nodes", get(nodes))
        .route("/v1/clt/path", get(path))
        .route("/v1/clt/stats", get(stats))
}
