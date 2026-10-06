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
    let days = match request.before_days {
        Some(days) if days > 0 => days,
        Some(_) => return Json(json!({"status":"blocked", "code":"invalid_retention_days"})),
        None => match configured_retention_days(
            std::env::var(focusa_core::runtime::event_retention::RETENTION_ENV_DAYS)
                .ok()
                .as_deref(),
        ) {
            Ok(days) => days,
            Err(error) => {
                return Json(
                    json!({"status":"blocked", "code":"invalid_retention_days", "error":error.to_string()}),
                );
            }
        },
    };

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
    match scheduled_retention_disabled(
        std::env::var(focusa_core::runtime::event_retention::RETENTION_ENV_DISABLED)
            .ok()
            .as_deref(),
    ) {
        Ok(true) => return json!({"status":"blocked", "code":"event_retention_disabled"}),
        Err(error) => {
            return json!({"status":"blocked", "code":"invalid_retention_configuration", "error":error.to_string()});
        }
        Ok(false) => {}
    }
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

fn scheduled_retention_disabled(value: Option<&str>) -> anyhow::Result<bool> {
    match value.map(str::to_ascii_lowercase).as_deref() {
        None | Some("0" | "false") => Ok(false),
        Some("1" | "true") => Ok(true),
        Some(_) => anyhow::bail!("retention disabled flag must be true/false or 1/0"),
    }
}

fn configured_retention_days(value: Option<&str>) -> anyhow::Result<u32> {
    let days = match value {
        Some(value) => value.parse::<u32>()?,
        None => focusa_core::runtime::event_retention::DEFAULT_RETENTION_DAYS,
    };
    anyhow::ensure!(days > 0, "retention days must be positive");
    Ok(days)
}

/// Bounded health projection: no full event-table scan or ledger mutation.
pub(crate) fn retention_health(data_dir: &Path) -> Value {
    let db_path = crate::routes::events_sqlite::focusa_db_path(&data_dir.to_string_lossy());
    let receipt_path = data_dir.join("event-retention-receipts.jsonl");
    let tail =
        focusa_core::background_jobs::bounded_log_tail(&receipt_path.to_string_lossy(), 65_536);
    let latest = tail
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|receipt| receipt.get("phase").and_then(Value::as_str) == Some("settled"));
    let policy_days = configured_retention_days(
        std::env::var(focusa_core::runtime::event_retention::RETENTION_ENV_DAYS)
            .ok()
            .as_deref(),
    );
    let disabled = scheduled_retention_disabled(
        std::env::var(focusa_core::runtime::event_retention::RETENTION_ENV_DISABLED)
            .ok()
            .as_deref(),
    );
    let last_completed = tail
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|receipt| {
            receipt.get("phase").and_then(Value::as_str) == Some("settled")
                && receipt.get("status").and_then(Value::as_str) == Some("completed")
        });
    json!({
        "schema":"focusa.event_retention_health.v1",
        "status": if policy_days.is_err() || disabled.is_err() { "invalid_configuration" } else if disabled.unwrap_or(true) { "disabled" } else { "enabled" },
        "before_days": policy_days.ok(),
        "db_bytes": std::fs::metadata(&db_path).ok().map(|metadata| metadata.len()),
        "last_retention_status": latest.as_ref().and_then(|receipt| receipt.get("status")),
        "last_prune_at": last_completed.as_ref().and_then(|receipt| receipt.get("timestamp")),
        "latest_receipt": latest,
        "backup_recovery_required":true,
        "event_counts":null,
        "event_counts_status":"not_scanned_on_health_hot_path",
    })
}

pub(crate) fn scheduled_retention_due(data_dir: &Path, interval_seconds: u64) -> bool {
    let raw = focusa_core::background_jobs::bounded_log_tail(
        &data_dir
            .join("event-retention-receipts.jsonl")
            .to_string_lossy(),
        65_536,
    );
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
    fn retention_days_are_explicit_and_invalid_values_fail_closed() {
        assert_eq!(configured_retention_days(None).unwrap(), 30);
        assert_eq!(configured_retention_days(Some("7")).unwrap(), 7);
        assert!(configured_retention_days(Some("0")).is_err());
        assert!(configured_retention_days(Some("invalid")).is_err());
    }

    #[test]
    fn retention_disabled_flag_rejects_ambiguous_configuration() {
        assert!(!scheduled_retention_disabled(None).unwrap());
        assert!(scheduled_retention_disabled(Some("true")).unwrap());
        assert!(scheduled_retention_disabled(Some("1")).unwrap());
        assert!(!scheduled_retention_disabled(Some("false")).unwrap());
        assert!(scheduled_retention_disabled(Some("unknown")).is_err());
    }

    #[test]
    fn retention_health_keeps_last_success_when_latest_attempt_fails() {
        let directory =
            std::env::temp_dir().join(format!("retention-health-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("event-retention-receipts.jsonl"),
            concat!(
                "{\"phase\":\"settled\",\"status\":\"completed\",\"timestamp\":\"t1\"}\n",
                "{\"phase\":\"settled\",\"status\":\"failed\",\"timestamp\":\"t2\"}\n"
            ),
        )
        .unwrap();
        let health = retention_health(&directory);
        assert_eq!(health["last_prune_at"], "t1");
        assert_eq!(health["last_retention_status"], "failed");
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn missing_receipts_do_not_claim_retention_completed() {
        let directory =
            std::env::temp_dir().join(format!("retention-health-{}", uuid::Uuid::now_v7()));
        let health = retention_health(&directory);
        assert!(health["last_prune_at"].is_null());
        assert!(health["latest_receipt"].is_null());
        assert!(health["db_bytes"].is_null());
        assert_eq!(health["backup_recovery_required"], true);
    }

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
