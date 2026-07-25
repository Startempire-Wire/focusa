//! Spec133 retention, export, ordinary deletion, and purge persistence.

use chrono::Utc;
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::runtime::persistence_sqlite::SqlitePersistence;

use super::{
    SilentSessionId, SilentSessionRunId, list_checkpoint_values, list_completion_evaluations,
    load_session, load_session_events,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SilentSessionRetentionRecord {
    pub session_id: String,
    pub evidence_hold: bool,
    pub hold_reason: Option<String>,
    pub hold_expires_at: Option<String>,
    pub deleted_at: Option<String>,
    pub delete_reason: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SilentSessionPurgePlan {
    pub session_id: String,
    pub evidence_hold: bool,
    pub table_counts: Vec<(String, u64)>,
    pub committed: bool,
}

pub fn load_retention_operation(
    persistence: &SqlitePersistence,
    session_id: SilentSessionId,
    action: &str,
    idempotency_key: &str,
    principal_id: &str,
) -> anyhow::Result<Option<(String, Value)>> {
    persistence.with_connection_mut(|connection| {
        connection
            .query_row(
                "SELECT request_hash,response_json FROM silent_session_control_retention_operations WHERE session_id=?1 AND action=?2 AND idempotency_key=?3 AND principal_id=?4",
                params![session_id.to_string(), action, idempotency_key, principal_id],
                |row| {
                    let request_hash: String = row.get(0)?;
                    let response_json: String = row.get(1)?;
                    let response = serde_json::from_str(&response_json).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            response_json.len(),
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?;
                    Ok((request_hash, response))
                },
            )
            .optional()
            .map_err(Into::into)
    })
}

pub fn save_retention_operation(
    persistence: &SqlitePersistence,
    session_id: SilentSessionId,
    action: &str,
    idempotency_key: &str,
    principal_id: &str,
    request_hash: &str,
    response: &Value,
) -> anyhow::Result<()> {
    persistence.with_connection_mut(|connection| {
        connection.execute(
            "INSERT INTO silent_session_control_retention_operations(session_id,action,idempotency_key,principal_id,request_hash,response_json,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![session_id.to_string(), action, idempotency_key, principal_id, request_hash, serde_json::to_string(response)?, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    })
}

pub fn load_retention_record(
    persistence: &SqlitePersistence,
    session_id: SilentSessionId,
) -> anyhow::Result<Option<SilentSessionRetentionRecord>> {
    persistence.with_connection_mut(|connection| {
        connection
            .query_row(
                "SELECT session_id,evidence_hold,hold_reason,hold_expires_at,deleted_at,delete_reason,updated_at FROM silent_session_control_retention WHERE session_id=?1",
                params![session_id.to_string()],
                |row| {
                    Ok(SilentSessionRetentionRecord {
                        session_id: row.get(0)?,
                        evidence_hold: row.get::<_, i64>(1)? != 0,
                        hold_reason: row.get(2)?,
                        hold_expires_at: row.get(3)?,
                        deleted_at: row.get(4)?,
                        delete_reason: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    })
}

pub fn set_evidence_hold(
    persistence: &SqlitePersistence,
    session_id: SilentSessionId,
    reason: &str,
    expires_at: Option<&str>,
) -> anyhow::Result<SilentSessionRetentionRecord> {
    let now = Utc::now().to_rfc3339();
    persistence.with_connection_mut(|connection| {
        connection.execute(
            "INSERT INTO silent_session_control_retention(session_id,evidence_hold,hold_reason,hold_expires_at,updated_at) VALUES (?1,1,?2,?3,?4) ON CONFLICT(session_id) DO UPDATE SET evidence_hold=1,hold_reason=excluded.hold_reason,hold_expires_at=excluded.hold_expires_at,updated_at=excluded.updated_at",
            params![session_id.to_string(), reason, expires_at, now],
        )?;
        Ok(())
    })?;
    load_retention_record(persistence, session_id)?
        .ok_or_else(|| anyhow::anyhow!("retention record missing after hold"))
}

pub fn ordinary_delete_session(
    persistence: &SqlitePersistence,
    session_id: SilentSessionId,
    reason: &str,
) -> anyhow::Result<SilentSessionRetentionRecord> {
    let now = Utc::now().to_rfc3339();
    persistence.with_connection_mut(|connection| {
        connection.execute(
            "INSERT INTO silent_session_control_retention(session_id,evidence_hold,deleted_at,delete_reason,updated_at) VALUES (?1,0,?2,?3,?2) ON CONFLICT(session_id) DO UPDATE SET deleted_at=excluded.deleted_at,delete_reason=excluded.delete_reason,updated_at=excluded.updated_at",
            params![session_id.to_string(), now, reason],
        )?;
        Ok(())
    })?;
    load_retention_record(persistence, session_id)?
        .ok_or_else(|| anyhow::anyhow!("retention record missing after delete"))
}

pub fn export_session_bundle(
    persistence: &SqlitePersistence,
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
) -> anyhow::Result<Value> {
    let session = load_session(persistence, session_id)?
        .ok_or_else(|| anyhow::anyhow!("silent session not found"))?;
    Ok(json!({
        "schema": "focusa.silent_session_export.v1",
        "session": session,
        "events": load_session_events(persistence, session_id)?,
        "checkpoints": list_checkpoint_values(persistence, session_id, run_id)?,
        "completion_evaluations": list_completion_evaluations(persistence, session_id, run_id)?,
        "retention": load_retention_record(persistence, session_id)?,
        "exported_at": Utc::now().to_rfc3339(),
    }))
}

pub fn purge_session(
    persistence: &SqlitePersistence,
    session_id: SilentSessionId,
    commit: bool,
) -> anyhow::Result<SilentSessionPurgePlan> {
    let retention = load_retention_record(persistence, session_id)?;
    let evidence_hold = retention.as_ref().is_some_and(|record| {
        record.evidence_hold
            && record.hold_expires_at.as_deref().is_none_or(|expires_at| {
                chrono::DateTime::parse_from_rfc3339(expires_at)
                    .map(|expires_at| expires_at > Utc::now())
                    .unwrap_or(true)
            })
    });
    if evidence_hold {
        anyhow::bail!("evidence_hold_active");
    }
    let tables = [
        ("silent_session_control_notifications", "silent_session_id"),
        (
            "silent_session_control_completion_evaluations",
            "silent_session_id",
        ),
        ("silent_session_control_checkpoints", "silent_session_id"),
        ("silent_session_control_stream_indexes", "silent_session_id"),
        ("silent_session_control_events", "silent_session_id"),
        ("silent_session_control_leases", "silent_session_id"),
        (
            "silent_session_control_backend_bindings",
            "silent_session_id",
        ),
        ("silent_session_control_approvals", "session_id"),
        ("silent_session_control_audits", "session_id"),
        ("silent_session_daemon_runs", "silent_session_id"),
        (
            "silent_session_control_config_revisions",
            "silent_session_id",
        ),
        ("silent_session_control_retention", "session_id"),
        ("silent_session_controls", "silent_session_id"),
    ];
    let session = session_id.to_string();
    let table_counts = persistence.with_connection_mut(|connection| {
        let transaction = connection.transaction()?;
        let mut counts = Vec::new();
        for (table, column) in tables {
            let count: i64 = transaction.query_row(
                &format!("SELECT COUNT(*) FROM {table} WHERE {column}=?1"),
                params![&session],
                |row| row.get(0),
            )?;
            counts.push((table.to_string(), count.max(0) as u64));
            if commit {
                transaction.execute(
                    &format!("DELETE FROM {table} WHERE {column}=?1"),
                    params![&session],
                )?;
            }
        }
        if commit {
            transaction.commit()?;
        } else {
            transaction.rollback()?;
        }
        Ok(counts)
    })?;
    Ok(SilentSessionPurgePlan {
        session_id: session,
        evidence_hold,
        table_counts,
        committed: commit,
    })
}
