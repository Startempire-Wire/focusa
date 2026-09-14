//! Isolated restore assembly and semantic verification for backup generations.

use super::backup::verify_generation;
use super::backup_contracts::*;
use super::backup_io::*;
use anyhow::{Context, Result, bail};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;
use uuid::Uuid;

pub fn restore_generation(
    backup_root: &Path,
    generation_id: &str,
    isolated_target: &Path,
    rto_seconds: u64,
) -> Result<BackupRestoreReceipt> {
    if isolated_target.exists() {
        bail!("isolated restore target already exists")
    }
    reject_symlink_components(isolated_target)?;
    let target_parent = isolated_target
        .parent()
        .context("isolated restore target has no parent")?;
    create_private_dir(target_parent)?;
    let _lock = MaintenanceLock::acquire(backup_root)?;
    let generation_dir = backup_root.join("generations").join(generation_id);
    let manifest = verify_generation(&generation_dir)?;
    let restore_id = Uuid::now_v7().to_string();
    let started_at = Utc::now();
    let started = Instant::now();
    let receipts = backup_root.join("receipts/restore-receipts.jsonl");
    let planned = BackupRestoreReceipt {
        schema: RESTORE_RECEIPT_SCHEMA.to_string(),
        restore_id: restore_id.clone(),
        generation_id: manifest.generation_id.clone(),
        generation_kind: manifest.generation_kind.clone(),
        manifest_sha256: manifest.manifest_sha256.clone(),
        isolated_target: isolated_target.display().to_string(),
        started_at,
        completed_at: started_at,
        restored_sha256: None,
        quick_check: None,
        event_count: None,
        event_chain_index: None,
        event_chain_hash: None,
        elapsed_seconds: 0,
        rto_status: "pending".to_string(),
        status: "planned".to_string(),
        error: None,
    };
    append_restore_receipt(&receipts, &planned)?;
    let partial = isolated_target.with_extension(format!("partial-{restore_id}"));
    let outcome = restore_and_verify(
        backup_root,
        &generation_dir,
        &manifest,
        &partial,
        isolated_target,
    );
    match outcome {
        Ok(verification) => {
            let elapsed = started.elapsed().as_secs();
            let receipt = BackupRestoreReceipt {
                schema: RESTORE_RECEIPT_SCHEMA.to_string(),
                restore_id,
                generation_id: manifest.generation_id,
                generation_kind: manifest.generation_kind,
                manifest_sha256: manifest.manifest_sha256,
                isolated_target: isolated_target.display().to_string(),
                started_at,
                completed_at: Utc::now(),
                restored_sha256: Some(verification.restored_sha256),
                quick_check: Some(verification.quick_check),
                event_count: Some(verification.event_count),
                event_chain_index: verification.event_chain_index,
                event_chain_hash: verification.event_chain_hash,
                elapsed_seconds: elapsed,
                rto_status: if elapsed <= rto_seconds {
                    "met"
                } else {
                    "breach"
                }
                .to_string(),
                status: "completed".to_string(),
                error: None,
            };
            append_restore_receipt(&receipts, &receipt)?;
            Ok(receipt)
        }
        Err(error) => {
            let _ = fs::remove_file(&partial);
            let receipt = BackupRestoreReceipt {
                status: "failed".to_string(),
                completed_at: Utc::now(),
                elapsed_seconds: started.elapsed().as_secs(),
                rto_status: "unproven".to_string(),
                error: Some(error.to_string()),
                ..planned
            };
            append_restore_receipt(&receipts, &receipt)?;
            Err(error)
        }
    }
}

struct RestoreVerification {
    restored_sha256: String,
    quick_check: String,
    event_count: u64,
    event_chain_index: Option<i64>,
    event_chain_hash: Option<String>,
}

fn restore_and_verify(
    backup_root: &Path,
    generation_dir: &Path,
    manifest: &BackupGenerationManifest,
    partial: &Path,
    target: &Path,
) -> Result<RestoreVerification> {
    if manifest.generation_kind == "full" {
        restore_full(generation_dir, manifest, partial)?;
    } else if manifest.generation_kind == "incremental_page_delta" {
        restore_chunks(backup_root, manifest, partial)?;
    } else {
        bail!("unsupported backup generation kind")
    }
    let restored_sha256 = sha256_file(partial)?;
    if restored_sha256 != manifest.source_snapshot_sha256 {
        bail!("restored database hash differs from generation manifest")
    }
    let conn = Connection::open_with_flags(partial, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let quick_check: String = conn.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if quick_check != "ok" {
        bail!("restored database quick_check failed: {quick_check}")
    }
    let event_count = query_u64_or_zero(&conn, "SELECT COUNT(*) FROM events")?;
    if event_count != manifest.event_count {
        bail!("restored event count differs from generation manifest")
    }
    let chain: Option<(i64, String)> = conn
        .query_row(
            "SELECT chain_index, event_hash FROM event_hash_chain ORDER BY chain_index DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    if chain.as_ref().map(|value| value.0) != manifest.event_chain_index
        || chain.as_ref().map(|value| &value.1) != manifest.event_chain_hash.as_ref()
    {
        bail!("restored event-chain head differs from generation manifest")
    }
    drop(conn);
    fs::rename(partial, target).context("commit isolated restore target")?;
    sync_dir(target.parent().context("restore target parent missing")?)?;
    Ok(RestoreVerification {
        restored_sha256,
        quick_check,
        event_count,
        event_chain_index: chain.as_ref().map(|value| value.0),
        event_chain_hash: chain.map(|value| value.1),
    })
}

fn restore_full(
    generation_dir: &Path,
    manifest: &BackupGenerationManifest,
    partial: &Path,
) -> Result<()> {
    let artifact = manifest
        .artifacts
        .first()
        .context("full generation has no artifact")?;
    if artifact.compression != "zstd" {
        bail!("unsupported full-generation compression")
    }
    let input = File::open(generation_dir.join(&artifact.path))?;
    let mut decoder = zstd::stream::read::Decoder::new(input)?;
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(partial)?;
    std::io::copy(&mut decoder, &mut output)?;
    output.sync_all()?;
    Ok(())
}

fn restore_chunks(
    backup_root: &Path,
    manifest: &BackupGenerationManifest,
    partial: &Path,
) -> Result<()> {
    if manifest.chunks.is_empty() {
        bail!("incremental generation has no chunks")
    }
    let mut chunks = manifest.chunks.clone();
    chunks.sort_by_key(|chunk| chunk.index);
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(partial)?;
    let mut expected_offset = 0_u64;
    for (expected_index, chunk) in chunks.iter().enumerate() {
        if chunk.index != expected_index as u64 || chunk.offset != expected_offset {
            bail!("incremental chunk sequence is discontinuous")
        }
        let input = File::open(backup_root.join(&chunk.storage_key))?;
        let mut decoder = zstd::stream::read::Decoder::new(input)?;
        let mut bytes = Vec::with_capacity(chunk.uncompressed_bytes as usize);
        decoder.read_to_end(&mut bytes)?;
        if bytes.len() as u64 != chunk.uncompressed_bytes
            || sha256_bytes(&bytes) != chunk.content_sha256
        {
            bail!("incremental chunk content verification failed")
        }
        output.write_all(&bytes)?;
        expected_offset += chunk.uncompressed_bytes;
    }
    output.sync_all()?;
    Ok(())
}

fn append_restore_receipt(path: &Path, receipt: &BackupRestoreReceipt) -> Result<()> {
    create_private_dir(path.parent().context("restore receipt parent missing")?)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, receipt)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}
