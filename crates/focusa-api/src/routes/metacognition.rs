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
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::{Json, Router, routing::post};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::cmp::Reverse;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone, Serialize)]
struct CaptureRecord {
    capture_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
    kind: String,
    content: String,
    rationale: Option<String>,
    confidence: Option<f64>,
    strategy_class: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ReflectionRecord {
    reflection_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
    turn_range: String,
    failure_classes: Vec<String>,
    hypotheses: Vec<String>,
    strategy_updates: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct AdjustmentRecord {
    adjustment_id: String,
    reflection_id: String,
    selected_updates: Vec<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Default)]
struct MetaStore {
    captures: Vec<CaptureRecord>,
    reflections: Vec<ReflectionRecord>,
    adjustments: Vec<AdjustmentRecord>,
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
    let rec = CaptureRecord {
        capture_id: capture_id.clone(),
        created_at: Utc::now(),
        kind: body.kind,
        content: body.content,
        rationale: body.rationale,
        confidence: body.confidence,
        strategy_class: body.strategy_class,
    };

    let mut s = store().lock().expect("metacog store poisoned");
    s.captures.push(rec);

    Ok(Json(json!({
        "capture_id": capture_id,
        "stored": true,
        "linked_turn_id": Value::Null,
    })))
}

#[derive(Debug, Deserialize)]
struct RetrieveBody {
    current_ask: String,
    #[serde(default)]
    scope_tags: Vec<String>,
    #[serde(default = "default_k")]
    k: usize,
}

fn default_k() -> usize {
    5
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

    let s = store().lock().expect("metacog store poisoned");
    let mut ranked = s
        .captures
        .iter()
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

    let candidates = ranked
        .into_iter()
        .take(body.k.max(1))
        .enumerate()
        .map(|(idx, (score, c))| {
            json!({
                "capture_id": c.capture_id,
                "summary": c.content,
                "score": score,
                "rank": idx + 1,
                "evidence_refs": []
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(json!({
        "candidates": candidates,
        "ranked_by": "keyword_similarity",
        "retrieval_budget": {
            "tokens_used": 0,
            "latency_ms": 0,
            "truncated": false
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

    let rec = ReflectionRecord {
        reflection_id: reflection_id.clone(),
        created_at: Utc::now(),
        turn_range: body.turn_range,
        failure_classes: body.failure_classes,
        hypotheses: vec!["strategy mismatch in recent turns".into()],
        strategy_updates: strategy_updates.clone(),
    };

    let mut s = store().lock().expect("metacog store poisoned");
    s.reflections.push(rec.clone());

    Ok(Json(json!({
        "reflection_id": reflection_id,
        "hypotheses": rec.hypotheses,
        "strategy_updates": strategy_updates,
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

    let s = store().lock().expect("metacog store poisoned");
    if !s
        .reflections
        .iter()
        .any(|r| r.reflection_id == body.reflection_id)
    {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "error",
                "code": "REFLECTION_NOT_FOUND",
                "reason": "reflection_id does not exist"
            })),
        ));
    }
    drop(s);

    let adjustment_id = format!("adj-{}", Utc::now().timestamp_nanos_opt().unwrap_or_default());
    let rec = AdjustmentRecord {
        adjustment_id: adjustment_id.clone(),
        reflection_id: body.reflection_id,
        selected_updates: body.selected_updates.clone(),
        created_at: Utc::now(),
    };
    let mut s = store().lock().expect("metacog store poisoned");
    s.adjustments.push(rec.clone());

    Ok(Json(json!({
        "adjustment_id": adjustment_id,
        "next_step_policy": rec.selected_updates,
        "expected_deltas": {
            "failed_turn_ratio": -0.1,
            "rework_loop_rate": -0.1,
        }
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

    let s = store().lock().expect("metacog store poisoned");
    if !s
        .adjustments
        .iter()
        .any(|a| a.adjustment_id == body.adjustment_id)
    {
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

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/metacognition/capture", post(capture))
        .route("/v1/metacognition/retrieve", post(retrieve))
        .route("/v1/metacognition/reflect", post(reflect))
        .route("/v1/metacognition/adjust", post(adjust))
        .route("/v1/metacognition/evaluate", post(evaluate))
}
