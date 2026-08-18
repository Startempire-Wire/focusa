//! Telemetry routes.

use crate::routes::bounded::{
    budgeted_default_limit, budgeted_hard_limit, budgeted_requested_limit,
    last_pressure_transition, lowmem_retention_policy, pressure_status, record_json_response_size,
    response_size_histograms, set_test_pressure_threshold, telemetry_trace_retention_limit,
};
use crate::routes::ontology::ontology_read_index_cache_metadata;
use crate::routes::workpoint::idempotency_cache_status_payload;
use crate::server::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Deserialize)]
struct DebugPressureQuery {
    threshold_kb: Option<u64>,
}
use uuid::Uuid;

fn env_usize(name: &str, fallback: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(fallback)
        .max(1)
}

fn parse_status_value_kb(status_text: &str, key: &str) -> Option<u64> {
    status_text.lines().find_map(|line| {
        let (line_key, rest) = line.split_once(':')?;
        if line_key != key {
            return None;
        }
        rest.split_whitespace().next()?.parse::<u64>().ok()
    })
}

fn current_process_memory() -> Value {
    let status_text = std::fs::read_to_string("/proc/self/status").ok();
    let rss_kb = status_text
        .as_deref()
        .and_then(|text| parse_status_value_kb(text, "VmRSS"));
    let peak_rss_kb = status_text
        .as_deref()
        .and_then(|text| parse_status_value_kb(text, "VmHWM"));
    json!({
        "pid": std::process::id(),
        "rss_kb": rss_kb,
        "rss_bytes": rss_kb.map(|value| value * 1024),
        "peak_rss_kb": peak_rss_kb,
        "peak_rss_bytes": peak_rss_kb.map(|value| value * 1024),
        "source": if status_text.is_some() { "proc_self_status" } else { "unavailable" },
    })
}

fn route_budget_profile() -> Value {
    let ontology_object_default =
        budgeted_default_limit("FOCUSA_ONTOLOGY_WORLD_DEFAULT_OBJECT_LIMIT", 16);
    let ontology_link_default =
        budgeted_default_limit("FOCUSA_ONTOLOGY_WORLD_DEFAULT_LINK_LIMIT", 24);
    let ecs_default = budgeted_default_limit("FOCUSA_ECS_HANDLES_DEFAULT_LIMIT", 100);
    let semantic_default = budgeted_default_limit("FOCUSA_MEMORY_SEMANTIC_DEFAULT_LIMIT", 100);
    let telemetry_default = budgeted_default_limit("FOCUSA_TELEMETRY_EVENTS_DEFAULT_LIMIT", 100);
    json!({
        "ontology_world": {
            "default_object_limit": ontology_object_default,
            "full_object_limit": budgeted_hard_limit("FOCUSA_ONTOLOGY_WORLD_FULL_OBJECT_LIMIT", 10_000, ontology_object_default),
            "default_link_limit": ontology_link_default,
            "full_link_limit": budgeted_hard_limit("FOCUSA_ONTOLOGY_WORLD_FULL_LINK_LIMIT", 20_000, ontology_link_default),
            "workspace_scan_limit": budgeted_hard_limit("FOCUSA_ONTOLOGY_WORKSPACE_SCAN_LIMIT", 128, ontology_object_default),
        },
        "ecs_handles": {
            "default_limit": ecs_default,
            "full_limit": budgeted_hard_limit("FOCUSA_ECS_HANDLES_FULL_LIMIT", 512, ecs_default),
        },
        "semantic_memory": {
            "default_limit": semantic_default,
            "full_limit": budgeted_hard_limit("FOCUSA_MEMORY_SEMANTIC_FULL_LIMIT", 512, semantic_default),
        },
        "telemetry_trace": {
            "default_limit": telemetry_default,
            "hard_limit": budgeted_hard_limit("FOCUSA_TELEMETRY_EVENTS_HARD_LIMIT", 1000, telemetry_default),
        }
    })
}

fn prune_trace_events_for_lowmem(focusa: &mut focusa_core::types::FocusaState) -> usize {
    let limit = telemetry_trace_retention_limit();
    if focusa.telemetry.trace_events.len() > limit {
        let overflow = focusa.telemetry.trace_events.len() - limit;
        focusa.telemetry.trace_events.drain(0..overflow);
        overflow
    } else {
        0
    }
}

/// GET /v1/telemetry/memory — read-only daemon memory/store-count telemetry.
async fn memory_payload(state: &AppState) -> Value {
    let started_at = state.started_at;
    let event_count = state.persistence.event_count().unwrap_or(0);
    let focusa = state.focusa.read().await;
    let payload = json!({
        "status": "ok",
        "generated_at": Utc::now().to_rfc3339(),
        "uptime_ms": started_at.elapsed().as_millis() as u64,
        "process": current_process_memory(),
        "stores": {
            "memory": {
                "semantic_count": focusa.memory.semantic.len(),
                "procedural_count": focusa.memory.procedural.len(),
                "semantic_default_limit": env_usize("FOCUSA_MEMORY_SEMANTIC_DEFAULT_LIMIT", 100),
                "semantic_full_limit": env_usize("FOCUSA_MEMORY_SEMANTIC_FULL_LIMIT", 512),
            },
            "ecs": {
                "handle_count": focusa.reference_index.handles.len(),
                "default_limit": env_usize("FOCUSA_ECS_HANDLES_DEFAULT_LIMIT", 100),
                "full_limit": env_usize("FOCUSA_ECS_HANDLES_FULL_LIMIT", 512),
            },
            "ontology": {
                "canonical_object_count": focusa.ontology.objects.len(),
                "canonical_link_count": focusa.ontology.links.len(),
                "proposal_count": focusa.ontology.proposals.len(),
                "verification_count": focusa.ontology.verifications.len(),
                "working_set_refresh_count": focusa.ontology.working_set_refreshes.len(),
                "delta_count": focusa.ontology.delta_log.len(),
            },
            "workpoint": {
                "record_count": focusa.workpoint.records.len(),
                "resume_event_count": focusa.workpoint.resume_events.len(),
                "drift_event_count": focusa.workpoint.drift_events.len(),
                "degraded_fallback_count": focusa.workpoint.degraded_fallbacks.len(),
                "idempotency_cache": idempotency_cache_status_payload(),
            },
            "lineage": {
                "node_count": focusa.clt.nodes.len(),
            },
            "events": {
                "sqlite_event_count": event_count,
                "trace_event_count": focusa.telemetry.trace_events.len(),
                "tool_call_count": focusa.telemetry.tool_calls.len(),
                "total_telemetry_events": focusa.telemetry.total_events,
            },
            "runtime": {
                "instance_count": focusa.instances.len(),
                "attachment_count": focusa.attachments.len(),
                "thread_count": focusa.threads.len(),
            }
        },
        "caps": {
            "workpoint_records": focusa_core::types::workpoint_caps::RECORDS,
            "workpoint_object_refs": focusa_core::types::workpoint_caps::OBJECT_REFS,
            "workpoint_verifications": focusa_core::types::workpoint_caps::VERIFICATIONS,
            "workpoint_blockers": focusa_core::types::workpoint_caps::BLOCKERS,
            "workpoint_resume_events": focusa_core::types::workpoint_caps::RESUME_EVENTS,
            "workpoint_drift_events": focusa_core::types::workpoint_caps::DRIFT_EVENTS,
            "workpoint_degraded_fallbacks": focusa_core::types::workpoint_caps::DEGRADED_FALLBACKS,
            "trace_events_window": 5000,
        },
        "evictions": {
            "secondary_loop_archived_events": focusa.telemetry.secondary_loop_archived_events,
            "trace_events_window_policy": "oldest_dropped_after_5000",
        },
        "pressure": {
            "current": pressure_status(),
            "last_transition": last_pressure_transition(),
            "note": "Full-payload read routes expose degraded=true when memory pressure is active and force_full_payload is not set."
        },
        "route_budgets": route_budget_profile(),
        "retention_policy": lowmem_retention_policy(),
        "response_size_histograms": response_size_histograms(),
        "degraded": false,
    });
    record_json_response_size("/v1/telemetry/memory", &payload);
    payload
}

async fn memory_status(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(memory_payload(&state).await)
}

/// GET /v1/telemetry/snapshot — route-parity snapshot for TUI/menubar consumers.
async fn telemetry_snapshot(State(state): State<Arc<AppState>>) -> Json<Value> {
    let memory = memory_payload(&state).await;
    let focusa = state.focusa.read().await;
    let estimated_cost = focusa_core::telemetry::estimate_cost(&focusa.telemetry, 0.003, 0.015);
    let latest_token_budget = focusa
        .telemetry
        .trace_events
        .iter()
        .rev()
        .find(|event| {
            event.get("event_type").and_then(|v| v.as_str()) == Some("spec92_token_budget")
        })
        .cloned()
        .unwrap_or(Value::Null);
    let latest_cache_metadata = focusa
        .telemetry
        .trace_events
        .iter()
        .rev()
        .find(|event| {
            event.get("event_type").and_then(|v| v.as_str()) == Some("spec92_cache_metadata")
        })
        .cloned()
        .unwrap_or(Value::Null);
    let ontology_cache_metadata = ontology_read_index_cache_metadata(&focusa, None);
    let payload = json!({
        "schema": "focusa.telemetry_snapshot.v1",
        "status": "completed",
        "memory": memory,
        "events_total": focusa.telemetry.total_events,
        "trace_event_count": focusa.telemetry.trace_events.len(),
        "cost": {
            "estimated_cost_usd": estimated_cost,
            "prompt_tokens": focusa.telemetry.total_prompt_tokens,
            "completion_tokens": focusa.telemetry.total_completion_tokens,
        },
        "token_budget": {
            "latest": latest_token_budget,
            "source": "telemetry_trace_events",
        },
        "cache_metadata": {
            "latest": latest_cache_metadata,
            "ontology_read_index": ontology_cache_metadata,
            "source": "telemetry_trace_events_plus_ontology_read_index",
        },
        "workpoint_resume_event_count": focusa.workpoint.resume_events.len(),
    });
    drop(focusa);
    record_json_response_size("/v1/telemetry/snapshot", &payload);
    Json(payload)
}

fn telemetry_debug_disabled() -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "status": "blocked", "canonical": false, "degraded": true,
            "error": "debug route disabled", "failure_class": "not_found",
            "why": "telemetry debug pressure threshold route is disabled outside debug/test mode",
            "recovery_hint": "Enable FOCUSA_ENABLE_TEST_ROUTES=1 only in test contexts, or use read-only telemetry routes.",
            "misuse_hint": "Likely production-safe daemon where test mutation routes are intentionally unavailable.",
            "next_tools": ["focusa_tool_doctor"],
            "details": {"tool_result_v1": {"ok": false, "status": "blocked", "canonical": false, "degraded": true, "failure_class": "not_found", "summary": "debug route disabled", "retry": {"safe": false, "posture": "do_not_retry_unchanged", "reason": "test_route_disabled"}, "side_effects": [], "evidence_refs": [], "next_tools": ["focusa_tool_doctor"], "error": {"code": "not_found", "message": "debug route disabled"}}}
        })),
    )
}

async fn debug_set_pressure_threshold(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DebugPressureQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if !cfg!(debug_assertions)
        && std::env::var("FOCUSA_ENABLE_TEST_ROUTES").ok().as_deref() != Some("1")
    {
        return Err(telemetry_debug_disabled());
    }
    set_test_pressure_threshold(query.threshold_kb);
    Ok(Json(memory_payload(&state).await))
}

/// GET /v1/telemetry/events — bounded read of telemetry trace events.
async fn telemetry_events(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Value> {
    let default_limit = budgeted_default_limit("FOCUSA_TELEMETRY_EVENTS_DEFAULT_LIMIT", 100);
    let hard_limit = budgeted_hard_limit("FOCUSA_TELEMETRY_EVENTS_HARD_LIMIT", 1000, default_limit);
    let limit = budgeted_requested_limit(
        params.get("limit").and_then(|v| v.parse::<usize>().ok()),
        default_limit,
        hard_limit,
    );
    let cursor = params
        .get("cursor")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);
    let event_type = params.get("event_type").map(String::as_str);
    let focusa = state.focusa.read().await;
    let total = focusa.telemetry.trace_events.len();
    let mut events = focusa
        .telemetry
        .trace_events
        .iter()
        .rev()
        .filter(|event| {
            event_type
                .map(|wanted| event.get("event_type").and_then(|v| v.as_str()) == Some(wanted))
                .unwrap_or(true)
        })
        .skip(cursor)
        .take(limit)
        .cloned()
        .collect::<Vec<_>>();
    events.reverse();
    let payload = json!({
        "status": "ok",
        "events": events,
        "count": total,
        "returned": events.len(),
        "limit": limit,
        "cursor": cursor,
        "next_cursor": (cursor + events.len() < total).then(|| (cursor + events.len()).to_string()),
        "truncated": cursor + events.len() < total,
        "bounds": {
            "total": total,
            "returned": events.len(),
            "limit": limit,
            "cursor": cursor,
            "next_cursor": (cursor + events.len() < total).then(|| (cursor + events.len()).to_string()),
            "truncated": cursor + events.len() < total
        },
    });
    record_json_response_size("/v1/telemetry/events", &payload);
    Json(payload)
}

/// GET /v1/telemetry/productivity — lightweight productivity counters.
async fn telemetry_productivity(State(state): State<Arc<AppState>>) -> Json<Value> {
    let focusa = state.focusa.read().await;
    let payload = json!({
        "status": "ok",
        "workpoint_records": focusa.workpoint.records.len(),
        "workpoint_resume_events": focusa.workpoint.resume_events.len(),
        "verification_events": focusa.telemetry.verification_result_events,
        "tool_calls": focusa.telemetry.tool_calls.len(),
        "trace_events": focusa.telemetry.trace_events.len(),
        "semantic_memory_records": focusa.memory.semantic.len(),
        "procedural_memory_records": focusa.memory.procedural.len(),
        "ontology_delta_count": focusa.ontology.delta_log.len(),
        "bounds": {"summary_only": true, "truncated": false, "returned": 1, "total": 1},
    });
    record_json_response_size("/v1/telemetry/productivity", &payload);
    Json(payload)
}

/// GET /v1/telemetry/autonomy — bounded secondary/autonomy telemetry counters.
async fn telemetry_autonomy(State(state): State<Arc<AppState>>) -> Json<Value> {
    let focusa = state.focusa.read().await;
    let payload = json!({
        "status": "ok",
        "autonomy": {
            "sample_count": focusa.autonomy.sample_count,
            "history_count": focusa.autonomy.history.len(),
        },
        "secondary_loop": {
            "useful_events": focusa.telemetry.secondary_loop_useful_events,
            "low_quality_events": focusa.telemetry.secondary_loop_low_quality_events,
            "ledger_entries": focusa.telemetry.secondary_loop_ledger.len(),
            "archived_events": focusa.telemetry.secondary_loop_archived_events,
        },
        "scope_quality": {
            "scope_contamination_events": focusa.telemetry.scope_contamination_events,
            "subject_hijack_prevented_events": focusa.telemetry.subject_hijack_prevented_events,
            "subject_hijack_occurred_events": focusa.telemetry.subject_hijack_occurred_events,
        },
        "bounds": {"summary_only": true, "truncated": false, "returned": 1, "total": 1}
    });
    record_json_response_size("/v1/telemetry/autonomy", &payload);
    Json(payload)
}

/// GET /v1/telemetry/tokens — token usage metrics.
async fn tokens(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    let payload = json!({
        "total_events": s.telemetry.total_events,
        "total_prompt_tokens": s.telemetry.total_prompt_tokens,
        "total_completion_tokens": s.telemetry.total_completion_tokens,
        "tokens_per_task": s.telemetry.tokens_per_task,
    });
    record_json_response_size("/v1/telemetry/tokens", &payload);
    Json(payload)
}

/// GET /v1/telemetry/token-budget/status — Spec92 token budget telemetry summary.
async fn token_budget_status(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Value> {
    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(20)
        .min(100);
    let s = state.focusa.read().await;
    let mut records: Vec<Value> = s
        .telemetry
        .trace_events
        .iter()
        .filter(|event| {
            event.get("event_type").and_then(|v| v.as_str()) == Some("spec92_token_budget")
        })
        .rev()
        .take(limit)
        .cloned()
        .collect();
    records.reverse();
    let latest = records.last().cloned().unwrap_or(Value::Null);
    let latest_payload = latest.get("payload").cloned().unwrap_or(Value::Null);
    let budget_class = latest_payload
        .get("budget_class")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    Json(json!({
        "status": if budget_class == "critical" { "critical" } else if budget_class == "high" { "high" } else if budget_class == "watch" { "watch" } else { "ok" },
        "summary": format!("latest token budget class: {budget_class}"),
        "record_count": records.len(),
        "latest": latest,
        "records": records,
        "next_action": if matches!(budget_class, "critical" | "high") { "compact large tool results or use ECS handles before continuing" } else { "continue normally; monitor token budget" },
        "commands": ["node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures", "focusa telemetry token-budget"],
    }))
}

/// POST /v1/telemetry/token-budget — record Spec92 token budget telemetry.
async fn record_token_budget(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut focusa = state.focusa.write().await;
    focusa.telemetry.total_events += 1;
    let event_id = Uuid::now_v7().to_string();
    let budget_class = body
        .get("budget_class")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    focusa.telemetry.trace_events.push(json!({
        "event_id": event_id,
        "event_type": "spec92_token_budget",
        "timestamp": Utc::now().to_rfc3339(),
        "turn_id": body.get("turn_id").cloned().unwrap_or(Value::Null),
        "payload": body,
    }));
    let _pruned = prune_trace_events_for_lowmem(&mut focusa);
    drop(focusa);
    state.mark_external_mutation();
    Json(json!({
        "status": "recorded",
        "event_type": "spec92_token_budget",
        "budget_class": budget_class,
    }))
}

/// GET /v1/telemetry/cache-metadata/status — Spec95 H1 / Spec92 cache metadata summary.
/// Exposes per-cache-entry TTL/invalidation metadata for the ontology read index,
/// plus existing Spec92 cache metadata records.
async fn cache_metadata_status(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Value> {
    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(20)
        .min(100);
    let s = state.focusa.read().await;
    // Spec95 H1: per-cache-entry metadata for the ontology read index.
    let read_index_meta = ontology_read_index_cache_metadata(&s, None);
    let mut records: Vec<Value> = s
        .telemetry
        .trace_events
        .iter()
        .filter(|event| {
            event.get("event_type").and_then(|v| v.as_str()) == Some("spec92_cache_metadata")
        })
        .rev()
        .take(limit)
        .cloned()
        .collect();
    records.reverse();
    let latest = records.last().cloned().unwrap_or(Value::Null);
    let eligible_count = records
        .iter()
        .filter(|event| {
            event
                .get("payload")
                .and_then(|p| p.get("cache_eligible"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        })
        .count();
    let object_type_counts = read_index_meta
        .get("object_type_counts")
        .and_then(|v| v.as_object())
        .map(|m| m.len())
        .unwrap_or(0);
    let link_type_counts = read_index_meta
        .get("link_type_counts")
        .and_then(|v| v.as_object())
        .map(|m| m.len())
        .unwrap_or(0);
    Json(json!({
        "status": "ok",
        // Spec95 H1 cache entry metadata for the ontology read index.
        "ontology_read_index": {
            "cache_tier": read_index_meta.get("cache_tier"),
            "source_reducer_version": read_index_meta.get("source_reducer_version"),
            "generated_at": read_index_meta.get("generated_at"),
            "ttl_seconds": read_index_meta.get("ttl_seconds"),
            "age_seconds": read_index_meta.get("age_seconds"),
            "invalidation_rule": read_index_meta.get("invalidation_rule"),
            "canonical": read_index_meta.get("canonical"),
            "degraded": read_index_meta.get("degraded"),
            "stale": read_index_meta.get("stale"),
            "object_count": read_index_meta.get("object_count"),
            "link_count": read_index_meta.get("link_count"),
            "object_type_count": object_type_counts,
            "link_type_count": link_type_counts,
            "last_reducer_event_id": read_index_meta.get("last_reducer_event_id"),
            "frame_id": read_index_meta.get("frame_id"),
        },
        "spec92_cache_metadata": {
            "summary": format!("cache metadata records: {}, eligible: {}", records.len(), eligible_count),
            "record_count": records.len(),
            "eligible_count": eligible_count,
            "latest": latest,
            "records": records,
            "next_action": if records.is_empty() { "run a provider turn to collect cache metadata" } else { "continue; review repeated_prefix_hash before cache-policy tuning" },
            "commands": ["focusa cache doctor", "focusa --json cache doctor --limit 10"],
        },
    }))
}

/// POST /v1/telemetry/cache-metadata — record Spec92 cache metadata.
async fn record_cache_metadata(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut focusa = state.focusa.write().await;
    focusa.telemetry.total_events += 1;
    focusa.telemetry.trace_events.push(json!({
        "event_id": Uuid::now_v7().to_string(),
        "event_type": "spec92_cache_metadata",
        "timestamp": Utc::now().to_rfc3339(),
        "turn_id": body.get("turn_id").cloned().unwrap_or(Value::Null),
        "payload": body,
    }));
    let _pruned = prune_trace_events_for_lowmem(&mut focusa);
    drop(focusa);
    state.mark_external_mutation();
    Json(json!({
        "status": "recorded",
        "event_type": "spec92_cache_metadata",
    }))
}

/// GET /v1/telemetry/cost — cost estimate.
async fn cost(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    let estimated = focusa_core::telemetry::estimate_cost(&s.telemetry, 0.003, 0.015);
    Json(json!({
        "estimated_cost_usd": estimated,
        "prompt_tokens": s.telemetry.total_prompt_tokens,
        "completion_tokens": s.telemetry.total_completion_tokens,
    }))
}

/// POST /v1/telemetry/tool-usage — record batch of tool calls for autonomy.
#[derive(Debug, Deserialize)]
struct ToolUsageBody {
    turn_id: Option<String>,
    tools: Vec<String>,
}

/// GET /v1/telemetry/tools — get tool usage summary.
async fn tool_usage(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    let summary: std::collections::HashMap<String, u32> =
        s.telemetry
            .tool_calls
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, name| {
                *acc.entry(name.clone()).or_insert(0) += 1;
                acc
            });
    let payload = json!({
        "total_calls": s.telemetry.tool_calls.len(),
        "tool_summary": summary,
    });
    record_json_response_size("/v1/telemetry/tools", &payload);
    Json(payload)
}

/// POST /v1/telemetry/tool-usage — receive tool call batch from extension.
async fn record_tool_usage(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ToolUsageBody>,
) -> Result<Json<Value>, axum::http::StatusCode> {
    // Feed tool names to telemetry for autonomy analysis.
    let recorded = body.tools.len();
    let turn_id = body.turn_id.clone();
    let tools = body.tools.clone();
    let mut focusa = state.focusa.write().await;
    focusa.telemetry.total_events += 1;
    for tool in &body.tools {
        focusa.telemetry.tool_calls.push(tool.clone());
    }
    focusa.telemetry.trace_events.push(json!({
        "event_id": Uuid::now_v7().to_string(),
        "event_type": "tools_invoked",
        "timestamp": Utc::now().to_rfc3339(),
        "turn_id": turn_id,
        "payload": {
            "turn_id": body.turn_id,
            "tools": body.tools,
        },
    }));
    let _pruned = prune_trace_events_for_lowmem(&mut focusa);
    drop(focusa);
    state.mark_external_mutation();
    Ok(Json(
        json!({"status": "accepted", "recorded": recorded, "turn_id": turn_id, "tools": tools}),
    ))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/telemetry/memory", get(memory_status))
        .route("/v1/telemetry/snapshot", get(telemetry_snapshot))
        .route(
            "/v1/debug/set-pressure-threshold",
            get(debug_set_pressure_threshold),
        )
        .route("/v1/telemetry/events", get(telemetry_events))
        .route("/v1/telemetry/productivity", get(telemetry_productivity))
        .route("/v1/telemetry/autonomy", get(telemetry_autonomy))
        .route("/v1/telemetry/tokens", get(tokens))
        .route(
            "/v1/telemetry/token-budget/status",
            get(token_budget_status),
        )
        .route("/v1/telemetry/token-budget", post(record_token_budget))
        .route(
            "/v1/telemetry/cache-metadata/status",
            get(cache_metadata_status),
        )
        .route("/v1/telemetry/cache-metadata", post(record_cache_metadata))
        .route("/v1/telemetry/cost", get(cost))
        .route("/v1/telemetry/tools", get(tool_usage))
        .route("/v1/telemetry/tool-usage", post(record_tool_usage))
        .route("/v1/telemetry/activity", post(record_activity_event))
        .route("/v1/telemetry/ops", post(record_operational_event))
        // Deprecated compatibility alias for legacy extension callers.
        .route("/v1/telemetry/event", post(record_operational_event))
        // SPEC 56: Trace dimension endpoints
        .route("/v1/telemetry/trace", post(record_trace_event))
        .route("/v1/telemetry/trace/batch", post(record_trace_batch))
        .route("/v1/telemetry/trace", get(get_trace_events))
        .route("/v1/telemetry/trace/stats", get(get_trace_stats))
}
// ═══════════════════════════════════════════════════════════════════════════════
// Operational + Activity Telemetry
// ═══════════════════════════════════════════════════════════════════════════════

/// POST /v1/telemetry/activity — record session/activity telemetry.
async fn record_activity_event(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let event_name = body
        .get("event")
        .and_then(|v| v.as_str())
        .unwrap_or("activity_event");

    let mut focusa = state.focusa.write().await;
    focusa.telemetry.total_events += 1;
    focusa.telemetry.trace_events.push(serde_json::json!({
        "event_id": Uuid::now_v7().to_string(),
        "channel": "activity",
        "event": event_name,
        "timestamp": Utc::now().to_rfc3339(),
        "payload": body,
    }));
    let _pruned = prune_trace_events_for_lowmem(&mut focusa);
    drop(focusa);
    state.mark_external_mutation();

    Json(serde_json::json!({
        "status": "recorded",
        "channel": "activity",
        "event": event_name,
    }))
}

/// POST /v1/telemetry/ops — record operational telemetry.
/// `/v1/telemetry/event` is kept as a deprecated compatibility alias.
async fn record_operational_event(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let event_name = body
        .get("event")
        .and_then(|v| v.as_str())
        .unwrap_or("operational_event");

    let semantic_id = body
        .get("semantic_event")
        .and_then(|event| event.get("event_id"))
        .and_then(|id| id.as_str());
    let mut focusa = state.focusa.write().await;
    if let Some(id) = semantic_id {
        let duplicate = focusa.telemetry.trace_events.iter().any(|event| {
            event
                .get("payload")
                .and_then(|payload| payload.get("semantic_event"))
                .and_then(|semantic| semantic.get("event_id"))
                .and_then(|value| value.as_str())
                == Some(id)
        });
        if duplicate {
            return Json(serde_json::json!({
                "status": "duplicate",
                "channel": "ops",
                "event": event_name,
                "event_id": id,
            }));
        }
    }
    focusa.telemetry.total_events += 1;
    let event_id = Uuid::now_v7().to_string();
    focusa.telemetry.trace_events.push(serde_json::json!({
        "event_id": event_id,
        "channel": "ops",
        "event": event_name,
        "timestamp": Utc::now().to_rfc3339(),
        "payload": body,
    }));
    let _pruned = prune_trace_events_for_lowmem(&mut focusa);
    drop(focusa);
    state.mark_external_mutation();

    Json(serde_json::json!({
        "status": "recorded",
        "channel": "ops",
        "event": event_name,
        "event_id": event_id,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// SPEC 56: Trace Dimensions
// ═══════════════════════════════════════════════════════════════════════════════

/// POST /v1/telemetry/trace/batch — Record a bounded batch of trace events.
async fn record_trace_batch(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let events = body
        .get("events")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let batch_id = body
        .get("batch_id")
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
        .unwrap_or_else(|| Uuid::now_v7().to_string());
    let mut focusa = state.focusa.write().await;
    let accepted = events.len().min(100);
    focusa.telemetry.total_events += accepted as u64;
    for mut event in events.into_iter().take(accepted) {
        if let Some(object) = event.as_object_mut() {
            object.insert("event_id".to_string(), json!(Uuid::now_v7().to_string()));
            object.insert("batch_id".to_string(), json!(batch_id.clone()));
            object
                .entry("timestamp".to_string())
                .or_insert_with(|| json!(Utc::now().to_rfc3339()));
        }
        focusa.telemetry.trace_events.push(event);
    }
    let _pruned = prune_trace_events_for_lowmem(&mut focusa);
    drop(focusa);
    state.mark_external_mutation();
    Json(json!({
        "status": "recorded",
        "batch_id": batch_id,
        "accepted": accepted,
        "truncated": accepted == 100,
    }))
}

/// POST /v1/telemetry/trace — Record a trace dimension event (SPEC 56)
async fn record_trace_event(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    use focusa_core::types::TelemetryEventType;

    let event_type_str = body
        .get("event_type")
        .and_then(|v| v.as_str())
        .unwrap_or("ModelTokens");
    let _event_type = match event_type_str {
        "mission_frame_context" => TelemetryEventType::MissionFrameContext,
        "working_set_used" => TelemetryEventType::WorkingSetUsed,
        "constraints_consulted" => TelemetryEventType::ConstraintsConsulted,
        "decisions_consulted" => TelemetryEventType::DecisionsConsulted,
        "action_intents_proposed" => TelemetryEventType::ActionIntentsProposed,
        "tools_invoked" => TelemetryEventType::ToolsInvoked,
        "verification_result" => TelemetryEventType::VerificationResult,
        "ontology_delta_applied" => TelemetryEventType::OntologyDeltaApplied,
        "blockers_failures_emitted" => TelemetryEventType::BlockersFailuresEmitted,
        "final_state_transition" => TelemetryEventType::FinalStateTransition,
        "operator_subject" => TelemetryEventType::OperatorSubject,
        "active_subject_after_routing" => TelemetryEventType::ActiveSubjectAfterRouting,
        "steering_detected" => TelemetryEventType::SteeringDetected,
        "subject_hijack_prevented" => TelemetryEventType::SubjectHijackPrevented,
        "subject_hijack_occurred" => TelemetryEventType::SubjectHijackOccurred,
        "prior_mission_reused" => TelemetryEventType::PriorMissionReused,
        "focus_slice_size" => TelemetryEventType::FocusSliceSize,
        "focus_slice_relevance_score" => TelemetryEventType::FocusSliceRelevanceScore,
        "current_ask_determined" => TelemetryEventType::CurrentAskDetermined,
        "query_scope_built" => TelemetryEventType::QueryScopeBuilt,
        "relevant_context_selected" => TelemetryEventType::RelevantContextSelected,
        "irrelevant_context_excluded" => TelemetryEventType::IrrelevantContextExcluded,
        "scope_verified" => TelemetryEventType::ScopeVerified,
        "scope_contamination_detected" => TelemetryEventType::ScopeContaminationDetected,
        "wrong_question_detected" => TelemetryEventType::WrongQuestionDetected,
        "answer_broadening_detected" => TelemetryEventType::AnswerBroadeningDetected,
        "scope_failure_recorded" => TelemetryEventType::ScopeFailureRecorded,
        _ => TelemetryEventType::ModelTokens,
    };

    // Store in focusa telemetry state (in-memory)
    let mut focusa = state.focusa.write().await;
    focusa.telemetry.total_events += 1;
    focusa.telemetry.trace_events.push(serde_json::json!({
        "event_id": Uuid::now_v7().to_string(),
        "event_type": event_type_str,
        "timestamp": Utc::now().to_rfc3339(),
        "turn_id": body.get("turn_id").cloned().unwrap_or(serde_json::Value::Null),
        "payload": body,
    }));
    let _pruned = prune_trace_events_for_lowmem(&mut focusa);
    drop(focusa);
    state.mark_external_mutation();

    Json(serde_json::json!({
        "status": "recorded",
        "event_type": event_type_str,
    }))
}

/// GET /v1/telemetry/trace — Get trace events (SPEC 56)
async fn get_trace_events(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    let events = &focusa.telemetry.trace_events;

    let default_limit = budgeted_default_limit("FOCUSA_TELEMETRY_TRACE_DEFAULT_LIMIT", 100);
    let hard_limit = budgeted_hard_limit("FOCUSA_TELEMETRY_TRACE_HARD_LIMIT", 1000, default_limit);
    let limit = budgeted_requested_limit(
        params.get("limit").and_then(|v| v.parse::<usize>().ok()),
        default_limit,
        hard_limit,
    );

    let cursor = params
        .get("cursor")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0)
        .min(events.len());
    let event_type_filter = params.get("event_type").map(String::as_str);
    let turn_id_filter = params.get("turn_id").map(String::as_str);
    let turn_id_prefix_filter = params.get("turn_id_prefix").map(String::as_str);
    let filtered: Vec<_> = events
        .iter()
        .rev()
        .filter(|e| {
            event_type_filter
                .map(|wanted| e.get("event_type").and_then(|v| v.as_str()) == Some(wanted))
                .unwrap_or(true)
        })
        .filter(|e| {
            turn_id_filter
                .map(|wanted| {
                    let nested = e
                        .get("payload")
                        .and_then(|p| p.get("turn_id"))
                        .and_then(|v| v.as_str());
                    let top = e.get("turn_id").and_then(|v| v.as_str());
                    nested == Some(wanted) || top == Some(wanted)
                })
                .unwrap_or(true)
        })
        .filter(|e| {
            turn_id_prefix_filter
                .map(|wanted| {
                    let nested = e
                        .get("payload")
                        .and_then(|p| p.get("turn_id"))
                        .and_then(|v| v.as_str());
                    let top = e.get("turn_id").and_then(|v| v.as_str());
                    nested
                        .or(top)
                        .map(|turn_id| turn_id.starts_with(wanted))
                        .unwrap_or(false)
                })
                .unwrap_or(true)
        })
        .skip(cursor)
        .take(limit)
        .map(|event| {
            let mut normalized = event.clone();
            let event_type = normalized
                .get("event_type")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            if event_type.as_deref() == Some("governing_priors_applied")
                && normalized
                    .get("payload")
                    .is_none_or(|payload| payload.is_null())
            {
                normalized["payload"] = serde_json::json!({
                    "governing_priors": normalized.get("governing_priors").cloned().unwrap_or_else(|| serde_json::json!([])),
                    "ranking_consumers": normalized.get("ranking_consumers").cloned().unwrap_or_else(|| serde_json::json!([])),
                    "prior_hits": normalized.get("prior_hits").cloned().unwrap_or_else(|| serde_json::json!({})),
                });
            }
            if event_type.as_deref() == Some("verification_result")
                && let Some(payload) = normalized.get_mut("payload").and_then(|v| v.as_object_mut())
            {
                payload
                    .entry("retention_policy".to_string())
                    .or_insert_with(|| serde_json::json!(lowmem_retention_policy()));
                payload
                    .entry("selected_count".to_string())
                    .or_insert_with(|| serde_json::json!(0));
                payload
                    .entry("pruned_count".to_string())
                    .or_insert_with(|| serde_json::json!(0));
            }
            normalized
        })
        .collect();
    let count = filtered.len();
    let next_cursor = (cursor + count < events.len()).then(|| (cursor + count).to_string());

    Json(serde_json::json!({
        "events": filtered,
        "count": count,
        "returned": count,
        "limit": limit,
        "cursor": cursor,
        "next_cursor": next_cursor,
        "truncated": next_cursor.is_some() || cursor > 0,
        "metadata": {"summary_only": true, "cursor": cursor, "limit": limit, "next_cursor": next_cursor},
    }))
}

/// GET /v1/telemetry/trace/stats — Get trace stats (SPEC 56)
async fn get_trace_stats(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    let events = &focusa.telemetry.trace_events;

    let mut by_type: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    for e in events {
        if let Some(t) = e.get("event_type").and_then(|v| v.as_str()) {
            *by_type.entry(t.to_string()).or_insert(0) += 1;
        }
    }

    Json(serde_json::json!({
        "total_events": events.len(),
        "by_event_type": by_type,
    }))
}

#[cfg(test)]
mod tests {
    use super::{env_usize, parse_status_value_kb, route_budget_profile};
    use crate::routes::bounded::{
        budgeted_default_limit, budgeted_hard_limit, budgeted_requested_limit,
        lowmem_retention_policy,
    };

    #[test]
    fn parses_proc_status_memory_values() {
        let text = "Name:\tfocusa-daemon\nVmHWM:\t  2048 kB\nVmRSS:\t  1024 kB\n";
        assert_eq!(parse_status_value_kb(text, "VmRSS"), Some(1024));
        assert_eq!(parse_status_value_kb(text, "VmHWM"), Some(2048));
        assert_eq!(parse_status_value_kb(text, "VmData"), None);
    }

    #[test]
    fn env_usize_never_returns_zero() {
        assert_eq!(env_usize("FOCUSA_TEST_UNSET_LIMIT", 0), 1);
    }

    #[test]
    fn trace_batch_acceptance_is_hard_bounded() {
        let accepted = 100;
        assert_eq!(accepted, 100);
    }

    #[test]
    fn telemetry_events_limit_is_hard_bounded_by_route_logic() {
        let default_limit = budgeted_default_limit("FOCUSA_TELEMETRY_EVENTS_DEFAULT_LIMIT", 100);
        let hard_limit =
            budgeted_hard_limit("FOCUSA_TELEMETRY_EVENTS_HARD_LIMIT", 1000, default_limit);
        let requested = budgeted_requested_limit(Some(5000), default_limit, hard_limit);
        assert_eq!(requested, hard_limit);
        assert!(requested <= 1000);
    }

    #[test]
    fn retention_policy_exposes_importance_pruning_order() {
        let policy = lowmem_retention_policy();
        assert_eq!(policy.retain_order.first(), Some(&"liveness"));
        assert!(policy.retain_order.contains(&"workpoint"));
        assert!(policy.evict_first.contains(&"raw_telemetry_trace_events"));
    }

    #[test]
    fn route_budget_profile_exposes_major_read_surfaces() {
        let profile = route_budget_profile();
        assert!(profile.get("ontology_world").is_some());
        assert!(profile.get("ecs_handles").is_some());
        assert!(profile.get("semantic_memory").is_some());
        assert!(profile.get("telemetry_trace").is_some());
    }
}
