//! Adapter registry routes (#254 slice 10): harnesses register capability
//! sets; CallGraph dispatch routes against the ledger-backed registry.

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::routing::{get, post};
use focusa_core::adapter_registry::AdapterRecord;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/adapters", post(register).get(list))
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(record): Json<AdapterRecord>,
) -> Json<Value> {
    // Trust boundary: the daemon binds loopback-only; registration feeds
    // ROUTING decisions, so reject empty identities and implausible
    // capability counts (bounded registry — never unbounded growth).
    if record.adapter_id.trim().is_empty() || record.model.trim().is_empty() {
        return Json(focusa_core::error_envelope::standard_error(
            "rejected",
            "adapter_identity_invalid",
            "do_not_retry_unchanged",
            "supply a non-empty adapter_id and model",
            "adapter registration requires non-empty identity",
        ));
    }
    if record.capabilities.len() > 64 {
        return Json(focusa_core::error_envelope::standard_error(
            "rejected",
            "adapter_capabilities_unbounded",
            "do_not_retry_unchanged",
            "register at most 64 capability refs per adapter",
            "capability count exceeds the registry bound",
        ));
    }
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
