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

fn prediction_payload(event: &Value) -> Option<&Value> {
    event
        .get("payload")
        .filter(|_| event.get("event_type").and_then(|v| v.as_str()) == Some("spec92_prediction"))
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
    let mut focusa = state.focusa.write().await;
    focusa.telemetry.total_events += 1;
    focusa.telemetry.trace_events.push(json!({
        "event_id": Uuid::now_v7().to_string(),
        "event_type": "spec92_prediction",
        "timestamp": Utc::now().to_rfc3339(),
        "payload": payload,
    }));
    if focusa.telemetry.trace_events.len() > 5000 {
        let overflow = focusa.telemetry.trace_events.len() - 5000;
        focusa.telemetry.trace_events.drain(0..overflow);
    }
    Json(json!({"status":"recorded", "prediction": payload}))
}

async fn recent(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Value> {
    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(20)
        .min(100);
    let s = state.focusa.read().await;
    let mut predictions: Vec<Value> = s
        .telemetry
        .trace_events
        .iter()
        .filter_map(prediction_payload)
        .rev()
        .take(limit)
        .cloned()
        .collect();
    predictions.reverse();
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
    let mut focusa = state.focusa.write().await;
    let mut updated = None;
    for event in focusa.telemetry.trace_events.iter_mut().rev() {
        if event.get("event_type").and_then(|v| v.as_str()) != Some("spec92_prediction") {
            continue;
        }
        let Some(payload) = event.get_mut("payload") else {
            continue;
        };
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
        Some(prediction) => Json(json!({"status":"evaluated", "prediction": prediction})),
        None => Json(
            json!({"status":"not_found", "prediction_id": prediction_id, "safe_recovery":"focusa predict recent --json"}),
        ),
    }
}

async fn stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    let predictions: Vec<&Value> = s
        .telemetry
        .trace_events
        .iter()
        .filter_map(prediction_payload)
        .collect();
    let evaluated: Vec<&Value> = predictions
        .iter()
        .copied()
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
        (k, json!({"total": total, "evaluated": eval, "accuracy": if eval > 0 { sum / eval as f64 } else { Value::Null.as_f64().unwrap_or(0.0) }}))
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
