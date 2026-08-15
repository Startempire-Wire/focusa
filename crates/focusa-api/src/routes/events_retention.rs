//! Event-ledger retention route: prune epoch-junk placeholders or export and
//! prune events older than the configured hot window. Long deletes run on
//! `spawn_blocking` in bounded batches so the daemon writer stays responsive.

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::server::AppState;

#[derive(Deserialize)]
pub struct PruneRequest {
    /// Remove placeholder events with epoch-0 timestamps (retired temporal
    /// fallback junk). Ignored unless true.
    pub epoch_junk: Option<bool>,
    /// Hot-window days; events older than this are exported and pruned.
    pub before_days: Option<u32>,
    /// Export pruned events to cold JSONL under <data>/events-cold.
    pub export: Option<bool>,
    /// Compute the cutoff and counts without mutating.
    pub dry_run: Option<bool>,
    /// Batch size for the delete loop.
    pub batch_size: Option<usize>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/events/prune", post(prune))
}

async fn prune(
    State(state): State<Arc<AppState>>,
    Json(request): Json<PruneRequest>,
) -> Json<Value> {
    let db_path = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let data_dir = state.config.data_dir.clone();
    let dry_run = request.dry_run.unwrap_or(false);
    let batch_size = request.batch_size.unwrap_or(5_000).clamp(100, 100_000);
    let epoch_junk = request.epoch_junk.unwrap_or(false);
    let export = request.export.unwrap_or(true);
    let days = request.before_days.unwrap_or(focusa_core::runtime::event_retention::DEFAULT_RETENTION_DAYS);

    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        if epoch_junk {
            let conn = rusqlite::Connection::open(&db_path)?;
            conn.busy_timeout(std::time::Duration::from_secs(30))?;
            let remaining: i64 = conn.query_row(
                "SELECT COUNT(*) FROM events WHERE ts < ?1",
                [focusa_core::runtime::event_retention::JUNK_CUTOFF],
                |row| row.get(0),
            )?;
            if dry_run {
                return Ok(json!({"dry_run": true, "epoch_junk_eligible": remaining}));
            }
            let summary =
                focusa_core::runtime::event_retention::prune_epoch_junk(&conn, batch_size)?;
            return Ok(json!({"pruned_epoch_junk": summary, "remaining_events": remaining - summary.deleted_events as i64}));
        }
        let cutoff = focusa_core::runtime::event_retention::retention_cutoff(days);
        if dry_run {
            return Ok(json!({"dry_run": true, "cutoff": cutoff}));
        }
        let conn = rusqlite::Connection::open(&db_path)?;
        conn.busy_timeout(std::time::Duration::from_secs(30))?;
        let export_dir = if export {
            Some(std::path::PathBuf::from(&data_dir).join("events-cold"))
        } else {
            None
        };
        let summary = focusa_core::runtime::event_retention::prune_before(
            &conn,
            &cutoff,
            export_dir.as_deref(),
            batch_size,
        )?;
        Ok(json!({"pruned_before": summary}))
    })
    .await;

    match result {
        Ok(Ok(value)) => Json(value),
        Ok(Err(error)) => Json(json!({"status": "failed", "error": error.to_string()})),
        Err(error) => Json(json!({"status": "failed", "error": format!("join error: {error}")})),
    }
}
