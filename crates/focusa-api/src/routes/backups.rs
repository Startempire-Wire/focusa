//! Thin authenticated backup routes backed by the canonical runtime authority.

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{Datelike, Timelike, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::server::AppState;
use focusa_core::runtime::backup::{
    BackupPolicy, backup_health, create_full_generation, list_verified_manifests, verify_generation,
};
use focusa_core::runtime::backup_incremental::create_incremental_generation;
use focusa_core::runtime::backup_offhost::{latest_off_host_receipt, settle_generation_off_host};
use focusa_core::runtime::backup_restore::restore_generation;
use focusa_core::runtime::backup_retention::{execute_retention, plan_retention};

#[derive(Debug, Deserialize)]
struct RunBackupRequest {
    slot_id: Option<String>,
    kind: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VerifyBackupRequest {
    generation_id: String,
}

#[derive(Debug, Deserialize)]
struct RestoreBackupRequest {
    generation_id: String,
}

#[derive(Debug, Deserialize)]
struct PruneBackupRequest {
    #[serde(default = "default_dry_run")]
    dry_run: bool,
}

/// Backup and restore operate on large SQLite snapshots. A dedicated bounded
/// stack prevents debug/static builds and unusually large schemas from
/// exhausting Tokio's general worker stack. The maintenance lock still limits
/// the system to one operation at a time.
pub(crate) async fn spawn_backup_operation<T, F>(
    label: &'static str,
    operation: F,
) -> anyhow::Result<anyhow::Result<T>>
where
    T: Send + 'static,
    F: FnOnce() -> anyhow::Result<T> + Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        let worker = std::thread::Builder::new()
            .name(format!("focusa-backup-{label}"))
            .stack_size(32 * 1024 * 1024)
            .spawn(operation)
            .map_err(|error| anyhow::anyhow!("start {label} worker: {error}"))?;
        worker
            .join()
            .map_err(|_| anyhow::anyhow!("{label} worker panicked"))
    })
    .await
    .map_err(|error| anyhow::anyhow!("join {label} coordinator: {error}"))?
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/backups/health", get(health))
        .route("/v1/backups/generations", get(generations))
        .route("/v1/backups/run", post(run))
        .route("/v1/backups/verify", post(verify))
        .route("/v1/backups/restore-drill", post(restore_drill))
        .route("/v1/backups/settle-off-host", post(settle_off_host))
        .route("/v1/backups/prune", post(prune))
}

async fn health(State(state): State<Arc<AppState>>) -> Json<Value> {
    let data_dir = PathBuf::from(&state.config.data_dir);
    match BackupPolicy::from_env(&data_dir) {
        Ok(policy) => Json(json!(backup_health(&policy))),
        Err(error) => Json(policy_error("health", error)),
    }
}

async fn generations(State(state): State<Arc<AppState>>) -> Json<Value> {
    let data_dir = PathBuf::from(&state.config.data_dir);
    let policy = match BackupPolicy::from_env(&data_dir) {
        Ok(policy) => policy,
        Err(error) => return Json(policy_error("generations", error)),
    };
    match list_verified_manifests(&policy.backup_root) {
        Ok(generations) => Json(json!({
            "schema": "focusa.backup_generation_list.v1",
            "policy_digest": policy.policy_digest,
            "generations": generations,
        })),
        Err(error) => Json(operation_error("list_failed", error)),
    }
}

async fn run(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RunBackupRequest>,
) -> Json<Value> {
    let data_dir = PathBuf::from(&state.config.data_dir);
    let source = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let policy = match BackupPolicy::from_env(&data_dir) {
        Ok(policy) => policy,
        Err(error) => return Json(policy_error("run", error)),
    };
    if !policy.enabled {
        return Json(operation_error_text(
            "policy_disabled",
            "backup policy is disabled",
        ));
    }
    let kind = request.kind.unwrap_or_else(|| "full".to_string());
    if kind != "full" && kind != "incremental_page_delta" {
        return Json(operation_error_text(
            "invalid_generation_kind",
            "kind must be full or incremental_page_delta",
        ));
    }
    let slot_id = request
        .slot_id
        .unwrap_or_else(|| current_slot(&policy, &kind));
    if !valid_id(&slot_id) {
        return Json(operation_error_text(
            "invalid_slot_id",
            "slot_id must be 1-128 ASCII letters, digits, dash, underscore, colon, or dot",
        ));
    }
    let version = env!("CARGO_PKG_VERSION").to_string();
    let result = spawn_backup_operation("run", move || {
        if kind == "full" {
            create_full_generation(&source, &policy, &slot_id, &version)
        } else {
            create_incremental_generation(&source, &policy, &slot_id, &version)
        }
    })
    .await;
    match result {
        Ok(Ok(manifest)) => Json(json!({
            "schema": "focusa.backup_run_result.v1",
            "status": "verified",
            "generation": manifest,
        })),
        Ok(Err(error)) => Json(operation_error("backup_failed", error)),
        Err(error) => Json(operation_error_text("join_failed", &error.to_string())),
    }
}

async fn verify(
    State(state): State<Arc<AppState>>,
    Json(request): Json<VerifyBackupRequest>,
) -> Json<Value> {
    if !valid_id(&request.generation_id) {
        return Json(operation_error_text(
            "invalid_generation_id",
            "generation_id is invalid",
        ));
    }
    let data_dir = PathBuf::from(&state.config.data_dir);
    let policy = match BackupPolicy::from_env(&data_dir) {
        Ok(policy) => policy,
        Err(error) => return Json(policy_error("verify", error)),
    };
    let generation = policy
        .backup_root
        .join("generations")
        .join(&request.generation_id);
    let result = spawn_backup_operation("verify", move || verify_generation(&generation)).await;
    match result {
        Ok(Ok(manifest)) => Json(json!({
            "schema": "focusa.backup_verify_result.v1",
            "status": "verified",
            "generation_id": manifest.generation_id,
            "manifest_sha256": manifest.manifest_sha256,
        })),
        Ok(Err(error)) => Json(operation_error("verification_failed", error)),
        Err(error) => Json(operation_error_text("join_failed", &error.to_string())),
    }
}

async fn prune(
    State(state): State<Arc<AppState>>,
    Json(request): Json<PruneBackupRequest>,
) -> Json<Value> {
    let data_dir = PathBuf::from(&state.config.data_dir);
    let policy = match BackupPolicy::from_env(&data_dir) {
        Ok(policy) => policy,
        Err(error) => return Json(policy_error("prune", error)),
    };
    if request.dry_run {
        return match plan_retention(&policy) {
            Ok(decision) => Json(json!(decision)),
            Err(error) => Json(operation_error("prune_plan_failed", error)),
        };
    }
    let result = spawn_backup_operation("prune", move || execute_retention(&policy)).await;
    match result {
        Ok(Ok(receipt)) => Json(json!({
            "schema": "focusa.backup_prune_result.v1",
            "status": receipt.status,
            "receipt": receipt,
        })),
        Ok(Err(error)) => Json(operation_error("prune_failed", error)),
        Err(error) => Json(operation_error_text("join_failed", &error.to_string())),
    }
}

fn default_dry_run() -> bool {
    true
}

async fn settle_off_host(
    State(state): State<Arc<AppState>>,
    Json(request): Json<VerifyBackupRequest>,
) -> Json<Value> {
    if !valid_id(&request.generation_id) {
        return Json(operation_error_text(
            "invalid_generation_id",
            "generation_id is invalid",
        ));
    }
    let data_dir = PathBuf::from(&state.config.data_dir);
    let policy = match BackupPolicy::from_env(&data_dir) {
        Ok(policy) => policy,
        Err(error) => return Json(policy_error("settle_off_host", error)),
    };
    let Some(remote) = policy.off_host_remote else {
        return Json(operation_error_text(
            "off_host_unconfigured",
            "FOCUSA_BACKUP_OFF_HOST_REMOTE is not configured",
        ));
    };
    let generation_id = request.generation_id;
    let root = policy.backup_root;
    let result = spawn_backup_operation("off-host", move || {
        settle_generation_off_host(&root, &generation_id, &remote)
    })
    .await;
    match result {
        Ok(Ok(receipt)) => Json(json!({
            "schema": "focusa.backup_off_host_result.v1",
            "status": "completed",
            "receipt": receipt,
        })),
        Ok(Err(error)) => Json(operation_error("off_host_settlement_failed", error)),
        Err(error) => Json(operation_error_text("join_failed", &error.to_string())),
    }
}

async fn restore_drill(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RestoreBackupRequest>,
) -> Json<Value> {
    if !valid_id(&request.generation_id) {
        return Json(operation_error_text(
            "invalid_generation_id",
            "generation_id is invalid",
        ));
    }
    let data_dir = PathBuf::from(&state.config.data_dir);
    let policy = match BackupPolicy::from_env(&data_dir) {
        Ok(policy) => policy,
        Err(error) => return Json(policy_error("restore_drill", error)),
    };
    let generation_id = request.generation_id;
    let target = policy.backup_root.join("restore-drills").join(format!(
        "{}-{}.sqlite",
        generation_id,
        uuid::Uuid::now_v7()
    ));
    let root = policy.backup_root.clone();
    let result = spawn_backup_operation("restore", move || {
        restore_generation(&root, &generation_id, &target, policy.rto_seconds)
            .map(|receipt| (receipt, target))
    })
    .await;
    match result {
        Ok(Ok((receipt, target))) => match std::fs::remove_file(&target) {
            Ok(()) => Json(json!({
                "schema": "focusa.backup_restore_drill_result.v1",
                "status": "completed",
                "output_removed": true,
                "receipt": receipt,
            })),
            Err(error) => Json(operation_error_text(
                "restore_cleanup_failed",
                &error.to_string(),
            )),
        },
        Ok(Err(error)) => Json(operation_error("restore_failed", error)),
        Err(error) => Json(operation_error_text("join_failed", &error.to_string())),
    }
}

fn current_hour_slot() -> String {
    let now = Utc::now();
    format!(
        "{:04}-{:02}-{:02}T{:02}",
        now.year(),
        now.month(),
        now.day(),
        now.hour()
    )
}

fn current_slot(policy: &BackupPolicy, kind: &str) -> String {
    let interval = if kind == "full" {
        policy.full_interval_seconds
    } else {
        policy.incremental_interval_seconds
    };
    format!(
        "manual-{kind}-{}",
        Utc::now().timestamp().max(0) as u64 / interval
    )
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':' | b'.'))
}

fn policy_error(operation: &str, error: anyhow::Error) -> Value {
    json!({
        "schema": "focusa.backup_error.v1",
        "status": "blocked",
        "code": "backup_policy_invalid",
        "operation": operation,
        "error": error.to_string(),
        "retry_safe": false,
    })
}

fn operation_error(code: &str, error: anyhow::Error) -> Value {
    operation_error_text(code, &error.to_string())
}

fn operation_error_text(code: &str, error: &str) -> Value {
    json!({
        "schema": "focusa.backup_error.v1",
        "status": "blocked",
        "code": code,
        "error": error,
        "retry_safe": false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_bounded_and_path_safe() {
        assert!(valid_id("2026-08-31T16"));
        assert!(valid_id("full-abc_123.def"));
        assert!(!valid_id("../escape"));
        assert!(!valid_id("with/slash"));
        assert!(!valid_id(""));
        assert!(!valid_id(&"a".repeat(129)));
    }

    #[test]
    fn hour_slots_are_bounded_ids() {
        assert!(valid_id(&current_hour_slot()));
    }

    #[test]
    fn backup_root_join_remains_under_generation_directory() {
        let root = Path::new("/backup/focusa");
        let generation = "full-abc";
        assert!(valid_id(generation));
        assert!(root.join("generations").join(generation).starts_with(root));
    }
}
