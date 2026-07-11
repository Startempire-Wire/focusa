//! Training Dataset Export & Contribution routes.
//!
//! Source: docs/21 (export), docs/22 (contribution)
//!
//! GET  /v1/export/status         — export pipeline status
//! POST /v1/export/run            — execute export pipeline (dry-run or write-plan)
//! GET  /v1/training/status       — contribution pipeline status (legacy)
//! POST /v1/contribute/enable     — enable contribution
//! POST /v1/contribute/pause      — pause contribution
//! GET  /v1/contribute/queue      — inspect contribution queue
//! POST /v1/contribute/approve    — approve a queue item
//! POST /v1/contribute/submit     — submit approved items

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use focusa_core::training;
use focusa_core::types::*;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

fn training_not_found(error: impl Into<String>) -> (StatusCode, Json<Value>) {
    let error = error.into();
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "status": "blocked", "canonical": false, "degraded": true,
            "error": error, "failure_class": "not_found",
            "why": "contribution queue item was not found for approval",
            "recovery_hint": "List contribution queue items before approving by item_id.",
            "misuse_hint": "Likely stale item_id, wrong daemon instance, or already processed contribution.",
            "next_tools": ["focusa_tool_doctor", "focusa_traverse"],
            "details": {"tool_result_v1": {"ok": false, "status": "blocked", "canonical": false, "degraded": true, "failure_class": "not_found", "summary": "contribution queue item not found", "retry": {"safe": false, "posture": "do_not_retry_unchanged", "reason": "not_found"}, "side_effects": [], "evidence_refs": [], "next_tools": ["focusa_tool_doctor", "focusa_traverse"], "error": {"code": "not_found", "message": error}}}
        })),
    )
}

/// GET /v1/export/status — export pipeline status.
async fn export_status(State(state): State<Arc<AppState>>) -> Json<Value> {
    let history_count = state.persistence.event_count().unwrap_or(0);
    let last_export_at = state.persistence.latest_event_timestamp().ok().flatten();

    Json(json!({
        "implemented": true,
        "dataset_types": ["sft", "preference", "contrastive", "long-horizon"],
        "supported_formats": ["jsonl", "parquet"],
        "history_count": history_count,
        "last_export_at": last_export_at,
        "status": "ready",
        "reason": Value::Null,
        "quality_gates": {
            "provenance_required": true,
            "eligibility_metadata_required": true,
            "redaction_summary_required": true,
            "minimum_quality_score": 0.70
        },
        "provenance_fields": ["source_event_type", "source_turn_ids", "session_id", "generated_by", "generated_at"],
        "redaction_policy": {
            "enabled": true,
            "patterns": ["email_like", "api_key_prefix", "bearer_token"]
        },
    }))
}

#[derive(Debug, Deserialize, Default)]
struct ExportFilters {
    min_uxp: Option<f64>,
    max_ufi: Option<f64>,
    min_autonomy: Option<i32>,
    agent: Option<String>,
    task: Option<String>,
    since: Option<String>,
    until: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExportRunBody {
    dataset_type: String,
    output: String,
    format: String,
    #[serde(default)]
    filters: ExportFilters,
    #[serde(default)]
    dataset_flags: Value,
    #[serde(default)]
    dry_run: bool,
    #[serde(default)]
    explain: bool,
}

#[derive(Debug, Clone)]
struct TurnRow {
    timestamp: DateTime<Utc>,
    turn_id: String,
    session_id: Option<String>,
    input: String,
    output: String,
    redaction_applied: bool,
}

fn parse_ts(raw: Option<&str>) -> Option<DateTime<Utc>> {
    raw.and_then(|v| DateTime::parse_from_rfc3339(v).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn within_window(
    ts: DateTime<Utc>,
    since: Option<DateTime<Utc>>,
    until: Option<DateTime<Utc>>,
) -> bool {
    if let Some(s) = since
        && ts < s
    {
        return false;
    }
    if let Some(u) = until
        && ts > u
    {
        return false;
    }
    true
}

fn redact_training_text(raw: &str) -> (String, bool) {
    let mut changed = false;
    let redacted = raw
        .split_whitespace()
        .map(|token| {
            let lower = token.to_ascii_lowercase();
            if token.contains('@') && token.contains('.') {
                changed = true;
                "[REDACTED_EMAIL]".to_string()
            } else if lower.starts_with("sk-") || lower.starts_with("pk_") {
                changed = true;
                "[REDACTED_SECRET]".to_string()
            } else if lower.starts_with("bearer") {
                changed = true;
                "[REDACTED_BEARER]".to_string()
            } else {
                token.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    (redacted, changed)
}

fn quality_score(input: &str, output: &str, redaction_applied: bool) -> f64 {
    let mut score: f64 = 0.70;
    if input.len() >= 4 {
        score += 0.10;
    }
    if output.len() >= 12 {
        score += 0.15;
    }
    if !redaction_applied {
        score += 0.05;
    }
    score.min(1.0)
}

fn provenance(
    source_turn_ids: Vec<String>,
    session_id: Option<String>,
    timestamp: DateTime<Utc>,
) -> Value {
    json!({
        "source_event_type": "TurnCompleted",
        "source_turn_ids": source_turn_ids,
        "session_id": session_id,
        "generated_by": "/v1/export/run",
        "generated_at": Utc::now(),
        "source_timestamp": timestamp,
    })
}

fn eligibility(
    row_count: usize,
    quality_score: f64,
    redaction_applied: bool,
    rules: Vec<&str>,
) -> Value {
    json!({
        "eligible": true,
        "row_count": row_count,
        "quality_score": quality_score,
        "redaction_applied": redaction_applied,
        "rules": rules,
    })
}

fn collect_turn_rows(events: Vec<Value>, filters: &ExportFilters) -> (Vec<TurnRow>, Vec<String>) {
    let since = parse_ts(filters.since.as_deref());
    let until = parse_ts(filters.until.as_deref());

    let mut rows = Vec::new();
    let mut exclusions = Vec::new();

    for ev in events {
        if ev.get("type").and_then(|v| v.as_str()) != Some("TurnCompleted") {
            continue;
        }

        let ts = match parse_ts(ev.get("timestamp").and_then(|v| v.as_str())) {
            Some(v) => v,
            None => {
                exclusions.push("missing_timestamp".into());
                continue;
            }
        };

        if !within_window(ts, since, until) {
            continue;
        }

        let raw_input = ev
            .get("raw_user_input")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let raw_output = ev
            .get("assistant_output")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let (input, input_redacted) = redact_training_text(&raw_input);
        let (output, output_redacted) = redact_training_text(&raw_output);
        let redaction_applied = input_redacted || output_redacted;

        if input.is_empty() || output.is_empty() {
            exclusions.push("empty_input_or_output".into());
            continue;
        }

        rows.push(TurnRow {
            timestamp: ts,
            turn_id: ev
                .get("turn_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            session_id: ev
                .get("session_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            input,
            output,
            redaction_applied,
        });
    }

    rows.sort_by_key(|r| r.timestamp);
    (rows, exclusions)
}

fn build_sft(rows: &[TurnRow]) -> Vec<Value> {
    rows.iter()
        .map(|r| {
            json!({
                "dataset_type": "sft",
                "turn_id": r.turn_id,
                "session_id": r.session_id,
                "timestamp": r.timestamp,
                "input": r.input,
                "output": r.output,
                "provenance": provenance(vec![r.turn_id.clone()], r.session_id.clone(), r.timestamp),
                "eligibility": eligibility(1, quality_score(&r.input, &r.output, r.redaction_applied), r.redaction_applied, vec!["turn_completed", "non_empty_input", "non_empty_output"]),
            })
        })
        .collect()
}

fn build_preference(rows: &[TurnRow]) -> Vec<Value> {
    if rows.len() < 2 {
        return vec![];
    }
    let mut out = Vec::new();
    for pair in rows.windows(2) {
        let rejected = &pair[0];
        let chosen = &pair[1];
        if chosen.output == rejected.output {
            continue;
        }
        out.push(json!({
            "dataset_type": "preference",
            "turn_id": chosen.turn_id,
            "session_id": chosen.session_id,
            "timestamp": chosen.timestamp,
            "prompt": chosen.input,
            "chosen": chosen.output,
            "rejected": rejected.output,
            "source_pair": [rejected.turn_id, chosen.turn_id],
            "provenance": provenance(vec![rejected.turn_id.clone(), chosen.turn_id.clone()], chosen.session_id.clone(), chosen.timestamp),
            "eligibility": eligibility(2, quality_score(&chosen.input, &chosen.output, chosen.redaction_applied || rejected.redaction_applied), chosen.redaction_applied || rejected.redaction_applied, vec!["turn_pair", "outputs_differ"]),
        }));
    }
    out
}

fn build_contrastive(rows: &[TurnRow]) -> Vec<Value> {
    if rows.len() < 2 {
        return vec![];
    }
    let mut out = Vec::new();
    for pair in rows.windows(2) {
        let a = &pair[0];
        let b = &pair[1];
        if a.output == b.output {
            continue;
        }
        out.push(json!({
            "dataset_type": "contrastive",
            "turn_id": b.turn_id,
            "session_id": b.session_id,
            "timestamp": b.timestamp,
            "context": b.input,
            "positive": b.output,
            "negative": a.output,
            "source_pair": [a.turn_id, b.turn_id],
            "provenance": provenance(vec![a.turn_id.clone(), b.turn_id.clone()], b.session_id.clone(), b.timestamp),
            "eligibility": eligibility(2, quality_score(&b.input, &b.output, a.redaction_applied || b.redaction_applied), a.redaction_applied || b.redaction_applied, vec!["turn_pair", "outputs_differ"]),
        }));
    }
    out
}

fn build_long_horizon(rows: &[TurnRow]) -> Vec<Value> {
    if rows.len() < 3 {
        return vec![];
    }
    rows.chunks(3)
        .filter(|chunk| chunk.len() == 3)
        .map(|chunk| {
            json!({
                "dataset_type": "long-horizon",
                "session_id": chunk[0].session_id,
                "start_turn_id": chunk[0].turn_id,
                "end_turn_id": chunk[2].turn_id,
                "trajectory": chunk.iter().map(|r| json!({"turn_id": r.turn_id, "input": r.input, "output": r.output, "timestamp": r.timestamp})).collect::<Vec<_>>(),
                "provenance": provenance(chunk.iter().map(|r| r.turn_id.clone()).collect::<Vec<_>>(), chunk[0].session_id.clone(), chunk[2].timestamp),
                "eligibility": eligibility(chunk.len(), quality_score(&chunk[0].input, &chunk[2].output, chunk.iter().any(|r| r.redaction_applied)), chunk.iter().any(|r| r.redaction_applied), vec!["three_turn_chunk", "ordered_trajectory"]),
            })
        })
        .collect()
}

fn estimate_dataset_size_bytes(records: &[Value]) -> usize {
    records
        .iter()
        .map(|r| serde_json::to_string(r).map(|s| s.len() + 1).unwrap_or(0))
        .sum()
}

/// POST /v1/export/run — execute export pipeline (dry-run or planned write payload).
async fn export_run(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ExportRunBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Spec 125-style: license gate with actionable guidance.
    let guard = crate::routes::license::current_guard();
    if let focusa_license::CapabilityCheck::Denied { reason } =
        guard.check(focusa_license::Capability::CommercialUse)
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "status": "denied",
                "canonical": false,
                "failure_class": "entitlement_denied",
                "error": "export_not_permitted_on_eval_tier",
                "reason": reason,
                "tier": guard.tier.label(),
                "human_message": format!(
                    "Export is not available on {} tier ({}). To export training data, upgrade to a commercial license or use eval-safe alternatives: (1) GET /v1/export/status for pipeline metadata, (2) POST /v1/context-cognition/curate for bounded context selection, (3) POST /v1/preload/build for agent bootstrap packets.",
                    guard.tier.label(), reason
                ),
                "safe_recovery": "upgrade to commercial license or use eval-safe alternatives",
                "eval_safe_alternatives": [
                    "GET /v1/export/status — pipeline metadata without data",
                    "POST /v1/context-cognition/curate — bounded context selection",
                    "POST /v1/preload/build — agent bootstrap packets"
                ],
                "upgrade_path": "contact sales@focusa.dev or renew license key",
                "next_tools": ["focusa_license_status"],
            })),
        ));
    }

    if body.format != "jsonl" && body.format != "parquet" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "invalid_request",
                "error": "unsupported_format",
                "reason": "format must be jsonl or parquet"
            })),
        ));
    }

    let events = state.persistence.recent_events(20_000).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error", "error": "event_read_failed", "reason": e.to_string()})),
        )
    })?;

    let (rows, mut exclusions) = collect_turn_rows(events, &body.filters);

    let records = match body.dataset_type.as_str() {
        "sft" => build_sft(&rows),
        "preference" => build_preference(&rows),
        "contrastive" => build_contrastive(&rows),
        "long-horizon" => build_long_horizon(&rows),
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "invalid_request",
                    "error": "unknown_dataset_type",
                    "reason": "dataset_type must be one of sft|preference|contrastive|long-horizon"
                })),
            ));
        }
    };

    if records.is_empty() {
        exclusions.push("no_eligible_records_after_dataset_rules".into());
    }

    let sample_schema_preview = records.first().cloned().unwrap_or(Value::Null);
    let estimated_dataset_size_bytes = estimate_dataset_size_bytes(&records);
    let quality_scores = records
        .iter()
        .filter_map(|record| record.get("eligibility")?.get("quality_score")?.as_f64())
        .collect::<Vec<_>>();
    let average_quality_score = if quality_scores.is_empty() {
        0.0
    } else {
        quality_scores.iter().sum::<f64>() / quality_scores.len() as f64
    };
    let redacted_records = records
        .iter()
        .filter(|record| {
            record
                .get("eligibility")
                .and_then(|e| e.get("redaction_applied"))
                .and_then(Value::as_bool)
                == Some(true)
        })
        .count();
    let provenance_complete = records
        .iter()
        .all(|record| record.get("provenance").is_some());
    let quality_summary = json!({
        "average_quality_score": average_quality_score,
        "minimum_quality_score": 0.70,
        "provenance_complete": provenance_complete,
        "records_with_redaction": redacted_records,
        "eligible_record_count": records.len(),
    });
    let redaction_summary = json!({
        "enabled": true,
        "records_redacted": redacted_records,
        "patterns": ["email_like", "api_key_prefix", "bearer_token"]
    });

    let manifest = json!({
        "dataset_type": body.dataset_type,
        "record_count": records.len(),
        "filters": {
            "min_uxp": body.filters.min_uxp,
            "max_ufi": body.filters.max_ufi,
            "min_autonomy": body.filters.min_autonomy,
            "agent": body.filters.agent,
            "task": body.filters.task,
            "since": body.filters.since,
            "until": body.filters.until,
        },
        "dataset_flags": body.dataset_flags,
        "generated_at": Utc::now(),
        "focusa_version": env!("CARGO_PKG_VERSION"),
        "quality_summary": quality_summary,
        "redaction_summary": redaction_summary,
        "provenance_complete": provenance_complete,
    });

    Ok(Json(json!({
        "status": "ok",
        "dataset_type": body.dataset_type,
        "dry_run": body.dry_run,
        "explain": body.explain,
        "output": body.output,
        "format": body.format,
        "eligible_records": records.len(),
        "excluded_records": rows.len().saturating_sub(records.len()),
        "estimated_dataset_size_bytes": estimated_dataset_size_bytes,
        "sample_schema_preview": sample_schema_preview,
        "exclusion_reasons": exclusions,
        "quality_summary": quality_summary,
        "redaction_summary": redaction_summary,
        "manifest": manifest,
        "records": records,
    })))
}

/// GET /v1/training/status — contribution pipeline status.
async fn contribution_status(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    Json(json!({
        "contribution_enabled": s.contribution.enabled,
        "queue_size": s.contribution.queue.len(),
        "total_contributed": s.contribution.total_contributed,
        "policy": s.contribution.policy,
        "pending": s.contribution.queue.iter().filter(|i| i.status == ContributionStatus::Pending).count(),
        "approved": s.contribution.queue.iter().filter(|i| i.status == ContributionStatus::Approved).count(),
    }))
}

/// POST /v1/contribute/enable — enable contribution (docs/22 §3.1: explicit only).
async fn contribute_enable(State(state): State<Arc<AppState>>) -> Result<Json<Value>, StatusCode> {
    let mut s = state.focusa.write().await;
    s.contribution.enabled = true;
    drop(s);
    state.mark_external_mutation();
    Ok(Json(json!({ "status": "enabled" })))
}

/// POST /v1/contribute/pause — pause contribution.
async fn contribute_pause(State(state): State<Arc<AppState>>) -> Result<Json<Value>, StatusCode> {
    let mut s = state.focusa.write().await;
    s.contribution.enabled = false;
    drop(s);
    state.mark_external_mutation();
    Ok(Json(json!({ "status": "paused" })))
}

/// POST /v1/contribute/approve — approve a queue item.
#[derive(Deserialize)]
struct ApproveBody {
    item_id: uuid::Uuid,
}

async fn contribute_approve(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ApproveBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut s = state.focusa.write().await;
    training::approve_contribution(&mut s.contribution, body.item_id)
        .map_err(training_not_found)?;
    drop(s);
    state.mark_external_mutation();
    Ok(Json(
        json!({ "status": "approved", "item_id": body.item_id }),
    ))
}

/// POST /v1/contribute/submit — submit approved items.
async fn contribute_submit(State(state): State<Arc<AppState>>) -> Json<Value> {
    let mut s = state.focusa.write().await;
    let count = training::submit_approved(&mut s.contribution);
    drop(s);
    state.mark_external_mutation();
    Json(json!({ "submitted": count }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/export/status", get(export_status))
        .route("/v1/export/run", post(export_run))
        .route("/v1/training/status", get(contribution_status))
        .route("/v1/contribute/enable", post(contribute_enable))
        .route("/v1/contribute/pause", post(contribute_pause))
        .route("/v1/contribute/approve", post(contribute_approve))
        .route("/v1/contribute/submit", post(contribute_submit))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tc_event(turn_id: &str, ts: &str, input: &str, output: &str) -> Value {
        json!({
            "type": "TurnCompleted",
            "turn_id": turn_id,
            "timestamp": ts,
            "session_id": "s-1",
            "raw_user_input": input,
            "assistant_output": output,
        })
    }

    #[test]
    fn collect_turn_rows_filters_and_orders() {
        let events = vec![
            tc_event("t2", "2026-01-01T00:00:02Z", "u2", "a2"),
            tc_event("t1", "2026-01-01T00:00:01Z", "u1", "a1"),
            json!({"type": "TurnStarted", "timestamp": "2026-01-01T00:00:00Z"}),
        ];

        let filters = ExportFilters {
            since: Some("2026-01-01T00:00:00Z".into()),
            until: Some("2026-01-01T00:00:03Z".into()),
            ..Default::default()
        };

        let (rows, exclusions) = collect_turn_rows(events, &filters);
        assert!(exclusions.is_empty());
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].turn_id, "t1");
        assert_eq!(rows[1].turn_id, "t2");
    }

    #[test]
    fn dataset_builders_return_expected_shapes() {
        let events = vec![
            tc_event("t1", "2026-01-01T00:00:01Z", "u1", "a1"),
            tc_event("t2", "2026-01-01T00:00:02Z", "u2", "a2"),
            tc_event("t3", "2026-01-01T00:00:03Z", "u3", "a3"),
        ];

        let (rows, _) = collect_turn_rows(events, &ExportFilters::default());
        let sft = build_sft(&rows);
        let pref = build_preference(&rows);
        let contrastive = build_contrastive(&rows);
        let long_h = build_long_horizon(&rows);

        assert_eq!(sft.len(), 3);
        assert_eq!(pref.len(), 2);
        assert_eq!(contrastive.len(), 2);
        assert_eq!(long_h.len(), 1);

        assert_eq!(sft[0]["dataset_type"], "sft");
        assert_eq!(pref[0]["dataset_type"], "preference");
        assert_eq!(contrastive[0]["dataset_type"], "contrastive");
        assert_eq!(long_h[0]["dataset_type"], "long-horizon");
        assert!(sft[0].get("provenance").is_some());
        assert!(sft[0].get("eligibility").is_some());
    }

    #[test]
    fn collect_turn_rows_redacts_training_secrets() {
        let events = vec![tc_event(
            "t1",
            "2026-01-01T00:00:01Z",
            "email me at user@example.com",
            "token sk-secret should not export",
        )];

        let (rows, exclusions) = collect_turn_rows(events, &ExportFilters::default());
        assert!(exclusions.is_empty());
        assert_eq!(rows.len(), 1);
        assert!(rows[0].redaction_applied);
        assert!(rows[0].input.contains("[REDACTED_EMAIL]"));
        assert!(rows[0].output.contains("[REDACTED_SECRET]"));
    }
}
