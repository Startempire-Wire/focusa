//! Application-consistent Focusa backup generation runtime.
//!
//! Full generations use SQLite's online backup API and never stop the daemon.
//! P1 health intentionally reports an RPO breach until bounded incremental
//! recovery points are implemented and restore-proven (Spec 181 §9).

pub use super::backup_contracts::*;
use super::backup_io::*;

use anyhow::{Context, Result, bail};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, backup::Backup};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::ffi::CString;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, UNIX_EPOCH};
use uuid::Uuid;

pub fn create_full_generation(
    source_database: &Path,
    policy: &BackupPolicy,
    slot_id: &str,
    runtime_version: &str,
) -> Result<BackupGenerationManifest> {
    if !policy.enabled {
        bail!("backup policy is disabled")
    }
    let source = canonical_regular_file(source_database)?;
    let data_dir = source.parent().context("database parent missing")?;
    policy.validate(data_dir)?;
    prepare_root(&policy.backup_root, data_dir)?;
    let _lock = MaintenanceLock::acquire(&policy.backup_root)?;
    let (free, capacity) = filesystem_space(&policy.backup_root)?;
    let free_percent = if capacity == 0 {
        0
    } else {
        ((free as u128 * 100) / capacity as u128) as u8
    };
    if free < policy.min_free_bytes || free_percent < policy.min_free_percent {
        bail!("backup headroom gate failed: {free} bytes and {free_percent}% free")
    }

    let source_identity = file_identity(&source)?;
    let generation_id =
        deterministic_generation_id(slot_id, &source_identity, &policy.policy_digest);
    let generation_dir = policy.backup_root.join("generations").join(&generation_id);
    if generation_dir.exists() {
        return verify_generation(&generation_dir);
    }

    let run_id = Uuid::now_v7().to_string();
    let receipts = policy.backup_root.join("receipts/backup-receipts.jsonl");
    let planned_receipt = BackupReceipt {
        schema: BACKUP_RECEIPT_SCHEMA.to_string(),
        run_id: run_id.clone(),
        generation_id: generation_id.clone(),
        slot_id: slot_id.to_string(),
        phase: "planned".to_string(),
        status: "planned".to_string(),
        timestamp: Utc::now(),
        policy_digest: policy.policy_digest.clone(),
        source_database: source.display().to_string(),
        bytes: 0,
        artifact_sha256: None,
        quick_check: None,
        error_code: None,
        error: None,
    };
    append_receipt(&receipts, &planned_receipt)?;
    let mut failure_guard = FailureReceiptGuard::new(receipts.clone(), planned_receipt.clone());

    let created_at = Utc::now();
    let staging = policy
        .backup_root
        .join("staging")
        .join(format!("{generation_id}-{run_id}"));
    create_private_dir(&staging)?;
    let staging_db = staging.join("focusa.sqlite");
    append_progress_receipt(&receipts, &planned_receipt, "snapshot_started", 0, None)?;
    online_backup(&source, &staging_db)?;
    let snapshot = Connection::open(&staging_db).context("open staged backup")?;
    let quick_check: String = snapshot.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if quick_check != "ok" {
        bail!("staged backup quick_check failed: {quick_check}")
    }
    append_progress_receipt(
        &receipts,
        &planned_receipt,
        "snapshot_completed",
        fs::metadata(&staging_db)?.len(),
        Some(quick_check.clone()),
    )?;
    let page_size = pragma_u64(&snapshot, "page_size")?;
    let page_count = pragma_u64(&snapshot, "page_count")?;
    let event_count = query_u64_or_zero(&snapshot, "SELECT COUNT(*) FROM events")?;
    let chain: Option<(i64, String)> = snapshot
        .query_row(
            "SELECT chain_index, event_hash FROM event_hash_chain ORDER BY chain_index DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let schema_version = query_meta(&snapshot, "schema_version")?;
    if schema_version.is_none() {
        bail!("staged backup is missing required schema_version metadata")
    }
    let chain_anchor = query_meta(&snapshot, "event_chain_anchor")?;

    let ecs_digest = inventory_digest(&data_dir.join("ecs"))?;
    let cold_digest = inventory_digest(&data_dir.join("events-cold"))?;
    if file_identity(&source)? != source_identity {
        bail!("source database identity drifted during backup")
    }
    let source_snapshot_sha256 = sha256_file(&staging_db)?;
    let compressed = staging.join("focusa.sqlite.zst");
    compress_file(&staging_db, &compressed, policy.compression_level)?;
    let artifact = BackupArtifact {
        path: "focusa.sqlite.zst".to_string(),
        bytes: fs::metadata(&compressed)?.len(),
        sha256: sha256_file(&compressed)?,
        media_type: "application/vnd.sqlite3".to_string(),
        compression: "zstd".to_string(),
    };
    fs::remove_file(&staging_db).context("remove uncompressed staging database")?;

    let mut manifest = BackupGenerationManifest {
        schema: BACKUP_MANIFEST_SCHEMA.to_string(),
        generation_id: generation_id.clone(),
        slot_id: slot_id.to_string(),
        generation_kind: "full".to_string(),
        state: "verified".to_string(),
        created_at,
        completed_at: Utc::now(),
        source_database: source.display().to_string(),
        source_file_identity: source_identity,
        runtime_version: runtime_version.to_string(),
        schema_version,
        page_size,
        page_count,
        event_count,
        event_chain_index: chain.as_ref().map(|value| value.0),
        event_chain_hash: chain.map(|value| value.1),
        persisted_chain_anchor: chain_anchor,
        ecs_inventory_digest: ecs_digest,
        cold_export_inventory_digest: cold_digest,
        artifacts: vec![artifact.clone()],
        chunks: Vec::new(),
        source_snapshot_sha256,
        parent_generation_id: None,
        policy_digest: policy.policy_digest.clone(),
        off_host_settlement_ref: None,
        restore_receipt_ref: None,
        manifest_sha256: String::new(),
    };
    manifest.manifest_sha256 = digest_serializable(&manifest)?;
    write_json_atomic(&staging.join("manifest.json"), &manifest)?;
    sync_dir(&staging)?;
    fs::rename(&staging, &generation_dir).context("commit backup generation")?;
    sync_dir(
        generation_dir
            .parent()
            .context("generation parent missing")?,
    )?;
    append_receipt(
        &receipts,
        &BackupReceipt {
            schema: BACKUP_RECEIPT_SCHEMA.to_string(),
            run_id,
            generation_id,
            slot_id: slot_id.to_string(),
            phase: "verified".to_string(),
            status: "completed".to_string(),
            timestamp: Utc::now(),
            policy_digest: policy.policy_digest.clone(),
            source_database: source.display().to_string(),
            bytes: artifact.bytes,
            artifact_sha256: Some(artifact.sha256),
            quick_check: Some(quick_check),
            error_code: None,
            error: None,
        },
    )?;
    failure_guard.settled = true;
    Ok(manifest)
}

pub fn backup_health(policy: &BackupPolicy) -> BackupHealth {
    let mut manifests = list_verified_manifests(&policy.backup_root).unwrap_or_default();
    manifests.sort_by_key(|manifest| manifest.completed_at);
    let latest = manifests.last();
    let latest_full = manifests
        .iter()
        .rev()
        .find(|manifest| manifest.generation_kind == "full");
    let latest_restore = latest_completed_restore(&policy.backup_root);
    let now = Utc::now();
    let full_status = latest_full
        .map(|manifest| {
            if (now - manifest.completed_at).num_seconds()
                <= policy.full_interval_seconds.saturating_mul(2) as i64
            {
                "ok"
            } else {
                "stale"
            }
        })
        .unwrap_or("missing")
        .to_string();
    let restore_status = latest_restore
        .as_ref()
        .map(|receipt| {
            if (now - receipt.completed_at).num_seconds() <= policy.restore_interval_seconds as i64
            {
                "ok"
            } else {
                "overdue"
            }
        })
        .unwrap_or("missing")
        .to_string();
    let incremental_restore_proven = latest_restore.as_ref().is_some_and(|receipt| {
        receipt.generation_kind == "incremental_page_delta"
            && receipt.rto_status == "met"
            && (now - receipt.completed_at).num_seconds() <= policy.restore_interval_seconds as i64
    });
    let rpo_status = if !policy.enabled {
        "disabled"
    } else if policy.incremental_strategy != "content_addressed_page_sink_v1" {
        "breach_incremental_not_implemented"
    } else if latest.is_none() {
        "breach_no_recovery_point"
    } else if latest.is_some_and(|manifest| {
        (now - manifest.completed_at).num_seconds() > policy.rpo_seconds as i64
    }) {
        "breach_stale_recovery_point"
    } else if !incremental_restore_proven {
        "breach_restore_unproven"
    } else {
        "ok"
    }
    .to_string();
    let off_host_status = if !policy.off_host_required {
        "not_required"
    } else if latest.is_some_and(|manifest| {
        super::backup_offhost::latest_off_host_receipt(&policy.backup_root, &manifest.generation_id)
            .is_some_and(|receipt| receipt.manifest_sha256 == manifest.manifest_sha256)
    }) {
        "ok"
    } else if policy.off_host_remote.is_none() {
        "required_unconfigured"
    } else {
        "required_unsettled"
    }
    .to_string();
    let overall_status = if !policy.enabled {
        "disabled"
    } else if rpo_status == "ok"
        && full_status == "ok"
        && restore_status == "ok"
        && (off_host_status == "ok" || off_host_status == "not_required")
    {
        "ok"
    } else if latest.is_some() {
        "degraded"
    } else {
        "failing"
    }
    .to_string();
    BackupHealth {
        schema: BACKUP_HEALTH_SCHEMA.to_string(),
        enabled: policy.enabled,
        policy_digest: policy.policy_digest.clone(),
        backup_root: policy.backup_root.display().to_string(),
        last_verified_generation_id: latest.map(|manifest| manifest.generation_id.clone()),
        last_verified_at: latest.map(|manifest| manifest.completed_at),
        verified_generation_count: manifests.len(),
        free_bytes: available_bytes(&policy.backup_root).ok(),
        overall_status,
        full_status,
        rpo_status,
        off_host_status,
        restore_status,
        last_failure: latest
            .is_none()
            .then(|| "no verified generation".to_string()),
    }
}

fn latest_completed_restore(root: &Path) -> Option<BackupRestoreReceipt> {
    let raw = fs::read_to_string(root.join("receipts/restore-receipts.jsonl")).ok()?;
    raw.lines()
        .filter_map(|line| serde_json::from_str::<BackupRestoreReceipt>(line).ok())
        .filter(|receipt| receipt.status == "completed")
        .max_by_key(|receipt| receipt.completed_at)
}

fn read_generation_manifest(dir: &Path) -> Result<BackupGenerationManifest> {
    let raw = fs::read(dir.join("manifest.json"))?;
    let manifest: BackupGenerationManifest = serde_json::from_slice(&raw)?;
    if manifest.schema != BACKUP_MANIFEST_SCHEMA {
        bail!("unsupported backup manifest schema")
    }
    let expected = digest_serializable(&manifest)?;
    if expected != manifest.manifest_sha256 {
        bail!("backup manifest digest mismatch")
    }
    Ok(manifest)
}

#[derive(Clone, PartialEq, Eq)]
struct VerificationFingerprint {
    manifest_sha256: String,
    files: Vec<(PathBuf, u64, u128)>,
}

const VERIFICATION_CACHE_MAX_ENTRIES: usize = 64;

static VERIFICATION_CACHE: OnceLock<Mutex<HashMap<PathBuf, VerificationFingerprint>>> =
    OnceLock::new();

fn verification_fingerprint(
    dir: &Path,
    manifest: &BackupGenerationManifest,
) -> Result<VerificationFingerprint> {
    let backup_root = dir
        .parent()
        .and_then(Path::parent)
        .context("generation directory is outside a backup root")?;
    let mut paths = manifest
        .artifacts
        .iter()
        .map(|artifact| dir.join(&artifact.path))
        .chain(
            manifest
                .chunks
                .iter()
                .map(|chunk| backup_root.join(&chunk.storage_key)),
        )
        .collect::<Vec<_>>();
    paths.sort();
    let mut files = Vec::with_capacity(paths.len());
    for path in paths {
        let metadata = fs::metadata(&path)?;
        let modified_ns = metadata
            .modified()?
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        files.push((path, metadata.len(), modified_ns));
    }
    Ok(VerificationFingerprint {
        manifest_sha256: manifest.manifest_sha256.clone(),
        files,
    })
}

fn verification_cache_hit(dir: &Path, fingerprint: &VerificationFingerprint) -> bool {
    VERIFICATION_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .is_ok_and(|cache| cache.get(dir) == Some(fingerprint))
}

fn record_verification(dir: &Path, fingerprint: VerificationFingerprint) {
    if let Ok(mut cache) = VERIFICATION_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
    {
        if cache.len() >= VERIFICATION_CACHE_MAX_ENTRIES && !cache.contains_key(dir) {
            // Verification results are an optimization only. Keep this process-wide
            // projection bounded and evict deterministically without touching any
            // durable backup, manifest, receipt, or cryptographic authority.
            if let Some(evicted) = cache.keys().min().cloned() {
                cache.remove(&evicted);
            }
        }
        cache.insert(dir.to_path_buf(), fingerprint);
    }
}

fn verify_generation_artifacts(dir: &Path, manifest: &BackupGenerationManifest) -> Result<()> {
    for artifact in &manifest.artifacts {
        let path = dir.join(&artifact.path);
        if fs::metadata(&path)?.len() != artifact.bytes || sha256_file(&path)? != artifact.sha256 {
            bail!("backup artifact integrity mismatch")
        }
    }
    let backup_root = dir
        .parent()
        .and_then(Path::parent)
        .context("generation directory is outside a backup root")?;
    for chunk in &manifest.chunks {
        let path = backup_root.join(&chunk.storage_key);
        if fs::metadata(&path)?.len() != chunk.compressed_bytes
            || sha256_file(&path)? != chunk.compressed_sha256
        {
            bail!("backup chunk integrity mismatch")
        }
    }
    Ok(())
}

fn verify_generation_cached(dir: &Path) -> Result<BackupGenerationManifest> {
    let manifest = read_generation_manifest(dir)?;
    let before = verification_fingerprint(dir, &manifest)?;
    if verification_cache_hit(dir, &before) {
        return Ok(manifest);
    }
    verify_generation_artifacts(dir, &manifest)?;
    let after = verification_fingerprint(dir, &manifest)?;
    if before != after {
        bail!("backup artifact changed during verification")
    }
    record_verification(dir, after);
    Ok(manifest)
}

pub fn list_verified_manifests(root: &Path) -> Result<Vec<BackupGenerationManifest>> {
    let generations = root.join("generations");
    if !generations.is_dir() {
        return Ok(Vec::new());
    }
    let mut manifests = Vec::new();
    for entry in fs::read_dir(generations)? {
        let entry = entry?;
        if entry.file_type()?.is_dir()
            && let Ok(manifest) = verify_generation_cached(&entry.path())
            && (manifest.state == "verified"
                || manifest.state == "off_host_settled"
                || manifest.state == "restore_proven")
        {
            manifests.push(manifest);
        }
    }
    Ok(manifests)
}

/// Perform a deep cryptographic verification even when metadata is unchanged.
pub fn verify_generation(dir: &Path) -> Result<BackupGenerationManifest> {
    let manifest = read_generation_manifest(dir)?;
    let before = verification_fingerprint(dir, &manifest)?;
    verify_generation_artifacts(dir, &manifest)?;
    let after = verification_fingerprint(dir, &manifest)?;
    if before != after {
        bail!("backup artifact changed during verification")
    }
    record_verification(dir, after);
    Ok(manifest)
}
