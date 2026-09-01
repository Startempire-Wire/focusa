//! Background job SQLite ledger — create, update, load, list, lapsed.
//!
//! One table, one writer per job lifecycle stage (the CLI monitor).
//! Completions are recorded durably BEFORE the SSE broadcast, mirroring
//! the silent-session completion boundary (#311).

use anyhow::Result;
use rusqlite::{Connection, OptionalExtension, params};

use crate::background_jobs::{BackgroundJobRecord, BackgroundJobStatus};

fn has_column(conn: &Connection, expected: &str) -> Result<bool> {
    Ok(conn
        .prepare("PRAGMA table_info(background_jobs)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .iter()
        .any(|name| name == expected))
}

pub fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS bg_job_stats (
            name TEXT PRIMARY KEY,
            runs INTEGER NOT NULL DEFAULT 0,
            total_ms INTEGER NOT NULL DEFAULT 0,
            ema_ms INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS background_jobs (
            job_id TEXT PRIMARY KEY,
            schema TEXT NOT NULL DEFAULT 'focusa.background_job.v1',
            name TEXT NOT NULL,
            command TEXT NOT NULL,
            cwd TEXT NOT NULL,
            attachment_json TEXT,
            status TEXT NOT NULL,
            exit_code INTEGER,
            pid INTEGER,
            log_path TEXT NOT NULL,
            started_at TEXT NOT NULL,
            completed_at TEXT,
            output_tail TEXT NOT NULL DEFAULT ''
        );
        "#,
    )?;
    for (column, declaration) in [
        (
            "schema",
            "schema TEXT NOT NULL DEFAULT 'focusa.background_job.v1'",
        ),
        ("output_tail", "output_tail TEXT NOT NULL DEFAULT ''"),
        ("attachment_json", "attachment_json TEXT"),
    ] {
        if !has_column(conn, column)? {
            if let Err(error) = conn.execute(
                &format!("ALTER TABLE background_jobs ADD COLUMN {declaration}"),
                [],
            ) {
                // A concurrent first-use migration may have added the column
                // after our initial read. Only accept that proven state.
                if !has_column(conn, column)? {
                    return Err(error.into());
                }
            }
        }
    }
    Ok(())
}

pub fn upsert_job(conn: &Connection, record: &BackgroundJobRecord) -> Result<()> {
    let attachment_json = record
        .attachment
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;
    conn.execute(
        "INSERT INTO background_jobs
         (job_id, name, command, cwd, attachment_json, status, exit_code, pid, log_path, started_at, completed_at, output_tail, schema)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(job_id) DO UPDATE SET
            name = excluded.name,
            command = excluded.command,
            cwd = excluded.cwd,
            attachment_json = excluded.attachment_json,
            status = excluded.status,
            exit_code = excluded.exit_code,
            pid = excluded.pid,
            log_path = excluded.log_path,
            completed_at = excluded.completed_at,
            output_tail = excluded.output_tail,
            schema = excluded.schema",
        params![
            record.job_id,
            record.name,
            record.command,
            record.cwd,
            attachment_json,
            record.status.as_str(),
            record.exit_code,
            record.pid,
            record.log_path,
            record.started_at,
            record.completed_at,
            record.output_tail,
            record.schema,
        ],
    )?;
    Ok(())
}

pub fn load_job(conn: &Connection, job_id: &str) -> Result<Option<BackgroundJobRecord>> {
    conn.query_row(
        "SELECT job_id, name, command, cwd, attachment_json, status, exit_code, pid, log_path, started_at, completed_at, output_tail, schema
         FROM background_jobs WHERE job_id = ?1",
        params![job_id],
        row_from,
    )
    .optional()
    .map_err(Into::into)
}

pub fn list_jobs(conn: &Connection) -> Result<Vec<BackgroundJobRecord>> {
    let mut stmt = conn.prepare(
        "SELECT job_id, name, command, cwd, attachment_json, status, exit_code, pid, log_path, started_at, completed_at, output_tail, schema
         FROM background_jobs ORDER BY started_at DESC",
    )?;
    let rows = stmt.query_map([], row_from)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

/// Running jobs whose monitor process is gone (monitor-lost detection is
/// the CLI's job; this just lists candidates).
/// Update per-name duration stats with an exponential moving average —
/// the ETA source for future runs of the same name.
pub fn record_job_duration(conn: &Connection, name: &str, duration_ms: i64) -> Result<i64> {
    conn.execute(
        "INSERT INTO bg_job_stats (name, runs, total_ms, ema_ms)
         VALUES (?1, 1, ?2, ?2)
         ON CONFLICT(name) DO UPDATE SET
            runs = runs + 1,
            total_ms = total_ms + excluded.total_ms,
            ema_ms = (ema_ms * 7 + excluded.ema_ms) / 8",
        params![name, duration_ms],
    )?;
    conn.query_row(
        "SELECT ema_ms FROM bg_job_stats WHERE name = ?1",
        params![name],
        |row| row.get(0),
    )
    .map_err(Into::into)
}

pub fn eta_ms_for(conn: &Connection, name: &str) -> Result<Option<i64>> {
    conn.query_row(
        "SELECT ema_ms FROM bg_job_stats WHERE name = ?1",
        params![name],
        |row| row.get::<_, i64>(0),
    )
    .optional()
    .map_err(Into::into)
}

pub fn list_running(conn: &Connection) -> Result<Vec<BackgroundJobRecord>> {
    let mut stmt = conn.prepare(
        "SELECT job_id, name, command, cwd, attachment_json, status, exit_code, pid, log_path, started_at, completed_at, output_tail, schema
         FROM background_jobs WHERE status = 'running' ORDER BY started_at",
    )?;
    let rows = stmt.query_map([], row_from)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn row_from(row: &rusqlite::Row<'_>) -> rusqlite::Result<BackgroundJobRecord> {
    let attachment_json: Option<String> = row.get(4)?;
    let attachment = attachment_json
        .map(|value| serde_json::from_str(&value))
        .transpose()
        .map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;
    Ok(BackgroundJobRecord {
        schema: row.get(12)?,
        job_id: row.get(0)?,
        name: row.get(1)?,
        command: row.get(2)?,
        cwd: row.get(3)?,
        attachment,
        status: BackgroundJobStatus::parse(&row.get::<_, String>(5)?),
        exit_code: row.get(6)?,
        pid: row.get(7)?,
        log_path: row.get(8)?,
        started_at: row.get(9)?,
        completed_at: row.get(10)?,
        output_tail: row.get(11)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::background_jobs::{BackgroundJobRecord, BackgroundJobStatus};
    use crate::scoped_state::{AttachmentKey, ScopeRef, WorkstreamKey};

    fn attachment() -> AttachmentKey {
        let root = std::env::temp_dir().join("focusa-bg-project");
        let scope =
            ScopeRef::project("project:bg", root, "Background Project", "fingerprint:bg").unwrap();
        AttachmentKey::new(
            WorkstreamKey::new(scope, "continuity-bg").unwrap(),
            "pi-42",
            "session-bg",
            "attachment-bg",
        )
        .unwrap()
    }

    fn sample(id: &str) -> BackgroundJobRecord {
        BackgroundJobRecord {
            schema: crate::background_jobs::BACKGROUND_JOB_SCHEMA.to_string(),
            job_id: id.to_string(),
            name: "gate".to_string(),
            command: "cargo test".to_string(),
            cwd: "/root/proj".to_string(),
            attachment: None,
            status: BackgroundJobStatus::Queued,
            exit_code: None,
            pid: None,
            log_path: format!("/tmp/{id}.log"),
            started_at: "t0".to_string(),
            completed_at: None,
            output_tail: String::new(),
        }
    }

    #[test]
    fn job_roundtrip_and_transitions() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        let mut job = sample("j1");
        upsert_job(&conn, &job).unwrap();
        let loaded = load_job(&conn, "j1").unwrap().expect("exists");
        assert_eq!(loaded.status, BackgroundJobStatus::Queued);
        job.status = BackgroundJobStatus::Running;
        job.pid = Some(42);
        upsert_job(&conn, &job).unwrap();
        job.status = BackgroundJobStatus::Completed;
        job.exit_code = Some(0);
        job.completed_at = Some("t1".to_string());
        upsert_job(&conn, &job).unwrap();
        let loaded = load_job(&conn, "j1").unwrap().expect("exists");
        assert_eq!(loaded.status, BackgroundJobStatus::Completed);
        assert_eq!(loaded.exit_code, Some(0));
        assert_eq!(loaded.completed_at.as_deref(), Some("t1"));
    }

    #[test]
    fn existing_schema_migrates_and_persists_monitor_tail() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE background_jobs (
                job_id TEXT PRIMARY KEY, name TEXT NOT NULL, command TEXT NOT NULL,
                cwd TEXT NOT NULL, status TEXT NOT NULL, exit_code INTEGER, pid INTEGER,
                log_path TEXT NOT NULL, started_at TEXT NOT NULL, completed_at TEXT
            );",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO background_jobs
             (job_id, name, command, cwd, status, log_path, started_at)
             VALUES ('legacy-schema', 'legacy', 'true', '.', 'queued', '/tmp/legacy.log', 't0')",
            [],
        )
        .unwrap();
        ensure_schema(&conn).unwrap();
        let legacy = load_job(&conn, "legacy-schema").unwrap().unwrap();
        assert_eq!(
            legacy.schema,
            crate::background_jobs::BACKGROUND_JOB_SCHEMA_V1
        );
        assert_eq!(legacy.attachment, None);
        assert_eq!(legacy.output_tail, "");

        let mut job = sample("current-schema");
        job.status = BackgroundJobStatus::Failed;
        job.exit_code = Some(1);
        job.output_tail = "compiler error".into();
        upsert_job(&conn, &job).unwrap();
        let current = load_job(&conn, "current-schema").unwrap().unwrap();
        assert_eq!(
            current.schema,
            crate::background_jobs::BACKGROUND_JOB_SCHEMA
        );
        assert_eq!(current.output_tail, "compiler error");
    }

    #[test]
    fn exact_attachment_round_trips_and_legacy_rows_remain_unscoped() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        let mut scoped = sample("scoped");
        scoped.attachment = Some(attachment());
        upsert_job(&conn, &scoped).unwrap();
        assert_eq!(
            load_job(&conn, "scoped").unwrap().unwrap().attachment,
            scoped.attachment
        );

        let mut legacy = sample("legacy");
        legacy.schema = crate::background_jobs::BACKGROUND_JOB_SCHEMA_V1.to_string();
        upsert_job(&conn, &legacy).unwrap();
        let loaded_legacy = load_job(&conn, "legacy").unwrap().unwrap();
        assert_eq!(loaded_legacy.attachment, None);
        assert_eq!(
            loaded_legacy.schema,
            crate::background_jobs::BACKGROUND_JOB_SCHEMA_V1
        );
        assert_eq!(
            load_job(&conn, "scoped").unwrap().unwrap().schema,
            crate::background_jobs::BACKGROUND_JOB_SCHEMA
        );
    }

    #[test]
    fn upsert_is_idempotent_per_job() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        upsert_job(&conn, &sample("j1")).unwrap();
        upsert_job(&conn, &sample("j1")).unwrap();
        assert_eq!(list_jobs(&conn).unwrap().len(), 1);
    }

    #[test]
    fn list_running_filters_by_status() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        let mut running = sample("r1");
        running.status = BackgroundJobStatus::Running;
        running.pid = Some(7);
        upsert_job(&conn, &sample("q1")).unwrap();
        upsert_job(&conn, &running).unwrap();
        let running_jobs = list_running(&conn).unwrap();
        assert_eq!(running_jobs.len(), 1);
        assert_eq!(running_jobs[0].job_id, "r1");
    }
}
