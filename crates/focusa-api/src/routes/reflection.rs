//! Reflection loop overlay routes.
//!
//! POST /v1/reflect/run
//! GET  /v1/reflect/history?limit=20
//! GET  /v1/reflect/status
//! GET  /v1/reflect/scheduler
//! POST /v1/reflect/scheduler
//! POST /v1/reflect/scheduler/tick

use crate::server::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ReflectMode {
    Manual,
    Scheduled,
}

#[derive(Debug, Clone, Deserialize)]
struct ReflectRunRequest {
    mode: ReflectMode,
    #[serde(default)]
    idempotency_key: Option<String>,
    #[serde(default)]
    window: Option<String>,
    #[serde(default)]
    budget: Option<u32>,
    #[serde(default)]
    context: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct HistoryParams {
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    stop_reason: Option<String>,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    since: Option<String>,
    #[serde(default)]
    until: Option<String>,
    #[serde(default)]
    cursor_before: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SchedulerUpdateRequest {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    interval_seconds: Option<u64>,
    #[serde(default)]
    max_iterations_per_window: Option<u32>,
    #[serde(default)]
    cooldown_seconds: Option<u64>,
    #[serde(default)]
    low_confidence_threshold: Option<f64>,
    #[serde(default)]
    no_delta_min_event_delta: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct SchedulerTickRequest {
    #[serde(default)]
    window: Option<String>,
    #[serde(default)]
    budget: Option<u32>,
}

#[derive(Debug, Clone)]
struct SchedulerConfig {
    enabled: bool,
    interval_seconds: u64,
    max_iterations_per_window: u32,
    cooldown_seconds: u64,
    low_confidence_threshold: f64,
    no_delta_min_event_delta: u32,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_seconds: 3600,
            max_iterations_per_window: 1,
            cooldown_seconds: 300,
            low_confidence_threshold: 0.5,
            no_delta_min_event_delta: 1,
        }
    }
}

fn default_limit() -> usize {
    20
}

fn parse_rfc3339_param(
    name: &str,
    raw: &str,
) -> Result<chrono::DateTime<chrono::FixedOffset>, (StatusCode, Json<Value>)> {
    let normalized = raw.trim().replace(' ', "+");
    chrono::DateTime::parse_from_rfc3339(&normalized).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"code":"invalid_time_filter","message": format!("{} must be RFC3339", name)})),
        )
    })
}

fn focusa_db_path(data_dir: &str) -> PathBuf {
    if let Some(rest) = data_dir.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return PathBuf::from(home).join(rest).join("focusa.sqlite");
    }
    PathBuf::from(data_dir).join("focusa.sqlite")
}

fn ensure_reflection_tables(conn: &Connection) -> anyhow::Result<()> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS reflection_runs (
          iteration_id TEXT PRIMARY KEY,
          mode TEXT NOT NULL,
          idempotency_key TEXT,
          window TEXT NOT NULL,
          budget INTEGER,
          result_json TEXT NOT NULL,
          created_at TEXT NOT NULL,
          UNIQUE(idempotency_key, window)
        )
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS reflection_scheduler_config (
          id INTEGER PRIMARY KEY CHECK(id=1),
          enabled INTEGER NOT NULL,
          interval_seconds INTEGER NOT NULL,
          max_iterations_per_window INTEGER NOT NULL,
          cooldown_seconds INTEGER NOT NULL,
          low_confidence_threshold REAL NOT NULL DEFAULT 0.5,
          no_delta_min_event_delta INTEGER NOT NULL DEFAULT 1,
          updated_at TEXT NOT NULL
        )
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS reflection_window_counters (
          window_key TEXT PRIMARY KEY,
          run_count INTEGER NOT NULL,
          last_run_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        )
        "#,
        [],
    )?;

    let mut has_low_conf = false;
    let mut has_no_delta = false;
    let mut pragma = conn.prepare("PRAGMA table_info(reflection_scheduler_config)")?;
    let cols = pragma.query_map([], |r| r.get::<_, String>(1))?;
    for c in cols.flatten() {
        if c == "low_confidence_threshold" {
            has_low_conf = true;
        }
        if c == "no_delta_min_event_delta" {
            has_no_delta = true;
        }
    }
    if !has_low_conf {
        conn.execute(
            "ALTER TABLE reflection_scheduler_config ADD COLUMN low_confidence_threshold REAL NOT NULL DEFAULT 0.5",
            [],
        )?;
    }
    if !has_no_delta {
        conn.execute(
            "ALTER TABLE reflection_scheduler_config ADD COLUMN no_delta_min_event_delta INTEGER NOT NULL DEFAULT 1",
            [],
        )?;
    }

    conn.execute(
        "INSERT OR IGNORE INTO reflection_scheduler_config(id,enabled,interval_seconds,max_iterations_per_window,cooldown_seconds,low_confidence_threshold,no_delta_min_event_delta,updated_at) VALUES (1,0,3600,1,300,0.5,1,?1)",
        [Utc::now().to_rfc3339()],
    )?;

    Ok(())
}

fn load_scheduler_config(conn: &Connection) -> anyhow::Result<SchedulerConfig> {
    let row: Option<(i64, i64, i64, i64, f64, i64)> = conn
        .query_row(
            "SELECT enabled, interval_seconds, max_iterations_per_window, cooldown_seconds, low_confidence_threshold, no_delta_min_event_delta FROM reflection_scheduler_config WHERE id=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
        )
        .optional()?;

    if let Some((enabled, interval, max_iter, cooldown, low_conf, no_delta)) = row {
        Ok(SchedulerConfig {
            enabled: enabled != 0,
            interval_seconds: interval.max(1) as u64,
            max_iterations_per_window: max_iter.max(1) as u32,
            cooldown_seconds: cooldown.max(0) as u64,
            low_confidence_threshold: low_conf.clamp(0.0, 1.0),
            no_delta_min_event_delta: no_delta.max(0) as u32,
        })
    } else {
        Ok(SchedulerConfig::default())
    }
}

fn save_scheduler_config(conn: &Connection, cfg: &SchedulerConfig) -> anyhow::Result<()> {
    conn.execute(
        "UPDATE reflection_scheduler_config SET enabled=?1, interval_seconds=?2, max_iterations_per_window=?3, cooldown_seconds=?4, low_confidence_threshold=?5, no_delta_min_event_delta=?6, updated_at=?7 WHERE id=1",
        params![
            if cfg.enabled { 1 } else { 0 },
            cfg.interval_seconds as i64,
            cfg.max_iterations_per_window as i64,
            cfg.cooldown_seconds as i64,
            cfg.low_confidence_threshold,
            cfg.no_delta_min_event_delta as i64,
            Utc::now().to_rfc3339(),
        ],
    )?;
    Ok(())
}

fn parse_window_to_secs(window: &str) -> u64 {
    if let Some(v) = window.strip_suffix('h').and_then(|s| s.parse::<u64>().ok()) {
        return (v.max(1)) * 3600;
    }
    if let Some(v) = window.strip_suffix('m').and_then(|s| s.parse::<u64>().ok()) {
        return (v.max(1)) * 60;
    }
    if let Some(v) = window.strip_suffix('d').and_then(|s| s.parse::<u64>().ok()) {
        return (v.max(1)) * 86_400;
    }
    3600
}

fn window_bucket_key(window: &str, now_epoch: i64) -> String {
    let span = parse_window_to_secs(window) as i64;
    let bucket = if span > 0 {
        now_epoch / span
    } else {
        now_epoch / 3600
    };
    format!("{}:{}", window, bucket)
}

fn latest_reflection_result_for_window(
    conn: &Connection,
    window: &str,
) -> anyhow::Result<Option<Value>> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT result_json FROM reflection_runs WHERE window = ?1 ORDER BY created_at DESC LIMIT 1",
            [window],
            |r| r.get(0),
        )
        .optional()?;

    Ok(raw.and_then(|s| serde_json::from_str::<Value>(&s).ok()))
}

fn extract_event_count(result: &Value) -> Option<u64> {
    if let Some(v) = result.get("event_count_snapshot").and_then(|v| v.as_u64()) {
        return Some(v);
    }

    result
        .get("observations")
        .and_then(|v| v.as_array())
        .and_then(|obs| {
            obs.iter().find_map(|o| {
                if o.get("kind").and_then(|k| k.as_str()) == Some("event_count") {
                    o.get("value").and_then(|v| v.as_u64())
                } else {
                    None
                }
            })
        })
}

fn latest_scheduler_counter(conn: &Connection) -> anyhow::Result<Option<(String, u32, String)>> {
    conn
        .query_row(
            "SELECT window_key, run_count, last_run_at FROM reflection_window_counters ORDER BY updated_at DESC LIMIT 1",
            [],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as u32, r.get::<_, String>(2)?)),
        )
        .optional()
        .map_err(Into::into)
}

fn stop_reason_counts(conn: &Connection) -> anyhow::Result<Value> {
    let mut stmt = conn.prepare("SELECT result_json FROM reflection_runs")?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;

    let mut counts = std::collections::BTreeMap::<String, u64>::new();
    for raw in rows.flatten() {
        if let Ok(v) = serde_json::from_str::<Value>(&raw)
            && let Some(reason) = v.get("stop_reason").and_then(|s| s.as_str())
        {
            *counts.entry(reason.to_string()).or_insert(0) += 1;
        }
    }

    Ok(json!(counts))
}

fn check_and_bump_guardrails(
    conn: &Connection,
    window: &str,
    max_iterations: u32,
    cooldown_seconds: u64,
) -> anyhow::Result<Option<String>> {
    let now = Utc::now().timestamp();
    let key = window_bucket_key(window, now);

    let row: Option<(i64, String)> = conn
        .query_row(
            "SELECT run_count, last_run_at FROM reflection_window_counters WHERE window_key=?1",
            [key.clone()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;

    if let Some((count, last_run_at)) = row {
        if count as u32 >= max_iterations {
            return Ok(Some("window_iteration_cap_reached".to_string()));
        }

        if let Ok(last) = chrono::DateTime::parse_from_rfc3339(&last_run_at) {
            let elapsed = now.saturating_sub(last.timestamp()) as u64;
            if elapsed < cooldown_seconds {
                return Ok(Some("cooldown_active".to_string()));
            }
        }

        conn.execute(
            "UPDATE reflection_window_counters SET run_count=run_count+1,last_run_at=?2,updated_at=?2 WHERE window_key=?1",
            params![key, Utc::now().to_rfc3339()],
        )?;
    } else {
        conn.execute(
            "INSERT INTO reflection_window_counters(window_key,run_count,last_run_at,updated_at) VALUES (?1,1,?2,?2)",
            params![key, Utc::now().to_rfc3339()],
        )?;
    }

    Ok(None)
}

struct ReflectionInputs {
    stack_depth: usize,
    active_frame_present: bool,
    latest_event_ts: Option<String>,
    event_count: u64,
}

async fn collect_reflection_inputs(state: &Arc<AppState>) -> ReflectionInputs {
    let (stack_depth, active_frame_present) = {
        let focusa = state.focusa.read().await;
        (
            focusa.focus_stack.frames.len(),
            focusa.focus_stack.active_id.is_some(),
        )
    };

    ReflectionInputs {
        stack_depth,
        active_frame_present,
        latest_event_ts: state.persistence.latest_event_timestamp().ok().flatten(),
        event_count: state.persistence.event_count().unwrap_or(0),
    }
}

fn execute_reflection(
    conn: &Connection,
    req: ReflectRunRequest,
    enforce_scheduler_guardrails: bool,
    inputs: &ReflectionInputs,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let window = req.window.clone().unwrap_or_else(|| "1h".to_string());

    let scheduler_cfg = load_scheduler_config(conn).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"scheduler_config_read_failed","message": format!("{e}")})),
        )
    })?;

    let idem = req
        .idempotency_key
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    if let Some(ref key) = idem {
        let existing: Option<String> = conn
            .query_row(
                "SELECT result_json FROM reflection_runs WHERE idempotency_key = ?1 AND window = ?2 LIMIT 1",
                params![key, window],
                |r| r.get(0),
            )
            .optional()
            .unwrap_or(None);

        if let Some(raw) = existing {
            let parsed: Value = serde_json::from_str(&raw).unwrap_or(json!({"raw": raw}));
            return Ok(Json(
                json!({"status":"accepted", "duplicate": true, "result": parsed}),
            ));
        }
    }

    if enforce_scheduler_guardrails {
        if !scheduler_cfg.enabled {
            return Ok(Json(json!({
                "status": "skipped",
                "reason": "scheduler_disabled",
                "duplicate": false,
            })));
        }

        if let Some(reason) = check_and_bump_guardrails(
            conn,
            &window,
            scheduler_cfg.max_iterations_per_window,
            scheduler_cfg.cooldown_seconds,
        )
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"code":"scheduler_guardrail_failed","message": format!("{e}")})),
            )
        })? {
            return Ok(Json(json!({
                "status": "skipped",
                "reason": reason,
                "duplicate": false,
            })));
        }
    }

    let observations = vec![
        json!({"kind":"focus_stack_depth","value":inputs.stack_depth}),
        json!({"kind":"event_count","value":inputs.event_count}),
    ];

    let mut risks = vec![];
    if !inputs.active_frame_present {
        risks.push(json!({"kind":"no_active_frame","severity":"medium"}));
    }

    let recommended = vec![
        json!({"action":"review_active_focus","advisory":true}),
        json!({"action":"check_recent_events","advisory":true}),
    ];
    let recommended_value = Value::Array(recommended.clone());

    let prior = latest_reflection_result_for_window(conn, &window)
        .ok()
        .flatten();
    let prior_event_count = prior.as_ref().and_then(extract_event_count);
    let event_delta = prior_event_count
        .map(|v| inputs.event_count.abs_diff(v))
        .unwrap_or(inputs.event_count);

    let no_evidence_delta = if inputs.event_count == 0 {
        true
    } else {
        prior_event_count
            .map(|_| event_delta < scheduler_cfg.no_delta_min_event_delta as u64)
            .unwrap_or(false)
    };

    let repeated_no_delta = no_evidence_delta
        && prior
            .as_ref()
            .map(|p| p.get("recommended_actions") == Some(&recommended_value))
            .unwrap_or(false);

    let confidence = if inputs.active_frame_present {
        0.82
    } else {
        0.66
    };
    let low_confidence = confidence < scheduler_cfg.low_confidence_threshold;

    let stop_reason = if repeated_no_delta {
        "repeated_recommendation_set"
    } else if low_confidence {
        "low_confidence"
    } else if no_evidence_delta {
        "no_evidence_delta"
    } else {
        "single_iteration_complete"
    };

    let iteration_id = Uuid::now_v7().to_string();
    let mode = match req.mode {
        ReflectMode::Manual => "manual",
        ReflectMode::Scheduled => "scheduled",
    };

    let result = json!({
        "iteration_id": iteration_id,
        "mode": mode,
        "window": window,
        "observations": observations,
        "risks": risks,
        "recommended_actions": recommended,
        "confidence": confidence,
        "stop_reason": stop_reason,
        "latest_event_ts": inputs.latest_event_ts,
        "event_count_snapshot": inputs.event_count,
        "event_delta": event_delta,
        "budget": req.budget,
        "context": req.context,
        "policy": {
            "low_confidence_threshold": scheduler_cfg.low_confidence_threshold,
            "no_delta_min_event_delta": scheduler_cfg.no_delta_min_event_delta
        }
    });

    conn.execute(
        "INSERT INTO reflection_runs(iteration_id, mode, idempotency_key, window, budget, result_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            result["iteration_id"].as_str().unwrap_or_default(),
            mode,
            idem,
            result["window"].as_str().unwrap_or("1h"),
            req.budget.map(|v| v as i64),
            result.to_string(),
            Utc::now().to_rfc3339(),
        ],
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"db_insert_failed","message": format!("{e}")})),
        )
    })?;

    Ok(Json(
        json!({"status":"accepted", "duplicate": false, "result": result}),
    ))
}

async fn run_reflection(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ReflectRunRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let inputs = collect_reflection_inputs(&state).await;
    let db_path = focusa_db_path(&state.config.data_dir);
    let conn = Connection::open(db_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"db_open_failed","message": format!("{e}")})),
        )
    })?;

    ensure_reflection_tables(&conn).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"db_schema_failed","message": format!("{e}")})),
        )
    })?;

    execute_reflection(&conn, req, false, &inputs)
}

async fn reflect_history(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HistoryParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let db_path = focusa_db_path(&state.config.data_dir);
    let conn = Connection::open(db_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"db_open_failed","message": format!("{e}")})),
        )
    })?;

    ensure_reflection_tables(&conn).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"db_schema_failed","message": format!("{e}")})),
        )
    })?;

    let mut stmt = conn
        .prepare(
            "SELECT iteration_id, mode, idempotency_key, window, budget, result_json, created_at FROM reflection_runs ORDER BY created_at DESC, iteration_id DESC LIMIT ?1",
        )
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"code":"db_query_failed","message": format!("{e}")})),
            )
        })?;

    let effective_limit = params.limit.clamp(1, 200);

    let rows = stmt
        .query_map([effective_limit as i64], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<i64>>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
            ))
        })
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"code":"db_read_failed","message": format!("{e}")})),
            )
        })?;

    let wanted_reason = params
        .stop_reason
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let wanted_mode = if let Some(m) = params
        .mode
        .as_ref()
        .map(|v| v.trim())
        .filter(|s| !s.is_empty())
    {
        let n = m.to_ascii_lowercase();
        if n != "manual" && n != "scheduled" {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(
                    json!({"code":"invalid_mode_filter","message":"mode must be manual|scheduled"}),
                ),
            ));
        }
        Some(n)
    } else {
        None
    };

    let since_ts = if let Some(s) = params
        .since
        .as_ref()
        .map(|v| v.trim())
        .filter(|s| !s.is_empty())
    {
        Some(parse_rfc3339_param("since", s)?)
    } else {
        None
    };

    let until_ts = if let Some(s) = params
        .until
        .as_ref()
        .map(|v| v.trim())
        .filter(|s| !s.is_empty())
    {
        Some(parse_rfc3339_param("until", s)?)
    } else {
        None
    };

    let cursor_before_ts = if let Some(s) = params
        .cursor_before
        .as_ref()
        .map(|v| v.trim())
        .filter(|s| !s.is_empty())
    {
        Some(parse_rfc3339_param("cursor_before", s)?)
    } else {
        None
    };

    let mut items = vec![];
    for row in rows.flatten() {
        let created_at = chrono::DateTime::parse_from_rfc3339(&row.6).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"code":"invalid_created_at","message":"stored created_at is invalid"})),
            )
        })?;

        if let Some(since) = since_ts.as_ref() {
            if created_at < *since {
                continue;
            }
        }
        if let Some(until) = until_ts.as_ref() {
            if created_at > *until {
                continue;
            }
        }
        if let Some(cursor_before) = cursor_before_ts.as_ref() {
            if created_at >= *cursor_before {
                continue;
            }
        }

        if let Some(want_mode) = wanted_mode.as_ref() {
            if row.1 != *want_mode {
                continue;
            }
        }

        let parsed: Value = serde_json::from_str(&row.5).unwrap_or(json!({"raw": row.5}));
        if let Some(want) = wanted_reason.as_ref() {
            let got = parsed
                .get("stop_reason")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if got != want {
                continue;
            }
        }

        items.push(json!({
            "iteration_id": row.0,
            "mode": row.1,
            "idempotency_key": row.2,
            "window": row.3,
            "budget": row.4,
            "created_at": row.6,
            "result": parsed,
        }));

        if items.len() >= effective_limit {
            break;
        }
    }

    let next_cursor = if items.len() >= effective_limit {
        items
            .last()
            .and_then(|it| it.get("created_at"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    } else {
        None
    };

    Ok(Json(json!({
        "items": items,
        "count": items.len(),
        "next_cursor": next_cursor,
        "applied_filters": {
            "limit": effective_limit,
            "requested_limit": params.limit,
            "stop_reason": wanted_reason,
            "mode": wanted_mode,
            "since": params.since,
            "until": params.until,
            "cursor_before": params.cursor_before
        }
    })))
}

async fn reflect_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let db_path = focusa_db_path(&state.config.data_dir);
    let conn = Connection::open(db_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"db_open_failed","message": format!("{e}")})),
        )
    })?;
    ensure_reflection_tables(&conn).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"db_schema_failed","message": format!("{e}")})),
        )
    })?;
    let cfg = load_scheduler_config(&conn).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"scheduler_config_read_failed","message": format!("{e}")})),
        )
    })?;
    let telemetry = latest_scheduler_counter(&conn).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"scheduler_counter_read_failed","message": format!("{e}")})),
        )
    })?;
    let reason_counts = stop_reason_counts(&conn).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"stop_reason_counts_read_failed","message": format!("{e}")})),
        )
    })?;

    Ok(Json(json!({
        "enabled": true,
        "scheduler": {
            "active": cfg.enabled,
            "mode": if cfg.enabled { "scheduled" } else { "manual_only" },
            "next_run": null,
            "interval_seconds": cfg.interval_seconds,
        },
        "guardrails": {
            "max_iterations_per_window": cfg.max_iterations_per_window,
            "cooldown_seconds": cfg.cooldown_seconds,
            "low_confidence_threshold": cfg.low_confidence_threshold,
            "no_delta_min_event_delta": cfg.no_delta_min_event_delta,
            "advisory_only": true,
            "auto_apply_disabled": true
        },
        "telemetry": {
            "latest_window_key": telemetry.as_ref().map(|t| t.0.clone()),
            "latest_window_run_count": telemetry.as_ref().map(|t| t.1),
            "last_scheduler_run_at": telemetry.as_ref().map(|t| t.2.clone()),
            "stop_reason_counts": reason_counts
        }
    })))
}

async fn reflect_scheduler_get(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let db_path = focusa_db_path(&state.config.data_dir);
    let conn = Connection::open(db_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"db_open_failed","message": format!("{e}")})),
        )
    })?;
    ensure_reflection_tables(&conn).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"db_schema_failed","message": format!("{e}")})),
        )
    })?;
    let cfg = load_scheduler_config(&conn).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"scheduler_config_read_failed","message": format!("{e}")})),
        )
    })?;
    let telemetry = latest_scheduler_counter(&conn).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"scheduler_counter_read_failed","message": format!("{e}")})),
        )
    })?;

    Ok(Json(json!({
        "enabled": cfg.enabled,
        "interval_seconds": cfg.interval_seconds,
        "max_iterations_per_window": cfg.max_iterations_per_window,
        "cooldown_seconds": cfg.cooldown_seconds,
        "low_confidence_threshold": cfg.low_confidence_threshold,
        "no_delta_min_event_delta": cfg.no_delta_min_event_delta,
        "telemetry": {
            "latest_window_key": telemetry.as_ref().map(|t| t.0.clone()),
            "latest_window_run_count": telemetry.as_ref().map(|t| t.1),
            "last_scheduler_run_at": telemetry.as_ref().map(|t| t.2.clone())
        }
    })))
}

async fn reflect_scheduler_update(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SchedulerUpdateRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let db_path = focusa_db_path(&state.config.data_dir);
    let conn = Connection::open(db_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"db_open_failed","message": format!("{e}")})),
        )
    })?;
    ensure_reflection_tables(&conn).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"db_schema_failed","message": format!("{e}")})),
        )
    })?;

    let mut cfg = load_scheduler_config(&conn).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"scheduler_config_read_failed","message": format!("{e}")})),
        )
    })?;

    if let Some(v) = req.enabled {
        cfg.enabled = v;
    }
    if let Some(v) = req.interval_seconds {
        cfg.interval_seconds = v.max(1);
    }
    if let Some(v) = req.max_iterations_per_window {
        cfg.max_iterations_per_window = v.max(1);
    }
    if let Some(v) = req.cooldown_seconds {
        cfg.cooldown_seconds = v;
    }
    if let Some(v) = req.low_confidence_threshold {
        cfg.low_confidence_threshold = v.clamp(0.0, 1.0);
    }
    if let Some(v) = req.no_delta_min_event_delta {
        cfg.no_delta_min_event_delta = v;
    }

    save_scheduler_config(&conn, &cfg).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"scheduler_config_save_failed","message": format!("{e}")})),
        )
    })?;
    let telemetry = latest_scheduler_counter(&conn).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"scheduler_counter_read_failed","message": format!("{e}")})),
        )
    })?;

    Ok(Json(json!({
        "status": "updated",
        "scheduler": {
            "enabled": cfg.enabled,
            "interval_seconds": cfg.interval_seconds,
            "max_iterations_per_window": cfg.max_iterations_per_window,
            "cooldown_seconds": cfg.cooldown_seconds,
            "low_confidence_threshold": cfg.low_confidence_threshold,
            "no_delta_min_event_delta": cfg.no_delta_min_event_delta,
        },
        "telemetry": {
            "latest_window_key": telemetry.as_ref().map(|t| t.0.clone()),
            "latest_window_run_count": telemetry.as_ref().map(|t| t.1),
            "last_scheduler_run_at": telemetry.as_ref().map(|t| t.2.clone())
        }
    })))
}

async fn reflect_scheduler_tick(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SchedulerTickRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let window = req.window.unwrap_or_else(|| "1h".to_string());
    let inputs = collect_reflection_inputs(&state).await;

    let db_path = focusa_db_path(&state.config.data_dir);
    let conn = Connection::open(db_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"db_open_failed","message": format!("{e}")})),
        )
    })?;

    ensure_reflection_tables(&conn).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code":"db_schema_failed","message": format!("{e}")})),
        )
    })?;

    execute_reflection(
        &conn,
        ReflectRunRequest {
            mode: ReflectMode::Scheduled,
            idempotency_key: None,
            window: Some(window),
            budget: req.budget,
            context: Some(json!({"source": "scheduler_tick"})),
        },
        true,
        &inputs,
    )
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/reflect/run", post(run_reflection))
        .route("/v1/reflect/history", get(reflect_history))
        .route("/v1/reflect/status", get(reflect_status))
        .route(
            "/v1/reflect/scheduler",
            get(reflect_scheduler_get).post(reflect_scheduler_update),
        )
        .route("/v1/reflect/scheduler/tick", post(reflect_scheduler_tick))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{AppState, build_router};
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use focusa_core::runtime::persistence_sqlite::SqlitePersistence;
    use focusa_core::types::{Action, FocusaConfig, FocusaState};
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Instant;
    use tokio::sync::{RwLock, broadcast, mpsc};
    use tower::ServiceExt;
    use uuid::Uuid;

    fn temp_config() -> FocusaConfig {
        let mut cfg = FocusaConfig::default();
        let dir = std::env::temp_dir().join(format!("focusa-reflect-test-{}", Uuid::now_v7()));
        cfg.data_dir = dir.to_string_lossy().to_string();
        cfg
    }

    async fn setup_app() -> axum::Router {
        let cfg = temp_config();
        let persistence = SqlitePersistence::new(&cfg).expect("persistence");
        let (tx, _rx) = mpsc::channel::<Action>(16);
        let (events_tx, _) = broadcast::channel::<String>(16);
        let focusa = Arc::new(RwLock::new(FocusaState::default()));

        let state = Arc::new(AppState {
            focusa,
            command_tx: tx,
            events_tx,
            config: cfg,
            persistence,
            command_store: Arc::new(RwLock::new(HashMap::new())),
            started_at: Instant::now(),
        });

        build_router(state)
    }

    #[tokio::test]
    async fn reflect_run_is_idempotent_by_key_and_window() {
        let app = setup_app().await;
        let rk = format!("rk-{}", Uuid::now_v7());
        let body = serde_json::json!({
            "mode": "manual",
            "idempotency_key": rk,
            "window": "1h",
            "budget": 200,
        })
        .to_string();

        let req1 = Request::builder()
            .method("POST")
            .uri("/v1/reflect/run")
            .header("content-type", "application/json")
            .body(Body::from(body.clone()))
            .expect("req1");
        let resp1 = app.clone().oneshot(req1).await.expect("resp1");
        assert_eq!(resp1.status(), StatusCode::OK);
        let b1 = to_bytes(resp1.into_body(), usize::MAX).await.expect("b1");
        let j1: Value = serde_json::from_slice(&b1).expect("j1");
        assert_eq!(j1.get("duplicate").and_then(|v| v.as_bool()), Some(false));

        let req2 = Request::builder()
            .method("POST")
            .uri("/v1/reflect/run")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .expect("req2");
        let resp2 = app.clone().oneshot(req2).await.expect("resp2");
        assert_eq!(resp2.status(), StatusCode::OK);
        let b2 = to_bytes(resp2.into_body(), usize::MAX).await.expect("b2");
        let j2: Value = serde_json::from_slice(&b2).expect("j2");
        assert_eq!(j2.get("duplicate").and_then(|v| v.as_bool()), Some(true));
    }

    #[tokio::test]
    async fn reflect_repeated_no_delta_sets_repeated_stop_reason() {
        let app = setup_app().await;

        let req1 = Request::builder()
            .method("POST")
            .uri("/v1/reflect/run")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "mode": "manual",
                    "idempotency_key": format!("rk-a-{}", Uuid::now_v7()),
                    "window": "1h",
                    "budget": 200
                })
                .to_string(),
            ))
            .expect("req1");
        let resp1 = app.clone().oneshot(req1).await.expect("resp1");
        assert_eq!(resp1.status(), StatusCode::OK);

        let req2 = Request::builder()
            .method("POST")
            .uri("/v1/reflect/run")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "mode": "manual",
                    "idempotency_key": format!("rk-b-{}", Uuid::now_v7()),
                    "window": "1h",
                    "budget": 200
                })
                .to_string(),
            ))
            .expect("req2");
        let resp2 = app.clone().oneshot(req2).await.expect("resp2");
        assert_eq!(resp2.status(), StatusCode::OK);
        let b2 = to_bytes(resp2.into_body(), usize::MAX).await.expect("b2");
        let j2: Value = serde_json::from_slice(&b2).expect("j2");
        assert_eq!(
            j2.get("result")
                .and_then(|v| v.get("stop_reason"))
                .and_then(|v| v.as_str()),
            Some("repeated_recommendation_set")
        );
    }

    #[tokio::test]
    async fn reflect_history_filters_by_stop_reason() {
        let app = setup_app().await;

        let policy = Request::builder()
            .method("POST")
            .uri("/v1/reflect/scheduler")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"low_confidence_threshold": 0.95}).to_string(),
            ))
            .expect("policy");
        let _ = app.clone().oneshot(policy).await.expect("policy resp");

        let run = Request::builder()
            .method("POST")
            .uri("/v1/reflect/run")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "mode": "manual",
                    "idempotency_key": format!("rk-filter-{}", Uuid::now_v7()),
                    "window": "1h"
                })
                .to_string(),
            ))
            .expect("run");
        let _ = app.clone().oneshot(run).await.expect("run resp");

        let hist = Request::builder()
            .method("GET")
            .uri("/v1/reflect/history?limit=20&stop_reason=low_confidence")
            .body(Body::empty())
            .expect("hist");
        let hist_resp = app.clone().oneshot(hist).await.expect("hist resp");
        assert_eq!(hist_resp.status(), StatusCode::OK);
        let hb = to_bytes(hist_resp.into_body(), usize::MAX)
            .await
            .expect("hb");
        let hj: Value = serde_json::from_slice(&hb).expect("hj");
        let items = hj["items"].as_array().cloned().unwrap_or_default();
        assert!(!items.is_empty());
        assert!(items.iter().all(|it| {
            it.get("result")
                .and_then(|r| r.get("stop_reason"))
                .and_then(|s| s.as_str())
                == Some("low_confidence")
        }));
    }

    #[tokio::test]
    async fn reflect_history_filters_by_mode() {
        let app = setup_app().await;

        let run = Request::builder()
            .method("POST")
            .uri("/v1/reflect/run")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "mode": "manual",
                    "idempotency_key": format!("rk-mode-m-{}", Uuid::now_v7()),
                    "window": "1h"
                })
                .to_string(),
            ))
            .expect("run");
        let _ = app.clone().oneshot(run).await.expect("run resp");

        let enable = Request::builder()
            .method("POST")
            .uri("/v1/reflect/scheduler")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"enabled": true, "cooldown_seconds": 0, "max_iterations_per_window": 10}).to_string(),
            ))
            .expect("enable");
        let _ = app.clone().oneshot(enable).await.expect("enable resp");

        let tick = Request::builder()
            .method("POST")
            .uri("/v1/reflect/scheduler/tick")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .expect("tick");
        let _ = app.clone().oneshot(tick).await.expect("tick resp");

        let hist = Request::builder()
            .method("GET")
            .uri("/v1/reflect/history?limit=20&mode=scheduled")
            .body(Body::empty())
            .expect("hist");
        let hist_resp = app.clone().oneshot(hist).await.expect("hist resp");
        assert_eq!(hist_resp.status(), StatusCode::OK);
        let hb = to_bytes(hist_resp.into_body(), usize::MAX)
            .await
            .expect("hb");
        let hj: Value = serde_json::from_slice(&hb).expect("hj");
        let items = hj["items"].as_array().cloned().unwrap_or_default();
        assert!(!items.is_empty());
        assert!(
            items
                .iter()
                .all(|it| it.get("mode").and_then(|v| v.as_str()) == Some("scheduled"))
        );
        assert_eq!(hj["applied_filters"]["mode"].as_str(), Some("scheduled"));
    }

    #[tokio::test]
    async fn reflect_history_cursor_before_paginates() {
        let app = setup_app().await;

        for i in 0..3 {
            let run = Request::builder()
                .method("POST")
                .uri("/v1/reflect/run")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "mode": "manual",
                        "idempotency_key": format!("rk-cursor-{}-{}", i, Uuid::now_v7()),
                        "window": "1h"
                    })
                    .to_string(),
                ))
                .expect("run");
            let _ = app.clone().oneshot(run).await.expect("run resp");
        }

        let p1 = Request::builder()
            .method("GET")
            .uri("/v1/reflect/history?limit=2")
            .body(Body::empty())
            .expect("p1");
        let p1_resp = app.clone().oneshot(p1).await.expect("p1 resp");
        assert_eq!(p1_resp.status(), StatusCode::OK);
        let b1 = to_bytes(p1_resp.into_body(), usize::MAX).await.expect("b1");
        let j1: Value = serde_json::from_slice(&b1).expect("j1");
        let c1 = j1["next_cursor"].as_str().unwrap_or("").to_string();
        assert!(!c1.is_empty());

        let p2 = Request::builder()
            .method("GET")
            .uri(&format!("/v1/reflect/history?limit=2&cursor_before={}", c1))
            .body(Body::empty())
            .expect("p2");
        let p2_resp = app.clone().oneshot(p2).await.expect("p2 resp");
        assert_eq!(p2_resp.status(), StatusCode::OK);
        let b2 = to_bytes(p2_resp.into_body(), usize::MAX).await.expect("b2");
        let j2: Value = serde_json::from_slice(&b2).expect("j2");
        let items2 = j2["items"].as_array().cloned().unwrap_or_default();
        let applied_cursor = j2["applied_filters"]["cursor_before"]
            .as_str()
            .unwrap_or("")
            .replace(' ', "+");
        assert_eq!(applied_cursor, c1);
        assert!(items2.iter().all(|it| {
            it.get("created_at")
                .and_then(|v| v.as_str())
                .map(|ts| ts < c1.as_str())
                .unwrap_or(false)
        }));
    }

    #[tokio::test]
    async fn reflect_history_limit_is_bounded() {
        let app = setup_app().await;

        let hist = Request::builder()
            .method("GET")
            .uri("/v1/reflect/history?limit=0")
            .body(Body::empty())
            .expect("hist");
        let hist_resp = app.clone().oneshot(hist).await.expect("hist resp");
        assert_eq!(hist_resp.status(), StatusCode::OK);
        let hb = to_bytes(hist_resp.into_body(), usize::MAX)
            .await
            .expect("hb");
        let hj: Value = serde_json::from_slice(&hb).expect("hj");
        assert_eq!(hj["applied_filters"]["limit"].as_u64(), Some(1));

        let hist2 = Request::builder()
            .method("GET")
            .uri("/v1/reflect/history?limit=500")
            .body(Body::empty())
            .expect("hist2");
        let hist2_resp = app.clone().oneshot(hist2).await.expect("hist2 resp");
        assert_eq!(hist2_resp.status(), StatusCode::OK);
        let h2b = to_bytes(hist2_resp.into_body(), usize::MAX)
            .await
            .expect("h2b");
        let h2j: Value = serde_json::from_slice(&h2b).expect("h2j");
        assert_eq!(h2j["applied_filters"]["limit"].as_u64(), Some(200));
    }

    #[tokio::test]
    async fn reflect_history_rejects_invalid_mode_filter() {
        let app = setup_app().await;

        let hist = Request::builder()
            .method("GET")
            .uri("/v1/reflect/history?limit=20&mode=bogus")
            .body(Body::empty())
            .expect("hist");
        let hist_resp = app.clone().oneshot(hist).await.expect("hist resp");
        assert_eq!(hist_resp.status(), StatusCode::BAD_REQUEST);
        let hb = to_bytes(hist_resp.into_body(), usize::MAX)
            .await
            .expect("hb");
        let hj: Value = serde_json::from_slice(&hb).expect("hj");
        assert_eq!(
            hj.get("code").and_then(|v| v.as_str()),
            Some("invalid_mode_filter")
        );
    }

    #[tokio::test]
    async fn reflect_history_filters_by_time_range() {
        let app = setup_app().await;

        let run = Request::builder()
            .method("POST")
            .uri("/v1/reflect/run")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "mode": "manual",
                    "idempotency_key": format!("rk-time-{}", Uuid::now_v7()),
                    "window": "1h"
                })
                .to_string(),
            ))
            .expect("run");
        let _ = app.clone().oneshot(run).await.expect("run resp");

        let hist = Request::builder()
            .method("GET")
            .uri("/v1/reflect/history?limit=20&since=2999-01-01T00:00:00Z")
            .body(Body::empty())
            .expect("hist");
        let hist_resp = app.clone().oneshot(hist).await.expect("hist resp");
        assert_eq!(hist_resp.status(), StatusCode::OK);
        let hb = to_bytes(hist_resp.into_body(), usize::MAX)
            .await
            .expect("hb");
        let hj: Value = serde_json::from_slice(&hb).expect("hj");
        assert_eq!(hj["count"].as_u64(), Some(0));
    }

    #[tokio::test]
    async fn reflect_history_rejects_invalid_time_filter() {
        let app = setup_app().await;

        let hist = Request::builder()
            .method("GET")
            .uri("/v1/reflect/history?limit=20&since=not-a-time")
            .body(Body::empty())
            .expect("hist");
        let hist_resp = app.clone().oneshot(hist).await.expect("hist resp");
        assert_eq!(hist_resp.status(), StatusCode::BAD_REQUEST);
        let hb = to_bytes(hist_resp.into_body(), usize::MAX)
            .await
            .expect("hb");
        let hj: Value = serde_json::from_slice(&hb).expect("hj");
        assert_eq!(
            hj.get("code").and_then(|v| v.as_str()),
            Some("invalid_time_filter")
        );
    }

    #[tokio::test]
    async fn reflect_low_confidence_threshold_sets_stop_reason() {
        let app = setup_app().await;

        let policy = Request::builder()
            .method("POST")
            .uri("/v1/reflect/scheduler")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"low_confidence_threshold": 0.9}).to_string(),
            ))
            .expect("policy");
        let policy_resp = app.clone().oneshot(policy).await.expect("policy resp");
        assert_eq!(policy_resp.status(), StatusCode::OK);

        let req = Request::builder()
            .method("POST")
            .uri("/v1/reflect/run")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "mode": "manual",
                    "idempotency_key": format!("rk-low-{}", Uuid::now_v7()),
                    "window": "1h"
                })
                .to_string(),
            ))
            .expect("req");
        let resp = app.clone().oneshot(req).await.expect("resp");
        assert_eq!(resp.status(), StatusCode::OK);
        let b = to_bytes(resp.into_body(), usize::MAX).await.expect("b");
        let j: Value = serde_json::from_slice(&b).expect("j");
        assert_eq!(
            j.get("result")
                .and_then(|v| v.get("stop_reason"))
                .and_then(|v| v.as_str()),
            Some("low_confidence")
        );
    }

    #[tokio::test]
    async fn scheduler_tick_respects_enable_and_cooldown() {
        let app = setup_app().await;

        let enable = Request::builder()
            .method("POST")
            .uri("/v1/reflect/scheduler")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"enabled": true, "cooldown_seconds": 3600}).to_string(),
            ))
            .expect("enable");
        let enable_resp = app.clone().oneshot(enable).await.expect("enable resp");
        assert_eq!(enable_resp.status(), StatusCode::OK);

        let tick1 = Request::builder()
            .method("POST")
            .uri("/v1/reflect/scheduler/tick")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .expect("tick1");
        let tick1_resp = app.clone().oneshot(tick1).await.expect("tick1 resp");
        assert_eq!(tick1_resp.status(), StatusCode::OK);

        let tick2 = Request::builder()
            .method("POST")
            .uri("/v1/reflect/scheduler/tick")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .expect("tick2");
        let tick2_resp = app.clone().oneshot(tick2).await.expect("tick2 resp");
        assert_eq!(tick2_resp.status(), StatusCode::OK);
        let b2 = to_bytes(tick2_resp.into_body(), usize::MAX)
            .await
            .expect("b2");
        let j2: Value = serde_json::from_slice(&b2).expect("j2");
        let status = j2.get("status").and_then(|v| v.as_str()).unwrap_or("");
        assert!(status == "skipped" || status == "accepted");
        if status == "accepted" {
            assert_eq!(j2.get("duplicate").and_then(|v| v.as_bool()), Some(true));
        }

        let sched = Request::builder()
            .method("GET")
            .uri("/v1/reflect/scheduler")
            .body(Body::empty())
            .expect("sched");
        let sched_resp = app.clone().oneshot(sched).await.expect("sched resp");
        assert_eq!(sched_resp.status(), StatusCode::OK);
        let sb = to_bytes(sched_resp.into_body(), usize::MAX)
            .await
            .expect("sb");
        let sj: Value = serde_json::from_slice(&sb).expect("sj");
        assert!(sj["telemetry"].is_object());
        assert!(sj["telemetry"]["latest_window_key"].is_string());
        assert!(sj["low_confidence_threshold"].is_number());
        assert!(sj["no_delta_min_event_delta"].is_number());

        let status = Request::builder()
            .method("GET")
            .uri("/v1/reflect/status")
            .body(Body::empty())
            .expect("status");
        let status_resp = app.clone().oneshot(status).await.expect("status resp");
        assert_eq!(status_resp.status(), StatusCode::OK);
        let stb = to_bytes(status_resp.into_body(), usize::MAX)
            .await
            .expect("stb");
        let stj: Value = serde_json::from_slice(&stb).expect("stj");
        assert!(stj["telemetry"]["stop_reason_counts"].is_object());
    }
}
