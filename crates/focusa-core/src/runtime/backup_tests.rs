use super::backup::{
    BACKUP_POLICY_SCHEMA, BackupPolicy, backup_health, create_full_generation, verify_generation,
};
use super::backup_contracts::digest_serializable;
use super::backup_incremental::create_incremental_generation;
use super::backup_restore::restore_generation;
use super::backup_retention::{execute_retention, plan_retention};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

fn approved_policy(backup_root: PathBuf) -> BackupPolicy {
    let mut policy = BackupPolicy {
        schema: BACKUP_POLICY_SCHEMA.to_string(),
        enabled: true,
        backup_root,
        rpo_seconds: 900,
        rto_seconds: 7_200,
        full_interval_seconds: 3_600,
        incremental_interval_seconds: 900,
        keep_hourly: 24,
        keep_daily: 14,
        keep_weekly: 8,
        keep_monthly: 12,
        restore_interval_seconds: 604_800,
        local_required: true,
        off_host_required: true,
        off_host_remote: None,
        min_free_bytes: 0,
        min_free_percent: 0,
        max_concurrent_operations: 1,
        compression: "zstd".to_string(),
        compression_level: 1,
        incremental_strategy: "experimental_full_snapshot_chunks_v0".to_string(),
        policy_digest: String::new(),
    };
    policy.policy_digest = digest_serializable(&policy).unwrap();
    policy
}

fn seeded_database(path: &Path) {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
         INSERT INTO meta VALUES('schema_version', '9');
         CREATE TABLE events(event_id TEXT PRIMARY KEY, payload_json TEXT NOT NULL);
         INSERT INTO events VALUES('event-1', '{}');
         CREATE TABLE event_hash_chain(
           event_id TEXT PRIMARY KEY, chain_index INTEGER NOT NULL, event_hash TEXT NOT NULL);
         INSERT INTO event_hash_chain VALUES('event-1', 1, 'hash-1');
         CREATE TABLE mutable_rows(id INTEGER PRIMARY KEY, value TEXT NOT NULL);
         INSERT INTO mutable_rows(value) VALUES('seed');",
    )
    .unwrap();
}

struct TestDir(PathBuf);
impl TestDir {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("focusa-backup-test-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn fixture() -> (TestDir, PathBuf, BackupPolicy) {
    let root = TestDir::new();
    let data = root.path().join("data");
    let backup = root.path().join("backup");
    std::fs::create_dir_all(data.join("ecs/objects")).unwrap();
    std::fs::write(data.join("ecs/objects/object-1"), b"object").unwrap();
    let db = data.join("focusa.sqlite");
    seeded_database(&db);
    (root, db, approved_policy(backup))
}

fn restore_compressed(generation_dir: &Path, destination: &Path) {
    let input = std::fs::File::open(generation_dir.join("focusa.sqlite.zst")).unwrap();
    let mut decoder = zstd::stream::read::Decoder::new(input).unwrap();
    let mut output = std::fs::File::create(destination).unwrap();
    std::io::copy(&mut decoder, &mut output).unwrap();
}

#[test]
fn full_generation_is_consistent_hashed_and_restorable() {
    let (root, db, policy) = fixture();
    let manifest = create_full_generation(&db, &policy, "2026-08-31T16", "test").unwrap();
    assert_eq!(manifest.state, "verified");
    assert_eq!(manifest.event_count, 1);
    assert_eq!(manifest.event_chain_index, Some(1));
    assert_eq!(manifest.event_chain_hash.as_deref(), Some("hash-1"));
    assert!(!manifest.ecs_inventory_digest.is_empty());
    assert_eq!(manifest.artifacts.len(), 1);

    let generation = policy
        .backup_root
        .join("generations")
        .join(&manifest.generation_id);
    let verified = verify_generation(&generation).unwrap();
    assert_eq!(verified.manifest_sha256, manifest.manifest_sha256);
    let restored = root.path().join("restored.sqlite");
    restore_compressed(&generation, &restored);
    let conn = Connection::open(restored).unwrap();
    let quick: String = conn
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .unwrap();
    assert_eq!(quick, "ok");
}

#[test]
fn same_slot_is_idempotent_and_does_not_duplicate_generation() {
    let (_root, db, policy) = fixture();
    let first = create_full_generation(&db, &policy, "slot-1", "test").unwrap();
    let second = create_full_generation(&db, &policy, "slot-1", "test").unwrap();
    assert_eq!(first.generation_id, second.generation_id);
    assert_eq!(
        std::fs::read_dir(policy.backup_root.join("generations"))
            .unwrap()
            .count(),
        1
    );
}

#[test]
fn artifact_corruption_fails_verification_and_health_excludes_generation() {
    let (_root, db, policy) = fixture();
    let manifest = create_full_generation(&db, &policy, "slot-corrupt", "test").unwrap();
    let generation = policy
        .backup_root
        .join("generations")
        .join(manifest.generation_id);
    std::fs::write(generation.join("focusa.sqlite.zst"), b"corrupt").unwrap();
    assert!(verify_generation(&generation).is_err());
    let health = backup_health(&policy);
    assert_eq!(health.verified_generation_count, 0);
    assert_eq!(health.full_status, "missing");
    assert_eq!(health.rpo_status, "breach_incremental_not_implemented");
}

#[test]
fn failed_snapshot_appends_failure_receipt() {
    let root = TestDir::new();
    let data = root.path().join("data");
    let backup = root.path().join("backup");
    std::fs::create_dir_all(&data).unwrap();
    let db = data.join("focusa.sqlite");
    std::fs::write(&db, b"not a sqlite database").unwrap();
    let policy = approved_policy(backup);
    assert!(create_full_generation(&db, &policy, "slot-invalid", "test").is_err());
    let receipts =
        std::fs::read_to_string(policy.backup_root.join("receipts/backup-receipts.jsonl")).unwrap();
    assert!(receipts.contains("\"phase\":\"planned\""));
    assert!(receipts.contains("\"phase\":\"failed\""));
    assert!(receipts.contains("operation_aborted"));
}

#[test]
fn non_wal_source_is_rejected() {
    let root = TestDir::new();
    let data = root.path().join("data");
    let backup = root.path().join("backup");
    std::fs::create_dir_all(&data).unwrap();
    let db = data.join("focusa.sqlite");
    let conn = Connection::open(&db).unwrap();
    conn.execute_batch(
        "PRAGMA journal_mode=DELETE;
         CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
         INSERT INTO meta VALUES('schema_version', '9');
         CREATE TABLE events(event_id TEXT PRIMARY KEY, payload_json TEXT NOT NULL);
         CREATE TABLE event_hash_chain(event_id TEXT PRIMARY KEY, chain_index INTEGER NOT NULL, event_hash TEXT NOT NULL);",
    )
    .unwrap();
    drop(conn);
    let error = create_full_generation(&db, &approved_policy(backup), "non-wal", "test")
        .unwrap_err()
        .to_string();
    assert!(error.contains("WAL journal mode"));
}

#[test]
fn manifest_tamper_is_rejected() {
    let (_root, db, policy) = fixture();
    let manifest = create_full_generation(&db, &policy, "tamper", "test").unwrap();
    let generation = policy
        .backup_root
        .join("generations")
        .join(manifest.generation_id);
    let manifest_path = generation.join("manifest.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    value["slot_id"] = serde_json::Value::String("tampered".to_string());
    std::fs::write(&manifest_path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    assert!(verify_generation(&generation).is_err());
}

#[test]
fn missing_incremental_chunk_blocks_restore() {
    let (root, db, policy) = fixture();
    let manifest = create_incremental_generation(&db, &policy, "missing-chunk", "test").unwrap();
    let chunk = manifest.chunks.first().unwrap();
    std::fs::remove_file(policy.backup_root.join(&chunk.storage_key)).unwrap();
    let error = restore_generation(
        &policy.backup_root,
        &manifest.generation_id,
        &root.path().join("missing-chunk-restore.sqlite"),
        policy.rto_seconds,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("No such file") || error.contains("integrity"));
}

#[test]
fn overlap_lock_fails_closed_before_snapshot_side_effects() {
    let (_root, db, policy) = fixture();
    std::fs::create_dir_all(policy.backup_root.join("locks")).unwrap();
    std::fs::create_dir_all(policy.backup_root.join("staging")).unwrap();
    std::fs::create_dir_all(policy.backup_root.join("generations")).unwrap();
    std::fs::create_dir_all(policy.backup_root.join("receipts")).unwrap();
    std::fs::write(policy.backup_root.join("locks/maintenance.lock"), b"active").unwrap();
    let error = create_full_generation(&db, &policy, "slot-locked", "test")
        .unwrap_err()
        .to_string();
    assert!(error.contains("already active"));
}

#[test]
fn online_backup_survives_concurrent_wal_writes() {
    let (root, db, policy) = fixture();
    let stop = Arc::new(AtomicBool::new(false));
    let writer_stop = stop.clone();
    let writer_db = db.clone();
    let writer = std::thread::spawn(move || {
        let conn = Connection::open(writer_db).unwrap();
        conn.busy_timeout(Duration::from_secs(2)).unwrap();
        let mut counter = 0_u64;
        while !writer_stop.load(Ordering::Relaxed) {
            let _ = conn.execute(
                "INSERT INTO mutable_rows(value) VALUES(?1)",
                [format!("value-{counter}")],
            );
            counter += 1;
            if counter % 50 == 0 {
                std::thread::yield_now();
            }
        }
    });
    let manifest = create_full_generation(&db, &policy, "slot-writers", "test").unwrap();
    stop.store(true, Ordering::Relaxed);
    writer.join().unwrap();
    let generation = policy
        .backup_root
        .join("generations")
        .join(manifest.generation_id);
    let restored = root.path().join("writer-restored.sqlite");
    restore_compressed(&generation, &restored);
    let conn = Connection::open(restored).unwrap();
    let quick: String = conn
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .unwrap();
    assert_eq!(quick, "ok");
}

#[test]
fn incremental_generations_reuse_unchanged_content_chunks() {
    let (_root, db, policy) = fixture();
    let conn = Connection::open(&db).unwrap();
    conn.execute_batch(
        "CREATE TABLE large_fixture(id INTEGER PRIMARY KEY, payload BLOB NOT NULL);
         INSERT INTO large_fixture(payload) VALUES(zeroblob(12582912));",
    )
    .unwrap();
    drop(conn);
    let first = create_incremental_generation(&db, &policy, "delta-1", "test").unwrap();
    let conn = Connection::open(&db).unwrap();
    conn.execute(
        "INSERT INTO mutable_rows(value) VALUES(?1)",
        ["one changed row"],
    )
    .unwrap();
    drop(conn);
    let second = create_incremental_generation(&db, &policy, "delta-2", "test").unwrap();
    assert_eq!(
        second.parent_generation_id,
        Some(first.generation_id.clone())
    );
    assert!(!first.chunks.is_empty());
    assert!(!second.chunks.is_empty());
    let first_keys = first
        .chunks
        .iter()
        .map(|chunk| &chunk.storage_key)
        .collect::<std::collections::HashSet<_>>();
    assert!(
        second
            .chunks
            .iter()
            .any(|chunk| first_keys.contains(&chunk.storage_key)),
        "at least one unchanged content chunk must be reused"
    );
}

#[test]
fn incremental_generation_restores_with_semantic_receipt() {
    let (root, db, policy) = fixture();
    let generation = create_incremental_generation(&db, &policy, "delta-restore", "test").unwrap();
    let target = root.path().join("restore-drill/restored.sqlite");
    let receipt = restore_generation(
        &policy.backup_root,
        &generation.generation_id,
        &target,
        policy.rto_seconds,
    )
    .unwrap();
    assert_eq!(receipt.status, "completed");
    assert_eq!(receipt.rto_status, "met");
    assert_eq!(receipt.quick_check.as_deref(), Some("ok"));
    assert_eq!(receipt.event_count, Some(generation.event_count));
    assert_eq!(receipt.event_chain_hash, generation.event_chain_hash);
    assert_eq!(
        receipt.restored_sha256.as_deref(),
        Some(generation.source_snapshot_sha256.as_str())
    );
    assert!(target.is_file());
    let health = backup_health(&policy);
    assert_eq!(health.rpo_status, "breach_incremental_not_implemented");
    assert_eq!(health.restore_status, "ok");
    assert_eq!(health.overall_status, "degraded");
    let receipts =
        std::fs::read_to_string(policy.backup_root.join("receipts/restore-receipts.jsonl"))
            .unwrap();
    assert!(receipts.contains("\"status\":\"planned\""));
    assert!(receipts.contains("\"status\":\"completed\""));
}

#[test]
fn policy_rejects_backup_root_inside_live_data() {
    let root = TestDir::new();
    let data = root.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    let policy = approved_policy(data.join("backups"));
    assert!(policy.validate(&data).is_err());
}

#[test]
fn retention_requires_newer_restore_and_never_deletes_last_generation() {
    let (root, db, mut policy) = fixture();
    policy.off_host_required = false;
    policy.policy_digest.clear();
    policy.policy_digest = digest_serializable(&policy).unwrap();
    let first = create_full_generation(&db, &policy, "retention-1", "test").unwrap();
    let second = create_full_generation(&db, &policy, "retention-2", "test").unwrap();
    let latest = create_full_generation(&db, &policy, "retention-3", "test").unwrap();
    let blocked = plan_retention(&policy).unwrap();
    assert!(blocked.candidate_generation_ids.is_empty());
    assert!(
        blocked
            .reasons
            .iter()
            .any(|reason| reason.contains("restore proof missing"))
    );

    let target = root.path().join("retention-restore/restored.sqlite");
    restore_generation(
        &policy.backup_root,
        &latest.generation_id,
        &target,
        policy.rto_seconds,
    )
    .unwrap();
    let approved = plan_retention(&policy).unwrap();
    assert!(
        approved
            .candidate_generation_ids
            .contains(&first.generation_id)
    );
    assert!(
        approved
            .candidate_generation_ids
            .contains(&second.generation_id)
    );
    assert!(
        !approved
            .candidate_generation_ids
            .contains(&latest.generation_id)
    );
    let receipt = execute_retention(&policy).unwrap();
    assert_eq!(receipt.status, "completed");
    assert_eq!(receipt.deleted_generation_ids.len(), 2);
    let remaining = std::fs::read_dir(policy.backup_root.join("generations"))
        .unwrap()
        .count();
    assert_eq!(remaining, 1);
    assert!(
        policy
            .backup_root
            .join("generations")
            .join(latest.generation_id)
            .is_dir()
    );
}

#[test]
fn retention_blocks_local_prune_without_required_off_host_settlement() {
    let (root, db, policy) = fixture();
    let first = create_full_generation(&db, &policy, "offhost-retention-1", "test").unwrap();
    let latest = create_full_generation(&db, &policy, "offhost-retention-2", "test").unwrap();
    restore_generation(
        &policy.backup_root,
        &latest.generation_id,
        &root.path().join("offhost-restore/restored.sqlite"),
        policy.rto_seconds,
    )
    .unwrap();
    let decision = plan_retention(&policy).unwrap();
    assert!(
        !decision
            .candidate_generation_ids
            .contains(&first.generation_id)
    );
    assert!(
        decision
            .reasons
            .iter()
            .any(|reason| reason.contains("off-host settlement missing"))
    );
}

#[test]
fn off_host_remote_policy_rejects_traversal_and_accepts_named_remote() {
    let root = TestDir::new();
    let data = root.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    let mut policy = approved_policy(root.path().join("backup"));
    for invalid in [
        "r2:../escape",
        ":local/path",
        "r2:s3:https://example.invalid",
        "r2:prefix?credential=value",
    ] {
        policy.off_host_remote = Some(invalid.to_string());
        assert!(policy.validate(&data).is_err(), "accepted {invalid}");
    }
    policy.off_host_remote = Some("focusa-r2:backups/kh".to_string());
    assert!(policy.validate(&data).is_ok());
}

#[cfg(unix)]
#[test]
fn symlinked_backup_root_is_rejected() {
    use std::os::unix::fs::symlink;
    let (_root, db, mut policy) = fixture();
    let real = policy.backup_root.with_extension("real");
    std::fs::create_dir_all(&real).unwrap();
    symlink(&real, &policy.backup_root).unwrap();
    policy.min_free_bytes = 0;
    let error = create_full_generation(&db, &policy, "slot-link", "test")
        .unwrap_err()
        .to_string();
    assert!(error.contains("symlink"));
}
