//! Spec96 public surgical traversal facade.
//!
//! Read-only bounded traversal over large Focusa surfaces. This route is a
//! facade; individual domain routes remain authoritative for mutations and
//! deep/cold reads.

use crate::routes::bounded::{
    BoundedReadOptions, bounded_metadata, bounded_window, budgeted_default_limit,
    budgeted_hard_limit, budgeted_requested_limit, field_projection,
    full_payload_blocked_by_pressure, project_json_fields,
};
use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::post};
use focusa_core::types::{CltNodeType, FocusaState};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Deserialize, Default)]
pub struct TraverseRequest {
    pub surface: String,
    pub selector: Option<String>,
    pub anchor: Option<String>,
    pub query: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
    pub depth: Option<usize>,
    pub radius: Option<usize>,
    #[serde(default)]
    pub fields: Vec<String>,
    #[serde(default)]
    pub tags: Vec<Value>,
    pub tag_mode: Option<String>,
    #[serde(default, alias = "include_payload")]
    pub include_full_payload: bool,
    #[serde(default)]
    pub include_rehydrate_refs: bool,
    pub budget_tokens: Option<usize>,
    pub session_identity: Option<Value>,
    #[serde(default)]
    pub force_full_payload: bool,
}

fn default_limit(surface: &str) -> usize {
    match surface {
        "snapshots" => budgeted_default_limit("FOCUSA_TRAVERSE_SNAPSHOTS_DEFAULT_LIMIT", 10),
        "trajectory" => budgeted_default_limit("FOCUSA_TRAVERSE_TRAJECTORY_DEFAULT_LIMIT", 5),
        "workpoints" | "metacognition" | "predictions" => {
            budgeted_default_limit("FOCUSA_TRAVERSE_DEFAULT_LIMIT", 10)
        }
        _ => budgeted_default_limit("FOCUSA_TRAVERSE_DEFAULT_LIMIT", 25),
    }
}

fn full_limit(surface: &str, default: usize) -> usize {
    match surface {
        "lineage" | "ontology" | "telemetry" | "evidence" | "references" => {
            budgeted_hard_limit("FOCUSA_TRAVERSE_FULL_LIMIT", 200, default)
        }
        _ => budgeted_hard_limit("FOCUSA_TRAVERSE_FULL_LIMIT", 100, default),
    }
}

fn normalize_surface(surface: &str) -> String {
    surface.trim().to_ascii_lowercase().replace('-', "_")
}

fn selector(req: &TraverseRequest) -> String {
    req.selector
        .as_deref()
        .unwrap_or("window")
        .trim()
        .to_ascii_lowercase()
}

fn fields_csv(fields: &[String]) -> Option<String> {
    (!fields.is_empty()).then(|| fields.join(","))
}

fn bounded_json_items(
    items: Vec<Value>,
    req: &TraverseRequest,
    surface: &str,
    default_fields: &[&str],
    allowed_fields: &[&str],
) -> (Vec<Value>, Value, Value, bool) {
    let default_limit = default_limit(surface);
    let full_limit = full_limit(surface, default_limit);
    let full_blocked =
        full_payload_blocked_by_pressure(req.include_full_payload, req.force_full_payload);
    let include_full_payload = req.include_full_payload && !full_blocked;
    let ceiling = if include_full_payload {
        full_limit
    } else {
        default_limit
    };
    let limit = budgeted_requested_limit(req.limit, default_limit.min(ceiling), ceiling);
    let fields = fields_csv(&req.fields);
    let projection = field_projection(fields.as_deref(), default_fields, allowed_fields);
    let projected = items
        .iter()
        .map(|item| project_json_fields(item, &projection))
        .collect::<Vec<_>>();
    let total = projected.len();
    let (window, cursor_window) = bounded_window(&projected, req.cursor.as_deref(), limit);
    let metadata = json!(bounded_metadata(
        total,
        window.len(),
        BoundedReadOptions {
            requested_limit: req.limit,
            include_full_payload,
            summary_only: !include_full_payload,
            cursor: req.cursor.clone(),
            next_cursor: cursor_window.next_cursor.clone(),
            default_limit,
            full_limit,
        },
    ));
    (window, metadata, json!(projection), full_blocked)
}

fn value_id(value: &Value) -> String {
    value
        .get("node_id")
        .or_else(|| value.get("id"))
        .or_else(|| value.get("workpoint_id"))
        .or_else(|| value.get("primitive_id"))
        .or_else(|| value.get("frame_id"))
        .or_else(|| value.get("prediction_id"))
        .and_then(Value::as_str)
        .unwrap_or("item")
        .to_string()
}

fn digest_text(text: &str, len: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let hex = hex::encode(hasher.finalize());
    hex.chars().take(len.clamp(8, 64)).collect()
}

fn stable_value_digest(value: &Value) -> String {
    digest_text(&serde_json::to_string(value).unwrap_or_default(), 24)
}

fn tag_component(value: &str) -> String {
    let clean = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    let clean = clean.trim_matches('_');
    if clean.is_empty() {
        "item".to_string()
    } else {
        clean.chars().take(96).collect()
    }
}

fn make_tag(surface: &str, selector: &str, mode: &str, anchor: &str, digest: &str) -> String {
    format!(
        "focusa://{}/{}/{}/{}/{}",
        tag_component(surface),
        tag_component(selector),
        tag_component(mode),
        tag_component(anchor),
        tag_component(digest)
    )
}

fn tag_record(
    surface: &str,
    selector: &str,
    mode: &str,
    anchor: &str,
    digest: &str,
    index: Option<usize>,
) -> Value {
    json!({
        "tag": make_tag(surface, selector, mode, anchor, digest),
        "tag_mode": mode,
        "surface": surface,
        "selector": selector,
        "anchor": anchor,
        "digest": digest,
        "index": index,
        "collision_policy": "sha256_24_hex_with_anchor; on collision request fields plus longer tag",
        "long_tag_policy": "stable 24-hex digest by default; clients may request full 64-hex verification via future tag_version",
    })
}

fn item_tags(surface: &str, selector: &str, items: &[Value]) -> Vec<Value> {
    items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let anchor = value_id(item);
            let digest = stable_value_digest(item);
            tag_record(surface, selector, "item", &anchor, &digest, Some(idx))
        })
        .collect()
}

fn aggregate_digest(items: &[Value]) -> String {
    let parts = items
        .iter()
        .map(|item| format!("{}:{}", value_id(item), stable_value_digest(item)))
        .collect::<Vec<_>>()
        .join("|");
    digest_text(&parts, 24)
}

fn aggregate_tags(surface: &str, selector: &str, items: &[Value], traversal: &Value) -> Vec<Value> {
    let cursor = traversal
        .get("cursor")
        .and_then(Value::as_str)
        .unwrap_or("0");
    let next_string = traversal
        .get("next_cursor")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            traversal
                .get("returned")
                .and_then(Value::as_u64)
                .map(|n| n.to_string())
        })
        .unwrap_or_else(|| "0".to_string());
    let next = next_string.as_str();
    let limit = traversal
        .get("limit")
        .and_then(Value::as_u64)
        .map(|n| n.to_string())
        .unwrap_or_else(|| items.len().to_string());
    let total = traversal
        .get("total")
        .and_then(Value::as_u64)
        .map(|n| n.to_string())
        .unwrap_or_else(|| items.len().to_string());
    let digest = aggregate_digest(items);
    vec![
        tag_record(
            surface,
            selector,
            "range",
            &format!("{cursor}-{next}"),
            &digest,
            None,
        ),
        tag_record(
            surface,
            selector,
            "window",
            &format!("{cursor}:{limit}"),
            &digest,
            None,
        ),
        tag_record(
            surface,
            selector,
            "surface",
            &format!("{surface}:{total}"),
            &digest_text(&format!("{surface}:{total}:{digest}"), 24),
            None,
        ),
    ]
}

fn parse_tag(tag: &str) -> Option<(String, String, String, String, String)> {
    let rest = tag.strip_prefix("focusa://")?;
    let parts = rest.split('/').collect::<Vec<_>>();
    if parts.len() != 5 {
        return None;
    }
    Some((
        parts[0].to_string(),
        parts[1].to_string(),
        parts[2].to_string(),
        parts[3].to_string(),
        parts[4].to_string(),
    ))
}

fn tag_index(records: &[Value]) -> BTreeMap<String, Value> {
    records
        .iter()
        .filter_map(|record| {
            record
                .get("tag")
                .and_then(Value::as_str)
                .map(|tag| (tag.to_string(), record.clone()))
        })
        .collect()
}

fn scope_from_item(item: &Value) -> Value {
    json!({
        "project_root": item.get("project_root").cloned().unwrap_or(Value::Null),
        "session_id": item.get("session_id").cloned().unwrap_or(Value::Null),
        "frame_id": item.get("frame_id").or_else(|| item.get("id")).cloned().unwrap_or(Value::Null),
        "workpoint_id": item.get("workpoint_id").cloned().unwrap_or(Value::Null),
    })
}

fn traversed_items(surface: &str, selector: &str, items: &[Value]) -> Vec<Value> {
    let item_records = item_tags(surface, selector, items);
    items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let tag = item_records
                .get(idx)
                .and_then(|record| record.get("tag"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            json!({
                "anchor": value_id(item),
                "ordinal": idx,
                "tag": tag,
                "surface_version": stable_value_digest(item),
                "freshness": "live",
                "scope": scope_from_item(item),
                "kind": item.get("kind").or_else(|| item.get("node_type")).or_else(|| item.get("status")).cloned().unwrap_or(Value::Null),
                "label": item.get("label").or_else(|| item.get("title")).or_else(|| item.get("work_item_id")).cloned().unwrap_or(Value::Null),
                "summary": item.get("summary").or_else(|| item.get("mission")).or_else(|| item.get("next_slice")).cloned().unwrap_or(Value::Null),
                "data": item,
            })
        })
        .collect()
}

fn requested_tag_strings(req: &TraverseRequest) -> Vec<String> {
    req.tags
        .iter()
        .filter_map(|tag| {
            tag.as_str()
                .map(str::to_string)
                .or_else(|| tag.get("tag").and_then(Value::as_str).map(str::to_string))
        })
        .collect()
}

fn adopt_verify_selector_from_requested_tags(req: &mut TraverseRequest) {
    if selector(req) != "tags_verify" {
        return;
    }
    if let Some(tag_selector) = requested_tag_strings(req)
        .into_iter()
        .find_map(|tag| parse_tag(&tag).map(|(_, selector, _, _, _)| selector))
        .filter(|selector| !selector.trim().is_empty())
    {
        req.selector = Some(tag_selector);
    }
}

fn verify_requested_tags(
    req: &TraverseRequest,
    items: &[Value],
    traversal: &Value,
) -> (Vec<Value>, Vec<Value>) {
    let surface = normalize_surface(&req.surface);
    let sel = selector(req);
    let mut current_records = item_tags(&surface, &sel, items);
    current_records.extend(aggregate_tags(&surface, &sel, items, traversal));
    let current = tag_index(&current_records);
    let mut verified = Vec::new();
    let mut stale = Vec::new();
    for tag in requested_tag_strings(req) {
        match parse_tag(&tag) {
            Some((tag_surface, tag_selector, mode, anchor, digest)) => {
                if let Some(record) = current.get(&tag) {
                    verified.push(json!({
                        "tag": tag,
                        "tag_mode": mode,
                        "surface": tag_surface,
                        "selector": tag_selector,
                        "anchor": anchor,
                        "digest": digest,
                        "verified": true,
                        "record": record,
                    }));
                } else {
                    stale.push(json!({
                        "tag": tag,
                        "tag_mode": mode,
                        "surface": tag_surface,
                        "selector": tag_selector,
                        "anchor": anchor,
                        "digest": digest,
                        "verified": false,
                        "reason": "tag digest, anchor, selector, or window no longer matches current bounded slice",
                    }));
                }
            }
            None => {
                stale.push(json!({"tag": tag, "verified": false, "reason": "invalid_tag_format"}))
            }
        }
    }
    (verified, stale)
}

fn active_frame_value(state: &FocusaState) -> Option<Value> {
    let frame = state
        .focus_stack
        .active_id
        .and_then(|id| state.focus_stack.frames.iter().find(|frame| frame.id == id))
        .or_else(|| state.focus_stack.frames.last())?;
    serde_json::to_value(frame).ok()
}

fn active_workpoint_value(state: &FocusaState) -> Option<Value> {
    let record = state
        .workpoint
        .active_workpoint_id
        .and_then(|id| {
            state
                .workpoint
                .records
                .iter()
                .find(|record| record.workpoint_id == id)
        })
        .or_else(|| state.workpoint.records.last())?;
    serde_json::to_value(record).ok()
}

fn trajectory_items(state: &FocusaState) -> Vec<Value> {
    let frame = active_frame_value(state);
    let workpoint = active_workpoint_value(state);
    let ladder = state.trajectory_ladder_context();
    vec![json!({
        "id": "active_project_trajectory",
        "project_identity": {
            "project_root": frame.as_ref().and_then(|f| f.get("project_root")).cloned().unwrap_or(Value::Null),
            "continuity_id": frame.as_ref().and_then(|f| f.get("continuity_id")).cloned().unwrap_or(Value::Null),
            "workpoint_id": workpoint.as_ref().and_then(|w| w.get("workpoint_id")).cloned().unwrap_or(Value::Null),
        },
        "trajectory": {
            "long_term_goal": ladder
                .as_ref()
                .and_then(|ctx| ctx.hlt.clone())
                .map(Value::String)
                .or_else(|| frame.as_ref().and_then(|f| f.get("goal")).cloned())
                .unwrap_or(Value::Null),
            "mid_level_goal": ladder
                .as_ref()
                .and_then(|ctx| ctx.mlg.clone())
                .map(Value::String)
                .unwrap_or(Value::Null),
            "short_term_goal": ladder
                .as_ref()
                .and_then(|ctx| ctx.stg.clone())
                .map(Value::String)
                .unwrap_or(Value::Null),
            "waypoints": ladder
                .as_ref()
                .map(|ctx| json!(ctx.waypoints))
                .unwrap_or(Value::Null),
            "current_state": frame
                .as_ref()
                .and_then(|f| f.pointer("/focus_state/current_state"))
                .cloned()
                .unwrap_or(Value::Null),
            "active_gap": workpoint
                .as_ref()
                .and_then(|w| w.get("next_slice"))
                .cloned()
                .unwrap_or(Value::Null),
            "workpoint_candidate": workpoint,
            "trajectory_ladder": ladder,
        },
        "advisory_only": true,
    })]
}

fn metacognition_items(state: &FocusaState) -> Vec<Value> {
    let Some(frame) = active_frame_value(state) else {
        return Vec::new();
    };
    let focus_state = frame.get("focus_state").cloned().unwrap_or(Value::Null);
    let mut out = Vec::new();
    for (kind, pointer) in [
        ("decision", "/decisions"),
        ("constraint", "/constraints"),
        ("failure", "/failures"),
        ("recent_result", "/recent_results"),
        ("open_question", "/open_questions"),
    ] {
        if let Some(values) = focus_state.pointer(pointer).and_then(Value::as_array) {
            for (idx, value) in values.iter().enumerate() {
                out.push(json!({
                    "id": format!("{kind}:{idx}"),
                    "kind": kind,
                    "content": value,
                    "source": "focus_state_projection",
                }));
            }
        }
    }
    out
}

fn prediction_items(state: &FocusaState) -> Vec<Value> {
    vec![json!({
        "id": "prediction_stats_summary",
        "kind": "prediction_stats",
        "summary": "Use /v1/predictions/recent or focusa_predict_recent for persisted prediction records.",
        "telemetry_total_events": state.telemetry.total_events,
        "verification_result_events": state.telemetry.verification_result_events,
    })]
}

fn snapshot_items(state: &FocusaState) -> Vec<Value> {
    vec![json!({
        "id": "snapshot_current_head_summary",
        "kind": "snapshot_summary",
        "lineage_head": state.clt.head_id,
        "state_version": state.version,
        "summary": "Use focusa_tree_recent_snapshots for persisted snapshot records; traverse exposes current head metadata only by default.",
    })]
}

fn reflex_primitive_items(req: &TraverseRequest, sel: &str) -> Vec<Value> {
    let registry: Value = serde_json::from_str(include_str!(
        "../../../../docs/current/focusa-reflex-primitives.json"
    ))
    .unwrap_or_else(|_| json!({"primitives": []}));
    let family = req
        .anchor
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let risk_or_object_query = req
        .query
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let primitives = registry
        .get("primitives")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    primitives
        .into_iter()
        .filter(|primitive| match sel {
            "family" | "children" => {
                family.is_empty()
                    || primitive
                        .get("family")
                        .and_then(Value::as_str)
                        .map(|value| value.eq_ignore_ascii_case(&family))
                        .unwrap_or(false)
            }
            _ => true,
        })
        .filter(|primitive| {
            risk_or_object_query.is_empty()
                || serde_json::to_string(primitive)
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .contains(&risk_or_object_query)
        })
        .map(|mut primitive| {
            if let Some(obj) = primitive.as_object_mut() {
                obj.insert(
                    "source".to_string(),
                    json!("spec97_reflex_primitive_registry"),
                );
                obj.insert("advisory_only".to_string(), json!(true));
            }
            primitive
        })
        .collect()
}

fn generic_filter_items(mut items: Vec<Value>, req: &TraverseRequest, sel: &str) -> Vec<Value> {
    let anchor = req.anchor.as_deref().unwrap_or_default();
    let query = req
        .query
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    match sel {
        "current" | "by_id" => {
            if anchor.is_empty() {
                items.into_iter().take(1).collect()
            } else {
                items
                    .into_iter()
                    .filter(|item| {
                        value_id(item) == anchor
                            || serde_json::to_string(item)
                                .unwrap_or_default()
                                .contains(anchor)
                    })
                    .collect()
            }
        }
        "search" => {
            if query.is_empty() {
                items
            } else {
                items
                    .into_iter()
                    .filter(|item| {
                        serde_json::to_string(item)
                            .unwrap_or_default()
                            .to_ascii_lowercase()
                            .contains(&query)
                    })
                    .collect()
            }
        }
        "recent" => {
            items.reverse();
            items
        }
        _ => items,
    }
}

fn lineage_items(state: &FocusaState, req: &TraverseRequest, sel: &str) -> Vec<Value> {
    let anchor = req.anchor.as_deref();
    let head = state.clt.head_id.as_deref();
    let radius = req.radius.unwrap_or(1).clamp(1, 8);
    let nodes = match sel {
        "head" => state
            .clt
            .nodes
            .iter()
            .rev()
            .filter(|node| Some(node.node_id.as_str()) == head)
            .cloned()
            .collect::<Vec<_>>(),
        "children" => state
            .clt
            .nodes
            .iter()
            .filter(|node| node.parent_id.as_deref() == anchor.or(head))
            .cloned()
            .collect::<Vec<_>>(),
        "summaries" => state
            .clt
            .nodes
            .iter()
            .filter(|node| node.node_type == CltNodeType::Summary)
            .cloned()
            .collect::<Vec<_>>(),
        "path" => focusa_core::clt::lineage_path(&state.clt)
            .into_iter()
            .take(req.depth.unwrap_or(64).clamp(1, 64))
            .cloned()
            .collect::<Vec<_>>(),
        "neighborhood" => {
            let target = anchor.or(head).unwrap_or_default();
            let mut out = Vec::new();
            for node in &state.clt.nodes {
                if node.node_id == target || node.parent_id.as_deref() == Some(target) {
                    out.push(node.clone());
                }
            }
            out.into_iter()
                .take(radius.saturating_mul(8))
                .collect::<Vec<_>>()
        }
        _ => state.clt.nodes.clone(),
    };
    nodes
        .iter()
        .filter_map(|node| serde_json::to_value(node).ok())
        .collect()
}

fn surface_items(
    state: &FocusaState,
    req: &TraverseRequest,
    surface: &str,
    sel: &str,
) -> Vec<Value> {
    let query = req
        .query
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let items = match surface {
        "trajectory" => trajectory_items(state),
        "lineage" | "tree" | "clt" => lineage_items(state, req, sel),
        "ontology" => match sel {
            "links" | "adjacency" | "neighborhood" => state.ontology.links.clone(),
            "proposals" => state
                .ontology
                .proposals
                .iter()
                .filter_map(|item| serde_json::to_value(item).ok())
                .collect(),
            _ => state.ontology.objects.clone(),
        },
        "focus_stack" | "frames" if sel == "current" => {
            active_frame_value(state).into_iter().collect()
        }
        "focus_stack" | "frames" => state
            .focus_stack
            .frames
            .iter()
            .filter_map(|frame| serde_json::to_value(frame).ok())
            .collect(),
        "workpoints" | "workpoint" if sel == "current" => {
            active_workpoint_value(state).into_iter().collect()
        }
        "workpoints" | "workpoint" => state
            .workpoint
            .records
            .iter()
            .filter_map(|record| serde_json::to_value(record).ok())
            .collect(),
        "evidence" | "ecs" | "references" => state
            .reference_index
            .handles
            .iter()
            .filter(|handle| query.is_empty() || handle.label.to_ascii_lowercase().contains(&query))
            .filter_map(|handle| serde_json::to_value(handle).ok())
            .collect(),
        "telemetry" | "turns" | "commands" => state.telemetry.trace_events.clone(),
        "metacognition" | "metacog" => metacognition_items(state),
        "predictions" | "prediction" => prediction_items(state),
        "snapshots" | "snapshot" => snapshot_items(state),
        "reflex" | "reflexes" | "reflex_primitives" => reflex_primitive_items(req, sel),
        "tool_registry" | "capabilities" => vec![json!({
            "id": "tool_registry_summary",
            "surface": "tool_registry",
            "summary": "Use /v1/ontology/tool-contracts or focusa_tool_doctor for the full bounded registry.",
            "next_tool": "focusa_tool_doctor"
        })],
        _ => Vec::new(),
    };
    generic_filter_items(items, req, sel)
}

fn surface_defaults(surface: &str) -> (&'static [&'static str], &'static [&'static str]) {
    match surface {
        "trajectory" => (
            &["id", "project_identity", "trajectory", "advisory_only"],
            &[
                "id",
                "project_identity",
                "trajectory",
                "advisory_only",
                "context_sufficiency",
            ],
        ),
        "lineage" | "tree" | "clt" => (
            &["node_id", "parent_id", "node_type", "summary", "created_at"],
            &[
                "node_id",
                "parent_id",
                "node_type",
                "payload",
                "summary",
                "created_at",
                "metadata",
            ],
        ),
        "focus_stack" | "frames" => (
            &["id", "title", "goal", "status", "continuity_id"],
            &[
                "id",
                "title",
                "goal",
                "status",
                "continuity_id",
                "project_root",
                "tags",
                "created_at",
            ],
        ),
        "reflex" | "reflexes" | "reflex_primitives" => (
            &[
                "primitive_id",
                "family",
                "trigger",
                "reflex_action",
                "advisory_only",
            ],
            &[
                "primitive_id",
                "family",
                "trigger",
                "context_inputs",
                "reflex_action",
                "evidence_output",
                "escalation_boundary",
                "authority_boundary",
                "hot_path_budget",
                "failure_envelope",
                "implementation_status",
                "source",
                "advisory_only",
            ],
        ),
        "workpoints" | "workpoint" => (
            &[
                "workpoint_id",
                "status",
                "mission",
                "next_slice",
                "updated_at",
            ],
            &[
                "workpoint_id",
                "work_item_id",
                "status",
                "mission",
                "next_slice",
                "canonical",
                "project_root",
                "continuity_id",
                "updated_at",
            ],
        ),
        "evidence" | "ecs" | "references" => (
            &["id", "kind", "label", "trajectory", "created_at"],
            &[
                "id",
                "kind",
                "label",
                "trajectory",
                "created_at",
                "pinned",
                "session_id",
                "size",
                "sha256",
            ],
        ),
        _ => (
            &["id", "label", "summary", "status"],
            &[
                "id",
                "label",
                "summary",
                "status",
                "payload",
                "created_at",
                "updated_at",
            ],
        ),
    }
}

fn traverse_response(state: &FocusaState, req: TraverseRequest, verify_only: bool) -> Value {
    let surface = normalize_surface(&req.surface);
    let sel = if verify_only {
        "tags_verify".to_string()
    } else {
        selector(&req)
    };
    let supported = matches!(
        surface.as_str(),
        "trajectory"
            | "lineage"
            | "tree"
            | "clt"
            | "ontology"
            | "focus_stack"
            | "frames"
            | "workpoints"
            | "workpoint"
            | "evidence"
            | "ecs"
            | "references"
            | "metacognition"
            | "metacog"
            | "predictions"
            | "prediction"
            | "telemetry"
            | "turns"
            | "commands"
            | "snapshots"
            | "snapshot"
            | "reflex"
            | "reflexes"
            | "reflex_primitives"
            | "tool_registry"
            | "capabilities"
    );
    if !supported {
        let reflex_suggestions =
            crate::routes::reflex::reflex_suggestions_for_failure("validation_rejected");
        return json!({
            "status": "validation_rejected",
            "canonical": false,
            "degraded": true,
            "failure_class": "validation_rejected",
            "items": [],
            "summary": "unsupported traversal surface or selector",
            "do_not_use": ["unsupported_surface"],
            "traversal": {
                "surface": surface,
                "selector": sel,
                "returned": 0,
                "total": 0,
                "truncated": false,
                "caps": {"limit": 0, "depth": 0, "radius": 0, "payload_bytes": 0, "budget_tokens": req.budget_tokens},
                "omitted": ["unsupported_surface"],
                "rehydrate_refs": [],
                "stale_tags": [],
                "verified_tags": []
            },
            "tag_scheme": {
                "version": "focusa-traverse-tag-v1",
                "algorithm": "opaque_version",
                "length": 24,
                "includes_anchor": true,
                "includes_surface_version": true,
                "collision_policy": "retry_with_longer_tag"
            },
            "next_tools": ["focusa_tool_doctor"],
            "reflex_suggestions": reflex_suggestions,
            "details": {"tool_result_v1": {"ok": false, "status": "validation_rejected", "failure_class": "validation_rejected", "canonical": false, "degraded": true, "reflex_suggestions": reflex_suggestions}}
        });
    }

    let raw_items = surface_items(state, &req, &surface, &selector(&req));
    let (default_fields, allowed_fields) = surface_defaults(&surface);
    let (items, metadata, field_projection, full_payload_blocked) =
        bounded_json_items(raw_items, &req, &surface, default_fields, allowed_fields);
    let returned = items.len();
    let total = metadata
        .get("total")
        .and_then(Value::as_u64)
        .unwrap_or(returned as u64);
    let truncated = metadata
        .get("truncated")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let limit_value = metadata.get("limit").cloned().unwrap_or(Value::Null);
    let omitted_count = metadata.get("omitted").and_then(Value::as_u64).unwrap_or(0);
    let omitted = if omitted_count > 0 {
        vec![format!("items_omitted:{omitted_count}")]
    } else {
        Vec::<String>::new()
    };
    let rehydrate_refs = if req.include_rehydrate_refs || truncated || full_payload_blocked {
        vec![format!(
            "focusa://traverse/{}/{}?cursor={}",
            surface,
            sel,
            metadata
                .get("next_cursor")
                .and_then(Value::as_str)
                .unwrap_or("0")
        )]
    } else {
        Vec::<String>::new()
    };
    let traversal_meta = json!({
        "surface": surface,
        "selector": sel,
        "anchor": req.anchor,
        "query": req.query,
        "cursor": metadata.get("cursor").cloned().unwrap_or(Value::Null),
        "next_cursor": metadata.get("next_cursor").cloned().unwrap_or(Value::Null),
        "returned": returned,
        "total": total,
        "total_known": total,
        "truncated": truncated,
        "limit": limit_value,
        "caps": {
            "limit": metadata.get("limit").and_then(Value::as_u64).unwrap_or(0),
            "depth": req.depth.unwrap_or(1).clamp(1, 64),
            "radius": req.radius.unwrap_or(1).clamp(1, 8),
            "payload_bytes": metadata.get("payload_bytes").cloned().unwrap_or(Value::Null),
            "budget_tokens": req.budget_tokens,
        },
        "depth": req.depth.unwrap_or(1).clamp(1, 64),
        "radius": req.radius.unwrap_or(1).clamp(1, 8),
        "fields": field_projection,
        "metadata": metadata,
        "omitted": omitted,
        "rehydrate_refs": rehydrate_refs,
    });
    let mut tags = item_tags(&surface, &sel, &items);
    tags.extend(aggregate_tags(&surface, &sel, &items, &traversal_meta));
    let (verified_tags, stale_tags) = verify_requested_tags(&req, &items, &traversal_meta);
    let mut traversal_meta = traversal_meta;
    if let Some(obj) = traversal_meta.as_object_mut() {
        obj.insert(
            "verified_tags".to_string(),
            Value::Array(verified_tags.clone()),
        );
        obj.insert("stale_tags".to_string(), Value::Array(stale_tags.clone()));
    }
    let response_items = if verify_only {
        Vec::<Value>::new()
    } else {
        traversed_items(&surface, &sel, &items)
    };
    let degraded = full_payload_blocked || !stale_tags.is_empty();
    let failure_class = if full_payload_blocked {
        json!("resource_exhausted")
    } else if !stale_tags.is_empty() {
        json!("read_model_lag")
    } else {
        Value::Null
    };
    json!({
        "status": if degraded { "degraded" } else { "completed" },
        "canonical": !degraded,
        "degraded": degraded,
        "failure_class": failure_class,
        "surface": surface,
        "selector": sel,
        "anchor": req.anchor,
        "project_identity": req.session_identity.as_ref().and_then(|value| value.get("project_identity")).cloned().unwrap_or(Value::Null),
        "items": response_items,
        "summary": format!("traverse surface={} selector={} returned={} truncated={}", surface, sel, returned, truncated),
        "do_not_use": if full_payload_blocked { vec!["full_payload_without_budget"] } else { Vec::<&str>::new() },
        "verified_tags": verified_tags,
        "stale_tags": stale_tags,
        "traversal": traversal_meta,
        "tag_scheme": {
            "version": "focusa-traverse-tag-v1",
            "algorithm": "opaque_version",
            "length": 24,
            "includes_anchor": true,
            "includes_surface_version": true,
            "collision_policy": "retry_with_longer_tag",
            "modes": ["item", "range", "window", "surface"],
            "requested_tag_mode": req.tag_mode.as_deref().unwrap_or("mixed"),
            "item_tag_format": "focusa://{surface}/{selector}/item/{anchor}/{sha256_24}",
            "range_tag_format": "focusa://{surface}/{selector}/range/{start-end}/{sha256_24}",
            "window_tag_format": "focusa://{surface}/{selector}/window/{cursor-limit}/{sha256_24}",
            "surface_tag_format": "focusa://{surface}/{selector}/surface/{surface-total}/{sha256_24}",
            "long_tag_policy": "stable 24-hex digest by default; future versions may use full 64-hex digest",
            "tags_verify_endpoint": "/v1/traverse/verify-tags"
        },
        "tags": tags,
        "next_tools": ["focusa_traverse", "focusa_trajectory_view", "focusa_workpoint_resume"],
        "reflex_suggestions": if full_payload_blocked { crate::routes::reflex::reflex_suggestions_for_failure("resource_exhausted") } else { Vec::new() },
        "details": {"tool_result_v1": {"ok": !degraded, "status": if degraded { "degraded" } else { "completed" }, "failure_class": failure_class, "canonical": !degraded, "degraded": degraded, "reflex_suggestions": if full_payload_blocked { crate::routes::reflex::reflex_suggestions_for_failure("resource_exhausted") } else { Vec::new() }}}
    })
}

async fn traverse(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TraverseRequest>,
) -> Json<Value> {
    let s = state.focusa.read().await;
    Json(traverse_response(&s, req, false))
}

async fn verify_tags(
    State(state): State<Arc<AppState>>,
    Json(mut req): Json<TraverseRequest>,
) -> Json<Value> {
    adopt_verify_selector_from_requested_tags(&mut req);
    let s = state.focusa.read().await;
    Json(traverse_response(&s, req, true))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/traverse", post(traverse))
        .route("/v1/traverse/verify-tags", post(verify_tags))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with_trajectory() -> FocusaState {
        let mut state = FocusaState::new();
        state.trajectory.active_trajectory_id = Some("traj-test".to_string());
        state
            .trajectory
            .records
            .push(focusa_core::types::TrajectoryProjectionRecord {
                trajectory_id: "traj-test".to_string(),
                project_root: Some("/tmp/focusa-test".to_string()),
                continuity_id: Some("cont-test".to_string()),
                long_term_goal: "High-level target".to_string(),
                mid_level_goal: Some("Mid-level target".to_string()),
                short_term_goal: Some("Short-term target".to_string()),
                waypoints: vec!["Waypoint A".to_string(), "Waypoint B".to_string()],
                ..focusa_core::types::TrajectoryProjectionRecord::default()
            });
        state
    }

    #[test]
    fn trajectory_surface_projects_ladder_context() {
        let state = state_with_trajectory();
        let res = traverse_response(
            &state,
            TraverseRequest {
                surface: "trajectory".to_string(),
                selector: Some("window".to_string()),
                limit: Some(1),
                ..TraverseRequest::default()
            },
            false,
        );
        let trajectory = res
            .pointer("/items/0/data/trajectory")
            .expect("trajectory projection");
        assert_eq!(
            trajectory.get("long_term_goal").and_then(Value::as_str),
            Some("High-level target")
        );
        assert_eq!(
            trajectory.get("mid_level_goal").and_then(Value::as_str),
            Some("Mid-level target")
        );
        assert_eq!(
            trajectory
                .pointer("/trajectory_ladder/trajectory_id")
                .and_then(Value::as_str),
            Some("traj-test")
        );
        assert_eq!(
            trajectory
                .get("waypoints")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(2)
        );
    }

    #[test]
    fn evidence_surface_default_projection_includes_trajectory_context() {
        let mut state = state_with_trajectory();
        let trajectory = state.trajectory_ladder_context();
        state
            .reference_index
            .handles
            .push(focusa_core::types::HandleRef {
                id: uuid::Uuid::now_v7(),
                kind: focusa_core::types::HandleKind::Text,
                label: "proof-handle".to_string(),
                size: 123,
                sha256: "deadbeef".to_string(),
                created_at: chrono::Utc::now(),
                session_id: None,
                pinned: false,
                trajectory,
            });
        let res = traverse_response(
            &state,
            TraverseRequest {
                surface: "evidence".to_string(),
                selector: Some("window".to_string()),
                limit: Some(1),
                ..TraverseRequest::default()
            },
            false,
        );
        let item = res.pointer("/items/0/data").expect("evidence item");
        assert_eq!(
            item.get("label").and_then(Value::as_str),
            Some("proof-handle")
        );
        assert_eq!(
            item.pointer("/trajectory/trajectory_id")
                .and_then(Value::as_str),
            Some("traj-test")
        );
        assert!(
            item.get("sha256").is_none(),
            "sha256 stays out of default projection"
        );
    }

    #[test]
    fn unsupported_surface_returns_blocked_tool_envelope() {
        let state = FocusaState::new();
        let res = traverse_response(
            &state,
            TraverseRequest {
                surface: "unknown".to_string(),
                ..TraverseRequest::default()
            },
            false,
        );
        assert_eq!(
            res.get("status").and_then(Value::as_str),
            Some("validation_rejected")
        );
        assert_eq!(
            res.pointer("/details/tool_result_v1/failure_class")
                .and_then(Value::as_str),
            Some("validation_rejected")
        );
    }

    #[test]
    fn tag_verify_preserves_item_tag_after_unrelated_change() {
        let mut state = FocusaState::new();
        state.clt.nodes.push(focusa_core::types::CltNode {
            node_id: "n1".to_string(),
            parent_id: None,
            node_type: CltNodeType::Interaction,
            created_at: chrono::Utc::now(),
            session_id: None,
            payload: focusa_core::types::CltPayload::Interaction {
                role: "user".to_string(),
                content_ref: Some("hello".to_string()),
            },
            metadata: focusa_core::types::CltMetadata::default(),
        });
        state.clt.head_id = Some("n1".to_string());
        let first = traverse_response(
            &state,
            TraverseRequest {
                surface: "lineage".to_string(),
                selector: Some("window".to_string()),
                limit: Some(1),
                ..TraverseRequest::default()
            },
            false,
        );
        let item_tag = first
            .get("tags")
            .and_then(Value::as_array)
            .and_then(|tags| {
                tags.iter()
                    .find(|tag| tag.get("tag_mode").and_then(Value::as_str) == Some("item"))
            })
            .and_then(|tag| tag.get("tag"))
            .and_then(Value::as_str)
            .expect("item tag")
            .to_string();
        state.clt.nodes.push(focusa_core::types::CltNode {
            node_id: "n2".to_string(),
            parent_id: Some("n1".to_string()),
            node_type: CltNodeType::Interaction,
            created_at: chrono::Utc::now(),
            session_id: None,
            payload: focusa_core::types::CltPayload::Interaction {
                role: "assistant".to_string(),
                content_ref: Some("unrelated".to_string()),
            },
            metadata: focusa_core::types::CltMetadata::default(),
        });
        let verified = traverse_response(
            &state,
            TraverseRequest {
                surface: "lineage".to_string(),
                selector: Some("window".to_string()),
                limit: Some(1),
                tags: vec![json!(item_tag)],
                ..TraverseRequest::default()
            },
            true,
        );
        assert_eq!(
            verified
                .get("items")
                .and_then(Value::as_array)
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            verified
                .get("verified_tags")
                .and_then(Value::as_array)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            verified
                .get("stale_tags")
                .and_then(Value::as_array)
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn major_surface_adapters_return_bounded_items() {
        let state = FocusaState::new();
        for surface in [
            "trajectory",
            "lineage",
            "ontology",
            "focus_stack",
            "workpoints",
            "evidence",
            "metacognition",
            "predictions",
            "telemetry",
            "snapshots",
            "reflex_primitives",
            "tool_registry",
        ] {
            let res = traverse_response(
                &state,
                TraverseRequest {
                    surface: surface.to_string(),
                    selector: Some("window".to_string()),
                    limit: Some(5),
                    ..TraverseRequest::default()
                },
                false,
            );
            assert_eq!(
                res.get("status").and_then(Value::as_str),
                Some("completed"),
                "surface={surface}"
            );
            assert!(
                res.get("traversal").and_then(Value::as_object).is_some(),
                "surface={surface}"
            );
        }
    }

    #[test]
    fn reflex_primitive_surface_returns_registry_backed_family_items() {
        let state = FocusaState::new();
        let res = traverse_response(
            &state,
            TraverseRequest {
                surface: "reflex_primitives".to_string(),
                selector: Some("family".to_string()),
                anchor: Some("recovery".to_string()),
                fields: vec![
                    "primitive_id".to_string(),
                    "family".to_string(),
                    "reflex_action".to_string(),
                ],
                limit: Some(8),
                ..TraverseRequest::default()
            },
            false,
        );
        assert_eq!(res.get("status").and_then(Value::as_str), Some("completed"));
        let items = res.get("items").and_then(Value::as_array).unwrap();
        assert!(items.iter().any(|item| {
            item.get("data")
                .and_then(|payload| payload.get("primitive_id"))
                .and_then(Value::as_str)
                == Some("route_noncanonical_result")
        }));
        assert!(items.iter().all(|item| {
            item.get("data")
                .and_then(|payload| payload.get("family"))
                .and_then(Value::as_str)
                == Some("recovery")
        }));
    }

    #[test]
    fn tag_verify_endpoint_adopts_selector_from_requested_tag() {
        let mut state = FocusaState::new();
        state.clt.nodes.push(focusa_core::types::CltNode {
            node_id: "n1".to_string(),
            parent_id: None,
            node_type: CltNodeType::Interaction,
            created_at: chrono::Utc::now(),
            session_id: None,
            payload: focusa_core::types::CltPayload::Interaction {
                role: "user".to_string(),
                content_ref: Some("hello".to_string()),
            },
            metadata: focusa_core::types::CltMetadata::default(),
        });
        state.clt.head_id = Some("n1".to_string());
        let first = traverse_response(
            &state,
            TraverseRequest {
                surface: "lineage".to_string(),
                selector: Some("window".to_string()),
                limit: Some(1),
                ..TraverseRequest::default()
            },
            false,
        );
        let item_tag = first
            .get("tags")
            .and_then(Value::as_array)
            .and_then(|tags| {
                tags.iter()
                    .find(|tag| tag.get("tag_mode").and_then(Value::as_str) == Some("item"))
            })
            .and_then(|tag| tag.get("tag"))
            .and_then(Value::as_str)
            .expect("item tag")
            .to_string();
        let mut req = TraverseRequest {
            surface: "lineage".to_string(),
            selector: Some("tags_verify".to_string()),
            limit: Some(1),
            tags: vec![json!({"tag": item_tag})],
            ..TraverseRequest::default()
        };
        adopt_verify_selector_from_requested_tags(&mut req);
        assert_eq!(selector(&req), "window");
        let verified = traverse_response(&state, req, true);
        assert_eq!(
            verified
                .get("verified_tags")
                .and_then(Value::as_array)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            verified
                .get("stale_tags")
                .and_then(Value::as_array)
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn tag_verify_reports_stale_tags() {
        let state = FocusaState::new();
        let res = traverse_response(
            &state,
            TraverseRequest {
                surface: "lineage".to_string(),
                selector: Some("window".to_string()),
                tags: vec![json!(
                    "focusa://lineage/window/item/missing/deadbeefdeadbeefdeadbeef"
                )],
                ..TraverseRequest::default()
            },
            true,
        );
        assert_eq!(
            res.get("verified_tags")
                .and_then(Value::as_array)
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            res.get("stale_tags")
                .and_then(Value::as_array)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn lineage_window_response_has_traversal_metadata_and_tags() {
        let mut state = FocusaState::new();
        state.clt.nodes.push(focusa_core::types::CltNode {
            node_id: "n1".to_string(),
            parent_id: None,
            node_type: CltNodeType::Interaction,
            created_at: chrono::Utc::now(),
            session_id: None,
            payload: focusa_core::types::CltPayload::Interaction {
                role: "user".to_string(),
                content_ref: Some("hello".to_string()),
            },
            metadata: focusa_core::types::CltMetadata::default(),
        });
        state.clt.head_id = Some("n1".to_string());
        let res = traverse_response(
            &state,
            TraverseRequest {
                surface: "lineage".to_string(),
                selector: Some("window".to_string()),
                limit: Some(1),
                ..TraverseRequest::default()
            },
            false,
        );
        assert_eq!(res.get("status").and_then(Value::as_str), Some("completed"));
        assert_eq!(
            res.pointer("/traversal/returned").and_then(Value::as_u64),
            Some(1)
        );
        assert!(res.get("tags").and_then(Value::as_array).unwrap().len() >= 4);
        assert_eq!(
            res.pointer("/tag_scheme/version").and_then(Value::as_str),
            Some("focusa-traverse-tag-v1")
        );
    }
}
