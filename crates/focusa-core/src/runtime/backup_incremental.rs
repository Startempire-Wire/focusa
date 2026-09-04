//! Content-addressed page-delta recovery points for the approved 15-minute RPO.
//!
//! Each run first obtains an application-consistent SQLite online snapshot,
//! then stores fixed-size chunks by uncompressed SHA-256. Unchanged chunks are
//! reused, bounding retained storage without depending on the live WAL's
//! checkpoint lifecycle.

use super::backup::{list_verified_manifests, verify_generation};
use super::backup_contracts::*;
use super::backup_io::*;
use anyhow::{Context, Result, bail};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use uuid::Uuid;

pub const DEFAULT_CHUNK_BYTES: usize = 4 * 1024 * 1024;

pub fn create_incremental_generation(
    source_database: &Path,
    policy: &BackupPolicy,
    slot_id: &str,
    runtime_version: &str,
) -> Result<BackupGenerationManifest> {
    if !policy.enabled {
        bail!("backup policy is disabled")
    }
    if policy.incremental_strategy != "experimental_full_snapshot_chunks_v0" {
        bail!("conforming incremental recovery strategy is not implemented")
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
        deterministic_incremental_id(slot_id, &source_identity, &policy.policy_digest);
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
    let snapshot = Connection::open(&staging_db).context("open staged incremental backup")?;
    let quick_check: String = snapshot.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if quick_check != "ok" {
        bail!("staged incremental quick_check failed: {quick_check}")
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
    drop(snapshot);

    if file_identity(&source)? != source_identity {
        bail!("source database identity drifted during backup")
    }
    let (chunks, source_snapshot_sha256) = store_snapshot_chunks(
        &staging_db,
        &policy.backup_root,
        policy.compression_level,
        DEFAULT_CHUNK_BYTES,
    )?;
    fs::remove_file(&staging_db).context("remove incremental staging database")?;
    let mut parents = list_verified_manifests(&policy.backup_root)?;
    parents.sort_by_key(|manifest| manifest.completed_at);
    let parent_generation_id = parents
        .last()
        .map(|manifest| manifest.generation_id.clone());
    let stored_bytes = chunks.iter().map(|chunk| chunk.compressed_bytes).sum();

    let mut manifest = BackupGenerationManifest {
        schema: BACKUP_MANIFEST_SCHEMA.to_string(),
        generation_id: generation_id.clone(),
        slot_id: slot_id.to_string(),
        generation_kind: "incremental_page_delta".to_string(),
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
        ecs_inventory_digest: inventory_digest(&data_dir.join("ecs"))?,
        cold_export_inventory_digest: inventory_digest(&data_dir.join("events-cold"))?,
        artifacts: Vec::new(),
        chunks,
        source_snapshot_sha256: source_snapshot_sha256.clone(),
        parent_generation_id,
        policy_digest: policy.policy_digest.clone(),
        off_host_settlement_ref: None,
        restore_receipt_ref: None,
        manifest_sha256: String::new(),
    };
    manifest.manifest_sha256 = digest_serializable(&manifest)?;
    write_json_atomic(&staging.join("manifest.json"), &manifest)?;
    sync_dir(&staging)?;
    fs::rename(&staging, &generation_dir).context("commit incremental generation")?;
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
            bytes: stored_bytes,
            artifact_sha256: Some(source_snapshot_sha256),
            quick_check: Some(quick_check),
            error_code: None,
            error: None,
        },
    )?;
    failure_guard.settled = true;
    Ok(manifest)
}

fn store_snapshot_chunks(
    snapshot: &Path,
    backup_root: &Path,
    compression_level: i32,
    chunk_bytes: usize,
) -> Result<(Vec<BackupChunkRef>, String)> {
    if chunk_bytes == 0 {
        bail!("chunk size must be non-zero")
    }
    let mut input = File::open(snapshot)?;
    let mut source_hasher = Sha256::new();
    let mut chunks = Vec::new();
    let mut buffer = vec![0_u8; chunk_bytes];
    let mut offset = 0_u64;
    loop {
        let mut filled = 0;
        while filled < buffer.len() {
            let read = input.read(&mut buffer[filled..])?;
            if read == 0 {
                break;
            }
            filled += read;
        }
        if filled == 0 {
            break;
        }
        let bytes = &buffer[..filled];
        source_hasher.update(bytes);
        chunks.push(store_chunk(
            backup_root,
            chunks.len() as u64,
            offset,
            bytes,
            compression_level,
        )?);
        offset += filled as u64;
    }
    Ok((chunks, hex::encode(source_hasher.finalize())))
}

fn store_chunk(
    backup_root: &Path,
    index: u64,
    offset: u64,
    bytes: &[u8],
    compression_level: i32,
) -> Result<BackupChunkRef> {
    let content_sha256 = sha256_bytes(bytes);
    let storage_key = format!(
        "chunks/sha256/{}/{}.zst",
        &content_sha256[..2],
        content_sha256
    );
    let path = backup_root.join(&storage_key);
    create_private_dir(path.parent().context("chunk parent missing")?)?;
    if path.exists() {
        verify_chunk_content(&path, &content_sha256)?;
    } else {
        let temporary = path.with_extension(format!("tmp-{}", Uuid::now_v7()));
        let compressed = zstd::stream::encode_all(bytes, compression_level)?;
        let mut output = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)?;
        output.write_all(&compressed)?;
        output.sync_all()?;
        fs::rename(&temporary, &path)?;
        sync_dir(path.parent().context("chunk parent missing")?)?;
    }
    Ok(BackupChunkRef {
        index,
        offset,
        uncompressed_bytes: bytes.len() as u64,
        content_sha256,
        storage_key,
        compressed_bytes: fs::metadata(&path)?.len(),
        compressed_sha256: sha256_file(&path)?,
    })
}

fn verify_chunk_content(path: &Path, expected: &str) -> Result<()> {
    let input = File::open(path)?;
    let mut decoder = zstd::stream::read::Decoder::new(input)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = decoder.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    if hex::encode(hasher.finalize()) != expected {
        bail!("content-addressed backup chunk hash mismatch")
    }
    Ok(())
}
