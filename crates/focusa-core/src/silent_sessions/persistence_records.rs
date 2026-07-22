use rusqlite::{OptionalExtension, params};
use serde::{Serialize, de::DeserializeOwned};

use crate::runtime::persistence_sqlite::SqlitePersistence;

use super::{
    CompletionEvaluation, CompletionEvaluationId, ConfigRevisionId, RuntimeCheckpoint,
    RuntimeCheckpointId, SilentSession, SilentSessionConfigRevision, SilentSessionEvent,
    SilentSessionId, SilentSessionLease, SilentSessionLeaseId, SilentSessionRun,
    SilentSessionRunId, SilentSessionWorkpointCheckpoint, WorkpointCheckpointId,
};

pub fn load_session(
    persistence: &SqlitePersistence,
    id: SilentSessionId,
) -> anyhow::Result<Option<SilentSession>> {
    persistence.with_connection_mut(|connection| {
        let json = connection
            .query_row(
                "SELECT snapshot_json FROM silent_sessions WHERE silent_session_id=?1",
                [id.to_string()],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        deserialize_optional(json)
    })
}

pub fn list_sessions(persistence: &SqlitePersistence) -> anyhow::Result<Vec<SilentSession>> {
    persistence.with_connection_mut(|connection| {
        let mut statement = connection.prepare(
            "SELECT snapshot_json FROM silent_sessions ORDER BY updated_at DESC, silent_session_id",
        )?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        rows.map(|row| {
            let json = row?;
            serde_json::from_str(&json).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
    })
}

pub fn load_session_by_idempotency_key(
    persistence: &SqlitePersistence,
    idempotency_key: &str,
) -> anyhow::Result<Option<(SilentSession, serde_json::Value)>> {
    persistence.with_connection_mut(|connection| {
        let mut statement = connection.prepare(
            r#"SELECT s.snapshot_json,e.payload_json
               FROM silent_session_events e
               JOIN silent_sessions s ON s.silent_session_id=e.silent_session_id
               WHERE e.idempotency_key=?1 ORDER BY e.occurred_at,e.event_id"#,
        )?;
        let rows = statement.query_map([idempotency_key], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut matches = rows.collect::<Result<Vec<_>, _>>()?;
        if matches.len() > 1 {
            anyhow::bail!("idempotency key is not unique across Silent Sessions");
        }
        matches
            .pop()
            .map(|(session, payload)| {
                Ok((
                    serde_json::from_str(&session)?,
                    serde_json::from_str(&payload)?,
                ))
            })
            .transpose()
    })
}

pub fn load_session_events(
    persistence: &SqlitePersistence,
    id: SilentSessionId,
) -> anyhow::Result<Vec<SilentSessionEvent>> {
    persistence.with_connection_mut(|connection| {
        let mut statement = connection.prepare(
            r#"SELECT event_id,run_id,sequence,event_schema_version,kind,payload_json,
               idempotency_key,previous_event_hash,event_hash,occurred_at
               FROM silent_session_events WHERE silent_session_id=?1 ORDER BY sequence"#,
        )?;
        let rows = statement.query_map([id.to_string()], |row| {
            let event_id: String = row.get(0)?;
            let run_id: Option<String> = row.get(1)?;
            let occurred_at: String = row.get(9)?;
            Ok((
                event_id,
                run_id,
                row.get::<_, u64>(2)?,
                row.get::<_, u32>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, String>(8)?,
                occurred_at,
            ))
        })?;
        rows.map(|row| {
            let (
                event_id,
                run_id,
                sequence,
                event_schema_version,
                kind,
                payload_json,
                idempotency_key,
                previous_event_hash,
                event_hash,
                occurred_at,
            ) = row?;
            Ok(SilentSessionEvent {
                event_schema_version,
                id: event_id.parse()?,
                silent_session_id: id,
                run_id: run_id.map(|value| value.parse()).transpose()?,
                sequence,
                kind,
                payload: serde_json::from_str(&payload_json)?,
                idempotency_key,
                previous_event_hash,
                event_hash,
                occurred_at: chrono::DateTime::parse_from_rfc3339(&occurred_at)?
                    .with_timezone(&chrono::Utc),
            })
        })
        .collect()
    })
}

pub fn save_run(persistence: &SqlitePersistence, run: &SilentSessionRun) -> anyhow::Result<()> {
    persistence.with_connection_mut(|connection| {
        connection.execute(
            r#"INSERT INTO silent_session_runs(
               run_id,silent_session_id,run_generation,actor_instance_id,config_revision_id,
               protocol_versions_json,run_json,started_at,ended_at
               ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
               ON CONFLICT(run_id) DO UPDATE SET run_json=excluded.run_json,ended_at=excluded.ended_at"#,
            params![
                run.id.to_string(),
                run.silent_session_id.to_string(),
                run.generation.get(),
                run.actor_instance_id.to_string(),
                run.config_revision_id.to_string(),
                serde_json::to_string(&run.protocol_versions)?,
                serde_json::to_string(run)?,
                run.started_at.to_rfc3339(),
                run.ended_at.map(|value| value.to_rfc3339()),
            ],
        )?;
        Ok(())
    })
}

pub fn load_run(
    persistence: &SqlitePersistence,
    id: SilentSessionRunId,
) -> anyhow::Result<Option<SilentSessionRun>> {
    load_json_by_id(
        persistence,
        "SELECT run_json FROM silent_session_runs WHERE run_id=?1",
        id.to_string(),
    )
}

pub fn save_config_revision(
    persistence: &SqlitePersistence,
    revision: &SilentSessionConfigRevision,
) -> anyhow::Result<()> {
    persistence.with_connection_mut(|connection| {
        connection.execute(
            r#"INSERT OR IGNORE INTO silent_session_config_revisions(
               config_revision_id,silent_session_id,revision,config_schema_version,config_json,
               redacted_config_hash,created_by,created_at
               ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)"#,
            params![
                revision.id.to_string(),
                revision.silent_session_id.to_string(),
                revision.revision,
                revision.config_schema_version,
                serde_json::to_string(&revision.config)?,
                revision.redacted_config_hash,
                revision.created_by.to_string(),
                revision.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    })
}

pub fn load_config_revision(
    persistence: &SqlitePersistence,
    id: ConfigRevisionId,
) -> anyhow::Result<Option<SilentSessionConfigRevision>> {
    persistence.with_connection_mut(|connection| {
        let row = connection
            .query_row(
                r#"SELECT silent_session_id,revision,config_schema_version,config_json,
                   redacted_config_hash,created_by,created_at
                   FROM silent_session_config_revisions WHERE config_revision_id=?1"#,
                [id.to_string()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, u64>(1)?,
                        row.get::<_, u32>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                    ))
                },
            )
            .optional()?;
        row.map(
            |(session_id, revision, schema, config, hash, actor, created)| {
                Ok(SilentSessionConfigRevision {
                    config_schema_version: schema,
                    id,
                    silent_session_id: session_id.parse()?,
                    revision,
                    config: serde_json::from_str(&config)?,
                    redacted_config_hash: hash,
                    created_by: actor.parse()?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created)?
                        .with_timezone(&chrono::Utc),
                })
            },
        )
        .transpose()
    })
}

pub fn load_runtime_checkpoint(
    persistence: &SqlitePersistence,
    id: RuntimeCheckpointId,
) -> anyhow::Result<Option<RuntimeCheckpoint>> {
    load_json_by_id(
        persistence,
        "SELECT checkpoint_json FROM silent_session_checkpoints WHERE checkpoint_id=?1 AND checkpoint_kind='runtime'",
        id.to_string(),
    )
}

pub fn load_workpoint_checkpoint(
    persistence: &SqlitePersistence,
    id: WorkpointCheckpointId,
) -> anyhow::Result<Option<SilentSessionWorkpointCheckpoint>> {
    load_json_by_id(
        persistence,
        "SELECT checkpoint_json FROM silent_session_checkpoints WHERE checkpoint_id=?1 AND checkpoint_kind='workpoint'",
        id.to_string(),
    )
}

pub fn save_runtime_checkpoint(
    persistence: &SqlitePersistence,
    checkpoint: &RuntimeCheckpoint,
) -> anyhow::Result<()> {
    save_checkpoint(
        persistence,
        checkpoint.id.to_string(),
        checkpoint.silent_session_id,
        Some(checkpoint.run_id.to_string()),
        "runtime",
        checkpoint.event_sequence,
        checkpoint,
        &checkpoint.runtime_state_hash,
        checkpoint.created_at,
    )
}

pub fn save_workpoint_checkpoint(
    persistence: &SqlitePersistence,
    checkpoint: &SilentSessionWorkpointCheckpoint,
) -> anyhow::Result<()> {
    save_checkpoint(
        persistence,
        checkpoint.id.to_string(),
        checkpoint.silent_session_id,
        None,
        "workpoint",
        0,
        checkpoint,
        &hash_json(checkpoint)?,
        checkpoint.created_at,
    )
}

pub fn save_lease(
    persistence: &SqlitePersistence,
    lease: &SilentSessionLease,
) -> anyhow::Result<()> {
    persistence.with_connection_mut(|connection| {
        connection.execute(
            r#"INSERT INTO silent_session_leases(
               lease_id,silent_session_id,run_id,owner_actor_instance_id,fencing_token,status,
               lease_json,issued_at,expires_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
               ON CONFLICT(lease_id) DO UPDATE SET status=excluded.status,
                 lease_json=excluded.lease_json,expires_at=excluded.expires_at"#,
            params![
                lease.id.to_string(),
                lease.silent_session_id.to_string(),
                lease.run_id.to_string(),
                lease.owner_actor_instance_id.to_string(),
                lease.fencing_token,
                enum_json(lease.status)?,
                serde_json::to_string(lease)?,
                lease.issued_at.to_rfc3339(),
                lease.expires_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    })
}

pub fn load_lease(
    persistence: &SqlitePersistence,
    id: SilentSessionLeaseId,
) -> anyhow::Result<Option<SilentSessionLease>> {
    load_json_by_id(
        persistence,
        "SELECT lease_json FROM silent_session_leases WHERE lease_id=?1",
        id.to_string(),
    )
}

pub fn save_completion_evaluation(
    persistence: &SqlitePersistence,
    evaluation: &CompletionEvaluation,
) -> anyhow::Result<()> {
    persistence.with_connection_mut(|connection| {
        connection.execute(
            r#"INSERT OR IGNORE INTO silent_session_completion_evaluations(
               completion_evaluation_id,silent_session_id,run_id,decision,evaluation_json,
               receipt_ready,evaluated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)"#,
            params![
                evaluation.id.to_string(),
                evaluation.silent_session_id.to_string(),
                evaluation.run_id.to_string(),
                enum_json(evaluation.decision)?,
                serde_json::to_string(evaluation)?,
                evaluation.receipt_ready,
                evaluation.evaluated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    })
}

pub fn load_completion_evaluation(
    persistence: &SqlitePersistence,
    id: CompletionEvaluationId,
) -> anyhow::Result<Option<CompletionEvaluation>> {
    load_json_by_id(
        persistence,
        "SELECT evaluation_json FROM silent_session_completion_evaluations WHERE completion_evaluation_id=?1",
        id.to_string(),
    )
}

pub fn list_checkpoint_values(
    persistence: &SqlitePersistence,
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
) -> anyhow::Result<Vec<serde_json::Value>> {
    persistence.with_connection_mut(|connection| {
        let mut statement = connection.prepare(
            r#"SELECT checkpoint_json FROM silent_session_checkpoints
               WHERE silent_session_id=?1 AND (run_id=?2 OR run_id IS NULL)
               ORDER BY event_sequence DESC,created_at DESC"#,
        )?;
        let rows = statement
            .query_map(params![session_id.to_string(), run_id.to_string()], |row| {
                row.get::<_, String>(0)
            })?;
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    })
}

pub fn list_completion_evaluations(
    persistence: &SqlitePersistence,
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
) -> anyhow::Result<Vec<CompletionEvaluation>> {
    persistence.with_connection_mut(|connection| {
        let mut statement = connection.prepare(
            r#"SELECT evaluation_json FROM silent_session_completion_evaluations
               WHERE silent_session_id=?1 AND run_id=?2 ORDER BY evaluated_at DESC"#,
        )?;
        let rows = statement
            .query_map(params![session_id.to_string(), run_id.to_string()], |row| {
                row.get::<_, String>(0)
            })?;
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    })
}

#[allow(clippy::too_many_arguments)]
fn save_checkpoint<T: Serialize>(
    persistence: &SqlitePersistence,
    checkpoint_id: String,
    session_id: SilentSessionId,
    run_id: Option<String>,
    kind: &str,
    event_sequence: u64,
    checkpoint: &T,
    checkpoint_hash: &str,
    created_at: chrono::DateTime<chrono::Utc>,
) -> anyhow::Result<()> {
    persistence.with_connection_mut(|connection| {
        connection.execute(
            r#"INSERT OR IGNORE INTO silent_session_checkpoints(
               checkpoint_id,silent_session_id,run_id,checkpoint_kind,event_sequence,
               checkpoint_json,checkpoint_hash,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)"#,
            params![
                checkpoint_id,
                session_id.to_string(),
                run_id,
                kind,
                event_sequence,
                serde_json::to_string(checkpoint)?,
                checkpoint_hash,
                created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    })
}

fn load_json_by_id<T: DeserializeOwned>(
    persistence: &SqlitePersistence,
    query: &str,
    id: String,
) -> anyhow::Result<Option<T>> {
    persistence.with_connection_mut(|connection| {
        let json = connection
            .query_row(query, [id], |row| row.get::<_, String>(0))
            .optional()?;
        deserialize_optional(json)
    })
}

fn deserialize_optional<T: DeserializeOwned>(json: Option<String>) -> anyhow::Result<Option<T>> {
    json.map(|value| serde_json::from_str(&value).map_err(Into::into))
        .transpose()
}

fn enum_json<T: Serialize>(value: T) -> anyhow::Result<String> {
    Ok(serde_json::to_string(&value)?.trim_matches('"').to_string())
}

fn hash_json<T: Serialize>(value: &T) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    Ok(hex::encode(Sha256::digest(serde_json::to_vec(value)?)))
}
