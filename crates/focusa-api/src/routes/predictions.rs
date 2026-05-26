use crate::routes::metacognition::capture_learning_signal;
use crate::server::AppState;
use axum::extract::{Path, Query, State};
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::types::TrajectoryLadderContext;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
struct PredictionBody {
    prediction_type: String,
    #[serde(default)]
    context_refs: Vec<String>,
    #[serde(default)]
    ontology_context: Value,
    predicted_outcome: String,
    confidence: f64,
    recommended_action: String,
    why: String,
}

#[derive(Debug, Deserialize)]
struct EvaluateBody {
    actual_outcome: String,
    #[serde(default)]
    score: Option<f64>,
    #[serde(default)]
    learning_signal_ref: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CaptureOutcomeBody {
    actual_outcome: String,
    #[serde(default)]
    prediction_type: Option<String>,
    #[serde(default)]
    context_refs: Vec<String>,
    #[serde(default)]
    ontology_context: Value,
    #[serde(default)]
    score: Option<f64>,
    #[serde(default)]
    learning_signal_ref: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
}

fn store_path() -> PathBuf {
    if let Some(home) = std::env::var_os("FOCUSA_HOME") {
        return PathBuf::from(home).join("data/spec92_predictions.json");
    }
    if let Some(data_home) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(data_home).join("focusa/spec92_predictions.json");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".local/share/focusa/spec92_predictions.json");
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data/spec92_predictions.json")
}

pub(crate) fn read_predictions() -> Vec<Value> {
    let path = store_path();
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<Vec<Value>>(&s).ok())
        .unwrap_or_default()
}

fn bound_predictions(mut predictions: Vec<Value>) -> Vec<Value> {
    if predictions.len() > 1000 {
        let overflow = predictions.len() - 1000;
        predictions.drain(0..overflow);
    }
    predictions
}

fn prediction_score(
    predicted_outcome: &str,
    actual_outcome: &str,
    explicit_score: Option<f64>,
) -> f64 {
    explicit_score
        .unwrap_or_else(|| {
            let predicted = predicted_outcome.to_lowercase();
            let actual = actual_outcome.to_lowercase();
            if !predicted.is_empty() && actual.contains(&predicted) {
                1.0
            } else {
                0.0
            }
        })
        .clamp(0.0, 1.0)
}

fn contexts_match(record_refs: &[String], outcome_refs: &[String]) -> bool {
    if outcome_refs.is_empty() {
        return true;
    }
    outcome_refs
        .iter()
        .any(|needle| record_refs.iter().any(|candidate| candidate == needle))
}

fn bound_string_array(value: &Value, key: &str, limit: usize) -> Vec<String> {
    value
        .get(key)
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    item.as_str()
                        .map(|s| s.chars().take(180).collect::<String>())
                })
                .take(limit)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn bound_ontology_context(value: Value) -> Value {
    if !value.is_object() {
        return Value::Null;
    }
    json!({
        "object_refs": bound_string_array(&value, "object_refs", 8),
        "action_refs": bound_string_array(&value, "action_refs", 8),
        "tool_refs": bound_string_array(&value, "tool_refs", 8),
        "evidence_refs": bound_string_array(&value, "evidence_refs", 8),
        "relation_refs": bound_string_array(&value, "relation_refs", 8),
    })
}

fn ontology_context_summary(value: &Value) -> String {
    if !value.is_object() {
        return "none".to_string();
    }
    let mut parts = Vec::new();
    for key in [
        "object_refs",
        "action_refs",
        "tool_refs",
        "evidence_refs",
        "relation_refs",
    ] {
        let count = value
            .get(key)
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        if count > 0 {
            parts.push(format!("{key}={count}"));
        }
    }
    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join(" ")
    }
}

fn trajectory_summary(trajectory: Option<TrajectoryLadderContext>) -> Value {
    match trajectory {
        Some(t) => json!({
            "trajectory_id": t.trajectory_id,
            "hlt": t.hlt,
            "mlg": t.mlg,
            "stg": t.stg,
            "waypoints": t.waypoints,
        }),
        None => Value::Null,
    }
}

fn write_predictions_to(
    path: &std::path::Path,
    predictions: Vec<Value>,
) -> std::io::Result<Vec<Value>> {
    let predictions = bound_predictions(predictions);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(&predictions)?)?;
    Ok(predictions)
}

pub(crate) fn write_predictions(predictions: Vec<Value>) -> std::io::Result<Vec<Value>> {
    write_predictions_to(&store_path(), predictions)
}

pub(crate) fn append_prediction_record(mut payload: Value) -> std::io::Result<Value> {
    if payload.get("prediction_id").is_none() {
        payload["prediction_id"] = json!(Uuid::now_v7().to_string());
    }
    if payload.get("ts").is_none() {
        payload["ts"] = json!(Utc::now().to_rfc3339());
    }
    let mut predictions = read_predictions();
    predictions.push(payload.clone());
    write_predictions(predictions).map(|_| payload)
}

async fn record(
    State(state): State<Arc<AppState>>,
    Json(body): Json<PredictionBody>,
) -> Json<Value> {
    let prediction_id = Uuid::now_v7().to_string();
    let confidence = body.confidence.clamp(0.0, 1.0);
    let trajectory = trajectory_summary(state.focusa.read().await.trajectory_ladder_context());
    let payload = json!({
        "prediction_id": prediction_id,
        "ts": Utc::now().to_rfc3339(),
        "prediction_type": body.prediction_type,
        "context_refs": body.context_refs,
        "ontology_context": bound_ontology_context(body.ontology_context),
        "predicted_outcome": body.predicted_outcome,
        "confidence": confidence,
        "recommended_action": body.recommended_action,
        "why": body.why,
        "trajectory": trajectory,
        "actual_outcome": null,
        "evaluated_at": null,
        "score": null,
        "learning_signal_ref": null,
        "outcome_capture": null,
    });
    let mut predictions = read_predictions();
    predictions.push(payload.clone());
    let status = match write_predictions(predictions) {
        Ok(_) => "recorded",
        Err(_) => "blocked",
    };
    let mut focusa = state.focusa.write().await;
    focusa.telemetry.total_events += 1;
    drop(focusa);
    state.mark_external_mutation();
    Json(json!({"status": status, "prediction": payload}))
}

async fn recent(Query(params): Query<HashMap<String, String>>) -> Json<Value> {
    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(20)
        .min(100);
    let mut predictions = read_predictions();
    if predictions.len() > limit {
        predictions = predictions.split_off(predictions.len() - limit);
    }
    Json(json!({
        "status": "completed",
        "summary": format!("{} prediction record(s)", predictions.len()),
        "predictions": predictions,
    }))
}

async fn evaluate(
    State(state): State<Arc<AppState>>,
    Path(prediction_id): Path<String>,
    Json(body): Json<EvaluateBody>,
) -> Json<Value> {
    let mut predictions = read_predictions();
    let mut updated = None;
    for payload in predictions.iter_mut().rev() {
        if payload.get("prediction_id").and_then(|v| v.as_str()) != Some(prediction_id.as_str()) {
            continue;
        }
        let score = prediction_score(
            payload
                .get("predicted_outcome")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            &body.actual_outcome,
            body.score,
        );
        payload["actual_outcome"] = json!(body.actual_outcome);
        payload["evaluated_at"] = json!(Utc::now().to_rfc3339());
        payload["score"] = json!(score);
        payload["learning_signal_ref"] = body
            .learning_signal_ref
            .map(Value::String)
            .unwrap_or(Value::Null);
        payload["outcome_capture"] = json!({"mode": "manual", "matched_by": "prediction_id"});
        updated = Some(payload.clone());
        break;
    }
    let mut promoted_capture_id = None;
    if let Some(prediction) = &updated {
        if prediction
            .get("score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            >= 0.5
        {
            promoted_capture_id = capture_learning_signal(
                &state,
                "prediction_outcome",
                &format!(
                    "Prediction {} scored {}. Expected: {}. Actual: {}. Recommended action was: {}. Ontology: {}",
                    prediction.get("prediction_id").and_then(|v| v.as_str()).unwrap_or("unknown"),
                    prediction.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    prediction.get("predicted_outcome").and_then(|v| v.as_str()).unwrap_or(""),
                    prediction.get("actual_outcome").and_then(|v| v.as_str()).unwrap_or(""),
                    prediction.get("recommended_action").and_then(|v| v.as_str()).unwrap_or(""),
                    ontology_context_summary(prediction.get("ontology_context").unwrap_or(&Value::Null))
                ),
                Some("Prediction evaluation fed the metacognition retrieval loop.".to_string()),
                prediction.get("score").and_then(|v| v.as_f64()),
                Some("prediction_metacog_flywheel".to_string()),
            ).await;
        }
    }
    match updated {
        Some(prediction) => match write_predictions(predictions) {
            Ok(_) => Json(
                json!({"status":"evaluated", "prediction": prediction, "metacog_capture_id": promoted_capture_id, "flywheel": {"prediction_to_metacog": promoted_capture_id.is_some(), "next_tools": ["focusa_metacog_retrieve", "focusa_predict_record"]}}),
            ),
            Err(err) => Json(
                json!({"status":"blocked", "what_failed":"write prediction store", "likely_why":err.to_string(), "safe_recovery":"check data directory permissions"}),
            ),
        },
        None => Json(
            json!({"status":"not_found", "prediction_id": prediction_id, "safe_recovery":"focusa predict recent --json"}),
        ),
    }
}

async fn capture_outcome(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CaptureOutcomeBody>,
) -> Json<Value> {
    let mut predictions = read_predictions();
    let mut updated = Vec::new();
    let limit = body.limit.unwrap_or(10).clamp(1, 50);
    let now = Utc::now().to_rfc3339();
    for payload in predictions.iter_mut().rev() {
        if updated.len() >= limit {
            break;
        }
        if !payload.get("score").unwrap_or(&Value::Null).is_null() {
            continue;
        }
        if let Some(prediction_type) = body.prediction_type.as_deref() {
            if payload.get("prediction_type").and_then(|v| v.as_str()) != Some(prediction_type) {
                continue;
            }
        }
        let record_refs = payload
            .get("context_refs")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if !contexts_match(&record_refs, &body.context_refs) {
            continue;
        }
        let score = prediction_score(
            payload
                .get("predicted_outcome")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            &body.actual_outcome,
            body.score,
        );
        payload["actual_outcome"] = json!(body.actual_outcome);
        payload["evaluated_at"] = json!(now);
        payload["score"] = json!(score);
        payload["learning_signal_ref"] = body
            .learning_signal_ref
            .as_ref()
            .map(|s| Value::String(s.clone()))
            .unwrap_or(Value::Null);
        payload["outcome_capture"] = json!({
            "mode": "auto_capture",
            "matched_by": if body.context_refs.is_empty() { "prediction_type_or_recent" } else { "context_refs" },
            "context_refs": body.context_refs,
            "ontology_context": bound_ontology_context(body.ontology_context.clone()),
        });
        updated.push(payload.clone());
    }
    let mut metacog_capture_ids = Vec::new();
    for prediction in &updated {
        if prediction
            .get("score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            >= 0.5
        {
            if let Some(capture_id) = capture_learning_signal(
                &state,
                "prediction_outcome",
                &format!(
                    "Auto-captured prediction outcome {} scored {}. Expected: {}. Actual: {}. Ontology: {}",
                    prediction.get("prediction_id").and_then(|v| v.as_str()).unwrap_or("unknown"),
                    prediction.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    prediction.get("predicted_outcome").and_then(|v| v.as_str()).unwrap_or(""),
                    prediction.get("actual_outcome").and_then(|v| v.as_str()).unwrap_or(""),
                    ontology_context_summary(prediction.get("ontology_context").unwrap_or(&Value::Null))
                ),
                Some("Prediction auto outcome capture fed metacognition retrieval memory.".to_string()),
                prediction.get("score").and_then(|v| v.as_f64()),
                Some("prediction_metacog_flywheel".to_string()),
            ).await {
                metacog_capture_ids.push(capture_id);
            }
        }
    }
    match write_predictions(predictions) {
        Ok(_) => Json(json!({
            "status": "completed",
            "summary": format!("auto-captured outcome for {} prediction(s)", updated.len()),
            "updated": updated,
            "metacog_capture_ids": metacog_capture_ids,
            "flywheel": {"prediction_to_metacog": !metacog_capture_ids.is_empty(), "next_tools": ["focusa_metacog_retrieve", "focusa_predict_stats"]},
            "next_action": if updated.is_empty() { "record a prediction with matching context_refs before capture-outcome" } else { "retrieve metacognition before the next decision and record the next prediction" },
        })),
        Err(err) => Json(json!({
            "status":"blocked",
            "what_failed":"write prediction store",
            "likely_why":err.to_string(),
            "safe_recovery":"check data directory permissions"
        })),
    }
}

async fn stats() -> Json<Value> {
    let predictions = read_predictions();
    let evaluated: Vec<&Value> = predictions
        .iter()
        .filter(|p| !p.get("score").unwrap_or(&Value::Null).is_null())
        .collect();
    let mut by_type: HashMap<String, (usize, usize, f64)> = HashMap::new();
    let mut by_trajectory: HashMap<String, (usize, usize, f64)> = HashMap::new();
    for p in &predictions {
        let ty = p
            .get("prediction_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let e = by_type.entry(ty).or_insert((0, 0, 0.0));
        e.0 += 1;
        if let Some(score) = p.get("score").and_then(|v| v.as_f64()) {
            e.1 += 1;
            e.2 += score;
        }
        let trajectory_key = p
            .pointer("/trajectory/trajectory_id")
            .and_then(|v| v.as_str())
            .or_else(|| p.pointer("/trajectory/hlt").and_then(|v| v.as_str()))
            .unwrap_or("none")
            .to_string();
        let te = by_trajectory.entry(trajectory_key).or_insert((0, 0, 0.0));
        te.0 += 1;
        if let Some(score) = p.get("score").and_then(|v| v.as_f64()) {
            te.1 += 1;
            te.2 += score;
        }
    }
    let by_type_json: HashMap<String, Value> = by_type.into_iter().map(|(k, (total, eval, sum))| {
        (k, json!({"total": total, "evaluated": eval, "accuracy": if eval > 0 { sum / eval as f64 } else { 0.0 }}))
    }).collect();
    let by_trajectory_json: HashMap<String, Value> = by_trajectory.into_iter().map(|(k, (total, eval, sum))| {
        (k, json!({"total": total, "evaluated": eval, "accuracy": if eval > 0 { sum / eval as f64 } else { 0.0 }}))
    }).collect();
    let score_sum: f64 = evaluated
        .iter()
        .filter_map(|p| p.get("score").and_then(|v| v.as_f64()))
        .sum();
    let accuracy = if evaluated.is_empty() {
        0.0
    } else {
        score_sum / evaluated.len() as f64
    };
    Json(json!({
        "status": "completed",
        "summary": format!("{} predictions, {} evaluated", predictions.len(), evaluated.len()),
        "accuracy": accuracy,
        "total": predictions.len(),
        "evaluated": evaluated.len(),
        "by_type": by_type_json,
        "by_trajectory": by_trajectory_json,
        "next_action": "Record predictions at decision points and use capture-outcome after proof/test/CI/evidence events; predictions guide but never override operator steering",
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/predictions", post(record))
        .route("/v1/predictions/capture-outcome", post(capture_outcome))
        .route("/v1/predictions/recent", get(recent))
        .route("/v1/predictions/stats", get(stats))
        .route("/v1/predictions/{prediction_id}/evaluate", post(evaluate))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bound_predictions_keeps_latest_thousand_records() {
        let records: Vec<Value> = (0..1005).map(|i| json!({"prediction_id": i})).collect();
        let bounded = bound_predictions(records);
        assert_eq!(bounded.len(), 1000);
        assert_eq!(
            bounded
                .first()
                .and_then(|v| v.get("prediction_id"))
                .and_then(|v| v.as_i64()),
            Some(5)
        );
        assert_eq!(
            bounded
                .last()
                .and_then(|v| v.get("prediction_id"))
                .and_then(|v| v.as_i64()),
            Some(1004)
        );
    }

    #[test]
    fn write_predictions_to_persists_json_records() {
        let dir = std::env::temp_dir().join(format!("focusa-pred-test-{}", Uuid::now_v7()));
        let path = dir.join("predictions.json");
        let records = vec![json!({"prediction_id":"p1","prediction_type":"token_risk"})];
        let written = write_predictions_to(&path, records).expect("write predictions");
        assert_eq!(written.len(), 1);
        let text = fs::read_to_string(&path).expect("read predictions");
        let parsed: Vec<Value> = serde_json::from_str(&text).expect("json predictions");
        assert_eq!(parsed[0]["prediction_id"], "p1");
        let _ = fs::remove_dir_all(dir);
    }
}
