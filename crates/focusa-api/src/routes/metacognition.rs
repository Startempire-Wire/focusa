//! Metacognition API surface for SPEC80.
//!
//! Endpoints:
//! - POST /v1/metacognition/capture
//! - POST /v1/metacognition/retrieve
//! - POST /v1/metacognition/reflect
//! - POST /v1/metacognition/adjust
//! - POST /v1/metacognition/evaluate

use crate::routes::permissions::{forbid, permission_context};
use crate::server::AppState;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Json, Router, routing::{get, post}};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdjustmentRecord {
    adjustment_id: String,
    reflection_id: String,
    selected_updates: Vec<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    storage_path: String,
}

#[derive(Debug, Default)]
struct MetaStore {
    captures: Vec<CaptureRecord>,
    reflections: Vec<ReflectionRecord>,
    adjustments: Vec<AdjustmentRecord>,
}

#[derive(Debug, Clone, Copy)]
struct MetaStoreConfig {
    max_captures: usize,
    max_reflections: usize,
    max_adjustments: usize,
    ttl_minutes: i64,
}

fn metacog_store_config() -> MetaStoreConfig {
    let max_captures = std::env::var("FOCUSA_METACOG_MAX_CAPTURES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(1000)
        .max(1);
    let max_reflections = std::env::var("FOCUSA_METACOG_MAX_REFLECTIONS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(500)
        .max(1);
    let max_adjustments = std::env::var("FOCUSA_METACOG_MAX_ADJUSTMENTS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(500)
        .max(1);
    let ttl_minutes = std::env::var("FOCUSA_METACOG_TTL_MINUTES")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(7 * 24 * 60)
        .max(1);

    MetaStoreConfig {
        max_captures,
        max_reflections,
        max_adjustments,
        ttl_minutes,
    }
}

fn retain_recent<T>(items: &mut Vec<T>, max_len: usize, cutoff: chrono::DateTime<chrono::Utc>, created_at: impl Fn(&T) -> chrono::DateTime<chrono::Utc>) {
    items.retain(|item| created_at(item) >= cutoff);
    if items.len() > max_len {
        let overflow = items.len() - max_len;
        items.drain(0..overflow);
    }
}

fn prune_metacog_store(store: &mut MetaStore, now: chrono::DateTime<chrono::Utc>, cfg: MetaStoreConfig) {
    let cutoff = now - chrono::Duration::minutes(cfg.ttl_minutes);
    retain_recent(&mut store.captures, cfg.max_captures, cutoff, |r| r.created_at);
    retain_recent(&mut store.reflections, cfg.max_reflections, cutoff, |r| r.created_at);
    retain_recent(&mut store.adjustments, cfg.max_adjustments, cutoff, |r| r.created_at);
}

fn metacog_base_dir(state: &AppState) -> PathBuf {
    Path::new(&state.config.data_dir).join("runtime").join("metacognition")
}

fn metacog_category_dir(state: &AppState, category: &str) -> PathBuf {
    metacog_base_dir(state).join(category)
}

fn metacog_record_path(state: &AppState, category: &str, id: &str) -> PathBuf {
    metacog_category_dir(state, category).join(format!("{id}.json"))
}

fn persist_json_record(path: &Path, payload: &Value) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(bytes) = serde_json::to_vec_pretty(payload) {
        let _ = fs::write(path, bytes);
    }
}

fn load_capture_records_from_disk(state: &AppState) -> Vec<CaptureRecord> {
    let mut out = Vec::new();
    let dir = metacog_category_dir(state, "captures");
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

fn load_reflection_records_from_disk(state: &AppState) -> Vec<ReflectionRecord> {
    let mut out = Vec::new();
    let dir = metacog_category_dir(state, "reflections");
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

fn load_adjustment_records_from_disk(state: &AppState) -> Vec<AdjustmentRecord> {
    let mut out = Vec::new();
    let dir = metacog_category_dir(state, "adjustments");
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

fn reflection_exists_on_disk(state: &AppState, reflection_id: &str) -> bool {
    metacog_record_path(state, "reflections", reflection_id).exists()
}

fn adjustment_exists_on_disk(state: &AppState, adjustment_id: &str) -> bool {
    metacog_record_path(state, "adjustments", adjustment_id).exists()
}

static METACOG_STORE: OnceLock<Mutex<MetaStore>> = OnceLock::new();

fn store() -> &'static Mutex<MetaStore> {
    METACOG_STORE.get_or_init(|| Mutex::new(MetaStore::default()))
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
    headers: HeaderMap,
    Json(body): Json<CaptureBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "metacognition:write")?;

    if body.kind.trim().is_empty() || body.content.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "code": "CAPTURE_SCHEMA_INVALID",
                "reason": "kind and content are required"
            })),
        ));
    }

    let capture_id = format!("cap-{}", Utc::now().timestamp_nanos_opt().unwrap_or_default());
    let storage_path = metacog_record_path(&state, "captures", &capture_id)
        .display()
        .to_string();
    let rec = CaptureRecord {
        capture_id: capture_id.clone(),
        created_at: Utc::now(),
        kind: body.kind,
        content: body.content,
        rationale: body.rationale,
        confidence: body.confidence,
        strategy_class: body.strategy_class,
        storage_path: storage_path.clone(),
    };

    persist_json_record(
        &metacog_record_path(&state, "captures", &capture_id),
        &json!(rec),
    );

    let mut s = store().lock().expect("metacog store poisoned");
    s.captures.push(rec);
    prune_metacog_store(&mut s, Utc::now(), metacog_store_config());

    Ok(Json(json!({
        "capture_id": capture_id,
        "stored": true,
        "linked_turn_id": Value::Null,
        "storage_path": storage_path,
    })))
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
    #[serde(default)]
    summary_only: bool,
}

fn default_k() -> usize {
    5
}

fn retrieve_max_k() -> usize {
    std::env::var("FOCUSA_METACOG_RETRIEVE_MAX_K")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(20)
        .max(1)
}

async fn retrieve(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<RetrieveBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "metacognition:read")?;

    let ask = body.current_ask.to_lowercase();
    let tags = body
        .scope_tags
        .iter()
        .map(|t| t.to_lowercase())
        .collect::<Vec<_>>();

    let in_memory_captures = {
        let s = store().lock().expect("metacog store poisoned");
        s.captures.clone()
    };

    let mut by_id = HashMap::new();
    for c in load_capture_records_from_disk(&state) {
        by_id.insert(c.capture_id.clone(), c);
    }
    for c in in_memory_captures {
        by_id.insert(c.capture_id.clone(), c);
    }

    let mut ranked = by_id
        .values()
        .map(|c| {
            let content = c.content.to_lowercase();
            let mut score = 0_i64;
            if !ask.is_empty() && content.contains(&ask) {
                score += 2;
            }
            for tag in &tags {
                if content.contains(tag) {
                    score += 1;
                }
            }
            (score, c)
        })
        .collect::<Vec<_>>();

    ranked.sort_by_key(|(score, _)| Reverse(*score));

    let cursor_offset = body
        .cursor
        .as_deref()
        .and_then(|c| c.parse::<usize>().ok())
        .unwrap_or(0);
    let page_size = body.k.max(1).min(retrieve_max_k());

    let total = ranked.len();
    let page = ranked
        .into_iter()
        .skip(cursor_offset)
        .take(page_size)
        .collect::<Vec<_>>();

    let candidates = page
        .iter()
        .enumerate()
        .map(|(idx, (score, c))| {
            let summary = if body.summary_only {
                c.content.chars().take(240).collect::<String>()
            } else {
                c.content.clone()
            };

            json!({
                "capture_id": c.capture_id,
                "kind": c.kind,
                "summary": summary,
                "score": score,
                "rank": cursor_offset + idx + 1,
                "confidence": c.confidence,
                "has_rationale": c.rationale.is_some(),
                "strategy_class": c.strategy_class,
                "evidence_refs": []
            })
        })
        .collect::<Vec<_>>();

    let next_cursor = if cursor_offset + page_size < total {
        Some((cursor_offset + page_size).to_string())
    } else {
        None
    };

    Ok(Json(json!({
        "candidates": candidates,
        "next_cursor": next_cursor,
        "page_size": page_size,
        "total_candidates": total,
        "ranked_by": "keyword_similarity",
        "retrieval_budget": {
            "tokens_used": 0,
            "latency_ms": 0,
            "truncated": next_cursor.is_some()
        }
    })))
}

#[derive(Debug, Deserialize)]
struct ReflectBody {
    turn_range: String,
    #[serde(default)]
    failure_classes: Vec<String>,
}

async fn reflect(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<ReflectBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "metacognition:write")?;

    if body.turn_range.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "code": "REFLECT_INPUT_INVALID",
                "reason": "turn_range is required"
            })),
        ));
    }

    let reflection_id = format!("refl-{}", Utc::now().timestamp_nanos_opt().unwrap_or_default());
    let strategy_updates = if body.failure_classes.is_empty() {
        vec!["increase verification checkpoints".to_string()]
    } else {
        body.failure_classes
            .iter()
            .map(|f| format!("mitigate {f}"))
            .collect::<Vec<_>>()
    };

    let storage_path = metacog_record_path(&state, "reflections", &reflection_id)
        .display()
        .to_string();
    let rec = ReflectionRecord {
        reflection_id: reflection_id.clone(),
        created_at: Utc::now(),
        turn_range: body.turn_range,
        failure_classes: body.failure_classes,
        hypotheses: vec!["strategy mismatch in recent turns".into()],
        strategy_updates: strategy_updates.clone(),
        storage_path: storage_path.clone(),
    };

    persist_json_record(
        &metacog_record_path(&state, "reflections", &reflection_id),
        &json!(rec),
    );

    let mut s = store().lock().expect("metacog store poisoned");
    s.reflections.push(rec.clone());
    prune_metacog_store(&mut s, Utc::now(), metacog_store_config());

    Ok(Json(json!({
        "reflection_id": reflection_id,
        "hypotheses": rec.hypotheses,
        "strategy_updates": strategy_updates,
        "storage_path": storage_path,
    })))
}

#[derive(Debug, Deserialize)]
struct AdjustBody {
    reflection_id: String,
    #[serde(default)]
    selected_updates: Vec<String>,
}

async fn adjust(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<AdjustBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "metacognition:write")?;

    let in_mem_exists = {
        let s = store().lock().expect("metacog store poisoned");
        s.reflections
            .iter()
            .any(|r| r.reflection_id == body.reflection_id)
    };
    if !in_mem_exists && !reflection_exists_on_disk(&state, &body.reflection_id) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "code": "REFLECTION_NOT_FOUND",
                "reason": "reflection_id does not exist"
            })),
        ));
    }

    let adjustment_id = format!("adj-{}", Utc::now().timestamp_nanos_opt().unwrap_or_default());
    let storage_path = metacog_record_path(&state, "adjustments", &adjustment_id)
        .display()
        .to_string();
    let rec = AdjustmentRecord {
        adjustment_id: adjustment_id.clone(),
        reflection_id: body.reflection_id,
        selected_updates: body.selected_updates.clone(),
        created_at: Utc::now(),
        storage_path: storage_path.clone(),
    };
    persist_json_record(
        &metacog_record_path(&state, "adjustments", &adjustment_id),
        &json!(rec),
    );
    let mut s = store().lock().expect("metacog store poisoned");
    s.adjustments.push(rec.clone());
    prune_metacog_store(&mut s, Utc::now(), metacog_store_config());

    Ok(Json(json!({
        "adjustment_id": adjustment_id,
        "next_step_policy": rec.selected_updates,
        "expected_deltas": {
            "failed_turn_ratio": -0.1,
            "rework_loop_rate": -0.1,
        },
        "storage_path": storage_path,
    })))
}

#[derive(Debug, Deserialize)]
struct EvaluateBody {
    adjustment_id: String,
    #[serde(default)]
    observed_metrics: Vec<String>,
}

async fn evaluate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<EvaluateBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "metacognition:write")?;

    let in_mem_exists = {
        let s = store().lock().expect("metacog store poisoned");
        s.adjustments
            .iter()
            .any(|a| a.adjustment_id == body.adjustment_id)
    };
    if !in_mem_exists && !adjustment_exists_on_disk(&state, &body.adjustment_id) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "code": "ADJUSTMENT_NOT_FOUND",
                "reason": "adjustment_id does not exist"
            })),
        ));
    }

    let promote = !body.observed_metrics.is_empty();

    Ok(Json(json!({
        "evaluation_id": format!("eval-{}", Utc::now().timestamp_nanos_opt().unwrap_or_default()),
        "delta_scorecard": {
            "metrics_observed": body.observed_metrics,
        },
        "result": if promote { "improved" } else { "inconclusive" },
        "promote_learning": promote,
    })))
}

#[derive(Debug, Deserialize)]
struct RecentMetacogQuery {
    limit: Option<usize>,
}

async fn recent_reflections(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<RecentMetacogQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "metacognition:read")?;

    let mut by_id: HashMap<String, ReflectionRecord> = HashMap::new();
    {
        let s = store().lock().expect("metacog store poisoned");
        for rec in &s.reflections {
            by_id.insert(rec.reflection_id.clone(), rec.clone());
        }
    }
    for rec in load_reflection_records_from_disk(&state) {
        by_id.entry(rec.reflection_id.clone()).or_insert(rec);
    }

    let limit = query.limit.unwrap_or(5).clamp(1, 20);
    let mut items = by_id.into_values().collect::<Vec<_>>();
    items.sort_by_key(|r| std::cmp::Reverse(r.created_at));
    items.truncate(limit);

    Ok(Json(json!({
        "status": "ok",
        "total": items.len(),
        "reflections": items.into_iter().map(|rec| json!({
            "reflection_id": rec.reflection_id,
            "created_at": rec.created_at,
            "turn_range": rec.turn_range,
            "failure_classes": rec.failure_classes,
            "strategy_updates": rec.strategy_updates,
        })).collect::<Vec<_>>()
    })))
}

async fn recent_adjustments(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<RecentMetacogQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_scope(&headers, &state, "metacognition:read")?;

    let mut by_id: HashMap<String, AdjustmentRecord> = HashMap::new();
    {
        let s = store().lock().expect("metacog store poisoned");
        for rec in &s.adjustments {
            by_id.insert(rec.adjustment_id.clone(), rec.clone());
        }
    }
    for rec in load_adjustment_records_from_disk(&state) {
        by_id.entry(rec.adjustment_id.clone()).or_insert(rec);
    }

    let limit = query.limit.unwrap_or(5).clamp(1, 20);
    let mut items = by_id.into_values().collect::<Vec<_>>();
    items.sort_by_key(|r| std::cmp::Reverse(r.created_at));
    items.truncate(limit);

    Ok(Json(json!({
        "status": "ok",
        "total": items.len(),
        "adjustments": items.into_iter().map(|rec| json!({
            "adjustment_id": rec.adjustment_id,
            "reflection_id": rec.reflection_id,
            "created_at": rec.created_at,
            "selected_updates": rec.selected_updates,
        })).collect::<Vec<_>>()
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/metacognition/capture", post(capture))
        .route("/v1/metacognition/retrieve", post(retrieve))
        .route("/v1/metacognition/reflect", post(reflect))
        .route("/v1/metacognition/reflections/recent", get(recent_reflections))
        .route("/v1/metacognition/adjust", post(adjust))
        .route("/v1/metacognition/adjustments/recent", get(recent_adjustments))
        .route("/v1/metacognition/evaluate", post(evaluate))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

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
        }
    }

    fn adjustment(id: &str, created_at: chrono::DateTime<chrono::Utc>) -> AdjustmentRecord {
        AdjustmentRecord {
            adjustment_id: id.to_string(),
            reflection_id: "refl".to_string(),
            selected_updates: vec![],
            created_at,
            storage_path: format!("/tmp/adjustment-{id}.json"),
        }
    }

    #[test]
    fn prune_metacog_store_applies_ttl() {
        let now = chrono::Utc.with_ymd_and_hms(2026, 4, 21, 20, 0, 0).unwrap();
        let mut store = MetaStore::default();
        store.captures.push(capture("old", now - Duration::minutes(200)));
        store.captures.push(capture("new", now - Duration::minutes(10)));

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
    }

    #[test]
    fn prune_metacog_store_applies_caps_per_collection() {
        let now = chrono::Utc.with_ymd_and_hms(2026, 4, 21, 20, 0, 0).unwrap();
        let mut store = MetaStore::default();
        store.captures.push(capture("c1", now - Duration::minutes(4)));
        store.captures.push(capture("c2", now - Duration::minutes(3)));
        store.captures.push(capture("c3", now - Duration::minutes(2)));

        store.reflections.push(reflection("r1", now - Duration::minutes(4)));
        store.reflections.push(reflection("r2", now - Duration::minutes(3)));

        store.adjustments.push(adjustment("a1", now - Duration::minutes(4)));
        store.adjustments.push(adjustment("a2", now - Duration::minutes(3)));

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
    }
}
