//! Single daemon maintenance coordinator for backup, restore, off-host,
//! retention, and event-ledger hygiene.

use crate::server::AppState;
use chrono::Utc;
use focusa_core::runtime::backup::{
    BackupPolicy, backup_health, create_full_generation, list_verified_manifests,
};
use focusa_core::runtime::backup_incremental::create_incremental_generation;
use focusa_core::runtime::backup_offhost::{latest_off_host_receipt, settle_generation_off_host};
use focusa_core::runtime::backup_restore::restore_generation;
use focusa_core::runtime::backup_retention::{execute_retention, retention_due};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub(crate) async fn maintenance_loop(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    // Tokio intervals yield their first tick immediately. Consume it so a
    // daemon restart never launches backup, restore, off-host, and retention
    // work in the same readiness burst.
    interval.tick().await;
    loop {
        interval.tick().await;
        if let Err(error) = maintenance_once(state.clone()).await {
            tracing::error!(error = %error, "backup maintenance cycle failed");
        }
    }
}

async fn maintenance_once(state: Arc<AppState>) -> anyhow::Result<()> {
    let data_dir = PathBuf::from(&state.config.data_dir);
    let policy = BackupPolicy::from_env(&data_dir)?;
    if !policy.enabled {
        return Ok(());
    }
    if let Some((mode, reason)) = crate::server::lowmem_background_throttle() {
        tracing::warn!(
            mode,
            reason,
            "backup maintenance deferred by resource policy"
        );
        return Ok(());
    }
    let source = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let version = env!("CARGO_PKG_VERSION").to_string();
    let now = Utc::now();
    let full_slot = format!(
        "full-slot-{}",
        now.timestamp().max(0) as u64 / policy.full_interval_seconds
    );
    let mut manifests = list_verified_manifests(&policy.backup_root)?;
    if !manifests
        .iter()
        .any(|manifest| manifest.slot_id == full_slot)
    {
        let source = source.clone();
        let operation_policy = policy.clone();
        let version = version.clone();
        let slot = full_slot.clone();
        let manifest = super::backups::spawn_backup_operation("scheduled-full", move || {
            create_full_generation(&source, &operation_policy, &slot, &version)
        })
        .await??;
        tracing::info!(generation_id = %manifest.generation_id, "verified full backup committed");
        manifests = list_verified_manifests(&policy.backup_root)?;
    }

    if policy.incremental_strategy == "experimental_full_snapshot_chunks_v0" {
        let recovery_slot_number =
            Utc::now().timestamp().max(0) as u64 / policy.incremental_interval_seconds;
        let recovery_in_slot = manifests.iter().any(|manifest| {
            manifest.completed_at.timestamp().max(0) as u64 / policy.incremental_interval_seconds
                == recovery_slot_number
        });
        if !recovery_in_slot {
            let slot = format!("recovery-slot-{recovery_slot_number}");
            let source = source.clone();
            let operation_policy = policy.clone();
            let version = version.clone();
            let manifest =
                super::backups::spawn_backup_operation("scheduled-incremental", move || {
                    create_incremental_generation(&source, &operation_policy, &slot, &version)
                })
                .await??;
            tracing::info!(generation_id = %manifest.generation_id, "experimental snapshot-chunk recovery point committed");
            manifests = list_verified_manifests(&policy.backup_root)?;
        }
    }

    manifests.sort_by_key(|manifest| manifest.completed_at);
    if let (Some(remote), Some(generation)) = (policy.off_host_remote.clone(), manifests.last()) {
        if latest_off_host_receipt(&policy.backup_root, &generation.generation_id).is_none() {
            let root = policy.backup_root.clone();
            let generation_id = generation.generation_id.clone();
            let receipt = super::backups::spawn_backup_operation("scheduled-off-host", move || {
                settle_generation_off_host(&root, &generation_id, &remote)
            })
            .await??;
            tracing::info!(generation_id = %receipt.generation_id, "backup off-host settlement verified");
        }
    }

    let health = backup_health(&policy);
    if health.restore_status != "ok" {
        manifests.sort_by_key(|manifest| manifest.completed_at);
        let restore_candidate = manifests
            .iter()
            .rev()
            .find(|manifest| manifest.generation_kind == "incremental_page_delta")
            .or_else(|| manifests.last());
        if let Some(generation) = restore_candidate {
            let generation_id = generation.generation_id.clone();
            let target = policy.backup_root.join("restore-drills").join(format!(
                "{}-{}.sqlite",
                generation_id,
                uuid::Uuid::now_v7()
            ));
            let root = policy.backup_root.clone();
            let rto_seconds = policy.rto_seconds;
            let receipt = super::backups::spawn_backup_operation("scheduled-restore", move || {
                restore_generation(&root, &generation_id, &target, rto_seconds)
                    .map(|receipt| (receipt, target))
            })
            .await??;
            if let Err(error) = std::fs::remove_file(&receipt.1) {
                tracing::error!(error = %error, target = %receipt.1.display(), "failed to remove isolated restore drill output");
            }
            tracing::info!(generation_id = %receipt.0.generation_id, "backup restore drill completed");
        }
    }
    if retention_due(&policy.backup_root, 86_400) {
        let policy = policy.clone();
        let receipt = super::backups::spawn_backup_operation("scheduled-prune", move || {
            execute_retention(&policy)
        })
        .await??;
        tracing::info!(
            status = %receipt.status,
            deleted_generations = receipt.deleted_generation_ids.len(),
            "backup retention cycle settled"
        );
    }
    let health = backup_health(&policy);
    let event_retention_safe = health.rpo_status == "ok"
        && health.restore_status == "ok"
        && (health.off_host_status == "ok" || health.off_host_status == "not_required");
    if event_retention_safe
        && crate::routes::events_retention::scheduled_retention_due(&data_dir, 86_400)
    {
        let result = crate::routes::events_retention::run_scheduled_retention(state).await;
        if result.get("status").and_then(serde_json::Value::as_str) == Some("blocked") {
            return Err(anyhow::anyhow!(
                "scheduled event retention was blocked: {result}"
            ));
        }
        tracing::info!("governed daily event-retention cycle settled");
    }
    Ok(())
}
