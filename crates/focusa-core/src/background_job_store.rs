//! Background job SQLite ledger — create, update, load, list, lapsed.
//!
//! One table, one writer per job lifecycle stage (the CLI monitor).
//! Completions are recorded durably BEFORE the SSE broadcast, mirroring
//! the silent-session completion boundary (#311).

use anyhow::Result;
use rusqlite::{Connection, OptionalExtension, params};

use crate::background_jobs::{BackgroundJobFailureClass, BackgroundJobRecord, BackgroundJobStatus};

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
            name TEXT NOT NULL,
            command TEXT NOT NULL,
            cwd TEXT NOT NULL,
            status TEXT NOT NULL,
            failure_class TEXT,
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
        ("output_tail", "output_tail TEXT NOT NULL DEFAULT ''"),
        ("failure_class", "failure_class TEXT"),
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
    conn.execute(
        "INSERT INTO background_jobs
         (job_id, name, command, cwd, status, failure_class, exit_code, pid, log_path, started_at, completed_at, output_tail)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
         ON CONFLICT(job_id) DO UPDATE SET
            name = excluded.name,
            command = excluded.command,
            cwd = excluded.cwd,
            status = excluded.status,
            failure_class = excluded.failure_class,
            exit_code = excluded.exit_code,
            pid = excluded.pid,
            log_path = excluded.log_path,
            completed_at = excluded.completed_at,
            output_tail = excluded.output_tail",
        params![
            record.job_id,
            record.name,
            record.command,
            record.cwd,
            record.status.as_str(),
            record.failure_class.map(BackgroundJobFailureClass::as_str),
            record.exit_code,
            record.pid,
            record.log_path,
            record.started_at,
            record.completed_at,
            record.output_tail,
        ],
    )?;
    Ok(())
}

pub fn load_job(conn: &Connection, job_id: &str) -> Result<Option<BackgroundJobRecord>> {
    conn.query_row(
        "SELECT job_id, name, command, cwd, status, exit_code, pid, log_path, started_at, completed_at, output_tail, failure_class
         FROM background_jobs WHERE job_id = ?1",
        params![job_id],
        row_from,
    )
    .optional()
    .map_err(Into::into)
}

pub fn list_jobs(conn: &Connection) -> Result<Vec<BackgroundJobRecord>> {
    let mut stmt = conn.prepare(
        "SELECT job_id, name, command, cwd, status, exit_code, pid, log_path, started_at, completed_at, output_tail, failure_class
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
        "SELECT job_id, name, command, cwd, status, exit_code, pid, log_path, started_at, completed_at, output_tail, failure_class
         FROM background_jobs WHERE status = 'running' ORDER BY started_at",
    )?;
    let rows = stmt.query_map([], row_from)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn row_from(row: &rusqlite::Row<'_>) -> rusqlite::Result<BackgroundJobRecord> {
    let failure_class = row
        .get::<_, Option<String>>(11)?
        .map(|value| {
            BackgroundJobFailureClass::parse(&value).ok_or_else(|| {
                rusqlite::Error::FromSqlConversionFailure(
                    11,
                    rusqlite::types::Type::Text,
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("unknown background job failure class: {value}"),
                    )
                    .into(),
                )
            })
        })
        .transpose()?;
    Ok(BackgroundJobRecord {
        schema: crate::background_jobs::BACKGROUND_JOB_SCHEMA.to_string(),
        job_id: row.get(0)?,
        name: row.get(1)?,
        command: row.get(2)?,
        cwd: row.get(3)?,
        status: BackgroundJobStatus::parse(&row.get::<_, String>(4)?),
        failure_class,
        exit_code: row.get(5)?,
        pid: row.get(6)?,
        log_path: row.get(7)?,
        started_at: row.get(8)?,
        completed_at: row.get(9)?,
        output_tail: row.get(10)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::background_jobs::{BackgroundJobRecord, BackgroundJobStatus};

    fn sample(id: &str) -> BackgroundJobRecord {
        BackgroundJobRecord {
            schema: crate::background_jobs::BACKGROUND_JOB_SCHEMA.to_string(),
            job_id: id.to_string(),
            name: "gate".to_string(),
            command: "cargo test".to_string(),
            cwd: "/root/proj".to_string(),
            status: BackgroundJobStatus::Queued,
            failure_class: None,
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
        ensure_schema(&conn).unwrap();
        let mut job = sample("legacy-schema");
        job.status = BackgroundJobStatus::Failed;
        job.failure_class = Some(BackgroundJobFailureClass::LaunchFailed);
        job.exit_code = Some(126);
        job.output_tail = "compiler error".into();
        upsert_job(&conn, &job).unwrap();
        let loaded = load_job(&conn, "legacy-schema").unwrap().unwrap();
        assert_eq!(loaded.output_tail, "compiler error");
        assert_eq!(
            loaded.failure_class,
            Some(BackgroundJobFailureClass::LaunchFailed)
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
