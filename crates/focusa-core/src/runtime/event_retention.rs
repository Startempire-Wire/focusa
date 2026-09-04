//! Event-ledger retention (DB-size architecture).
//!
//! Keeps the events SQLite ledger bounded:
//! - cold export: events older than the hot window are exported to
//!   append-only JSONL files under the configured export directory;
//! - hot-window pruning: exported (or junk) rows are deleted in bounded
//!   batches so the daemon's writer is never starved by one giant
//!   transaction;
//! - hash-chain anchoring: old chain rows are dropped while the latest
//!   checkpoint hash is preserved in `meta`, keeping forward integrity
//!   without unbounded chain growth;
//! - incremental vacuum: freed pages are returned to the filesystem in
//!   bounded steps (no-op unless `PRAGMA auto_vacuum=INCREMENTAL`).
//!
//! All functions are synchronous and batch-bounded; callers run them via
//! `spawn_blocking`.

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::io::Write as _;
use std::path::{Path, PathBuf};

pub const RETENTION_ENV_DAYS: &str = "FOCUSA_EVENT_RETENTION_DAYS";
pub const RETENTION_ENV_DISABLED: &str = "FOCUSA_EVENT_RETENTION_DISABLED";
pub const DEFAULT_RETENTION_DAYS: u32 = 30;
pub const CHAIN_ANCHOR_KEEP: i64 = 2000;
pub const DEFAULT_BATCH_SIZE: usize = 5_000;
pub const META_ANCHOR_KEY: &str = "event_chain_anchor";

/// Placeholder events written by the retired temporal fallback carried
/// epoch-0 (1970) timestamps. Never exported — they carry no durable signal.
pub const JUNK_CUTOFF: &str = "2000-01-01T00:00:00+00:00";

#[derive(Debug, Clone, serde::Serialize)]
pub struct RetentionSummary {
    pub cutoff_ts: String,
    pub exported_events: u64,
    pub deleted_events: u64,
    pub deleted_chain_rows: u64,
    pub anchor_chain_index: Option<i64>,
    pub vacuumed_pages: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct JunkPruneSummary {
    pub deleted_events: u64,
    pub deleted_chain_rows: u64,
}

/// Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (y + i64::from(m <= 2), m, d)
}

/// ISO-8601 UTC timestamp `days` ago at midnight. Lexicographically
/// comparable with the `events.ts` TEXT column.
pub fn retention_cutoff(days: u32) -> String {
    let days = if days == 0 {
        DEFAULT_RETENTION_DAYS
    } else {
        days
    };
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default() as i64;
    let target_days = now_secs / 86_400 - i64::from(days);
    let (year, month, day) = civil_from_days(target_days);
    format!("{year:04}-{month:02}-{day:02}T00:00:00+00:00")
}

fn batch_ids(conn: &Connection, where_clause: &str, limit: usize) -> Result<Vec<String>> {
    let sql =
        format!("SELECT event_id FROM events WHERE {where_clause} ORDER BY ts, event_id LIMIT ?");
    let mut statement = conn
        .prepare(&sql)
        .with_context(|| format!("prepare batch select: {sql}"))?;
    let ids = statement
        .query_map([limit as i64], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(ids)
}

/// Delete one batch of events and their chain rows. Returns the number of
/// events removed. Explicit per-id chain deletes make the operation immune
/// to foreign_keys pragma differences.
fn delete_batch(conn: &Connection, ids: &[String]) -> Result<u64> {
    let tx = conn
        .unchecked_transaction()
        .context("begin delete transaction")?;
    let mut events_deleted = 0_u64;
    {
        let mut delete_event = tx.prepare("DELETE FROM events WHERE event_id = ?")?;
        let mut delete_chain = tx.prepare("DELETE FROM event_hash_chain WHERE event_id = ?")?;
        for id in ids {
            events_deleted += delete_event.execute([id])? as u64;
            let _ = delete_chain.execute([id])?;
        }
    }
    tx.commit().context("commit delete transaction")?;
    Ok(events_deleted)
}

/// Remove chain rows whose event row is already gone (orphans left by legacy
/// writers), in bounded batches.
fn sweep_orphan_chain_rows(conn: &Connection, batch_size: usize) -> Result<u64> {
    let mut total = 0_u64;
    loop {
        let removed = conn.execute(
            "DELETE FROM event_hash_chain WHERE rowid IN (
               SELECT rowid FROM event_hash_chain
               WHERE event_id NOT IN (SELECT event_id FROM events)
               LIMIT ?)",
            [batch_size as i64],
        )?;
        total += removed as u64;
        if removed == 0 {
            break;
        }
    }
    Ok(total)
}

/// Delete epoch-timestamped placeholder events (and their chain rows) in
/// bounded batches. Safe against the live daemon writer: each batch is its
/// own short transaction.
pub fn prune_epoch_junk(conn: &Connection, batch_size: usize) -> Result<JunkPruneSummary> {
    let batch_size = if batch_size == 0 {
        DEFAULT_BATCH_SIZE
    } else {
        batch_size
    };
    let mut summary = JunkPruneSummary {
        deleted_events: 0,
        deleted_chain_rows: 0,
    };
    loop {
        let ids = batch_ids(conn, &format!("ts < '{JUNK_CUTOFF}'"), batch_size)?;
        if ids.is_empty() {
            break;
        }
        let count = ids.len() as u64;
        delete_batch(conn, &ids)?;
        summary.deleted_events += count;
        summary.deleted_chain_rows += count;
    }
    summary.deleted_chain_rows += sweep_orphan_chain_rows(conn, batch_size)?;
    Ok(summary)
}

fn export_batch(
    conn: &Connection,
    cutoff: &str,
    export_file: &Path,
    batch_size: usize,
) -> Result<Vec<String>> {
    let mut statement = conn.prepare(
        "SELECT event_id, ts, origin, correlation_id, payload_json, machine_id, instance_id, session_id, thread_id, is_observation
         FROM events WHERE ts < ? ORDER BY ts, event_id LIMIT ?",
    )?;
    let rows = statement
        .query_map(rusqlite::params![cutoff, batch_size as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, i64>(9)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(export_file)
        .with_context(|| format!("open cold export {}", export_file.display()))?;
    let mut ids = Vec::with_capacity(rows.len());
    for (id, ts, origin, correlation, payload, machine, instance, session, thread, is_obs) in rows {
        let line = serde_json::json!({
            "event_id": id,
            "ts": ts,
            "origin": origin,
            "correlation_id": correlation,
            "payload_json": payload,
            "machine_id": machine,
            "instance_id": instance,
            "session_id": session,
            "thread_id": thread,
            "is_observation": is_obs,
        });
        writeln!(file, "{line}")?;
        ids.push(id);
    }
    file.flush()?;
    file.sync_all()?;
    Ok(ids)
}

/// Export events older than `cutoff` to the cold JSONL directory and delete
/// them from the hot SQLite ledger in bounded batches. Then anchor the hash
/// chain and return pages to the filesystem via incremental vacuum.
pub fn prune_before(
    conn: &Connection,
    cutoff: &str,
    export_dir: Option<&Path>,
    batch_size: usize,
) -> Result<RetentionSummary> {
    let batch_size = if batch_size == 0 {
        DEFAULT_BATCH_SIZE
    } else {
        batch_size
    };
    let mut summary = RetentionSummary {
        cutoff_ts: cutoff.to_string(),
        exported_events: 0,
        deleted_events: 0,
        deleted_chain_rows: 0,
        anchor_chain_index: None,
        vacuumed_pages: 0,
    };
    let export_file = match export_dir {
        Some(dir) => {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("create cold export directory {}", dir.display()))?;
            let day = cutoff.get(..10).unwrap_or("archive").replace('-', "");
            Some(dir.join(format!("events-cold-{day}.jsonl")))
        }
        None => None,
    };
    loop {
        let ids = match &export_file {
            Some(file) => export_batch(conn, cutoff, file, batch_size)?,
            None => batch_ids(conn, &format!("ts < '{cutoff}'"), batch_size)?,
        };
        if ids.is_empty() {
            break;
        }
        let count = ids.len() as u64;
        delete_batch(conn, &ids)?;
        summary.deleted_events += count;
        summary.deleted_chain_rows += count;
        if export_file.is_some() {
            summary.exported_events += count;
        }
    }
    summary.deleted_chain_rows += sweep_orphan_chain_rows(conn, batch_size)?;
    summary.anchor_chain_index = anchor_hash_chain(conn, CHAIN_ANCHOR_KEEP)?;
    summary.vacuumed_pages = incremental_vacuum(conn, 10_000)?;
    Ok(summary)
}

/// Drop old hash-chain rows, preserving the latest checkpoint hash in `meta`
/// so forward integrity remains provable after pruning.
pub fn anchor_hash_chain(conn: &Connection, keep: i64) -> Result<Option<i64>> {
    let max_index: Option<i64> = conn
        .query_row("SELECT MAX(chain_index) FROM event_hash_chain", [], |row| {
            row.get(0)
        })
        .unwrap_or(None);
    let Some(max_index) = max_index else {
        return Ok(None);
    };
    let floor = max_index.saturating_sub(keep);
    if floor > 0 {
        conn.execute(
            "DELETE FROM event_hash_chain WHERE chain_index < ?",
            [floor],
        )
        .context("anchor hash chain")?;
    }
    let head_hash: Option<String> = conn
        .query_row(
            "SELECT event_hash FROM event_hash_chain WHERE chain_index = ?",
            [max_index],
            |row| row.get(0),
        )
        .unwrap_or(None);
    if let Some(hash) = &head_hash {
        conn.execute(
            "INSERT INTO meta(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![META_ANCHOR_KEY, format!("{max_index}|{hash}")],
        )
        .context("persist chain anchor")?;
    }
    Ok(Some(max_index))
}

/// Return freed pages to the filesystem. No-op unless the database was
/// created (or vacuued) with `PRAGMA auto_vacuum=INCREMENTAL`.
pub fn incremental_vacuum(conn: &Connection, pages: i64) -> Result<u32> {
    // PRAGMA incremental_vacuum returns no rows when auto_vacuum is NONE;
    // treat that as a no-op returning zero pages.
    let returned: i64 = conn
        .prepare(&format!("PRAGMA incremental_vacuum({pages})"))?
        .query_row([], |row| row.get(0))
        .unwrap_or(0);
    Ok(returned.max(0) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             CREATE TABLE events (
               event_id TEXT PRIMARY KEY,
               ts TEXT NOT NULL,
               origin TEXT NOT NULL,
               correlation_id TEXT,
               payload_json TEXT NOT NULL,
               machine_id TEXT, instance_id TEXT, session_id TEXT, thread_id TEXT,
               is_observation INTEGER NOT NULL DEFAULT 0);
             CREATE INDEX idx_events_ts ON events(ts);
             CREATE TABLE event_hash_chain (
               event_id TEXT PRIMARY KEY,
               chain_index INTEGER NOT NULL,
               previous_hash TEXT NOT NULL,
               payload_sha256 TEXT NOT NULL,
               event_hash TEXT NOT NULL,
               created_at TEXT NOT NULL,
               FOREIGN KEY(event_id) REFERENCES events(event_id) ON DELETE CASCADE);
             CREATE INDEX idx_event_hash_chain_index ON event_hash_chain(chain_index);
             CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        )
        .unwrap();
        conn
    }

    fn seed(conn: &Connection, prefix: &str, n: usize, ts: &str) {
        let mut ins_event = conn
            .prepare("INSERT INTO events (event_id, ts, origin, payload_json) VALUES (?1, ?2, 'test', '{}')")
            .unwrap();
        let mut ins_chain = conn
            .prepare("INSERT INTO event_hash_chain (event_id, chain_index, previous_hash, payload_sha256, event_hash, created_at) VALUES (?1, ?2, 'prev', 'sha', 'hash', 'ts')")
            .unwrap();
        for i in 0..n {
            let id = format!("{prefix}-{i}");
            ins_event.execute(rusqlite::params![id, ts]).unwrap();
            ins_chain.execute(rusqlite::params![id, i as i64]).unwrap();
        }
    }

    #[test]
    fn junk_prune_removes_only_epoch_rows_and_chain() {
        let conn = schema_conn();
        seed(&conn, "junk", 50, "1970-01-01T00:00:00+00:00");
        seed(&conn, "real", 10, "2026-08-01T00:00:00+00:00");
        let summary = prune_epoch_junk(&conn, 7).unwrap();
        assert_eq!(summary.deleted_events, 50);
        let remaining: i64 = conn
            .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(remaining, 10);
        let chain_rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM event_hash_chain", [], |r| r.get(0))
            .unwrap();
        assert_eq!(chain_rows, 10, "only junk chain rows removed");
    }

    #[test]
    fn prune_before_exports_cold_jsonl_and_anchors_chain() {
        let conn = schema_conn();
        seed(&conn, "old", 30, "2026-07-01T00:00:00+00:00");
        seed(&conn, "new", 10, "2026-08-10T00:00:00+00:00");
        let dir = std::env::temp_dir().join(format!(
            "focusa-retention-{}-{}",
            std::process::id(),
            now_unix()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let summary = prune_before(&conn, "2026-08-01T00:00:00+00:00", Some(&dir), 8).unwrap();
        assert_eq!(summary.deleted_events, 30);
        assert_eq!(summary.exported_events, 30);
        let remaining: i64 = conn
            .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(remaining, 10);
        let exported = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".jsonl"))
            .count();
        assert_eq!(exported, 1);
        let anchor: Option<String> = conn
            .query_row(
                "SELECT value FROM meta WHERE key = ?1",
                [META_ANCHOR_KEY],
                |r| r.get(0),
            )
            .unwrap_or(None);
        assert!(anchor.is_some(), "anchor must be persisted");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn retention_cutoff_is_lexicographically_comparable() {
        let cutoff = retention_cutoff(30);
        assert_eq!(cutoff.len(), "2026-08-15T00:00:00+00:00".len());
        assert!(cutoff.as_str() < "2026-08-15T00:00:00+00:00");
    }

    #[test]
    fn orphan_chain_rows_are_swept() {
        let conn = schema_conn();
        conn.execute_batch("PRAGMA foreign_keys=OFF;").unwrap();
        conn.execute(
            "INSERT INTO event_hash_chain (event_id, chain_index, previous_hash, payload_sha256, event_hash, created_at)
             VALUES ('ghost-1', 1, 'p', 's', 'h', 't')",
            [],
        )
        .unwrap();
        let swept = sweep_orphan_chain_rows(&conn, 10).unwrap();
        assert_eq!(swept, 1);
    }

    fn now_unix() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default()
    }
}
