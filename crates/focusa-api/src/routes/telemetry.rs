//! Telemetry routes.

use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::{get, post}};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /v1/telemetry/tokens — token usage metrics.
async fn tokens(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    Json(json!({
        "total_events": s.telemetry.total_events,
        "total_prompt_tokens": s.telemetry.total_prompt_tokens,
        "total_completion_tokens": s.telemetry.total_completion_tokens,
        "tokens_per_task": s.telemetry.tokens_per_task,
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
    let summary: std::collections::HashMap<String, u32> = s
        .telemetry
        .tool_calls
        .iter()
        .fold(std::collections::HashMap::new(), |mut acc, name| {
            *acc.entry(name.clone()).or_insert(0) += 1;
            acc
        });
    Json(json!({
        "total_calls": s.telemetry.tool_calls.len(),
        "tool_summary": summary,
    }))
}

/// POST /v1/telemetry/tool-usage — receive tool call batch from extension.
async fn record_tool_usage(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ToolUsageBody>,
) -> Result<Json<Value>, axum::http::StatusCode> {
    // Feed tool names to telemetry for autonomy analysis.
    let mut focusa = state.focusa.write().await;
    for tool in &body.tools {
        focusa.telemetry.tool_calls.push(tool.clone());
    }
    Ok(Json(json!({"status": "accepted", "recorded": body.tools.len()})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/telemetry/tokens", get(tokens))
        .route("/v1/telemetry/cost", get(cost))
        .route("/v1/telemetry/tools", get(tool_usage))
        .route("/v1/telemetry/tool-usage", post(record_tool_usage))
}
