//! Metacognition API surface for SPEC80.
//!
//! Endpoints:
//! - POST /v1/metacognition/capture
//! - POST /v1/metacognition/retrieve
//! - POST /v1/metacognition/reflect
//! - POST /v1/metacognition/adjust
//! - POST /v1/metacognition/evaluate

use crate::routes::bounded::{
    budgeted_default_limit, budgeted_hard_limit, budgeted_requested_limit,
};
use crate::routes::permissions::{forbid, permission_context};
use crate::routes::predictions::append_prediction_record_scoped;
use crate::scope::ScopeContext;
use crate::server::AppState;
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::prediction::{PredictionOntologyContext, PredictionValue};
use focusa_core::scoped_state::{ScopeRef, WorkstreamKey};
use focusa_core::types::TrajectoryLadderContext;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::Digest;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CaptureRecord {
    capture_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
    kind: String,
    content: String,
    rationale: Option<String>,
    confidence: Option<f64>,
    strategy_class: Option<String>,
    storage_path: String,
    #[serde(default)]
    trajectory: Option<TrajectoryLadderContext>,
    #[serde(default)]
    scope: Option<WorkstreamKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReflectionRecord {
    reflection_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
    turn_range: String,
    failure_classes: Vec<String>,
    hypotheses: Vec<String>,
    strategy_updates: Vec<String>,
    storage_path: String,
    #[serde(default)]
    trajectory: Option<TrajectoryLadderContext>,
    #[serde(default)]
    scope: Option<WorkstreamKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdjustmentRecord {
    adjustment_id: String,
    reflection_id: String,
    selected_updates: Vec<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    storage_path: String,
    #[serde(default)]
    trajectory: Option<TrajectoryLadderContext>,
    #[serde(default)]
    scope: Option<WorkstreamKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EvaluationRecord {
    evaluation_id: String,
    adjustment_id: String,
    observed_metrics: Vec<String>,
    result: String,
    promote_learning: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    storage_path: String,
    #[serde(default)]
    trajectory: Option<TrajectoryLadderContext>,
    #[serde(default)]
    scope: Option<WorkstreamKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CaptureIndexEntry {
    capture_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
    kind: String,
    tags: Vec<String>,
    summary: String,
    confidence: Option<f64>,
    strategy_class: Option<String>,
    #[serde(default)]
    has_rationale: bool,
    storage_path: String,
    #[serde(default)]
    trajectory: Option<TrajectoryLadderContext>,
    #[serde(default)]
    scope: Option<WorkstreamKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetacogEvictionEvent {
    collection: String,
    evicted_count: usize,
    reason: String,
    occurred_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Default)]
pub(crate) struct MetaStore {
    captures: Vec<CaptureRecord>,
    reflections: Vec<ReflectionRecord>,
    adjustments: Vec<AdjustmentRecord>,
    evaluations: Vec<EvaluationRecord>,
    capture_hot_index: Vec<CaptureIndexEntry>,
    eviction_events: Vec<MetacogEvictionEvent>,
}

#[derive(Debug, Clone, Copy)]
struct MetaStoreConfig {
    max_captures: usize,
    max_reflections: usize,
    max_adjustments: usize,
    ttl_minutes: i64,
}

fn metacog_store_config(config: &focusa_core::types::FocusaConfig) -> MetaStoreConfig {
    // Env vars override FocusaConfig values (allows testing and legacy compat).
    let max_captures = std::env::var("FOCUSA_METACOG_MAX_CAPTURES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(config.metacog_max_captures)
        .max(1);
    let max_reflections = std::env::var("FOCUSA_METACOG_MAX_REFLECTIONS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(config.metacog_max_reflections)
        .max(1);
    let max_adjustments = std::env::var("FOCUSA_METACOG_MAX_ADJUSTMENTS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(config.metacog_max_adjustments)
        .max(1);
    let ttl_minutes = std::env::var("FOCUSA_METACOG_TTL_MINUTES")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(config.metacog_ttl_minutes)
        .max(1);

    MetaStoreConfig {
        max_captures,
        max_reflections,
        max_adjustments,
        ttl_minutes,
    }
}

fn retain_recent<T>(
    items: &mut Vec<T>,
    max_len: usize,
    cutoff: chrono::DateTime<chrono::Utc>,
    created_at: impl Fn(&T) -> chrono::DateTime<chrono::Utc>,
) -> usize {
    let before_ttl = items.len();
    items.retain(|item| created_at(item) >= cutoff);
    let ttl_removed = before_ttl.saturating_sub(items.len());
    let cap_removed = if items.len() > max_len {
        let overflow = items.len() - max_len;
        items.drain(0..overflow);
        overflow
    } else {
        0
    };
    ttl_removed + cap_removed
}

fn summarize_content(content: &str) -> String {
    content.chars().take(240).collect()
}

fn with_human_readable(mut response: Value, message: impl Into<String>) -> Value {
    if let Some(object) = response.as_object_mut() {
        object.insert("human_readable".to_string(), json!(message.into()));
    }
    response
}

fn tags_for_capture(capture: &CaptureRecord) -> Vec<String> {
    let mut tags = vec![capture.kind.to_ascii_lowercase()];
    if let Some(strategy) = &capture.strategy_class {
        tags.push(strategy.to_ascii_lowercase());
    }
    if let Some(trajectory) = &capture.trajectory {
        for value in [
            trajectory.trajectory_id.as_deref(),
            trajectory.hlt.as_deref(),
            trajectory.mlg.as_deref(),
            trajectory.stg.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            for word in value
                .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
                .filter(|w| w.len() >= 3)
                .take(4)
            {
                let tag = word.to_ascii_lowercase();
                if !tags.contains(&tag) {
                    tags.push(tag);
                }
            }
        }
    }
    for word in capture
        .content
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .filter(|w| w.len() >= 3)
        .take(8)
    {
        let tag = word.to_ascii_lowercase();
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    tags
}

fn capture_index_entry(capture: &CaptureRecord) -> CaptureIndexEntry {
    CaptureIndexEntry {
        capture_id: capture.capture_id.clone(),
        created_at: capture.created_at,
        kind: capture.kind.clone(),
        tags: tags_for_capture(capture),
        summary: summarize_content(&capture.content),
        confidence: capture.confidence,
        strategy_class: capture.strategy_class.clone(),
        has_rationale: capture.rationale.is_some(),
        storage_path: capture.storage_path.clone(),
        trajectory: capture.trajectory.clone(),
        scope: capture.scope.clone(),
    }
}

fn rebuild_capture_hot_index(
    captures: &[CaptureRecord],
    cfg: MetaStoreConfig,
    now: chrono::DateTime<chrono::Utc>,
) -> Vec<CaptureIndexEntry> {
    let cutoff = now - chrono::Duration::minutes(cfg.ttl_minutes);
    let mut items = captures
        .iter()
        .filter(|capture| capture.created_at >= cutoff)
        .map(capture_index_entry)
        .collect::<Vec<_>>();
    items.sort_by_key(|entry| Reverse(entry.created_at));
    items.truncate(cfg.max_captures);
    items
}

fn record_eviction(
    store: &mut MetaStore,
    collection: &str,
    evicted_count: usize,
    reason: &str,
    now: chrono::DateTime<chrono::Utc>,
) {
    if evicted_count == 0 {
        return;
    }
    store.eviction_events.push(MetacogEvictionEvent {
        collection: collection.to_string(),
        evicted_count,
        reason: reason.to_string(),
        occurred_at: now,
    });
    if store.eviction_events.len() > 100 {
        let overflow = store.eviction_events.len() - 100;
        store.eviction_events.drain(0..overflow);
    }
}

fn prune_metacog_store(
    store: &mut MetaStore,
    now: chrono::DateTime<chrono::Utc>,
    cfg: MetaStoreConfig,
) {
    let cutoff = now - chrono::Duration::minutes(cfg.ttl_minutes);
    let capture_evicted = retain_recent(&mut store.captures, cfg.max_captures, cutoff, |r| {
        r.created_at
    });
    let reflection_evicted =
        retain_recent(&mut store.reflections, cfg.max_reflections, cutoff, |r| {
            r.created_at
        });
    let adjustment_evicted =
        retain_recent(&mut store.adjustments, cfg.max_adjustments, cutoff, |r| {
            r.created_at
        });
    let evaluation_evicted =
        retain_recent(&mut store.evaluations, cfg.max_adjustments, cutoff, |r| {
            r.created_at
        });
    store.capture_hot_index = rebuild_capture_hot_index(&store.captures, cfg, now);
    record_eviction(store, "captures", capture_evicted, "ttl_or_cap", now);
    record_eviction(store, "reflections", reflection_evicted, "ttl_or_cap", now);
    record_eviction(store, "adjustments", adjustment_evicted, "ttl_or_cap", now);
    record_eviction(store, "evaluations", evaluation_evicted, "ttl_or_cap", now);
}

fn metacog_base_dir(state: &AppState, scope: &WorkstreamKey) -> PathBuf {
    Path::new(&state.config.data_dir)
        .join("runtime")
        .join("scoped-metacog")
        .join(scope.storage_key())
}

fn metacog_category_dir(state: &AppState, scope: &WorkstreamKey, category: &str) -> PathBuf {
    metacog_base_dir(state, scope).join(category)
}

fn metacog_record_path(
    state: &AppState,
    scope: &WorkstreamKey,
    category: &str,
    id: &str,
) -> PathBuf {
    metacog_category_dir(state, scope, category).join(format!("{id}.json"))
}

fn metacog_capture_index_path(state: &AppState, scope: &WorkstreamKey) -> PathBuf {
    metacog_base_dir(state, scope).join("capture-hot-index.jsonl")
}

fn persist_json_record(path: &Path, payload: &Value) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(bytes) = serde_json::to_vec_pretty(payload) {
        let _ = fs::write(path, bytes);
    }
}

fn append_capture_index_entry(state: &AppState, scope: &WorkstreamKey, entry: &CaptureIndexEntry) {
    let path = metacog_capture_index_path(state, scope);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(line) = serde_json::to_string(entry) {
        use std::io::Write;
        if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{line}");
        }
    }
}

fn load_capture_index_from_disk(
    state: &AppState,
    scope: &WorkstreamKey,
    cfg: MetaStoreConfig,
    now: chrono::DateTime<chrono::Utc>,
) -> Vec<CaptureIndexEntry> {
    let path = metacog_capture_index_path(state, scope);
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let cutoff = now - chrono::Duration::minutes(cfg.ttl_minutes);
    let mut by_id = HashMap::new();
    for line in text.lines() {
        let Ok(entry) = serde_json::from_str::<CaptureIndexEntry>(line) else {
            continue;
        };
        if entry.created_at >= cutoff {
            by_id.insert(entry.capture_id.clone(), entry);
        }
    }
    let mut entries = by_id.into_values().collect::<Vec<_>>();
    entries.sort_by_key(|entry| Reverse(entry.created_at));
    entries.truncate(cfg.max_captures);
    entries
}

fn load_capture_record_from_path(path: &str) -> Option<CaptureRecord> {
    let bytes = fs::read(path).ok()?;
    serde_json::from_slice::<CaptureRecord>(&bytes).ok()
}

fn load_capture_records_from_disk(state: &AppState, scope: &WorkstreamKey) -> Vec<CaptureRecord> {
    let mut out = Vec::new();
    let dir = metacog_category_dir(state, scope, "captures");
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };

    for entry in entries.flatten() {
        let Ok(bytes) = fs::read(entry.path()) else {
            continue;
        };
        let Ok(rec) = serde_json::from_slice::<CaptureRecord>(&bytes) else {
            continue;
        };
        out.push(rec);
    }

    out.sort_by_key(|r| r.created_at);
    out
}

fn load_reflection_records_from_disk(
    state: &AppState,
    scope: &WorkstreamKey,
) -> Vec<ReflectionRecord> {
    let mut out = Vec::new();
    let dir = metacog_category_dir(state, scope, "reflections");
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };

    for entry in entries.flatten() {
        let Ok(bytes) = fs::read(entry.path()) else {
            continue;
        };
        let Ok(rec) = serde_json::from_slice::<ReflectionRecord>(&bytes) else {
            continue;
        };
        out.push(rec);
    }

    out.sort_by_key(|r| r.created_at);
    out
}

fn load_adjustment_records_from_disk(
    state: &AppState,
    scope: &WorkstreamKey,
) -> Vec<AdjustmentRecord> {
    let mut out = Vec::new();
    let dir = metacog_category_dir(state, scope, "adjustments");
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };

    for entry in entries.flatten() {
        let Ok(bytes) = fs::read(entry.path()) else {
            continue;
        };
        let Ok(rec) = serde_json::from_slice::<AdjustmentRecord>(&bytes) else {
            continue;
        };
        out.push(rec);
    }

    out.sort_by_key(|r| r.created_at);
    out
}

fn reflection_exists_on_disk(state: &AppState, scope: &WorkstreamKey, reflection_id: &str) -> bool {
    metacog_record_path(state, scope, "reflections", reflection_id).exists()
}

fn load_reflection_record(
    state: &AppState,
    scope: &WorkstreamKey,
    reflection_id: &str,
) -> Option<ReflectionRecord> {
    let in_mem = state
        .metacog_by_scope
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(scope)
        .and_then(|store| {
            store
                .reflections
                .iter()
                .find(|rec| rec.reflection_id == reflection_id)
                .cloned()
        });
    in_mem.or_else(|| {
        let path = metacog_record_path(state, scope, "reflections", reflection_id);
        fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<ReflectionRecord>(&bytes).ok())
    })
}

fn load_evaluation_records_from_disk(
    state: &AppState,
    scope: &WorkstreamKey,
) -> Vec<EvaluationRecord> {
    let mut out = Vec::new();
    let dir = metacog_category_dir(state, scope, "evaluations");
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };

    for entry in entries.flatten() {
        let Ok(bytes) = fs::read(entry.path()) else {
            continue;
        };
        let Ok(rec) = serde_json::from_slice::<EvaluationRecord>(&bytes) else {
            continue;
        };
        out.push(rec);
    }

    out.sort_by_key(|r| r.created_at);
    out
}

fn load_adjustment_record(
    state: &AppState,
    scope: &WorkstreamKey,
    adjustment_id: &str,
) -> Option<AdjustmentRecord> {
    let in_mem = state
        .metacog_by_scope
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(scope)
        .and_then(|store| {
            store
                .adjustments
                .iter()
                .find(|rec| rec.adjustment_id == adjustment_id)
                .cloned()
        });
    in_mem.or_else(|| {
        let path = metacog_record_path(state, scope, "adjustments", adjustment_id);
        fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<AdjustmentRecord>(&bytes).ok())
    })
}

fn promotion_score(observed_metrics: &[String], selected_updates: &[String]) -> f64 {
    let metric_score = (observed_metrics.len() as f64 * 0.25).min(0.75);
    let update_score = if selected_updates.is_empty() {
        0.0
    } else {
        0.25
    };
    (metric_score + update_score).clamp(0.0, 1.0)
}

fn token_enabled(state: &AppState) -> bool {
    state.config.auth_token.is_some() || std::env::var("FOCUSA_AUTH_TOKEN").is_ok()
}

fn require_scope(
    headers: &HeaderMap,
    state: &AppState,
    scope: &str,
) -> Result<(), (StatusCode, Json<Value>)> {
    let permissions = permission_context(headers, token_enabled(state));
    if permissions.allows(scope) {
        Ok(())
    } else {
        Err(forbid(scope))
    }
}

async fn active_trajectory_context(state: &AppState) -> Option<TrajectoryLadderContext> {
    state.focusa.read().await.trajectory_ladder_context()
}

fn scope_required_response(reason: String) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "status": "error",
            "code": "SCOPE_REQUIRED",
            "reason": reason,
            "human_readable": "Metacognition requires a verified project and continuity scope. Next: resume or checkpoint the current Workpoint."
        })),
    )
}

fn workstream_from_trajectory(trajectory: &TrajectoryLadderContext) -> Option<WorkstreamKey> {
    let root = trajectory.project_root.as_deref()?;
    let continuity_id = trajectory.continuity_id.as_deref()?;
    let canonical_name = root
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("project");
    let fingerprint = format!(
        "sha256:{}",
        hex::encode(sha2::Sha256::digest(root.trim_end_matches('/').as_bytes()))
    );
    let root_scope = ScopeRef::project(
        format!("project:{fingerprint}"),
        root,
        canonical_name,
        fingerprint,
    )
    .ok()?;
    WorkstreamKey::new(root_scope, continuity_id).ok()
}

pub(crate) async fn capture_learning_signal(
    state: &AppState,
    kind: &str,
    content: &str,
    rationale: Option<String>,
    confidence: Option<f64>,
    strategy_class: Option<String>,
) -> Option<String> {
    if kind.trim().is_empty() || content.trim().is_empty() {
        return None;
    }
    let trajectory = active_trajectory_context(state).await;
    let scope = trajectory.as_ref().and_then(workstream_from_trajectory)?;
    capture_learning_signal_scoped(
        state,
        scope,
        kind,
        content,
        rationale,
        confidence,
        strategy_class,
    )
    .await
}

pub(crate) async fn capture_learning_signal_scoped(
    state: &AppState,
    scope: WorkstreamKey,
    kind: &str,
    content: &str,
    rationale: Option<String>,
    confidence: Option<f64>,
    strategy_class: Option<String>,
) -> Option<String> {
    if scope.validate().is_err() || kind.trim().is_empty() || content.trim().is_empty() {
        return None;
    }
    let capture_id = format!(
        "scoped-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let storage_path = metacog_record_path(state, &scope, "captures", &capture_id);
    let trajectory = active_trajectory_context(state).await.filter(|trajectory| {
        trajectory.project_root.as_deref()
            == Some(scope.root_scope.root_path.to_string_lossy().as_ref())
            && trajectory.continuity_id.as_deref() == Some(scope.continuity_id.as_str())
    });
    let rec = CaptureRecord {
        capture_id: capture_id.clone(),
        created_at: Utc::now(),
        kind: kind.to_string(),
        content: content.to_string(),
        rationale,
        confidence,
        strategy_class,
        storage_path: storage_path.display().to_string(),
        trajectory,
        scope: Some(scope.clone()),
    };
    persist_json_record(&storage_path, &json!(rec));
    let index_entry = capture_index_entry(&rec);
    append_capture_index_entry(state, &scope, &index_entry);
    let mut stores = state
        .metacog_by_scope
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let store = stores.entry(scope).or_default();
    store.captures.push(rec);
    store.capture_hot_index.push(index_entry);
    prune_metacog_store(store, Utc::now(), metacog_store_config(&state.config));
    Some(capture_id)
}

#[derive(Debug, Deserialize)]
struct CaptureBody {
    kind: String,
    content: String,
    #[serde(default)]
    rationale: Option<String>,
    #[serde(default)]
    confidence: Option<f64>,
    #[serde(default)]
    strategy_class: Option<String>,
}

async fn capture(
    State(state): State<Arc<AppState>>,
    scope_context: ScopeContext,
    headers: HeaderMap,
    Json(body): Json<CaptureBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "metacognition:write")?;
    let scope = scope_context
        .require_workstream_key()
        .map_err(scope_required_response)?;

    if body.kind.trim().is_empty() || body.content.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "code": "CAPTURE_SCHEMA_INVALID",
                "reason": "kind and content are required",
                "human_readable": "Metacognition capture was rejected because kind and content are required. Next: provide both bounded fields."
            })),
        ));
    }

    let capture_id = format!(
        "cap-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let storage_path = metacog_record_path(&state, &scope, "captures", &capture_id)
        .display()
        .to_string();
    let trajectory = active_trajectory_context(&state).await;
    let rec = CaptureRecord {
        capture_id: capture_id.clone(),
        created_at: Utc::now(),
        kind: body.kind,
        content: body.content,
        rationale: body.rationale,
        confidence: body.confidence,
        strategy_class: body.strategy_class,
        storage_path: storage_path.clone(),
        trajectory: trajectory.clone(),
        scope: Some(scope.clone()),
    };

    persist_json_record(
        &metacog_record_path(&state, &scope, "captures", &capture_id),
        &json!(rec),
    );

    let index_entry = capture_index_entry(&rec);
    append_capture_index_entry(&state, &scope, &index_entry);

    let mut stores = state
        .metacog_by_scope
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let s = stores.entry(scope).or_default();
    s.captures.push(rec);
    s.capture_hot_index.push(index_entry);
    prune_metacog_store(s, Utc::now(), metacog_store_config(&state.config));

    Ok(Json(with_human_readable(
        json!({
            "capture_id": capture_id,
            "stored": true,
            "linked_turn_id": Value::Null,
            "storage_path": storage_path,
            "trajectory": trajectory,
        }),
        "Metacognition capture stored. Next: retrieve it when a related ask needs the lesson.",
    )))
}

#[derive(Debug, Deserialize)]
struct RetrieveBody {
    current_ask: String,
    #[serde(default)]
    scope_tags: Vec<String>,
    #[serde(default = "default_k")]
    k: usize,
    #[serde(default)]
    cursor: Option<String>,
    #[serde(default = "default_summary_only")]
    summary_only: bool,
}

fn default_k() -> usize {
    budgeted_default_limit("FOCUSA_METACOG_RETRIEVE_DEFAULT_K", 5)
}

fn default_summary_only() -> bool {
    true
}

fn retrieve_max_k() -> usize {
    // The hard cap must be allowed below the default; callers clamp the
    // default/request to this value. Passing default_k() as a floor made
    // FOCUSA_METACOG_RETRIEVE_MAX_K=2 silently become 5.
    budgeted_hard_limit("FOCUSA_METACOG_RETRIEVE_MAX_K", 20, 1)
}

fn recent_artifacts_default_limit() -> usize {
    budgeted_default_limit("FOCUSA_METACOG_RECENT_DEFAULT_LIMIT", 5)
}

fn recent_artifacts_hard_limit() -> usize {
    budgeted_hard_limit(
        "FOCUSA_METACOG_RECENT_HARD_LIMIT",
        20,
        recent_artifacts_default_limit(),
    )
}

async fn retrieve(
    State(state): State<Arc<AppState>>,
    scope_context: ScopeContext,
    headers: HeaderMap,
    Json(body): Json<RetrieveBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "metacognition:read")?;
    let scope = scope_context
        .require_workstream_key()
        .map_err(scope_required_response)?;

    let ask = body.current_ask.to_lowercase();
    let tags = body
        .scope_tags
        .iter()
        .map(|t| t.to_lowercase())
        .collect::<Vec<_>>();

    let cfg = metacog_store_config(&state.config);
    let now = Utc::now();
    let (in_memory_records, in_memory_index) = {
        let mut stores = state
            .metacog_by_scope
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let s = stores.entry(scope.clone()).or_default();
        if s.capture_hot_index.is_empty() && !s.captures.is_empty() {
            s.capture_hot_index = rebuild_capture_hot_index(&s.captures, cfg, now);
        }
        let records = if body.summary_only {
            Vec::new()
        } else {
            s.captures.clone()
        };
        (records, s.capture_hot_index.clone())
    };

    let mut by_id = HashMap::new();
    let mut index_by_id = HashMap::new();
    for entry in load_capture_index_from_disk(&state, &scope, cfg, now) {
        index_by_id.insert(entry.capture_id.clone(), entry);
    }
    for entry in in_memory_index {
        index_by_id.insert(entry.capture_id.clone(), entry);
    }
    for capture in in_memory_records {
        by_id.insert(capture.capture_id.clone(), capture);
    }
    let mut hot_index = index_by_id.into_values().collect::<Vec<_>>();
    hot_index.sort_by_key(|entry| Reverse(entry.created_at));
    hot_index.truncate(cfg.max_captures);

    let mut ranked = hot_index
        .iter()
        .map(|entry| {
            let haystack = format!(
                "{} {} {}",
                entry.summary.to_ascii_lowercase(),
                entry.kind.to_ascii_lowercase(),
                entry.tags.join(" ")
            );
            let mut score = 0_i64;
            if !ask.is_empty() && haystack.contains(&ask) {
                score += 2;
            }
            for tag in &tags {
                if haystack.contains(tag) {
                    score += 1;
                }
            }
            (score, entry)
        })
        .collect::<Vec<_>>();

    ranked.sort_by_key(|(score, entry)| (Reverse(*score), Reverse(entry.created_at)));

    let cursor_offset = body
        .cursor
        .as_deref()
        .and_then(|c| c.parse::<usize>().ok())
        .unwrap_or(0);
    let hard_limit = retrieve_max_k();
    let page_size = budgeted_requested_limit(Some(body.k), default_k().min(hard_limit), hard_limit);

    let total = ranked.len();
    let page = ranked
        .into_iter()
        .skip(cursor_offset)
        .take(page_size)
        .collect::<Vec<_>>();

    let candidates = page
        .iter()
        .enumerate()
        .map(|(idx, (score, entry))| {
            let full_record = if body.summary_only {
                None
            } else {
                by_id.get(&entry.capture_id).cloned().or_else(|| {
                    load_capture_record_from_path(&entry.storage_path)
                })
            };
            let summary = if body.summary_only {
                entry.summary.clone()
            } else {
                full_record
                    .as_ref()
                    .map(|record| record.content.clone())
                    .unwrap_or_else(|| entry.summary.clone())
            };

            json!({
                "capture_id": entry.capture_id,
                "kind": entry.kind,
                "summary": summary,
                "summary_only": body.summary_only,
                "score": score,
                "rank": cursor_offset + idx + 1,
                "confidence": entry.confidence,
                "has_rationale": full_record.as_ref().map(|record| record.rationale.is_some()).unwrap_or(entry.has_rationale),
                "strategy_class": entry.strategy_class,
                "tags": entry.tags,
                "storage_path": entry.storage_path,
                "rehydrate": {"route": "/v1/metacognition/captures", "capture_id": entry.capture_id},
                "evidence_refs": []
            })
        })
        .collect::<Vec<_>>();

    let next_cursor = if cursor_offset + page_size < total {
        Some((cursor_offset + page_size).to_string())
    } else {
        None
    };

    Ok(Json(with_human_readable(
        json!({
            "candidates": candidates,
            "next_cursor": next_cursor,
            "page_size": page_size,
            "total_candidates": total,
            "ranked_by": "hot_index_keyword_similarity",
            "index": {
                "kind": "capture_hot_index",
                "summary_only_default": true,
                "indexed_items": hot_index.len(),
                "cap": cfg.max_captures,
                "ttl_minutes": cfg.ttl_minutes
            },
            "retrieval_budget": {
                "tokens_used": 0,
                "latency_ms": 0,
                "truncated": next_cursor.is_some()
            }
        }),
        format!(
            "Retrieved {} metacognition candidate(s). Next: inspect the top lesson or reflect if evidence is weak.",
            page_size
        ),
    )))
}

#[derive(Debug, Deserialize)]
struct ReflectBody {
    turn_range: String,
    #[serde(default)]
    failure_classes: Vec<String>,
}

async fn reflect(
    State(state): State<Arc<AppState>>,
    scope_context: ScopeContext,
    headers: HeaderMap,
    Json(body): Json<ReflectBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "metacognition:write")?;
    let scope = scope_context
        .require_workstream_key()
        .map_err(scope_required_response)?;

    if body.turn_range.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "code": "REFLECT_INPUT_INVALID",
                "reason": "turn_range is required",
                "human_readable": "Metacognition reflection was rejected because turn_range is required. Next: provide a bounded turn range."
            })),
        ));
    }

    let reflection_id = format!(
        "refl-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let strategy_updates = if body.failure_classes.is_empty() {
        vec!["increase verification checkpoints".to_string()]
    } else {
        body.failure_classes
            .iter()
            .map(|f| format!("mitigate {f}"))
            .collect::<Vec<_>>()
    };

    let storage_path = metacog_record_path(&state, &scope, "reflections", &reflection_id)
        .display()
        .to_string();
    let trajectory = active_trajectory_context(&state).await;
    let rec = ReflectionRecord {
        reflection_id: reflection_id.clone(),
        created_at: Utc::now(),
        turn_range: body.turn_range,
        failure_classes: body.failure_classes,
        hypotheses: vec!["strategy mismatch in recent turns".into()],
        strategy_updates: strategy_updates.clone(),
        storage_path: storage_path.clone(),
        trajectory: trajectory.clone(),
        scope: Some(scope.clone()),
    };

    persist_json_record(
        &metacog_record_path(&state, &scope, "reflections", &reflection_id),
        &json!(rec),
    );

    let mut stores = state
        .metacog_by_scope
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let s = stores.entry(scope).or_default();
    s.reflections.push(rec.clone());
    prune_metacog_store(s, Utc::now(), metacog_store_config(&state.config));

    Ok(Json(with_human_readable(
        json!({
            "reflection_id": reflection_id,
            "hypotheses": rec.hypotheses,
            "strategy_updates": strategy_updates,
            "storage_path": storage_path,
            "trajectory": trajectory,
        }),
        "Metacognition reflection created. Next: select bounded strategy updates for an adjustment.",
    )))
}

#[derive(Debug, Deserialize)]
struct AdjustBody {
    reflection_id: String,
    #[serde(default)]
    selected_updates: Vec<String>,
}

async fn adjust(
    State(state): State<Arc<AppState>>,
    scope_context: ScopeContext,
    headers: HeaderMap,
    Json(body): Json<AdjustBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "metacognition:write")?;
    let scope = scope_context
        .require_workstream_key()
        .map_err(scope_required_response)?;

    let in_mem_exists = state
        .metacog_by_scope
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&scope)
        .is_some_and(|s| {
            s.reflections
                .iter()
                .any(|r| r.reflection_id == body.reflection_id)
        });
    if !in_mem_exists && !reflection_exists_on_disk(&state, &scope, &body.reflection_id) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "code": "REFLECTION_NOT_FOUND",
                "reason": "reflection_id does not exist",
                "human_readable": "The requested metacognition reflection was not found in this scope. Next: list recent reflections and retry with a valid id."
            })),
        ));
    }

    let adjustment_id = format!(
        "adj-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let storage_path = metacog_record_path(&state, &scope, "adjustments", &adjustment_id)
        .display()
        .to_string();
    let trajectory = active_trajectory_context(&state).await;
    let rec = AdjustmentRecord {
        adjustment_id: adjustment_id.clone(),
        reflection_id: body.reflection_id,
        selected_updates: body.selected_updates.clone(),
        created_at: Utc::now(),
        storage_path: storage_path.clone(),
        trajectory: trajectory.clone(),
        scope: Some(scope.clone()),
    };
    persist_json_record(
        &metacog_record_path(&state, &scope, "adjustments", &adjustment_id),
        &json!(rec),
    );
    let mut stores = state
        .metacog_by_scope
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let s = stores.entry(scope).or_default();
    s.adjustments.push(rec.clone());
    prune_metacog_store(s, Utc::now(), metacog_store_config(&state.config));

    Ok(Json(with_human_readable(
        json!({
            "adjustment_id": adjustment_id,
            "next_step_policy": rec.selected_updates,
            "expected_deltas": {
                "failed_turn_ratio": -0.1,
                "rework_loop_rate": -0.1,
            },
            "storage_path": storage_path,
            "trajectory": trajectory,
        }),
        "Metacognition adjustment created. Next: apply it and evaluate observed outcome metrics.",
    )))
}

#[derive(Debug, Deserialize)]
struct EvaluateBody {
    adjustment_id: String,
    #[serde(default)]
    observed_metrics: Vec<String>,
}

async fn evaluate(
    State(state): State<Arc<AppState>>,
    scope_context: ScopeContext,
    headers: HeaderMap,
    Json(body): Json<EvaluateBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "metacognition:write")?;
    let scope = scope_context
        .require_workstream_key()
        .map_err(scope_required_response)?;

    let adjustment = load_adjustment_record(&state, &scope, &body.adjustment_id);
    let Some(adjustment) = adjustment else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "code": "ADJUSTMENT_NOT_FOUND",
                "reason": "adjustment_id does not exist",
                "human_readable": "The requested metacognition adjustment was not found in this scope. Next: list recent adjustments and retry with a valid id."
            })),
        ));
    };
    let reflection = load_reflection_record(&state, &scope, &adjustment.reflection_id);
    let score = promotion_score(&body.observed_metrics, &adjustment.selected_updates);
    let promote = score >= 0.5;
    let now = Utc::now();
    let evaluation_id = format!("eval-{}", now.timestamp_nanos_opt().unwrap_or_default());
    let storage_path = metacog_record_path(&state, &scope, "evaluations", &evaluation_id)
        .display()
        .to_string();
    let trajectory = active_trajectory_context(&state).await;
    let rec = EvaluationRecord {
        evaluation_id: evaluation_id.clone(),
        adjustment_id: body.adjustment_id,
        observed_metrics: body.observed_metrics,
        result: if promote { "improved" } else { "inconclusive" }.to_string(),
        promote_learning: promote,
        created_at: now,
        storage_path: storage_path.clone(),
        trajectory: trajectory.clone(),
        scope: Some(scope.clone()),
    };
    persist_json_record(
        &metacog_record_path(&state, &scope, "evaluations", &evaluation_id),
        &json!(rec),
    );

    let promoted_capture = if rec.promote_learning {
        let capture_id = format!("promoted-{}", now.timestamp_nanos_opt().unwrap_or_default());
        let capture_storage_path = metacog_record_path(&state, &scope, "captures", &capture_id)
            .display()
            .to_string();
        let capture = CaptureRecord {
            capture_id: capture_id.clone(),
            created_at: now,
            kind: "promoted_learning".to_string(),
            content: format!(
                "Promoted adjustment {}. Selected updates: {}. Observed metrics: {}. Reflection hypotheses: {}",
                rec.adjustment_id,
                adjustment.selected_updates.join("; "),
                rec.observed_metrics.join(", "),
                reflection
                    .as_ref()
                    .map(|r| r.hypotheses.join("; "))
                    .unwrap_or_default()
            ),
            rationale: Some(format!(
                "Evaluation {} marked result={} with promotion_score={:.2}; promoted learning is now retrievable.",
                rec.evaluation_id, rec.result, score
            )),
            confidence: Some(score),
            strategy_class: Some("metacognition_evaluation".to_string()),
            storage_path: capture_storage_path,
            trajectory: trajectory.clone(),
            scope: adjustment.scope.clone(),
        };
        persist_json_record(
            &metacog_record_path(&state, &scope, "captures", &capture_id),
            &json!(capture),
        );
        let index_entry = capture_index_entry(&capture);
        append_capture_index_entry(&state, &scope, &index_entry);
        Some((capture, index_entry))
    } else {
        None
    };

    {
        let mut stores = state
            .metacog_by_scope
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let s = stores.entry(scope.clone()).or_default();
        s.evaluations.push(rec.clone());
        if let Some((capture, index_entry)) = promoted_capture.clone() {
            s.captures.push(capture);
            s.capture_hot_index.push(index_entry);
        }
        prune_metacog_store(s, now, metacog_store_config(&state.config));
    }

    let promoted_capture_id = promoted_capture
        .as_ref()
        .map(|(capture, _)| capture.capture_id.clone());

    let follow_up_prediction = if rec.promote_learning {
        if let Some(scope) = rec.scope.clone() {
            append_prediction_record_scoped(
                &state,
                scope,
                PredictionValue {
                    prediction_type: "metacog_learning_transfer".into(),
                    context_refs: vec![rec.evaluation_id.clone(), rec.adjustment_id.clone()],
                    ontology_context: PredictionOntologyContext {
                        object_refs: vec!["MetacognitionEvaluation".into(), "PredictionMetacogFlywheel".into()],
                        action_refs: vec!["evaluate_outcome".into(), "promote_learning".into(), "record_next_prediction".into()],
                        tool_refs: vec!["focusa_metacog_evaluate_outcome".into(), "focusa_predict_record".into()],
                        evidence_refs: vec![storage_path.clone()],
                        relation_refs: vec!["evaluation_promotes_capture".into(), "capture_informs_prediction".into()],
                    },
                    predicted_outcome: "promoted learning improves the next similar action".into(),
                    confidence: score,
                    recommended_action: "retrieve promoted metacognition before the next similar decision and record the next prediction".into(),
                    why: "A scoped metacognition evaluation promoted a bounded learning signal.".into(),
                    trajectory: trajectory.clone(),
                    actual_outcome: None,
                    evaluated_at: None,
                    score: None,
                    learning_signal_ref: promoted_capture_id.clone(),
                    outcome_capture: None,
                },
            )
            .await
            .ok()
            .map(|record| json!({"record_id": record.record_id, "scope": record.scope}))
        } else {
            None
        }
    } else {
        None
    };

    Ok(Json(with_human_readable(
        json!({
            "evaluation_id": evaluation_id,
            "adjustment_id": rec.adjustment_id,
            "delta_scorecard": {
                "metrics_observed": rec.observed_metrics,
                "selected_updates": adjustment.selected_updates,
                "promotion_score": score,
                "threshold": 0.5,
            },
            "result": rec.result,
            "promote_learning": rec.promote_learning,
            "promoted_capture_id": promoted_capture_id,
            "follow_up_prediction": follow_up_prediction,
            "flywheel": {"metacog_to_prediction": follow_up_prediction.is_some(), "next_tools": ["focusa_predict_recent", "focusa_predict_evaluate", "focusa_metacog_retrieve"]},
            "storage_path": storage_path,
            "trajectory": trajectory,
            "next_step_hint": if rec.promote_learning { "promoted learning was written back into metacognition retrieval memory and a follow-up prediction was recorded" } else { "collect observed_metrics before promoting this learning signal" }
        }),
        if rec.promote_learning {
            "Metacognition evaluation passed and learning was promoted. Next: evaluate the follow-up prediction."
        } else {
            "Metacognition evaluation completed without promotion. Next: collect stronger observed metrics."
        },
    )))
}

#[derive(Debug, Deserialize)]
struct RecentMetacogQuery {
    limit: Option<usize>,
    cursor: Option<usize>,
}

async fn metacog_status(
    State(state): State<Arc<AppState>>,
    scope_context: ScopeContext,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "metacognition:read")?;
    let scope = scope_context
        .require_workstream_key()
        .map_err(scope_required_response)?;
    let cfg = metacog_store_config(&state.config);
    let disk_captures = load_capture_records_from_disk(&state, &scope);
    let disk_evaluations = load_evaluation_records_from_disk(&state, &scope);
    let mut stores = state
        .metacog_by_scope
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let s = stores.entry(scope).or_default();
    let mut by_id: HashMap<String, CaptureRecord> = HashMap::new();
    for rec in disk_captures {
        by_id.insert(rec.capture_id.clone(), rec);
    }
    for rec in &s.captures {
        by_id.insert(rec.capture_id.clone(), rec.clone());
    }
    let mut evaluations_by_id: HashMap<String, EvaluationRecord> = HashMap::new();
    for rec in disk_evaluations {
        evaluations_by_id.insert(rec.evaluation_id.clone(), rec);
    }
    for rec in &s.evaluations {
        evaluations_by_id.insert(rec.evaluation_id.clone(), rec.clone());
    }
    s.capture_hot_index = rebuild_capture_hot_index(
        &by_id.values().cloned().collect::<Vec<_>>(),
        cfg,
        Utc::now(),
    );
    let promoted_evaluations = evaluations_by_id
        .values()
        .filter(|rec| rec.promote_learning)
        .count();
    Ok(Json(with_human_readable(
        json!({
            "status": "ok",
            "caps": {
                "max_captures": cfg.max_captures,
                "max_reflections": cfg.max_reflections,
                "max_adjustments": cfg.max_adjustments,
                "max_evaluations": cfg.max_adjustments,
                "ttl_minutes": cfg.ttl_minutes,
                "retrieve_max_k": retrieve_max_k()
            },
            "hot_index": {
                "captures_indexed": s.capture_hot_index.len(),
                "summary_chars": 240,
                "full_content_rehydrate_route": "/v1/metacognition/captures/{capture_id}"
            },
            "evaluation_memory": {
                "evaluations_recorded": evaluations_by_id.len(),
                "promoted_evaluations": promoted_evaluations,
                "storage_category": "evaluations",
                "persistence": "json_record"
            },
            "eviction_telemetry": s.eviction_events.iter().rev().take(10).cloned().collect::<Vec<_>>(),
        }),
        "Metacognition store is healthy. Next: retrieve lessons for the current ask or capture new evidence-backed learning.",
    )))
}

async fn get_capture(
    State(state): State<Arc<AppState>>,
    scope_context: ScopeContext,
    headers: HeaderMap,
    AxumPath(capture_id): AxumPath<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "metacognition:read")?;
    let scope = scope_context
        .require_workstream_key()
        .map_err(scope_required_response)?;
    let in_mem = state
        .metacog_by_scope
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&scope)
        .and_then(|s| {
            s.captures
                .iter()
                .find(|rec| rec.capture_id == capture_id)
                .cloned()
        });
    let rec = in_mem.or_else(|| {
        let path = metacog_record_path(&state, &scope, "captures", &capture_id);
        fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<CaptureRecord>(&bytes).ok())
    });
    let Some(rec) = rec else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "code": "CAPTURE_NOT_FOUND",
                "reason": "capture_id does not exist",
                "human_readable": "The requested metacognition capture was not found in this scope. Next: retrieve recent candidates and retry with a valid id."
            })),
        ));
    };
    Ok(Json(with_human_readable(
        json!({
            "status": "ok",
            "capture": rec,
        }),
        "Metacognition capture loaded. Next: apply the lesson only when it matches the current evidence and scope.",
    )))
}

async fn recent_reflections(
    State(state): State<Arc<AppState>>,
    scope_context: ScopeContext,
    headers: HeaderMap,
    Query(query): Query<RecentMetacogQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "metacognition:read")?;
    let scope = scope_context
        .require_workstream_key()
        .map_err(scope_required_response)?;

    let mut by_id: HashMap<String, ReflectionRecord> = HashMap::new();
    if let Some(s) = state
        .metacog_by_scope
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&scope)
    {
        for rec in &s.reflections {
            by_id.insert(rec.reflection_id.clone(), rec.clone());
        }
    }
    for rec in load_reflection_records_from_disk(&state, &scope) {
        by_id.entry(rec.reflection_id.clone()).or_insert(rec);
    }

    let limit = budgeted_requested_limit(
        query.limit,
        recent_artifacts_default_limit(),
        recent_artifacts_hard_limit(),
    );
    let total = by_id.len();
    let cursor = query.cursor.unwrap_or(0).min(total);
    let mut items = by_id.into_values().collect::<Vec<_>>();
    items.sort_by_key(|r| std::cmp::Reverse(r.created_at));
    let window = items
        .into_iter()
        .skip(cursor)
        .take(limit)
        .collect::<Vec<_>>();
    let next_cursor = (cursor + window.len() < total).then(|| (cursor + window.len()).to_string());

    Ok(Json(with_human_readable(
        json!({
            "status": "ok",
            "total": total,
            "returned": window.len(),
            "limit": limit,
            "cursor": cursor,
            "next_cursor": next_cursor,
            "truncated": next_cursor.is_some() || cursor > 0,
            "metadata": {"summary_only": true, "cursor": cursor, "limit": limit, "next_cursor": next_cursor},
            "rehydrate": {"route": "/v1/metacognition/captures/{capture_id}"},
            "reflections": window.into_iter().map(|rec| json!({
                "reflection_id": rec.reflection_id,
                "created_at": rec.created_at,
                "turn_range": rec.turn_range,
                "failure_classes": rec.failure_classes,
                "strategy_updates": rec.strategy_updates,
            })).collect::<Vec<_>>()
        }),
        "Recent metacognition reflections loaded. Next: choose one reflection before creating an adjustment.",
    )))
}

async fn recent_evaluations(
    State(state): State<Arc<AppState>>,
    scope_context: ScopeContext,
    headers: HeaderMap,
    Query(query): Query<RecentMetacogQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "metacognition:read")?;
    let scope = scope_context
        .require_workstream_key()
        .map_err(scope_required_response)?;

    let mut by_id: HashMap<String, EvaluationRecord> = HashMap::new();
    if let Some(s) = state
        .metacog_by_scope
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&scope)
    {
        for rec in &s.evaluations {
            by_id.insert(rec.evaluation_id.clone(), rec.clone());
        }
    }
    for rec in load_evaluation_records_from_disk(&state, &scope) {
        by_id.entry(rec.evaluation_id.clone()).or_insert(rec);
    }

    let limit = budgeted_requested_limit(
        query.limit,
        recent_artifacts_default_limit(),
        recent_artifacts_hard_limit(),
    );
    let total = by_id.len();
    let cursor = query.cursor.unwrap_or(0).min(total);
    let mut items = by_id.into_values().collect::<Vec<_>>();
    items.sort_by_key(|r| std::cmp::Reverse(r.created_at));
    let window = items
        .into_iter()
        .skip(cursor)
        .take(limit)
        .collect::<Vec<_>>();
    let next_cursor = (cursor + window.len() < total).then(|| (cursor + window.len()).to_string());

    Ok(Json(with_human_readable(
        json!({
            "status": "ok",
            "total": total,
            "returned": window.len(),
            "limit": limit,
            "cursor": cursor,
            "next_cursor": next_cursor,
            "truncated": next_cursor.is_some() || cursor > 0,
            "metadata": {"summary_only": true, "cursor": cursor, "limit": limit, "next_cursor": next_cursor},
            "rehydrate": {"route": "/v1/metacognition/evaluations/recent", "include_full_record": true},
            "evaluations": window.into_iter().map(|rec| json!({
                "evaluation_id": rec.evaluation_id,
                "adjustment_id": rec.adjustment_id,
                "created_at": rec.created_at,
                "result": rec.result,
                "promote_learning": rec.promote_learning,
                "observed_metrics": rec.observed_metrics,
                "storage_path": rec.storage_path,
            })).collect::<Vec<_>>()
        }),
        "Recent metacognition evaluations loaded. Next: inspect promotion evidence or collect stronger outcome metrics.",
    )))
}

async fn recent_adjustments(
    State(state): State<Arc<AppState>>,
    scope_context: ScopeContext,
    headers: HeaderMap,
    Query(query): Query<RecentMetacogQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "metacognition:read")?;
    let scope = scope_context
        .require_workstream_key()
        .map_err(scope_required_response)?;

    let mut by_id: HashMap<String, AdjustmentRecord> = HashMap::new();
    if let Some(s) = state
        .metacog_by_scope
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&scope)
    {
        for rec in &s.adjustments {
            by_id.insert(rec.adjustment_id.clone(), rec.clone());
        }
    }
    for rec in load_adjustment_records_from_disk(&state, &scope) {
        by_id.entry(rec.adjustment_id.clone()).or_insert(rec);
    }

    let limit = budgeted_requested_limit(
        query.limit,
        recent_artifacts_default_limit(),
        recent_artifacts_hard_limit(),
    );
    let total = by_id.len();
    let cursor = query.cursor.unwrap_or(0).min(total);
    let mut items = by_id.into_values().collect::<Vec<_>>();
    items.sort_by_key(|r| std::cmp::Reverse(r.created_at));
    let window = items
        .into_iter()
        .skip(cursor)
        .take(limit)
        .collect::<Vec<_>>();
    let next_cursor = (cursor + window.len() < total).then(|| (cursor + window.len()).to_string());

    Ok(Json(with_human_readable(
        json!({
            "status": "ok",
            "total": total,
            "returned": window.len(),
            "limit": limit,
            "cursor": cursor,
            "next_cursor": next_cursor,
            "truncated": next_cursor.is_some() || cursor > 0,
            "metadata": {"summary_only": true, "cursor": cursor, "limit": limit, "next_cursor": next_cursor},
            "rehydrate": {"route": "/v1/metacognition/captures/{capture_id}"},
            "adjustments": window.into_iter().map(|rec| json!({
                "adjustment_id": rec.adjustment_id,
                "reflection_id": rec.reflection_id,
                "created_at": rec.created_at,
                "selected_updates": rec.selected_updates,
            })).collect::<Vec<_>>()
        }),
        "Recent metacognition adjustments loaded. Next: evaluate one adjustment against observed metrics.",
    )))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/metacognition/status", get(metacog_status))
        .route("/v1/metacognition/capture", post(capture))
        .route("/v1/metacognition/captures/{capture_id}", get(get_capture))
        .route("/v1/metacognition/retrieve", post(retrieve))
        .route("/v1/metacognition/reflect", post(reflect))
        .route(
            "/v1/metacognition/reflections/recent",
            get(recent_reflections),
        )
        .route("/v1/metacognition/adjust", post(adjust))
        .route(
            "/v1/metacognition/adjustments/recent",
            get(recent_adjustments),
        )
        .route(
            "/v1/metacognition/evaluations/recent",
            get(recent_evaluations),
        )
        .route("/v1/metacognition/evaluate", post(evaluate))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    #[test]
    fn human_readable_field_is_stable_and_top_level() {
        let response = with_human_readable(
            json!({"status":"ok","capture_id":"cap-test"}),
            "Metacognition capture stored. Next: retrieve it.",
        );
        assert_eq!(
            response.get("human_readable").and_then(Value::as_str),
            Some("Metacognition capture stored. Next: retrieve it.")
        );
        assert_eq!(response.get("status").and_then(Value::as_str), Some("ok"));
    }

    fn capture(id: &str, created_at: chrono::DateTime<chrono::Utc>) -> CaptureRecord {
        CaptureRecord {
            capture_id: id.to_string(),
            created_at,
            kind: "kind".to_string(),
            content: format!("content-{id}"),
            rationale: None,
            confidence: None,
            strategy_class: None,
            storage_path: format!("/tmp/capture-{id}.json"),
            trajectory: None,
            scope: None,
        }
    }

    fn reflection(id: &str, created_at: chrono::DateTime<chrono::Utc>) -> ReflectionRecord {
        ReflectionRecord {
            reflection_id: id.to_string(),
            created_at,
            turn_range: "1..2".to_string(),
            failure_classes: vec![],
            hypotheses: vec![],
            strategy_updates: vec![],
            storage_path: format!("/tmp/reflection-{id}.json"),
            trajectory: None,
            scope: None,
        }
    }

    fn adjustment(id: &str, created_at: chrono::DateTime<chrono::Utc>) -> AdjustmentRecord {
        AdjustmentRecord {
            adjustment_id: id.to_string(),
            reflection_id: "refl".to_string(),
            selected_updates: vec![],
            created_at,
            storage_path: format!("/tmp/adjustment-{id}.json"),
            trajectory: None,
            scope: None,
        }
    }

    fn evaluation(id: &str, created_at: chrono::DateTime<chrono::Utc>) -> EvaluationRecord {
        EvaluationRecord {
            evaluation_id: id.to_string(),
            adjustment_id: "adj".to_string(),
            observed_metrics: vec!["metric".to_string()],
            result: "improved".to_string(),
            promote_learning: true,
            created_at,
            storage_path: format!("/tmp/evaluation-{id}.json"),
            trajectory: None,
            scope: None,
        }
    }

    #[test]
    fn capture_tags_include_trajectory_ladder_words() {
        let mut rec = capture("with-trajectory", chrono::Utc::now());
        rec.trajectory = Some(TrajectoryLadderContext {
            trajectory_id: Some("traj-test".to_string()),
            hlt: Some("High-level target".to_string()),
            mlg: Some("Mid-level objective".to_string()),
            stg: Some("Short-term checkpoint".to_string()),
            ..TrajectoryLadderContext::default()
        });
        let tags = tags_for_capture(&rec);
        assert!(tags.contains(&"traj-test".to_string()));
        assert!(tags.contains(&"high-level".to_string()));
        assert!(tags.contains(&"short-term".to_string()));
    }

    #[test]
    fn prune_metacog_store_applies_ttl() {
        let now = chrono::Utc.with_ymd_and_hms(2026, 4, 21, 20, 0, 0).unwrap();
        let mut store = MetaStore::default();
        store
            .captures
            .push(capture("old", now - Duration::minutes(200)));
        store
            .captures
            .push(capture("new", now - Duration::minutes(10)));

        prune_metacog_store(
            &mut store,
            now,
            MetaStoreConfig {
                max_captures: 10,
                max_reflections: 10,
                max_adjustments: 10,
                ttl_minutes: 60,
            },
        );

        assert_eq!(store.captures.len(), 1);
        assert_eq!(store.captures[0].capture_id, "new");
        assert_eq!(store.capture_hot_index.len(), 1);
        assert_eq!(store.eviction_events[0].collection, "captures");
    }

    #[test]
    fn capture_hot_index_is_summary_bounded_and_recent_first() {
        let now = chrono::Utc.with_ymd_and_hms(2026, 4, 21, 20, 0, 0).unwrap();
        let mut old = capture("old", now - Duration::minutes(4));
        old.content = "old ".repeat(100);
        let mut new = capture("new", now - Duration::minutes(1));
        new.content = "new learning signal with retrieval governor".to_string();
        let index = rebuild_capture_hot_index(
            &[old, new],
            MetaStoreConfig {
                max_captures: 10,
                max_reflections: 10,
                max_adjustments: 10,
                ttl_minutes: 60,
            },
            now,
        );
        assert_eq!(index[0].capture_id, "new");
        assert!(index[1].summary.chars().count() <= 240);
        assert!(index[0].tags.contains(&"retrieval".to_string()));
    }

    #[test]
    fn prune_metacog_store_applies_caps_per_collection() {
        let now = chrono::Utc.with_ymd_and_hms(2026, 4, 21, 20, 0, 0).unwrap();
        let mut store = MetaStore::default();
        store
            .captures
            .push(capture("c1", now - Duration::minutes(4)));
        store
            .captures
            .push(capture("c2", now - Duration::minutes(3)));
        store
            .captures
            .push(capture("c3", now - Duration::minutes(2)));

        store
            .reflections
            .push(reflection("r1", now - Duration::minutes(4)));
        store
            .reflections
            .push(reflection("r2", now - Duration::minutes(3)));

        store
            .adjustments
            .push(adjustment("a1", now - Duration::minutes(4)));
        store
            .adjustments
            .push(adjustment("a2", now - Duration::minutes(3)));
        store
            .evaluations
            .push(evaluation("e1", now - Duration::minutes(4)));
        store
            .evaluations
            .push(evaluation("e2", now - Duration::minutes(3)));

        prune_metacog_store(
            &mut store,
            now,
            MetaStoreConfig {
                max_captures: 2,
                max_reflections: 1,
                max_adjustments: 1,
                ttl_minutes: 60,
            },
        );

        assert_eq!(store.captures.len(), 2);
        assert_eq!(store.captures[0].capture_id, "c2");
        assert_eq!(store.captures[1].capture_id, "c3");

        assert_eq!(store.reflections.len(), 1);
        assert_eq!(store.reflections[0].reflection_id, "r2");

        assert_eq!(store.adjustments.len(), 1);
        assert_eq!(store.adjustments[0].adjustment_id, "a2");
        assert_eq!(store.evaluations.len(), 1);
        assert_eq!(store.evaluations[0].evaluation_id, "e2");
    }
}
