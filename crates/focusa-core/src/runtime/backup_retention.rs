//! GFS retention planning and receipt-bound pruning for verified generations.

use super::backup::verify_generation;
use super::backup_contracts::*;
use super::backup_io::*;
use super::backup_offhost::latest_off_host_receipt;
use anyhow::{Context, Result, bail};
use chrono::{Datelike, Timelike, Utc};
use std::collections::{BTreeSet, HashSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const PRUNE_RECEIPT_SCHEMA: &str = "focusa.backup_prune_receipt.v1";

pub fn plan_retention(policy: &BackupPolicy) -> Result<BackupPruneDecision> {
    let mut manifests = strict_manifests(&policy.backup_root)?;
    manifests.sort_by_key(|manifest| manifest.completed_at);
    let Some(latest) = manifests.last() else {
        return Ok(BackupPruneDecision {
            schema: PRUNE_DECISION_SCHEMA.to_string(),
            policy_digest: policy.policy_digest.clone(),
            decision: "blocked_no_verified_generation".to_string(),
            retained_generation_ids: Vec::new(),
            candidate_generation_ids: Vec::new(),
            reasons: vec!["never delete without a verified generation".to_string()],
        });
    };
    let latest_id = latest.generation_id.clone();
    let mut retained = gfs_retained(&manifests, policy);
    retained.insert(latest_id.clone());
    let latest_restore = latest_completed_restore(&policy.backup_root);
    let mut candidates = Vec::new();
    let mut reasons = Vec::new();
    for manifest in &manifests {
        if retained.contains(&manifest.generation_id) {
            continue;
        }
        let off_host_safe = !policy.off_host_required
            || latest_off_host_receipt(&policy.backup_root, &manifest.generation_id)
                .is_some_and(|receipt| receipt.manifest_sha256 == manifest.manifest_sha256);
        let newer_restore_safe = latest_restore.as_ref().is_some_and(|receipt| {
            receipt.status == "completed"
                && receipt.rto_status == "met"
                && receipt.completed_at > manifest.completed_at
        });
        if off_host_safe && newer_restore_safe && manifest.generation_id != latest_id {
            candidates.push(manifest.generation_id.clone());
        } else {
            retained.insert(manifest.generation_id.clone());
            if !off_host_safe {
                reasons.push(format!(
                    "{} retained: off-host settlement missing",
                    manifest.generation_id
                ));
            }
            if !newer_restore_safe {
                reasons.push(format!(
                    "{} retained: newer restore proof missing",
                    manifest.generation_id
                ));
            }
        }
    }
    if candidates.len() >= manifests.len() {
        bail!("retention plan attempted to delete every verified generation")
    }
    Ok(BackupPruneDecision {
        schema: PRUNE_DECISION_SCHEMA.to_string(),
        policy_digest: policy.policy_digest.clone(),
        decision: if candidates.is_empty() {
            "no_safe_candidates"
        } else {
            "approved"
        }
        .to_string(),
        retained_generation_ids: retained.into_iter().collect(),
        candidate_generation_ids: candidates,
        reasons,
    })
}

pub fn execute_retention(policy: &BackupPolicy) -> Result<BackupPruneReceipt> {
    reject_symlink_components(&policy.backup_root)?;
    if !policy.backup_root.is_dir() {
        bail!("backup root does not exist")
    }
    let _lock = MaintenanceLock::acquire(&policy.backup_root)?;
    let plan = plan_retention(policy)?;
    let plan_digest = digest_serializable(&plan)?;
    let prune_id = Uuid::now_v7().to_string();
    let started_at = Utc::now();
    let receipts = policy.backup_root.join("receipts/prune-receipts.jsonl");
    let planned = BackupPruneReceipt {
        schema: PRUNE_RECEIPT_SCHEMA.to_string(),
        prune_id,
        policy_digest: policy.policy_digest.clone(),
        plan_digest,
        started_at,
        completed_at: started_at,
        status: "planned".to_string(),
        deleted_generation_ids: Vec::new(),
        deleted_chunk_count: 0,
        reclaimed_bytes: 0,
        error: None,
    };
    append_prune_receipt(&receipts, &planned)?;
    if plan.candidate_generation_ids.is_empty() {
        let receipt = BackupPruneReceipt {
            status: "completed_noop".to_string(),
            completed_at: Utc::now(),
            ..planned
        };
        append_prune_receipt(&receipts, &receipt)?;
        return Ok(receipt);
    }

    let mut deleted = Vec::new();
    let mut reclaimed_bytes = 0_u64;
    for generation_id in &plan.candidate_generation_ids {
        let directory = policy.backup_root.join("generations").join(generation_id);
        if verify_generation(&directory)?.generation_id != *generation_id {
            bail!("generation identity changed after retention planning")
        }
        let generation_bytes = tree_bytes(&directory)?;
        if let Err(error) = fs::remove_dir_all(&directory) {
            let receipt = BackupPruneReceipt {
                status: "failed_partial".to_string(),
                completed_at: Utc::now(),
                deleted_generation_ids: deleted,
                reclaimed_bytes,
                error: Some(error.to_string()),
                ..planned
            };
            append_prune_receipt(&receipts, &receipt)?;
            return Err(error.into());
        }
        reclaimed_bytes = reclaimed_bytes.saturating_add(generation_bytes);
        deleted.push(generation_id.clone());
    }
    let (deleted_chunk_count, chunk_bytes) = match prune_unreferenced_chunks(&policy.backup_root) {
        Ok(result) => result,
        Err(error) => {
            let receipt = BackupPruneReceipt {
                status: "failed_partial".to_string(),
                completed_at: Utc::now(),
                deleted_generation_ids: deleted,
                reclaimed_bytes,
                error: Some(error.to_string()),
                ..planned
            };
            append_prune_receipt(&receipts, &receipt)?;
            return Err(error);
        }
    };
    reclaimed_bytes = reclaimed_bytes.saturating_add(chunk_bytes);
    let remaining = strict_manifests(&policy.backup_root)?;
    if remaining.is_empty() {
        bail!("retention violated last-verified-generation invariant")
    }
    let receipt = BackupPruneReceipt {
        status: "completed".to_string(),
        completed_at: Utc::now(),
        deleted_generation_ids: deleted,
        deleted_chunk_count,
        reclaimed_bytes,
        ..planned
    };
    append_prune_receipt(&receipts, &receipt)?;
    Ok(receipt)
}

pub fn retention_due(backup_root: &Path, interval_seconds: u64) -> bool {
    let raw = match fs::read_to_string(backup_root.join("receipts/prune-receipts.jsonl")) {
        Ok(raw) => raw,
        Err(_) => return true,
    };
    let latest = raw
        .lines()
        .filter_map(|line| serde_json::from_str::<BackupPruneReceipt>(line).ok())
        .filter(|receipt| receipt.status.starts_with("completed"))
        .map(|receipt| receipt.completed_at)
        .max();
    latest.is_none_or(|completed_at| {
        (Utc::now() - completed_at).num_seconds() >= interval_seconds as i64
    })
}

fn gfs_retained(manifests: &[BackupGenerationManifest], policy: &BackupPolicy) -> BTreeSet<String> {
    let mut retained = BTreeSet::new();
    let mut hours = BTreeSet::new();
    let mut days = BTreeSet::new();
    let mut weeks = BTreeSet::new();
    let mut months = BTreeSet::new();
    for manifest in manifests.iter().rev() {
        let time = manifest.completed_at;
        retain_bucket(
            &mut retained,
            &mut hours,
            format!(
                "{}-{:02}-{:02}T{:02}",
                time.year(),
                time.month(),
                time.day(),
                time.hour()
            ),
            policy.keep_hourly,
            &manifest.generation_id,
        );
        retain_bucket(
            &mut retained,
            &mut days,
            format!("{}-{:02}-{:02}", time.year(), time.month(), time.day()),
            policy.keep_daily,
            &manifest.generation_id,
        );
        let week = time.iso_week();
        retain_bucket(
            &mut retained,
            &mut weeks,
            format!("{}-W{:02}", week.year(), week.week()),
            policy.keep_weekly,
            &manifest.generation_id,
        );
        retain_bucket(
            &mut retained,
            &mut months,
            format!("{}-{:02}", time.year(), time.month()),
            policy.keep_monthly,
            &manifest.generation_id,
        );
    }
    retained
}

fn retain_bucket(
    retained: &mut BTreeSet<String>,
    buckets: &mut BTreeSet<String>,
    bucket: String,
    limit: usize,
    generation_id: &str,
) {
    if buckets.contains(&bucket) || buckets.len() >= limit {
        return;
    }
    buckets.insert(bucket);
    retained.insert(generation_id.to_string());
}

fn strict_manifests(root: &Path) -> Result<Vec<BackupGenerationManifest>> {
    let generations = root.join("generations");
    if !generations.is_dir() {
        return Ok(Vec::new());
    }
    let mut manifests = Vec::new();
    for entry in fs::read_dir(generations)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            bail!("unexpected non-directory in generations root")
        }
        manifests.push(verify_generation(&entry.path())?);
    }
    Ok(manifests)
}

fn latest_completed_restore(root: &Path) -> Option<BackupRestoreReceipt> {
    let raw = fs::read_to_string(root.join("receipts/restore-receipts.jsonl")).ok()?;
    raw.lines()
        .filter_map(|line| serde_json::from_str::<BackupRestoreReceipt>(line).ok())
        .filter(|receipt| receipt.status == "completed")
        .max_by_key(|receipt| receipt.completed_at)
}

fn prune_unreferenced_chunks(root: &Path) -> Result<(usize, u64)> {
    let referenced = strict_manifests(root)?
        .into_iter()
        .flat_map(|manifest| manifest.chunks.into_iter().map(|chunk| chunk.storage_key))
        .collect::<HashSet<_>>();
    let chunks_root = root.join("chunks");
    if !chunks_root.is_dir() {
        return Ok((0, 0));
    }
    let mut files = Vec::new();
    collect_chunk_files(&chunks_root, &mut files)?;
    let mut count = 0;
    let mut bytes = 0_u64;
    for path in files {
        let relative = path
            .strip_prefix(root)?
            .to_string_lossy()
            .replace('\\', "/");
        if !referenced.contains(&relative) {
            bytes = bytes.saturating_add(fs::metadata(&path)?.len());
            fs::remove_file(&path)?;
            count += 1;
        }
    }
    Ok((count, bytes))
}

fn collect_chunk_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            bail!("chunk store contains symlink")
        }
        if file_type.is_dir() {
            collect_chunk_files(&entry.path(), files)?;
        } else if file_type.is_file() {
            files.push(entry.path());
        } else {
            bail!("chunk store contains unsupported filesystem entry")
        }
    }
    Ok(())
}

fn tree_bytes(directory: &Path) -> Result<u64> {
    let mut bytes = 0_u64;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            bail!("generation contains symlink")
        }
        if file_type.is_dir() {
            bytes = bytes.saturating_add(tree_bytes(&entry.path())?);
        } else if file_type.is_file() {
            bytes = bytes.saturating_add(entry.metadata()?.len());
        }
    }
    Ok(bytes)
}

fn append_prune_receipt(path: &Path, receipt: &BackupPruneReceipt) -> Result<()> {
    create_private_dir(path.parent().context("prune receipt parent missing")?)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, receipt)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}
