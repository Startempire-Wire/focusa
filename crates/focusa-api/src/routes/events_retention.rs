//! Event-ledger retention route: prune epoch-junk placeholders or export and
//! prune events older than the configured hot window. Long deletes run on
//! `spawn_blocking` in bounded batches so the daemon writer stays responsive.

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use std::io::Write;
use std::path::{Path, PathBuf};
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
    let dry_run = request.dry_run.unwrap_or(true);
    let batch_size = request.batch_size.unwrap_or(5_000).clamp(100, 100_000);
    let epoch_junk = request.epoch_junk.unwrap_or(false);
    let export = request.export.unwrap_or(true);
    let days = request
        .before_days
        .unwrap_or(focusa_core::runtime::event_retention::DEFAULT_RETENTION_DAYS);

    let mut backup_guard = None;
    if !dry_run {
        if !epoch_junk && !export {
            return Json(json!({
                "status": "blocked",
                "code": "cold_export_required",
                "error": "event retention cannot delete governed events without a cold export",
            }));
        }
        let policy = match focusa_core::runtime::backup::BackupPolicy::from_env(
            std::path::Path::new(&state.config.data_dir),
        ) {
            Ok(policy) => policy,
            Err(error) => {
                return Json(json!({
                    "status": "blocked",
                    "code": "backup_policy_invalid",
                    "error": error.to_string(),
                }));
            }
        };
        let health = focusa_core::runtime::backup::backup_health(&policy);
        if !health.enabled
            || health.rpo_status != "ok"
            || health.restore_status != "ok"
            || (health.off_host_status != "ok" && health.off_host_status != "not_required")
        {
            return Json(json!({
                "status": "blocked",
                "code": "backup_recovery_gate_not_met",
                "backup_health": health,
                "error": "fresh, restore-proven, off-host-settled backup required before event retention",
            }));
        }
        backup_guard = health.last_verified_generation_id.clone();
    }

    let retention_run_id = uuid::Uuid::now_v7().to_string();
    let receipt_path = PathBuf::from(&state.config.data_dir).join("event-retention-receipts.jsonl");
    if !dry_run {
        let planned = json!({
            "schema": "focusa.event_retention_receipt.v1",
            "run_id": retention_run_id,
            "phase": "planned",
            "status": "planned",
            "timestamp": chrono::Utc::now(),
            "backup_generation_id": backup_guard,
            "epoch_junk": epoch_junk,
            "before_days": days,
            "export": export,
            "batch_size": batch_size,
        });
        if let Err(error) = append_receipt(&receipt_path, &planned) {
            return Json(focusa_core::error_envelope::internal_error(
                "event_retention_receipt",
                &error.to_string(),
            ));
        }
    }

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

    let (value, phase, status) = match result {
        Ok(Ok(value)) => (value, "settled", "completed"),
        Ok(Err(error)) => (
            focusa_core::error_envelope::internal_error("route", &error.to_string()),
            "settled",
            "failed",
        ),
        Err(error) => (
            focusa_core::error_envelope::internal_error("join", &format!("join error: {error}")),
            "settled",
            "failed",
        ),
    };
    if !dry_run {
        let receipt = json!({
            "schema": "focusa.event_retention_receipt.v1",
            "run_id": retention_run_id,
            "phase": phase,
            "status": status,
            "timestamp": chrono::Utc::now(),
            "result": &value,
        });
        if let Err(error) = append_receipt(&receipt_path, &receipt) {
            return Json(focusa_core::error_envelope::internal_error(
                "event_retention_receipt",
                &error.to_string(),
            ));
        }
    }
    Json(value)
}

pub(crate) async fn run_scheduled_retention(state: Arc<AppState>) -> Value {
    prune(
        State(state),
        Json(PruneRequest {
            epoch_junk: Some(false),
            before_days: None,
            export: Some(true),
            dry_run: Some(false),
            batch_size: Some(focusa_core::runtime::event_retention::DEFAULT_BATCH_SIZE),
        }),
    )
    .await
    .0
}

pub(crate) fn scheduled_retention_due(data_dir: &Path, interval_seconds: u64) -> bool {
    let raw = match std::fs::read_to_string(data_dir.join("event-retention-receipts.jsonl")) {
        Ok(raw) => raw,
        Err(_) => return true,
    };
    let latest = raw
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|receipt| receipt.get("status").and_then(Value::as_str) == Some("completed"))
        .filter_map(|receipt| {
            receipt
                .get("timestamp")
                .and_then(Value::as_str)
                .and_then(|timestamp| chrono::DateTime::parse_from_rfc3339(timestamp).ok())
        })
        .max();
    latest.is_none_or(|completed_at| {
        (chrono::Utc::now() - completed_at.with_timezone(&chrono::Utc)).num_seconds()
            >= interval_seconds as i64
    })
}

fn append_receipt(path: &Path, receipt: &Value) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    serde_json::to_writer(&mut file, receipt)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutation_defaults_to_dry_run() {
        let request = PruneRequest {
            epoch_junk: None,
            before_days: None,
            export: None,
            dry_run: None,
            batch_size: None,
        };
        assert!(request.dry_run.unwrap_or(true));
        assert!(request.export.unwrap_or(true));
    }

    #[test]
    fn scheduled_retention_uses_durable_completed_receipt_cadence() {
        let directory = std::env::temp_dir().join(format!(
            "focusa-event-retention-due-{}",
            uuid::Uuid::now_v7()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        assert!(scheduled_retention_due(&directory, 86_400));
        let receipt = json!({
            "schema": "focusa.event_retention_receipt.v1",
            "status": "completed",
            "timestamp": chrono::Utc::now(),
        });
        append_receipt(&directory.join("event-retention-receipts.jsonl"), &receipt).unwrap();
        assert!(!scheduled_retention_due(&directory, 86_400));
        std::fs::remove_dir_all(directory).unwrap();
    }
}
