//! Verified off-host settlement through an operator-configured rclone remote.
//!
//! Credentials remain in rclone's provider-owned configuration and never enter
//! Focusa manifests, arguments, logs, or receipts. The configured value is only
//! a remote name and object prefix such as `focusa-r2:backups/kh`.

use super::backup::verify_generation;
use super::backup_contracts::*;
use super::backup_io::*;
use anyhow::{Context, Result, bail};
use chrono::Utc;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

pub const OFF_HOST_RECEIPT_SCHEMA: &str = "focusa.backup_off_host_receipt.v1";

pub fn settle_generation_off_host(
    backup_root: &Path,
    generation_id: &str,
    remote: &str,
) -> Result<BackupOffHostReceipt> {
    validate_remote(remote)?;
    let _lock = MaintenanceLock::acquire(backup_root)?;
    let generation_dir = backup_root.join("generations").join(generation_id);
    let manifest = verify_generation(&generation_dir)?;
    let settlement_id = Uuid::now_v7().to_string();
    let started_at = Utc::now();
    let receipts = backup_root.join("receipts/off-host-receipts.jsonl");
    let planned = BackupOffHostReceipt {
        schema: OFF_HOST_RECEIPT_SCHEMA.to_string(),
        settlement_id: settlement_id.clone(),
        generation_id: manifest.generation_id.clone(),
        manifest_sha256: manifest.manifest_sha256.clone(),
        remote: redact_remote(remote),
        started_at,
        completed_at: started_at,
        status: "planned".to_string(),
        verification: "pending".to_string(),
        files: 0,
        bytes: 0,
        error: None,
    };
    append_off_host_receipt(&receipts, &planned)?;
    let bundle = backup_root
        .join("staging")
        .join(format!("off-host-{generation_id}-{settlement_id}"));
    let outcome = settle_bundle(backup_root, &generation_dir, &bundle, &manifest, remote);
    match outcome {
        Ok((files, bytes)) => {
            let receipt = BackupOffHostReceipt {
                status: "completed".to_string(),
                verification: "rclone_check_checksum".to_string(),
                completed_at: Utc::now(),
                files,
                bytes,
                ..planned
            };
            upload_receipt(remote, &receipt, backup_root)?;
            append_off_host_receipt(&receipts, &receipt)?;
            if let Err(error) = fs::remove_dir_all(&bundle) {
                tracing::error!(error = %error, path = %bundle.display(), "failed to remove off-host staging bundle");
            }
            Ok(receipt)
        }
        Err(error) => {
            let receipt = BackupOffHostReceipt {
                status: "failed".to_string(),
                verification: "failed".to_string(),
                completed_at: Utc::now(),
                error: Some(error.to_string()),
                ..planned
            };
            append_off_host_receipt(&receipts, &receipt)?;
            if let Err(cleanup_error) = fs::remove_dir_all(&bundle) {
                if bundle.exists() {
                    tracing::error!(error = %cleanup_error, path = %bundle.display(), "failed to remove failed off-host staging bundle");
                }
            }
            Err(error)
        }
    }
}

pub fn latest_off_host_receipt(
    backup_root: &Path,
    generation_id: &str,
) -> Option<BackupOffHostReceipt> {
    let raw = fs::read_to_string(backup_root.join("receipts/off-host-receipts.jsonl")).ok()?;
    raw.lines()
        .filter_map(|line| serde_json::from_str::<BackupOffHostReceipt>(line).ok())
        .filter(|receipt| receipt.generation_id == generation_id && receipt.status == "completed")
        .max_by_key(|receipt| receipt.completed_at)
}

fn settle_bundle(
    backup_root: &Path,
    generation_dir: &Path,
    bundle: &Path,
    manifest: &BackupGenerationManifest,
    remote: &str,
) -> Result<(usize, u64)> {
    create_private_dir(bundle)?;
    let generation_bundle = bundle.join("generation");
    create_private_dir(&generation_bundle)?;
    hard_link_verified(
        &generation_dir.join("manifest.json"),
        &generation_bundle.join("manifest.json"),
    )?;
    let mut files = 1_usize;
    let mut bytes = fs::metadata(generation_dir.join("manifest.json"))?.len();
    for artifact in &manifest.artifacts {
        let source = generation_dir.join(&artifact.path);
        let target = generation_bundle.join(&artifact.path);
        hard_link_verified(&source, &target)?;
        files += 1;
        bytes = bytes.saturating_add(artifact.bytes);
    }
    for chunk in &manifest.chunks {
        let source = backup_root.join(&chunk.storage_key);
        let target = bundle.join(&chunk.storage_key);
        if let Some(parent) = target.parent() {
            create_private_dir(parent)?;
        }
        hard_link_verified(&source, &target)?;
        files += 1;
        bytes = bytes.saturating_add(chunk.compressed_bytes);
    }
    let target = format!(
        "{}/generations/{}",
        remote.trim_end_matches('/'),
        manifest.generation_id
    );
    run_rclone([
        "copy",
        path_arg(bundle)?,
        target.as_str(),
        "--checksum",
        "--immutable",
    ])?;
    run_rclone([
        "check",
        path_arg(bundle)?,
        target.as_str(),
        "--one-way",
        "--checksum",
    ])?;
    Ok((files, bytes))
}

fn upload_receipt(remote: &str, receipt: &BackupOffHostReceipt, backup_root: &Path) -> Result<()> {
    let temp = backup_root
        .join("staging")
        .join(format!("off-host-receipt-{}.json", receipt.settlement_id));
    write_json_atomic(&temp, receipt)?;
    let destination = format!(
        "{}/receipts/{}.json",
        remote.trim_end_matches('/'),
        receipt.settlement_id
    );
    let result = run_rclone([
        "copyto",
        path_arg(&temp)?,
        destination.as_str(),
        "--checksum",
    ]);
    if let Err(error) = fs::remove_file(&temp) {
        tracing::error!(error = %error, path = %temp.display(), "failed to remove off-host receipt staging file");
    }
    result
}

fn run_rclone<const N: usize>(args: [&str; N]) -> Result<()> {
    let output = Command::new("rclone")
        .args(args)
        .output()
        .context("execute rclone off-host settlement")?;
    if !output.status.success() {
        let stderr_sha256 = sha256_bytes(&output.stderr);
        bail!(
            "rclone off-host settlement failed: status={} stderr_bytes={} stderr_sha256={}",
            output.status,
            output.stderr.len(),
            stderr_sha256
        )
    }
    Ok(())
}

fn hard_link_verified(source: &Path, target: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(source)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        bail!("off-host bundle source must be a regular non-symlink file")
    }
    fs::hard_link(source, target).with_context(|| {
        format!(
            "hard-link off-host bundle file {} to {}",
            source.display(),
            target.display()
        )
    })?;
    Ok(())
}

fn append_off_host_receipt(path: &Path, receipt: &BackupOffHostReceipt) -> Result<()> {
    create_private_dir(path.parent().context("off-host receipt parent missing")?)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, receipt)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}

fn validate_remote(remote: &str) -> Result<()> {
    let valid = remote.split_once(':').is_some_and(|(name, prefix)| {
        !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
            && !prefix.is_empty()
            && !prefix.starts_with('/')
            && !prefix.contains(':')
            && !prefix.contains("..")
            && prefix.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'/' | b'.')
            })
    });
    if remote.len() > 240 || !valid {
        bail!("off-host remote is invalid")
    }
    Ok(())
}

fn redact_remote(remote: &str) -> String {
    remote
        .split_once(':')
        .map(|(name, _)| format!("{name}:<configured-prefix>"))
        .unwrap_or_else(|| "<invalid>".to_string())
}

fn path_arg(path: &Path) -> Result<&str> {
    path.to_str().context("off-host path is not valid UTF-8")
}
