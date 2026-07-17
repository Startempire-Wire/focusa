//! CLT routes — Context Lineage Tree inspection.

use crate::routes::bounded::{
    BoundedReadOptions, bounded_metadata, bounded_window, budgeted_default_limit,
    budgeted_hard_limit, budgeted_requested_limit, field_projection,
    full_payload_blocked_by_pressure, project_json_fields, traversal_bounds,
};
use crate::scope::ScopeContext;
use crate::server::AppState;
use axum::extract::{Query, State};
use axum::{Json, Router, routing::get};
use focusa_core::types::{CltPayload, CltState};
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
    let content_ref = payload
        .get("content_ref")
        .and_then(Value::as_str)
        .map(str::to_string);
    let summary = content_ref
        .clone()
        .or_else(|| {
            payload
                .get("summary")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .or_else(|| {
            payload
                .get("reason")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| {
            let kind = value
                .get("node_type")
                .and_then(Value::as_str)
                .unwrap_or("node");
            let created = value
                .get("created_at")
                .and_then(Value::as_str)
                .unwrap_or("unknown_time");
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

pub(crate) fn scoped_clt_state(clt: &CltState, scope: &ScopeContext) -> CltState {
    let project_root = scope
        .project_root
        .as_deref()
        .map(str::trim)
        .unwrap_or_default();
    let continuity_id = scope
        .continuity_id
        .as_deref()
        .map(str::trim)
        .unwrap_or_default();
    let mut nodes = clt
        .nodes
        .iter()
        .filter(|node| {
            let trajectory = node.metadata.trajectory.as_ref();
            let is_guardian_service_warning = matches!(
                &node.payload,
                CltPayload::Interaction {
                    content_ref: Some(content_ref),
                    ..
                } if content_ref.contains("summary=Guardian: service ")
            );
            !project_root.is_empty()
                && !continuity_id.is_empty()
                && !is_guardian_service_warning
                && trajectory
                    .and_then(|ctx| ctx.project_root.as_deref())
                    .map(str::trim)
                    == Some(project_root)
                && trajectory
                    .and_then(|ctx| ctx.continuity_id.as_deref())
                    .map(str::trim)
                    == Some(continuity_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut parent_id = None;
    for node in &mut nodes {
        node.parent_id = parent_id.clone();
        parent_id = Some(node.node_id.clone());
    }
    CltState {
        nodes,
        head_id: parent_id,
    }
}

/// GET /v1/clt/nodes — capped CLT nodes by default; opt into larger payload with include_full_payload=true.
async fn nodes(
    scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Query(query): Query<NodesQuery>,
) -> Json<Value> {
    let s = state.focusa.read().await;
    let clt = scoped_clt_state(&s.clt, &scope);
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
    let total = clt.nodes.len();
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
    let (nodes, window) = bounded_window(&clt.nodes, query.cursor.as_deref(), limit);
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
        "head_id": clt.head_id,
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
async fn path(
    scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Query(query): Query<PathQuery>,
) -> Json<Value> {
    let s = state.focusa.read().await;
    let clt = scoped_clt_state(&s.clt, &scope);
    let bounds = traversal_bounds(query.path.as_deref(), query.depth, 64, 64);
    let path = focusa_core::clt::lineage_path(&clt);
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
async fn stats(scope: ScopeContext, State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    let clt = scoped_clt_state(&s.clt, &scope);
    let (interactions, summaries, markers) = focusa_core::clt::node_counts(&clt);
    Json(json!({
        "interactions": interactions,
        "summaries": summaries,
        "branch_markers": markers,
        "total": clt.nodes.len(),
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/clt/nodes", get(nodes))
        .route("/v1/clt/path", get(path))
        .route("/v1/clt/stats", get(stats))
}

#[cfg(test)]
mod tests {
    use super::*;
    use focusa_core::clt::append_interaction;
    use focusa_core::types::{CltMetadata, TrajectoryLadderContext};

    fn metadata(project_root: &str, continuity_id: &str) -> CltMetadata {
        CltMetadata {
            trajectory: Some(TrajectoryLadderContext {
                project_root: Some(project_root.to_string()),
                continuity_id: Some(continuity_id.to_string()),
                ..TrajectoryLadderContext::default()
            }),
            ..CltMetadata::default()
        }
    }

    #[test]
    fn scoped_clt_drops_other_workstreams_and_rebuilds_path() {
        let mut clt = CltState::default();
        let first = append_interaction(
            &mut clt,
            None,
            "assistant",
            Some("first"),
            metadata("/repo/focusa", "cont-a"),
        );
        append_interaction(
            &mut clt,
            None,
            "system",
            Some("other"),
            metadata("/repo/other", "cont-b"),
        );
        append_interaction(
            &mut clt,
            None,
            "system",
            Some(
                "intuition_signal type=Warning severity=info summary=Guardian: service spamd is DOWN",
            ),
            metadata("/repo/focusa", "cont-a"),
        );
        let last = append_interaction(
            &mut clt,
            None,
            "assistant",
            Some("last"),
            metadata("/repo/focusa", "cont-a"),
        );
        let scope = ScopeContext {
            project_root: Some("/repo/focusa".to_string()),
            continuity_id: Some("cont-a".to_string()),
            ..ScopeContext::default()
        };

        let scoped = scoped_clt_state(&clt, &scope);

        assert_eq!(scoped.nodes.len(), 2);
        assert_eq!(scoped.nodes[0].node_id, first);
        assert_eq!(scoped.nodes[0].parent_id, None);
        assert_eq!(scoped.nodes[1].parent_id.as_deref(), Some(first.as_str()));
        assert_eq!(scoped.head_id.as_deref(), Some(last.as_str()));
    }
}
