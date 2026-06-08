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
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

const DEFAULT_AGENT_ID: &str = "focusa-default";

fn token_enabled(state: &AppState) -> bool {
    state.config.auth_token.is_some() || std::env::var("FOCUSA_AUTH_TOKEN").is_ok()
}

fn capabilities_blocked(
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
    next_tools: Vec<&'static str>,
) -> Value {
    let error = error.into();
    let why = why.into();
    let next_tools_value = json!(next_tools);
    let retry_safe = !matches!(failure_class, "validation_rejected" | "not_found");
    let retry_posture = if retry_safe {
        "safe_retry"
    } else {
        "do_not_retry_unchanged"
    };
    json!({
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
                "retry": {"safe": retry_safe, "posture": retry_posture, "reason": failure_class},
                "recovery_hint": recovery_hint,
                "misuse_hint": misuse_hint,
                "side_effects": [],
                "evidence_refs": [],
                "next_tools": next_tools_value,
                "error": {"code": failure_class, "message": error}
            }
        }
    })
}

fn agent_not_found(agent_id: &str) -> Value {
    capabilities_blocked(
        "agent_id not found",
        "not_found",
        format!("agent_id {agent_id} is not registered on this Focusa daemon"),
        "Use /v1/agents to discover the valid agent_id before requesting agent details.",
        "Likely stale agent id, wrong daemon instance, or unsupported multi-agent query.",
        vec!["focusa_tool_doctor", "focusa_project_identity"],
    )
}

fn clt_node_not_found(clt_node_id: &str) -> Value {
    capabilities_blocked(
        "clt_node_id not found",
        "not_found",
        format!("CLT node {clt_node_id} is not present in the current lineage tree"),
        "Use /v1/lineage/head or /v1/lineage/tree to discover valid node ids before node lookup.",
        "Likely stale lineage node id, wrong session tree, or pruned lineage window.",
        vec![
            "focusa_tree_head",
            "focusa_lineage_tree",
            "focusa_tool_doctor",
        ],
    )
}

fn invalid_ref_id(ref_id: &str) -> Value {
    capabilities_blocked(
        "invalid ref_id",
        "validation_rejected",
        format!("reference id {ref_id} is not a valid UUID"),
        "Use a UUID from /v1/references/search or /v1/references before requesting reference details.",
        "Likely malformed ref_id, stale docs, or route parameter mix-up.",
        vec!["focusa_tool_doctor", "focusa_traverse"],
    )
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
        return Ok(Json(agent_not_found(&agent_id)));
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
        return Ok(Json(agent_not_found(&agent_id)));
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
        return Ok(Json(agent_not_found(&agent_id)));
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

#[derive(Debug, Deserialize, Default)]
struct SessionScopedQuery {
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    max_nodes: Option<usize>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    cursor: Option<usize>,
    #[serde(default)]
    selector: Option<String>,
    #[serde(default)]
    anchor: Option<String>,
    #[serde(default)]
    depth: Option<usize>,
    #[serde(default)]
    radius: Option<usize>,
    #[serde(default)]
    include_full_payload: bool,
}

fn lineage_default_max_nodes() -> usize {
    std::env::var("FOCUSA_LINEAGE_DEFAULT_MAX_NODES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(50)
        .max(1)
}

fn lineage_full_max_nodes() -> usize {
    std::env::var("FOCUSA_LINEAGE_FULL_MAX_NODES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(2000)
        .max(lineage_default_max_nodes())
}

fn lineage_node_cap(q: &SessionScopedQuery) -> usize {
    let ceiling = if q.include_full_payload {
        lineage_full_max_nodes()
    } else {
        lineage_default_max_nodes()
    };
    q.limit.or(q.max_nodes).unwrap_or(ceiling).clamp(1, ceiling)
}

fn enriched_lineage_node_value(node: &focusa_core::types::CltNode) -> Value {
    let mut value = serde_json::to_value(node).unwrap_or(Value::Null);
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
    value
}

fn traversal_caps(limit: usize, depth: Option<usize>, radius: Option<usize>) -> Value {
    json!({
        "limit": limit,
        "depth": depth.unwrap_or(1).clamp(1, 64),
        "radius": radius.unwrap_or(1).clamp(1, 8),
    })
}

struct TraversalMetadataArgs<'a> {
    surface: &'a str,
    selector: &'a str,
    anchor: Option<&'a str>,
    returned: usize,
    total_known: usize,
    cursor: Option<usize>,
    next_cursor: Option<usize>,
    limit: usize,
    depth: Option<usize>,
    radius: Option<usize>,
    omitted: Vec<&'a str>,
}

fn traversal_metadata(args: TraversalMetadataArgs<'_>) -> Value {
    json!({
        "surface": args.surface,
        "selector": args.selector,
        "window_kind": args.selector,
        "anchor": args.anchor,
        "returned": args.returned,
        "total_known": args.total_known,
        "cursor": args.cursor,
        "next_cursor": args.next_cursor,
        "truncated": args.next_cursor.is_some() || args.returned < args.total_known,
        "caps": traversal_caps(args.limit, args.depth, args.radius),
        "omitted": args.omitted,
        "rehydrate_refs": Vec::<String>::new(),
    })
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
    let total = s.clt.nodes.len();
    let cap = lineage_node_cap(&q);
    let cursor = q.cursor.unwrap_or(0).min(total);
    let selector = q.selector.as_deref().unwrap_or("window");
    let head = s.clt.head_id.clone();
    let root = s
        .clt
        .nodes
        .iter()
        .find(|node| node.parent_id.is_none())
        .map(|node| node.node_id.clone())
        .or_else(|| head.clone());
    let anchor = q.anchor.as_deref().or(head.as_deref());

    let nodes: Vec<_> = match selector {
        "head" => s
            .clt
            .nodes
            .iter()
            .rev()
            .filter(|node| Some(node.node_id.as_str()) == head.as_deref())
            .take(cap)
            .cloned()
            .collect(),
        "children" => s
            .clt
            .nodes
            .iter()
            .filter(|node| node.parent_id.as_deref() == anchor)
            .skip(cursor)
            .take(cap)
            .cloned()
            .collect(),
        "summaries" => s
            .clt
            .nodes
            .iter()
            .filter(|node| node.node_type == CltNodeType::Summary)
            .skip(cursor)
            .take(cap)
            .cloned()
            .collect(),
        _ => s.clt.nodes.iter().skip(cursor).take(cap).cloned().collect(),
    };
    let next_cursor = (cursor + nodes.len() < total).then_some(cursor + nodes.len());
    let metadata = traversal_metadata(TraversalMetadataArgs {
        surface: "lineage",
        selector,
        anchor,
        returned: nodes.len(),
        total_known: total,
        cursor: Some(cursor),
        next_cursor,
        limit: cap,
        depth: q.depth,
        radius: q.radius,
        omitted: if q.include_full_payload {
            Vec::new()
        } else {
            vec!["full_payload"]
        },
    });

    let nodes = nodes.iter().map(enriched_lineage_node_value).collect::<Vec<_>>();
    let returned = nodes.len();
    Ok(Json(json!({
        "session_id": q.session_id,
        "root": root,
        "head": head,
        "nodes": nodes,
        "total": total,
        "returned": returned,
        "truncated": next_cursor.is_some(),
        "max_nodes": cap,
        "next_cursor": next_cursor,
        "window_kind": selector,
        "full_payload_cold_opt_in": q.include_full_payload,
        "traversal": metadata,
    })))
}

async fn lineage_neighborhood(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(clt_node_id): Path<String>,
    Query(q): Query<SessionScopedQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "lineage:read")?;
    let s = state.focusa.read().await;
    let cap = lineage_node_cap(&q);
    let radius = q.radius.unwrap_or(1).clamp(1, 8);
    let index: HashMap<&str, _> = s
        .clt
        .nodes
        .iter()
        .map(|n| (n.node_id.as_str(), n))
        .collect();
    let mut selected = Vec::new();
    let mut seen = HashSet::new();

    if let Some(anchor) = index.get(clt_node_id.as_str()) {
        selected.push((*anchor).clone());
        seen.insert(anchor.node_id.clone());
        let mut current = anchor.parent_id.as_deref();
        for _ in 0..radius {
            let Some(id) = current else {
                break;
            };
            let Some(node) = index.get(id) else {
                break;
            };
            if seen.insert(node.node_id.clone()) {
                selected.push((*node).clone());
            }
            current = node.parent_id.as_deref();
        }
        for child in s
            .clt
            .nodes
            .iter()
            .filter(|node| node.parent_id.as_deref() == Some(clt_node_id.as_str()))
            .take(cap.saturating_sub(selected.len()))
        {
            if seen.insert(child.node_id.clone()) {
                selected.push(child.clone());
            }
        }
    }
    selected.truncate(cap);
    let metadata = traversal_metadata(TraversalMetadataArgs {
        surface: "lineage",
        selector: "neighborhood",
        anchor: Some(clt_node_id.as_str()),
        returned: selected.len(),
        total_known: s.clt.nodes.len(),
        cursor: None,
        next_cursor: None,
        limit: cap,
        depth: q.depth,
        radius: Some(radius),
        omitted: vec!["full_tree"],
    });
    let selected = selected.iter().map(enriched_lineage_node_value).collect::<Vec<_>>();
    Ok(Json(json!({
        "anchor": clt_node_id,
        "nodes": selected,
        "returned": metadata["returned"],
        "truncated": metadata["truncated"],
        "traversal": metadata,
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
        Some(n) => Ok(Json(json!({"node": enriched_lineage_node_value(n)}))),
        None => Ok(Json(clt_node_not_found(&clt_node_id))),
    }
}

async fn lineage_path(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(clt_node_id): Path<String>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "lineage:read")?;
    let s = state.focusa.read().await;
    let index: HashMap<&str, _> = s
        .clt
        .nodes
        .iter()
        .map(|n| (n.node_id.as_str(), n))
        .collect();
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    let mut current = Some(clt_node_id);
    let mut truncated = false;
    const MAX_LINEAGE_PATH_DEPTH: usize = 512;

    while let Some(id) = current {
        if out.len() >= MAX_LINEAGE_PATH_DEPTH {
            truncated = true;
            break;
        }
        if !seen.insert(id.clone()) {
            truncated = true;
            break;
        }
        if let Some(node) = index.get(id.as_str()) {
            out.push((*node).clone());
            current = node.parent_id.clone();
        } else {
            break;
        }
    }

    let out = out.iter().map(enriched_lineage_node_value).collect::<Vec<_>>();
    let depth = out.len();
    Ok(Json(json!({
        "path": out,
        "depth": depth,
        "truncated": truncated,
    })))
}

async fn lineage_children(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(clt_node_id): Path<String>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "lineage:read")?;
    let s = state.focusa.read().await;
    let all_children = s
        .clt
        .nodes
        .iter()
        .filter(|n| n.parent_id.as_deref() == Some(clt_node_id.as_str()));
    let cap = lineage_default_max_nodes();
    let mut children = Vec::new();
    let mut total = 0_usize;
    for child in all_children {
        total += 1;
        if children.len() < cap {
            children.push(child.clone());
        }
    }

    let next_cursor = (children.len() < total).then_some(children.len());
    let children = children.iter().map(enriched_lineage_node_value).collect::<Vec<_>>();
    let returned = children.len();
    Ok(Json(json!({
        "children": children,
        "total": total,
        "returned": returned,
        "truncated": total > returned,
        "max_nodes": cap,
        "traversal": traversal_metadata(TraversalMetadataArgs {
            surface: "lineage",
            selector: "children",
            anchor: Some(clt_node_id.as_str()),
            returned,
            total_known: total,
            cursor: Some(0),
            next_cursor,
            limit: cap,
            depth: Some(1),
            radius: Some(1),
            omitted: vec!["full_tree"],
        }),
    })))
}

async fn lineage_summaries(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<SessionScopedQuery>,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    require_scope(&headers, &state, "lineage:read")?;
    let s = state.focusa.read().await;
    let cap = lineage_node_cap(&q);
    let mut summaries = Vec::new();
    let mut total = 0_usize;
    for node in s
        .clt
        .nodes
        .iter()
        .filter(|n| n.node_type == CltNodeType::Summary)
    {
        total += 1;
        if summaries.len() < cap {
            summaries.push(node.clone());
        }
    }

    let summaries = summaries.iter().map(enriched_lineage_node_value).collect::<Vec<_>>();
    let returned = summaries.len();
    Ok(Json(json!({
        "session_id": q.session_id,
        "summaries": summaries,
        "total": total,
        "returned": returned,
        "truncated": total > returned,
        "max_nodes": cap,
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
            return Ok(Json(invalid_ref_id(&ref_id)));
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
            return Ok(Json(invalid_ref_id(&ref_id)));
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
        .route(
            "/v1/lineage/neighborhood/{clt_node_id}",
            get(lineage_neighborhood),
        )
        .route("/v1/lineage/path/{clt_node_id}", get(lineage_path))
        .route("/v1/lineage/children/{clt_node_id}", get(lineage_children))
        .route("/v1/lineage/summaries", get(lineage_summaries))
        .route("/v1/references", get(references))
        .route("/v1/references/search", get(reference_search))
        .route("/v1/references/{ref_id}", get(reference_by_id))
        .route("/v1/references/{ref_id}/meta", get(reference_meta))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use focusa_core::types::{CltMetadata, CltNode, CltPayload};

    #[test]
    fn traversal_metadata_reports_window_caps_and_cursor() {
        let payload = traversal_metadata(TraversalMetadataArgs {
            surface: "lineage",
            selector: "window",
            anchor: Some("node-1"),
            returned: 5,
            total_known: 20,
            cursor: Some(0),
            next_cursor: Some(5),
            limit: 5,
            depth: Some(2),
            radius: Some(1),
            omitted: vec!["full_tree"],
        });
        assert_eq!(payload["surface"].as_str(), Some("lineage"));
        assert_eq!(payload["selector"].as_str(), Some("window"));
        assert_eq!(payload["returned"].as_u64(), Some(5));
        assert_eq!(payload["next_cursor"].as_u64(), Some(5));
        assert_eq!(payload["truncated"].as_bool(), Some(true));
        assert_eq!(payload["caps"]["limit"].as_u64(), Some(5));
        assert!(
            payload["omitted"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v.as_str() == Some("full_tree"))
        );
    }

    #[test]
    fn lineage_node_cap_defaults_to_surgical_window() {
        let q = SessionScopedQuery::default();
        assert!(lineage_node_cap(&q) <= 50);
    }

    #[test]
    fn lineage_node_cap_respects_limit_under_default_ceiling() {
        let q = SessionScopedQuery {
            limit: Some(3),
            ..SessionScopedQuery::default()
        };
        assert_eq!(lineage_node_cap(&q), 3);
    }

    #[test]
    fn clt_nodes_can_be_summarized_without_payload_expansion() {
        let node = CltNode {
            node_id: "node-1".to_string(),
            node_type: CltNodeType::Interaction,
            parent_id: None,
            created_at: Utc::now(),
            session_id: None,
            payload: CltPayload::Interaction {
                role: "user".to_string(),
                content_ref: Some("handle-1".to_string()),
            },
            metadata: CltMetadata::default(),
        };
        assert_eq!(node.node_id, "node-1");
        assert_eq!(node.parent_id, None);
    }
}
