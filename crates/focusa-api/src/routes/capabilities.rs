//! Capabilities API read domains (docs/23 initial closure tranche).
//!
//! Implemented endpoints:
//! - /v1/agents
//! - /v1/agents/{agent_id}
//! - /v1/agents/{agent_id}/constitution
//! - /v1/agents/{agent_id}/capabilities
//! - /v1/state/current
//! - /v1/state/history
//! - /v1/state/stack
//! - /v1/state/diff
//! - /v1/lineage/head
//! - /v1/lineage/tree
//! - /v1/lineage/node/{clt_node_id}
//! - /v1/lineage/path/{clt_node_id}
//! - /v1/lineage/children/{clt_node_id}
//! - /v1/lineage/summaries
//! - /v1/references

use crate::routes::permissions::{forbid, permission_context};
use crate::server::AppState;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::{Json, Router, routing::get};
use focusa_core::types::{CltNodeType, FrameRecord};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

const DEFAULT_AGENT_ID: &str = "focusa-default";

fn token_enabled(state: &AppState) -> bool {
    state.config.auth_token.is_some() || std::env::var("FOCUSA_AUTH_TOKEN").is_ok()
}

fn require_scope(
    headers: &HeaderMap,
    state: &AppState,
    scope: &str,
) -> Result<(), (axum::http::StatusCode, axum::Json<Value>)> {
    let permissions = permission_context(headers, token_enabled(state));
    if permissions.allows(scope) {
        Ok(())
    } else {
        Err(forbid(scope))
    }
}

#[derive(Debug, Deserialize)]
struct AgentsQuery {
    #[serde(default)]
    active: Option<bool>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    cursor: Option<String>,
}

fn active_frame(frames: &[FrameRecord], active_id: Option<uuid::Uuid>) -> Option<&FrameRecord> {
    let id = active_id?;
    frames.iter().find(|f| f.id == id)
}

async fn list_agents(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<AgentsQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "agents:read")?;
    let s = state.focusa.read().await;
    let is_active = s.session.is_some();

    if let Some(active_filter) = q.active
        && active_filter != is_active
    {
        return Ok(Json(json!({"agents": [], "next_cursor": Value::Null})));
    }

    let cap_level = if state.config.auth_token.is_some() {
        "restricted"
    } else {
        "owner_local"
    };

    let mut agents = vec![json!({
        "agent_id": DEFAULT_AGENT_ID,
        "active": is_active,
        "autonomy_level": s.autonomy.level,
        "ari_score": s.autonomy.ari_score,
        "constitution_active_version": s.constitution.active_version,
        "capability_profile": cap_level,
    })];

    if let Some(limit) = q.limit {
        agents.truncate(limit);
    }

    Ok(Json(json!({
        "agents": agents,
        "next_cursor": q.cursor.and(Some("end")),
    })))
}

async fn get_agent(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(agent_id): Path<String>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "agents:read")?;
    let s = state.focusa.read().await;
    if agent_id != DEFAULT_AGENT_ID {
        return Ok(Json(json!({"error": "agent_id not found"})));
    }

    Ok(Json(json!({
        "agent_id": DEFAULT_AGENT_ID,
        "active": s.session.is_some(),
        "autonomy": {
            "level": s.autonomy.level,
            "ari_score": s.autonomy.ari_score,
            "dimensions": s.autonomy.dimensions,
        },
        "constitution": {
            "active_version": s.constitution.active_version,
            "version_count": s.constitution.versions.len(),
        },
    })))
}

async fn get_agent_constitution(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(agent_id): Path<String>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "agents:read")?;
    let s = state.focusa.read().await;
    if agent_id != DEFAULT_AGENT_ID {
        return Ok(Json(json!({"error": "agent_id not found"})));
    }

    Ok(Json(json!({
        "agent_id": DEFAULT_AGENT_ID,
        "active_version": s.constitution.active_version,
        "versions": s.constitution.versions,
    })))
}

async fn get_agent_capabilities(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(agent_id): Path<String>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "agents:read")?;
    if agent_id != DEFAULT_AGENT_ID {
        return Ok(Json(json!({"error": "agent_id not found"})));
    }

    let permissions = permission_context(&headers, token_enabled(&state));

    Ok(Json(json!({
        "agent_id": DEFAULT_AGENT_ID,
        "token_protected": token_enabled(&state),
        "permissions": {
            "effective": permissions.list(),
        },
    })))
}

async fn state_current(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "state:read")?;
    let s = state.focusa.read().await;
    let active = active_frame(&s.focus_stack.frames, s.focus_stack.active_id);
    let focus_state = active.map(|f| &f.focus_state);

    Ok(Json(json!({
        "focus_state_id": active.map(|f| f.id.to_string()).unwrap_or_else(|| "none".to_string()),
        "revision": s.version,
        "agent_id": DEFAULT_AGENT_ID,
        "intent": focus_state.map(|f| f.intent.clone()).unwrap_or_default(),
        "constraints": focus_state.map(|f| f.constraints.clone()).unwrap_or_default(),
        "active_frame": s.focus_stack.active_id.map(|id| id.to_string()),
        "lineage_head": s.clt.head_id,
        "salient_refs": s.reference_index.handles.iter().take(25).map(|h| h.id.to_string()).collect::<Vec<_>>(),
        "confidence": (s.autonomy.ari_score / 100.0).clamp(0.0, 1.0),
        "timestamp": chrono::Utc::now(),
    })))
}

#[derive(Debug, Deserialize)]
struct HistoryQuery {
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    cursor: Option<String>,
    #[serde(default)]
    since: Option<String>,
    #[serde(default)]
    until: Option<String>,
}

async fn state_history(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "state:read")?;
    Ok(Json(json!({
        "items": Vec::<Value>::new(),
        "next_cursor": Value::Null,
        "limit": q.limit.unwrap_or(0),
        "since": q.since,
        "until": q.until,
        "cursor": q.cursor,
    })))
}

#[derive(Debug, Deserialize)]
struct DiffQuery {
    from: u64,
    to: u64,
}

async fn state_stack(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "state:read")?;
    let s = state.focusa.read().await;
    Ok(Json(json!({"stack": s.focus_stack})))
}

async fn state_diff(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<DiffQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "state:read")?;
    let s = state.focusa.read().await;
    Ok(Json(json!({
        "from": q.from,
        "to": q.to,
        "current_revision": s.version,
        "changed": q.from != q.to,
        "note": "state revision snapshots are not yet persisted; returning coarse diff metadata",
    })))
}

#[derive(Debug, Deserialize)]
struct SessionScopedQuery {
    #[serde(default)]
    session_id: Option<String>,
}

async fn lineage_head(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<SessionScopedQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "lineage:read")?;
    let s = state.focusa.read().await;
    Ok(Json(json!({
        "session_id": q.session_id,
        "head": s.clt.head_id,
    })))
}

async fn lineage_tree(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<SessionScopedQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "lineage:read")?;
    let s = state.focusa.read().await;
    let nodes: Vec<_> = s.clt.nodes.iter().cloned().collect();
    let head = s.clt.head_id.clone();
    let root = nodes
        .iter()
        .find(|node| node.parent_id.is_none())
        .map(|node| node.node_id.clone())
        .or_else(|| head.clone());

    Ok(Json(json!({
        "session_id": q.session_id,
        "root": root,
        "head": head,
        "nodes": nodes,
        "total": s.clt.nodes.len(),
    })))
}

async fn lineage_node(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(clt_node_id): Path<String>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "lineage:read")?;
    let s = state.focusa.read().await;
    let node = s.clt.nodes.iter().find(|n| n.node_id == clt_node_id);
    match node {
        Some(n) => Ok(Json(json!({"node": n}))),
        None => Ok(Json(json!({"error": "clt_node_id not found"}))),
    }
}

async fn lineage_path(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(clt_node_id): Path<String>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "lineage:read")?;
    let s = state.focusa.read().await;
    let mut out = Vec::new();
    let mut current = Some(clt_node_id);

    while let Some(id) = current {
        if let Some(node) = s.clt.nodes.iter().find(|n| n.node_id == id) {
            out.push(node.clone());
            current = node.parent_id.clone();
        } else {
            break;
        }
    }

    Ok(Json(json!({
        "path": out,
        "depth": out.len(),
    })))
}

async fn lineage_children(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(clt_node_id): Path<String>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "lineage:read")?;
    let s = state.focusa.read().await;
    let children: Vec<_> = s
        .clt
        .nodes
        .iter()
        .filter(|n| n.parent_id.as_deref() == Some(clt_node_id.as_str()))
        .cloned()
        .collect();

    Ok(Json(json!({"children": children, "total": children.len()})))
}

async fn lineage_summaries(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<SessionScopedQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "lineage:read")?;
    let s = state.focusa.read().await;
    let summaries: Vec<_> = s
        .clt
        .nodes
        .iter()
        .filter(|n| n.node_type == CltNodeType::Summary)
        .cloned()
        .collect();

    Ok(Json(json!({
        "session_id": q.session_id,
        "summaries": summaries,
        "total": summaries.len(),
    })))
}

#[derive(Debug, Deserialize)]
struct ReferencesQuery {
    #[serde(default)]
    r#type: Option<String>,
    #[serde(default)]
    tag: Option<String>,
}

async fn references(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<ReferencesQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "references:read")?;
    let s = state.focusa.read().await;

    let refs: Vec<_> = s
        .reference_index
        .handles
        .iter()
        .filter(|h| {
            q.r#type
                .as_ref()
                .map(|t| {
                    serde_json::to_value(h.kind)
                        .ok()
                        .and_then(|v| v.as_str().map(|x| x == t))
                        .unwrap_or(false)
                })
                .unwrap_or(true)
        })
        .filter(|h| {
            q.tag
                .as_ref()
                .map(|tag| h.label.to_lowercase().contains(&tag.to_lowercase()))
                .unwrap_or(true)
        })
        .cloned()
        .collect();

    Ok(Json(json!({
        "references": refs,
        "total": refs.len(),
    })))
}

#[derive(Debug, Deserialize)]
struct ReferenceSearchQuery {
    #[serde(default)]
    q: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
}

async fn reference_by_id(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(ref_id): Path<String>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "references:read")?;
    let s = state.focusa.read().await;
    let id = match uuid::Uuid::parse_str(&ref_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(Json(json!({"error": "invalid ref_id"})));
        }
    };
    let handle = s.reference_index.handles.iter().find(|h| h.id == id);
    Ok(Json(json!({"reference": handle})))
}

async fn reference_meta(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(ref_id): Path<String>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "references:read")?;
    let s = state.focusa.read().await;
    let id = match uuid::Uuid::parse_str(&ref_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(Json(json!({"error": "invalid ref_id"})));
        }
    };
    let handle = s.reference_index.handles.iter().find(|h| h.id == id);
    Ok(Json(json!({"meta": handle})))
}

async fn reference_search(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<ReferenceSearchQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "references:read")?;
    let s = state.focusa.read().await;
    let term = q.q.unwrap_or_default().to_lowercase();
    let mut hits: Vec<_> = s
        .reference_index
        .handles
        .iter()
        .filter(|h| {
            term.is_empty()
                || h.label.to_lowercase().contains(&term)
                || h.sha256.to_lowercase().contains(&term)
        })
        .cloned()
        .collect();

    if let Some(limit) = q.limit {
        hits.truncate(limit);
    }

    Ok(Json(json!({"results": hits, "total": hits.len()})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/agents", get(list_agents))
        .route("/v1/agents/{agent_id}", get(get_agent))
        .route(
            "/v1/agents/{agent_id}/constitution",
            get(get_agent_constitution),
        )
        .route(
            "/v1/agents/{agent_id}/capabilities",
            get(get_agent_capabilities),
        )
        .route("/v1/state/current", get(state_current))
        .route("/v1/state/history", get(state_history))
        .route("/v1/state/stack", get(state_stack))
        .route("/v1/state/diff", get(state_diff))
        .route("/v1/lineage/head", get(lineage_head))
        .route("/v1/lineage/tree", get(lineage_tree))
        .route("/v1/lineage/node/{clt_node_id}", get(lineage_node))
        .route("/v1/lineage/path/{clt_node_id}", get(lineage_path))
        .route("/v1/lineage/children/{clt_node_id}", get(lineage_children))
        .route("/v1/lineage/summaries", get(lineage_summaries))
        .route("/v1/references", get(references))
        .route("/v1/references/search", get(reference_search))
        .route("/v1/references/{ref_id}", get(reference_by_id))
        .route("/v1/references/{ref_id}/meta", get(reference_meta))
}
