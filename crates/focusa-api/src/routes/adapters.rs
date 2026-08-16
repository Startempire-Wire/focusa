//! Adapter registry routes (#254 slice 10): harnesses register capability
//! sets; CallGraph dispatch routes against the ledger-backed registry.

use axum::extract::State;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use focusa_core::adapter_registry::AdapterRecord;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/adapters", post(register).get(list))
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(record): Json<AdapterRecord>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::adapter_registry::ensure_schema(&conn)?;
        focusa_core::adapter_registry::upsert_adapter(&conn, &record)?;
        Ok(json!({"status": "registered", "adapter_id": record.adapter_id, "model": record.model}))
    })
    .await;
    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(json!({"status": "failed", "error": error.to_string()})),
        Err(error) => Json(json!({"status": "failed", "error": format!("join error: {error}")})),
    }
}

async fn list(State(state): State<Arc<AppState>>) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::adapter_registry::ensure_schema(&conn)?;
        let adapters = focusa_core::adapter_registry::list_adapters(&conn)?;
        Ok(json!({"status": "ok", "adapters": adapters}))
    })
    .await;
    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(json!({"status": "failed", "error": error.to_string()})),
        Err(error) => Json(json!({"status": "failed", "error": format!("join error: {error}")})),
    }
}
