use crate::server::AppState;
use axum::extract::{Path, Query, State};
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
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

fn store_path() -> PathBuf {
    let home = std::env::var("FOCUSA_HOME").unwrap_or_else(|_| "/home/wirebot/focusa".to_string());
    PathBuf::from(home).join("data/spec92_predictions.json")
}

fn read_predictions() -> Vec<Value> {
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

fn write_predictions(predictions: Vec<Value>) -> std::io::Result<Vec<Value>> {
    write_predictions_to(&store_path(), predictions)
}

async fn record(
    State(state): State<Arc<AppState>>,
    Json(body): Json<PredictionBody>,
) -> Json<Value> {
    let prediction_id = Uuid::now_v7().to_string();
    let confidence = body.confidence.clamp(0.0, 1.0);
    let payload = json!({
        "prediction_id": prediction_id,
        "ts": Utc::now().to_rfc3339(),
        "prediction_type": body.prediction_type,
        "context_refs": body.context_refs,
        "predicted_outcome": body.predicted_outcome,
        "confidence": confidence,
        "recommended_action": body.recommended_action,
        "why": body.why,
        "actual_outcome": null,
        "evaluated_at": null,
        "score": null,
        "learning_signal_ref": null,
    });
    let mut predictions = read_predictions();
    predictions.push(payload.clone());
    let status = match write_predictions(predictions) {
        Ok(_) => "recorded",
        Err(_) => "blocked",
    };
    let mut focusa = state.focusa.write().await;
    focusa.telemetry.total_events += 1;
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
    Path(prediction_id): Path<String>,
    Json(body): Json<EvaluateBody>,
) -> Json<Value> {
    let mut predictions = read_predictions();
    let mut updated = None;
    for payload in predictions.iter_mut().rev() {
        if payload.get("prediction_id").and_then(|v| v.as_str()) != Some(prediction_id.as_str()) {
            continue;
        }
        let score = body
            .score
            .unwrap_or_else(|| {
                let predicted = payload
                    .get("predicted_outcome")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_lowercase();
                let actual = body.actual_outcome.to_lowercase();
                if !predicted.is_empty() && actual.contains(&predicted) {
                    1.0
                } else {
                    0.0
                }
            })
            .clamp(0.0, 1.0);
        payload["actual_outcome"] = json!(body.actual_outcome);
        payload["evaluated_at"] = json!(Utc::now().to_rfc3339());
        payload["score"] = json!(score);
        payload["learning_signal_ref"] = body
            .learning_signal_ref
            .map(Value::String)
            .unwrap_or(Value::Null);
        updated = Some(payload.clone());
        break;
    }
    match updated {
        Some(prediction) => match write_predictions(predictions) {
            Ok(_) => Json(json!({"status":"evaluated", "prediction": prediction})),
            Err(err) => Json(
                json!({"status":"blocked", "what_failed":"write prediction store", "likely_why":err.to_string(), "safe_recovery":"check data directory permissions"}),
            ),
        },
        None => Json(
            json!({"status":"not_found", "prediction_id": prediction_id, "safe_recovery":"focusa predict recent --json"}),
        ),
    }
}

async fn stats() -> Json<Value> {
    let predictions = read_predictions();
    let evaluated: Vec<&Value> = predictions
        .iter()
        .filter(|p| !p.get("score").unwrap_or(&Value::Null).is_null())
        .collect();
    let mut by_type: HashMap<String, (usize, usize, f64)> = HashMap::new();
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
    }
    let by_type_json: HashMap<String, Value> = by_type.into_iter().map(|(k, (total, eval, sum))| {
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
        "next_action": "Record/evaluate predictions at decision points; predictions guide but never override operator steering",
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/predictions", post(record))
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
