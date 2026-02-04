//! Memory routes.
//!
//! GET  /v1/memory/semantic             — list semantic memory
//! POST /v1/memory/semantic/upsert      — upsert a key=value
//! GET  /v1/memory/procedural           — list procedural rules
//! POST /v1/memory/procedural/reinforce — reinforce a rule

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router, routing::{get, post}};
use focusa_core::types::{Action, MemorySource};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

async fn semantic(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    Json(json!({
        "semantic": focusa.memory.semantic,
    }))
}

#[derive(Deserialize)]
struct UpsertBody {
    key: String,
    value: String,
    #[serde(default = "default_source")]
    source: MemorySource,
}

fn default_source() -> MemorySource {
    MemorySource::User
}

async fn upsert_semantic(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpsertBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .command_tx
        .send(Action::UpsertSemantic {
            key: body.key,
            value: body.value,
            source: body.source,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

async fn procedural(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    Json(json!({
        "procedural": focusa.memory.procedural,
    }))
}

#[derive(Deserialize)]
struct ReinforceBody {
    rule_id: String,
}

async fn reinforce_rule(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ReinforceBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .command_tx
        .send(Action::ReinforceRule {
            rule_id: body.rule_id,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/memory/semantic", get(semantic))
        .route("/v1/memory/semantic/upsert", post(upsert_semantic))
        .route("/v1/memory/procedural", get(procedural))
        .route("/v1/memory/procedural/reinforce", post(reinforce_rule))
}
