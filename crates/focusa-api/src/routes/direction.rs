//! Direction Workbench API — #291 slice 3: typed steering/adjudication/
//! review operations with receipts, projected from the ledger.

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::routing::{get, post};
use focusa_core::direction_operations::DirectionOperation;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/direction/operations", post(record).get(list))
}

async fn record(
    State(state): State<Arc<AppState>>,
    Json(operation): Json<DirectionOperation>,
) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::direction_ledger::ensure_schema(&conn)?;
        let receipt = focusa_core::direction_ledger::record_operation(&conn, &operation)?;
        Ok(json!({"status": "recorded", "receipt": receipt}))
    })
    .await;
    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(focusa_core::error_envelope::standard_error(
            "rejected",
            "direction_verification_failed",
            "do_not_retry_unchanged",
            "add the required typed fields or evidence ref",
            &error.to_string(),
        )),
        Err(error) => Json(focusa_core::error_envelope::internal_error(
            "join",
            &format!("{error}"),
        )),
    }
}

async fn list(State(state): State<Arc<AppState>>) -> Json<Value> {
    let path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(path)?;
        focusa_core::direction_ledger::ensure_schema(&conn)?;
        let operations = focusa_core::direction_ledger::list_operations(&conn)?;
        Ok(json!({"status": "ok", "operations": operations}))
    })
    .await;
    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error(
            "route",
            &error.to_string(),
        )),
        Err(error) => Json(focusa_core::error_envelope::internal_error(
            "join",
            &format!("{error}"),
        )),
    }
}
