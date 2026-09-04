//! Background job SQLite ledger — create, update, load, list, lapsed.
//!
//! One table, one writer per job lifecycle stage (the CLI monitor).
//! Completions are recorded durably BEFORE the SSE broadcast, mirroring
//! the silent-session completion boundary (#311).

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};

use crate::background_jobs::{
    BackgroundJobFailureClass, BackgroundJobRecord, BackgroundJobStatus, ProcessIdentityStatus,
    process_identity_status,
};

const NONTERMINAL_GRACE_SECONDS: i64 = 30;

fn has_column(conn: &Connection, expected: &str) -> Result<bool> {
    Ok(conn
        .prepare("PRAGMA table_info(background_jobs)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .iter()
        .any(|name| name == expected))
}

/// Project every supported background-job schema into the current row shape
/// without migrating it. Recovery-only GET routes use this projection through
/// a read-only SQLite connection, so legacy v1/v2 rows remain inspectable while
/// the damaged authority plane cannot authorize schema or lifecycle writes.
fn read_projection(conn: &Connection) -> Result<String> {
    let attachment = if has_column(conn, "attachment_json")? {
        "attachment_json"
    } else {
        "NULL AS attachment_json"
    };
    let output_tail = if has_column(conn, "output_tail")? {
        "output_tail"
    } else {
        "'' AS output_tail"
    };
    let schema = if has_column(conn, "schema")? {
        "schema"
    } else {
        "'focusa.background_job.v1' AS schema"
    };
    let failure_class = if has_column(conn, "failure_class")? {
        "failure_class"
    } else {
        "NULL AS failure_class"
    };
    let process_start_token = if has_column(conn, "process_start_token")? {
        "process_start_token"
    } else {
        "NULL AS process_start_token"
    };
    Ok(format!(
        "job_id, name, command, cwd, {attachment}, status, exit_code, pid, \
         log_path, started_at, completed_at, {output_tail}, {schema}, \
         {failure_class}, {process_start_token}"
    ))
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
            failure_class TEXT,
            exit_code INTEGER,
            pid INTEGER,
            log_path TEXT NOT NULL,
            started_at TEXT NOT NULL,
            completed_at TEXT,
            output_tail TEXT NOT NULL DEFAULT '',
            process_start_token TEXT
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
        ("failure_class", "failure_class TEXT"),
        ("process_start_token", "process_start_token TEXT"),
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
         (job_id, name, command, cwd, attachment_json, status, failure_class, exit_code, pid, log_path, started_at, completed_at, output_tail, schema, process_start_token)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
         ON CONFLICT(job_id) DO UPDATE SET
            name = excluded.name,
            command = excluded.command,
            cwd = excluded.cwd,
            attachment_json = excluded.attachment_json,
            status = excluded.status,
            failure_class = excluded.failure_class,
            exit_code = excluded.exit_code,
            pid = excluded.pid,
            log_path = excluded.log_path,
            completed_at = excluded.completed_at,
            output_tail = excluded.output_tail,
            schema = excluded.schema,
            process_start_token = excluded.process_start_token",
        params![
            record.job_id,
            record.name,
            record.command,
            record.cwd,
            attachment_json,
            record.status.as_str(),
            record.failure_class.map(BackgroundJobFailureClass::as_str),
            record.exit_code,
            record.pid,
            record.log_path,
            record.started_at,
            record.completed_at,
            record.output_tail,
            record.schema,
            record.process_start_token,
        ],
    )?;
    Ok(())
}

pub fn load_job(conn: &Connection, job_id: &str) -> Result<Option<BackgroundJobRecord>> {
    let columns = read_projection(conn)?;
    conn.query_row(
        &format!("SELECT {columns} FROM background_jobs WHERE job_id = ?1"),
        params![job_id],
        row_from,
    )
    .optional()
    .map_err(Into::into)
}

pub fn list_jobs(conn: &Connection) -> Result<Vec<BackgroundJobRecord>> {
    let columns = read_projection(conn)?;
    let mut stmt = conn.prepare(&format!(
        "SELECT {columns} FROM background_jobs ORDER BY started_at DESC"
    ))?;
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
    let stats_exist = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'bg_job_stats')",
        [],
        |row| row.get::<_, bool>(0),
    )?;
    if !stats_exist {
        return Ok(None);
    }
    conn.query_row(
        "SELECT ema_ms FROM bg_job_stats WHERE name = ?1",
        params![name],
        |row| row.get::<_, i64>(0),
    )
    .optional()
    .map_err(Into::into)
}

pub fn list_running(conn: &Connection) -> Result<Vec<BackgroundJobRecord>> {
    let columns = read_projection(conn)?;
    let mut stmt = conn.prepare(&format!(
        "SELECT {columns} FROM background_jobs WHERE status = 'running' ORDER BY started_at"
    ))?;
    let rows = stmt.query_map([], row_from)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

pub fn list_nonterminal(conn: &Connection) -> Result<Vec<BackgroundJobRecord>> {
    let columns = read_projection(conn)?;
    let mut stmt = conn.prepare(&format!(
        "SELECT {columns} FROM background_jobs \
         WHERE status IN ('queued', 'running') ORDER BY started_at"
    ))?;
    let rows = stmt.query_map([], row_from)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

/// Settle stale rows only when their recorded process is absent or its
/// start identity mismatches. Legacy queued rows without a PID settle
/// after the grace period because they have no possible lifecycle owner.
pub fn reconcile_stale_jobs(
    conn: &Connection,
    now: DateTime<Utc>,
) -> Result<Vec<BackgroundJobRecord>> {
    ensure_schema(conn)?;
    let mut settled = Vec::new();
    for mut record in list_nonterminal(conn)? {
        let age_seconds = record
            .started_at
            .parse::<DateTime<Utc>>()
            .map(|started| (now - started).num_seconds())
            .unwrap_or_default();
        if record.status == BackgroundJobStatus::Queued && age_seconds < NONTERMINAL_GRACE_SECONDS {
            continue;
        }
        let identity = record.pid.map_or(ProcessIdentityStatus::Missing, |pid| {
            process_identity_status(pid, record.process_start_token.as_deref())
        });
        if matches!(
            identity,
            ProcessIdentityStatus::Match | ProcessIdentityStatus::Unknown
        ) {
            continue;
        }
        match record.status {
            BackgroundJobStatus::Queued => {
                record.status = BackgroundJobStatus::Failed;
                record.failure_class = Some(BackgroundJobFailureClass::LaunchFailed);
                record.exit_code = Some(BackgroundJobFailureClass::LaunchFailed.exit_code());
                record.output_tail =
                    "[launch_failed:daemon_reconcile] lifecycle owner is missing".to_string();
            }
            BackgroundJobStatus::Running => {
                record.status = BackgroundJobStatus::MonitorLost;
                record.failure_class = Some(BackgroundJobFailureClass::MonitorFailed);
                record.exit_code = Some(BackgroundJobFailureClass::MonitorFailed.exit_code());
                record.output_tail =
                    "[monitor_failed:daemon_reconcile] lifecycle owner is missing".to_string();
            }
            _ => continue,
        }
        record.completed_at = Some(now.to_rfc3339());
        upsert_job(conn, &record)?;
        settled.push(record);
    }
    Ok(settled)
}

fn row_from(row: &rusqlite::Row<'_>) -> rusqlite::Result<BackgroundJobRecord> {
    let failure_class = row
        .get::<_, Option<String>>(13)?
        .map(|value| {
            BackgroundJobFailureClass::parse(&value).ok_or_else(|| {
                rusqlite::Error::FromSqlConversionFailure(
                    13,
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
        failure_class,
        exit_code: row.get(6)?,
        pid: row.get(7)?,
        process_start_token: row.get(14)?,
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
            failure_class: None,
            exit_code: None,
            pid: None,
            process_start_token: None,
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
    fn legacy_schema_reads_without_migration() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE background_jobs (
                job_id TEXT PRIMARY KEY, name TEXT NOT NULL, command TEXT NOT NULL,
                cwd TEXT NOT NULL, status TEXT NOT NULL, exit_code INTEGER, pid INTEGER,
                log_path TEXT NOT NULL, started_at TEXT NOT NULL, completed_at TEXT
            );
            INSERT INTO background_jobs
                (job_id, name, command, cwd, status, log_path, started_at)
                VALUES ('legacy-read', 'legacy', 'true', '.', 'queued', '/tmp/legacy.log', 't0');",
        )
        .unwrap();
        let columns_before = conn
            .prepare("PRAGMA table_info(background_jobs)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();

        let loaded = load_job(&conn, "legacy-read").unwrap().unwrap();
        assert_eq!(
            loaded.schema,
            crate::background_jobs::BACKGROUND_JOB_SCHEMA_V1
        );
        assert_eq!(loaded.attachment, None);
        assert_eq!(loaded.output_tail, "");
        assert_eq!(loaded.failure_class, None);
        assert_eq!(loaded.process_start_token, None);
        assert_eq!(list_jobs(&conn).unwrap().len(), 1);
        assert_eq!(list_nonterminal(&conn).unwrap().len(), 1);
        assert_eq!(eta_ms_for(&conn, "legacy").unwrap(), None);

        let columns_after = conn
            .prepare("PRAGMA table_info(background_jobs)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(columns_after, columns_before);
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
        assert_eq!(legacy.process_start_token, None);

        let mut job = sample("current-schema");
        job.status = BackgroundJobStatus::Failed;
        job.failure_class = Some(BackgroundJobFailureClass::LaunchFailed);
        job.exit_code = Some(126);
        job.output_tail = "compiler error".into();
        upsert_job(&conn, &job).unwrap();
        let current = load_job(&conn, "current-schema").unwrap().unwrap();
        assert_eq!(
            current.schema,
            crate::background_jobs::BACKGROUND_JOB_SCHEMA
        );
        assert_eq!(current.output_tail, "compiler error");
        assert_eq!(
            current.failure_class,
            Some(BackgroundJobFailureClass::LaunchFailed)
        );
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

    #[test]
    fn reconciliation_settles_legacy_rows_without_live_owners() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        let mut queued = sample("q1");
        queued.started_at = "2026-01-01T00:00:00Z".to_string();
        let mut running = sample("r1");
        running.status = BackgroundJobStatus::Running;
        running.pid = Some(u32::MAX);
        running.started_at = "2026-01-01T00:00:00Z".to_string();
        upsert_job(&conn, &queued).unwrap();
        upsert_job(&conn, &running).unwrap();

        let settled = reconcile_stale_jobs(&conn, "2026-01-02T00:00:00Z".parse().unwrap()).unwrap();
        assert_eq!(settled.len(), 2);
        let queued = load_job(&conn, "q1").unwrap().unwrap();
        assert_eq!(queued.status, BackgroundJobStatus::Failed);
        assert_eq!(
            queued.failure_class,
            Some(BackgroundJobFailureClass::LaunchFailed)
        );
        let running = load_job(&conn, "r1").unwrap().unwrap();
        assert_eq!(running.status, BackgroundJobStatus::MonitorLost);
        assert_eq!(
            running.failure_class,
            Some(BackgroundJobFailureClass::MonitorFailed)
        );
        assert!(list_nonterminal(&conn).unwrap().is_empty());
    }

    #[test]
    fn reconciliation_preserves_a_matching_live_process() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        let Some(token) = crate::background_jobs::current_process_start_token() else {
            return;
        };
        let mut running = sample("live");
        running.status = BackgroundJobStatus::Running;
        running.pid = Some(std::process::id());
        running.process_start_token = Some(token);
        running.started_at = "2026-01-01T00:00:00Z".to_string();
        upsert_job(&conn, &running).unwrap();

        let settled = reconcile_stale_jobs(&conn, "2026-01-02T00:00:00Z".parse().unwrap()).unwrap();
        assert!(settled.is_empty());
        assert_eq!(
            load_job(&conn, "live").unwrap().unwrap().status,
            BackgroundJobStatus::Running
        );
    }
}
