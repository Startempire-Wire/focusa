//! Training Dataset Export & Contribution routes.
//!
//! Source: docs/21 (export), docs/22 (contribution)
//!
//! GET  /v1/export/status         — export pipeline status
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
use focusa_core::training;
use focusa_core::types::*;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /v1/export/status — export pipeline status.
async fn export_status(State(_state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({
        "implemented": false,
        "dataset_types": ["sft", "preference", "contrastive", "long-horizon"],
        "supported_formats": ["jsonl", "parquet"],
        "history_count": 0,
        "last_export_at": Value::Null,
        "status": "not_implemented",
        "reason": "dataset export pipeline not implemented yet",
    }))
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
    Ok(Json(json!({ "status": "enabled" })))
}

/// POST /v1/contribute/pause — pause contribution.
async fn contribute_pause(State(state): State<Arc<AppState>>) -> Result<Json<Value>, StatusCode> {
    let mut s = state.focusa.write().await;
    s.contribution.enabled = false;
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
        .map_err(|e| (StatusCode::NOT_FOUND, Json(json!({ "error": e }))))?;
    Ok(Json(
        json!({ "status": "approved", "item_id": body.item_id }),
    ))
}

/// POST /v1/contribute/submit — submit approved items.
async fn contribute_submit(State(state): State<Arc<AppState>>) -> Json<Value> {
    let mut s = state.focusa.write().await;
    let count = training::submit_approved(&mut s.contribution);
    Json(json!({ "submitted": count }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/export/status", get(export_status))
        .route("/v1/training/status", get(contribution_status))
        .route("/v1/contribute/enable", post(contribute_enable))
        .route("/v1/contribute/pause", post(contribute_pause))
        .route("/v1/contribute/approve", post(contribute_approve))
        .route("/v1/contribute/submit", post(contribute_submit))
}
