//! Memory routes.
//!
//! GET  /v1/memory/semantic             — list semantic memory
//! POST /v1/memory/semantic/upsert      — upsert a key=value
//! GET  /v1/memory/procedural           — list procedural rules
//! POST /v1/memory/procedural/reinforce — reinforce a rule

use crate::routes::bounded::{
    BoundedReadOptions, bounded_metadata, env_limit, full_payload_blocked_by_pressure,
    pressure_status, record_json_response_size,
};
use crate::server::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use focusa_core::types::{Action, MemorySource, SemanticRecord};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

const DEFAULT_SEMANTIC_LIMIT: usize = 100;
const MAX_SEMANTIC_LIMIT: usize = 512;

type MemoryResult = Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)>;

fn memory_failure(
    http_status: StatusCode,
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
) -> (StatusCode, Json<serde_json::Value>) {
    let error = error.into();
    let why = why.into();
    (
        http_status,
        Json(json!({
            "status": "blocked", "canonical": false, "degraded": true,
            "error": error, "failure_class": failure_class, "why": why,
            "recovery_hint": recovery_hint, "misuse_hint": misuse_hint,
            "next_tools": ["focusa_tool_doctor", "focusa_metacog_retrieve"],
            "details": {"tool_result_v1": {"ok": false, "status": "blocked", "canonical": false, "degraded": true, "failure_class": failure_class, "summary": why, "retry": {"safe": true, "posture": "safe_retry", "reason": failure_class}, "recovery_hint": recovery_hint, "misuse_hint": misuse_hint, "side_effects": [], "evidence_refs": [], "next_tools": ["focusa_tool_doctor", "focusa_metacog_retrieve"], "error": {"code": failure_class, "message": error}}}
        })),
    )
}

fn memory_dispatch_failed(error: impl std::fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    memory_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("memory dispatch failed: {error}"),
        "daemon_unavailable",
        "Memory action could not be dispatched to daemon command channel.",
        "Check daemon health and retry after command channel recovery is clear.",
        "Likely daemon command channel closed, runtime shutdown, or writer/transport ownership issue.",
    )
}

fn memory_dispatch_timeout() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::ACCEPTED,
        Json(json!({
            "status": "pending",
            "canonical": false,
            "degraded": true,
            "failure_class": "resource_exhausted",
            "retry_posture": "safe_retry",
            "retry": {"safe": true, "posture": "safe_retry", "reason": "daemon command channel is saturated"},
            "side_effects": [],
            "next_tools": ["focusa_tool_doctor", "focusa_resource_mode", "focusa_metacog_retrieve"],
            "next_step_hint": "retry after daemon command backlog drains; memory action was not enqueued"
        })),
    )
}

async fn dispatch_memory_action(state: &Arc<AppState>, action: Action) -> MemoryResult {
    match tokio::time::timeout(Duration::from_millis(1500), state.command_tx.send(action)).await {
        Ok(Ok(())) => Ok(Json(json!({"status": "accepted"}))),
        Ok(Err(error)) => Err(memory_dispatch_failed(error)),
        Err(_) => Err(memory_dispatch_timeout()),
    }
}

fn rule_not_found(rule_id: &str) -> (StatusCode, Json<serde_json::Value>) {
    memory_failure(
        StatusCode::NOT_FOUND,
        format!("procedural rule not found: {rule_id}"),
        "not_found",
        format!("Procedural rule {rule_id} is absent, so reinforcement would be a no-op."),
        "List /v1/memory/procedural or create the intended rule before retrying reinforcement.",
        "Likely stale rule id, wrong daemon data_dir, or cross-session memory reference.",
    )
}

#[derive(Debug, Clone, Deserialize, Default)]
struct SemanticQuery {
    limit: Option<usize>,
    cursor: Option<usize>,
    #[serde(default = "default_true")]
    summary_only: bool,
    #[serde(default)]
    include_full_payload: bool,
    #[serde(default)]
    force_full_payload: bool,
}

fn default_true() -> bool {
    true
}

fn semantic_default_limit() -> usize {
    env_limit(
        "FOCUSA_MEMORY_SEMANTIC_DEFAULT_LIMIT",
        DEFAULT_SEMANTIC_LIMIT,
    )
}

fn semantic_full_limit() -> usize {
    env_limit("FOCUSA_MEMORY_SEMANTIC_FULL_LIMIT", MAX_SEMANTIC_LIMIT).max(semantic_default_limit())
}

fn limit_page<T: Clone>(items: &[T], cursor: usize, limit: usize) -> (Vec<T>, Option<String>) {
    let total = items.len();
    let start = cursor.min(total);
    let end = (start + limit).min(total);
    let out = items
        .iter()
        .rev()
        .skip(start)
        .take(end.saturating_sub(start))
        .cloned()
        .collect::<Vec<_>>();
    let next_cursor = (end < total).then(|| end.to_string());
    (out, next_cursor)
}

fn semantic_summary(records: &[SemanticRecord]) -> Vec<serde_json::Value> {
    records
        .iter()
        .map(|record| {
            json!({
                "key": record.key,
                "value": record.value,
                "updated_at": record.updated_at,
                "pinned": record.pinned,
            })
        })
        .collect()
}

async fn semantic(
    Query(query): Query<SemanticQuery>,
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    let total = focusa.memory.semantic.len();
    let default_limit = semantic_default_limit();
    let full_limit = semantic_full_limit();
    let full_payload_blocked =
        full_payload_blocked_by_pressure(query.include_full_payload, query.force_full_payload);
    let effective_include_full_payload = query.include_full_payload && !full_payload_blocked;
    let effective_summary_only =
        (query.summary_only && !effective_include_full_payload) || full_payload_blocked;
    let pressure = pressure_status();
    let mut options = BoundedReadOptions {
        requested_limit: query.limit,
        include_full_payload: effective_include_full_payload,
        summary_only: effective_summary_only,
        cursor: query.cursor.map(|v| v.to_string()),
        next_cursor: None,
        default_limit,
        full_limit,
    };
    let resolved_limit = options.resolved_limit();
    let (semantic, next_cursor) = limit_page(
        &focusa.memory.semantic,
        query.cursor.unwrap_or(0),
        resolved_limit,
    );
    options.next_cursor = next_cursor;
    let bounds = bounded_metadata(total, semantic.len(), options);
    let payload = json!({
        "semantic": if effective_summary_only { json!(semantic_summary(&semantic)) } else { json!(semantic) },
        "count": total,
        "bounds": bounds,
        "pressure": pressure,
        "degraded": full_payload_blocked,
        "full_payload_blocked_by_pressure": full_payload_blocked,
    });
    record_json_response_size("/v1/memory/semantic", &payload);
    Json(payload)
}

#[derive(Deserialize)]
struct UpsertBody {
    key: String,
    value: String,
    #[serde(default = "default_source")]
    source: MemorySource,
}

fn default_source() -> MemorySource {
    MemorySource::User
}

async fn upsert_semantic(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpsertBody>,
) -> MemoryResult {
    dispatch_memory_action(
        &state,
        Action::UpsertSemantic {
            key: body.key,
            value: body.value,
            source: body.source,
        },
    )
    .await
}

async fn procedural(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    Json(json!({
        "procedural": focusa.memory.procedural,
    }))
}

#[derive(Deserialize)]
struct ReinforceBody {
    rule_id: String,
}

async fn reinforce_rule(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ReinforceBody>,
) -> MemoryResult {
    let rule_id = body.rule_id.trim().to_string();
    if rule_id.is_empty() {
        return Err(memory_failure(
            StatusCode::BAD_REQUEST,
            "rule_id is required",
            "validation_rejected",
            "Procedural reinforcement requires a non-empty rule_id.",
            "Pass a rule_id returned by /v1/memory/procedural.",
            "Likely malformed request body or empty stale variable.",
        ));
    }

    {
        let focusa = state.focusa.read().await;
        if !focusa
            .memory
            .procedural
            .iter()
            .any(|rule| rule.id == rule_id)
        {
            return Err(rule_not_found(&rule_id));
        }
    }

    dispatch_memory_action(&state, Action::ReinforceRule { rule_id }).await
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/memory/semantic", get(semantic))
        .route("/v1/memory/semantic/upsert", post(upsert_semantic))
        .route("/v1/memory/procedural", get(procedural))
        .route("/v1/memory/procedural/reinforce", post(reinforce_rule))
}

#[cfg(test)]
mod tests {
    use super::{limit_page, rule_not_found, semantic_summary};
    use chrono::Utc;
    use focusa_core::types::{MemorySource, SemanticRecord};

    fn record(key: &str, value: &str, pinned: bool) -> SemanticRecord {
        SemanticRecord {
            key: key.to_string(),
            value: value.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            source: MemorySource::User,
            confidence: 1.0,
            ttl: None,
            tags: vec![],
            pinned,
        }
    }

    #[test]
    fn limit_page_returns_cursor_window() {
        let items = vec![
            record("a", "1", false),
            record("b", "2", true),
            record("c", "3", false),
        ];
        let (limited, next_cursor) = limit_page(&items, 1, 2);
        assert_eq!(limited.len(), 2);
        assert_eq!(limited[0].key, "b");
        assert_eq!(limited[1].key, "a");
        assert_eq!(next_cursor, None);
    }

    #[test]
    fn semantic_summary_strips_heavy_fields() {
        let items = vec![record("alpha", "beta", true)];
        let summary = semantic_summary(&items);
        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0]["key"], "alpha");
        assert_eq!(summary[0]["value"], "beta");
        assert_eq!(summary[0]["pinned"], true);
        assert!(summary[0].get("tags").is_none());
        assert!(summary[0].get("confidence").is_none());
    }

    #[test]
    fn missing_rule_reinforce_returns_not_found_failure() {
        let (status, body) = rule_not_found("bogus-rule");
        assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
        assert_eq!(body.0["failure_class"], "not_found");
        assert!(body.0["why"].as_str().unwrap().contains("would be a no-op"));
    }
}
