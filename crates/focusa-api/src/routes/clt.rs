//! CLT routes — Context Lineage Tree inspection.

use crate::routes::bounded::{
    BoundedReadOptions, bounded_metadata, bounded_window, budgeted_default_limit,
    budgeted_hard_limit, budgeted_requested_limit, field_projection,
    full_payload_blocked_by_pressure, project_json_fields, traversal_bounds,
};
use crate::server::AppState;
use axum::extract::{Query, State};
use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug, serde::Deserialize)]
struct NodesQuery {
    limit: Option<usize>,
    cursor: Option<String>,
    #[serde(default)]
    include_full_payload: bool,
    #[serde(default)]
    force_full_payload: bool,
    fields: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct PathQuery {
    path: Option<String>,
    depth: Option<usize>,
}

fn default_nodes_limit() -> usize {
    budgeted_default_limit("FOCUSA_LINEAGE_DEFAULT_MAX_NODES", 200)
}

fn enrich_clt_node_for_recovery(value: &mut Value) {
    let payload = value.get("payload").cloned().unwrap_or(Value::Null);
    let content_ref = payload.get("content_ref").and_then(Value::as_str).map(str::to_string);
    let summary = content_ref
        .clone()
        .or_else(|| payload.get("summary").and_then(Value::as_str).map(str::to_string))
        .or_else(|| payload.get("reason").and_then(Value::as_str).map(str::to_string))
        .unwrap_or_else(|| {
            let kind = value.get("node_type").and_then(Value::as_str).unwrap_or("node");
            let created = value.get("created_at").and_then(Value::as_str).unwrap_or("unknown_time");
            format!("{kind} at {created}")
        });
    if let Some(obj) = value.as_object_mut() {
        obj.insert("summary".to_string(), json!(summary));
        if let Some(content_ref) = content_ref {
            obj.insert("content_ref".to_string(), json!(content_ref));
        }
    }
}

fn full_nodes_limit() -> usize {
    budgeted_hard_limit("FOCUSA_LINEAGE_FULL_MAX_NODES", 2000, default_nodes_limit())
}

/// GET /v1/clt/nodes — capped CLT nodes by default; opt into larger payload with include_full_payload=true.
async fn nodes(State(state): State<Arc<AppState>>, Query(query): Query<NodesQuery>) -> Json<Value> {
    let s = state.focusa.read().await;
    let requested_full_payload = query.include_full_payload;
    let full_payload_blocked =
        full_payload_blocked_by_pressure(requested_full_payload, query.force_full_payload);
    let include_full_payload = requested_full_payload && !full_payload_blocked;
    let default_limit = default_nodes_limit();
    let full_limit = full_nodes_limit();
    let ceiling = if include_full_payload {
        full_limit
    } else {
        default_limit
    };
    let limit = budgeted_requested_limit(query.limit, default_limit.min(ceiling), ceiling);
    let total = s.clt.nodes.len();
    let fields = query.fields.clone();
    let field_projection = field_projection(
        fields.as_deref(),
        &["node_id", "parent_id", "kind", "summary", "created_at"],
        &[
            "node_id",
            "parent_id",
            "children",
            "kind",
            "content_ref",
            "summary",
            "created_at",
            "metadata",
        ],
    );
    let (nodes, window) = bounded_window(&s.clt.nodes, query.cursor.as_deref(), limit);
    let nodes = nodes
        .iter()
        .map(|node| {
            let mut value = serde_json::to_value(node).unwrap_or(Value::Null);
            enrich_clt_node_for_recovery(&mut value);
            project_json_fields(&value, &field_projection)
        })
        .collect::<Vec<_>>();
    let metadata = bounded_metadata(
        total,
        nodes.len(),
        BoundedReadOptions {
            requested_limit: query.limit,
            include_full_payload,
            summary_only: !include_full_payload,
            cursor: query.cursor,
            next_cursor: window.next_cursor.clone(),
            default_limit,
            full_limit,
        },
    );
    Json(json!({
        "nodes": nodes,
        "head_id": s.clt.head_id,
        "total": total,
        "returned": metadata.returned,
        "truncated": metadata.truncated,
        "limit": metadata.limit,
        "next_cursor": metadata.next_cursor,
        "metadata": metadata,
        "field_projection": field_projection,
        "full_payload_blocked": full_payload_blocked,
    }))
}

/// GET /v1/clt/path — lineage path from head to root.
async fn path(State(state): State<Arc<AppState>>, Query(query): Query<PathQuery>) -> Json<Value> {
    let s = state.focusa.read().await;
    let bounds = traversal_bounds(query.path.as_deref(), query.depth, 64, 64);
    let path = focusa_core::clt::lineage_path(&s.clt);
    let ids: Vec<&str> = path
        .iter()
        .take(bounds.depth)
        .map(|n| n.node_id.as_str())
        .collect();
    Json(json!({
        "path": ids,
        "depth": ids.len(),
        "total_depth": path.len(),
        "truncated": ids.len() < path.len(),
        "traversal_bounds": bounds,
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
