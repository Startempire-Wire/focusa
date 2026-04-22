//! Memory routes.
//!
//! GET  /v1/memory/semantic             — list semantic memory
//! POST /v1/memory/semantic/upsert      — upsert a key=value
//! GET  /v1/memory/procedural           — list procedural rules
//! POST /v1/memory/procedural/reinforce — reinforce a rule

use crate::server::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use focusa_core::types::{Action, MemorySource, SemanticRecord};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

const MAX_SEMANTIC_LIMIT: usize = 512;

#[derive(Debug, Clone, Deserialize, Default)]
struct SemanticQuery {
    limit: Option<usize>,
    #[serde(default)]
    summary_only: bool,
}

fn limit_tail<T: Clone>(items: &[T], limit: Option<usize>) -> Vec<T> {
    match limit.map(|value| value.min(MAX_SEMANTIC_LIMIT)) {
        Some(0) => Vec::new(),
        Some(n) => items
            .iter()
            .rev()
            .take(n)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect(),
        None => items.to_vec(),
    }
}

fn semantic_summary(records: &[SemanticRecord]) -> Vec<serde_json::Value> {
    records
        .iter()
        .map(|record| {
            json!({
                "key": record.key,
                "value": record.value,
                "updated_at": record.updated_at,
                "pinned": record.pinned,
            })
        })
        .collect()
}

async fn semantic(Query(query): Query<SemanticQuery>, State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    let total = focusa.memory.semantic.len();
    let semantic = limit_tail(&focusa.memory.semantic, query.limit);
    Json(json!({
        "semantic": if query.summary_only { json!(semantic_summary(&semantic)) } else { json!(semantic) },
        "count": total,
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

#[cfg(test)]
mod tests {
    use super::{limit_tail, semantic_summary};
    use chrono::Utc;
    use focusa_core::types::{MemorySource, SemanticRecord};

    fn record(key: &str, value: &str, pinned: bool) -> SemanticRecord {
        SemanticRecord {
            key: key.to_string(),
            value: value.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            source: MemorySource::User,
            confidence: 1.0,
            ttl: None,
            tags: vec![],
            pinned,
        }
    }

    #[test]
    fn limit_tail_keeps_most_recent_records() {
        let items = vec![record("a", "1", false), record("b", "2", true), record("c", "3", false)];
        let limited = limit_tail(&items, Some(2));
        assert_eq!(limited.len(), 2);
        assert_eq!(limited[0].key, "b");
        assert_eq!(limited[1].key, "c");
    }

    #[test]
    fn semantic_summary_strips_heavy_fields() {
        let items = vec![record("alpha", "beta", true)];
        let summary = semantic_summary(&items);
        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0]["key"], "alpha");
        assert_eq!(summary[0]["value"], "beta");
        assert_eq!(summary[0]["pinned"], true);
        assert!(summary[0].get("tags").is_none());
        assert!(summary[0].get("confidence").is_none());
    }
}
