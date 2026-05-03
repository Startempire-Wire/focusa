//! CLT routes — Context Lineage Tree inspection.

use crate::server::AppState;
use axum::extract::{Query, State};
use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug, serde::Deserialize)]
struct NodesQuery {
    limit: Option<usize>,
    #[serde(default)]
    include_full_payload: bool,
}

fn default_nodes_limit() -> usize {
    std::env::var("FOCUSA_LINEAGE_DEFAULT_MAX_NODES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(200)
        .max(1)
}

fn full_nodes_limit() -> usize {
    std::env::var("FOCUSA_LINEAGE_FULL_MAX_NODES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(2000)
        .max(default_nodes_limit())
}

/// GET /v1/clt/nodes — capped CLT nodes by default; opt into larger payload with include_full_payload=true.
async fn nodes(State(state): State<Arc<AppState>>, Query(query): Query<NodesQuery>) -> Json<Value> {
    let s = state.focusa.read().await;
    let ceiling = if query.include_full_payload {
        full_nodes_limit()
    } else {
        default_nodes_limit()
    };
    let limit = query.limit.unwrap_or(ceiling).clamp(1, ceiling);
    let total = s.clt.nodes.len();
    let nodes = s.clt.nodes.iter().take(limit).cloned().collect::<Vec<_>>();
    Json(json!({
        "nodes": nodes,
        "head_id": s.clt.head_id,
        "total": total,
        "returned": nodes.len(),
        "truncated": total > nodes.len(),
        "limit": limit,
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
