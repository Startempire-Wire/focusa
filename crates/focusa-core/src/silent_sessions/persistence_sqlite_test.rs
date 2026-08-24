use chrono::{TimeZone, Utc};
use rusqlite::Connection;
use serde_json::json;
use uuid::Uuid;

use crate::{
    runtime::persistence_sqlite::SqlitePersistence,
    silent_sessions::{
        ActorInstanceId, AppendOutcome, CompletionDecision, CompletionEvaluation,
        CompletionEvaluationId, ConfigRevisionId, EVENT_SCHEMA_VERSION, HarnessConfig, HarnessKind,
        IdentityConfig, LeaseStatus, MigrationMode, ModelConfig, ModelFallbackPolicy,
        ModelSelectionPolicy, NativeResumePolicy, ProtocolVersions, RunGeneration,
        RuntimeCheckpoint, RuntimeCheckpointId, SilentSession, SilentSessionAuthority,
        SilentSessionConfig, SilentSessionConfigRevision, SilentSessionEvent, SilentSessionEventId,
        SilentSessionLease, SilentSessionLeaseId, SilentSessionLifecycle, SilentSessionRun,
        SilentSessionRunId, SilentSessionWorkpointCheckpoint, WorkpointCheckpointId,
        append_config_revision_event_and_project, append_create_event_and_project,
        append_reducer_event_and_project, append_restart_event_and_project, list_checkpoint_values,
        list_completion_evaluations, list_sessions, load_completion_evaluation,
        load_config_revision, load_lease, load_run, load_runtime_checkpoint, load_session,
        load_session_by_idempotency_key, load_session_events, load_usage_summary,
        load_workpoint_checkpoint, migrate_silent_session_schema, save_completion_evaluation,
        save_config_revision, save_lease, save_run, save_runtime_checkpoint,
        save_workpoint_checkpoint,
    },
    types::FocusaConfig,
};

fn temp_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("focusa-spec133-{}", Uuid::now_v7()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn persistence() -> (std::path::PathBuf, SqlitePersistence) {
    let dir = temp_dir();
    let config = FocusaConfig {
        data_dir: dir.to_string_lossy().into_owned(),
        ..FocusaConfig::default()
    };
    let persistence = SqlitePersistence::new(&config).unwrap();
    (dir, persistence)
}

fn session() -> SilentSession {
    SilentSession::draft(
        SilentSessionAuthority::new("/repo/focusa", "continuity-1").unwrap(),
        "proof",
        "prove persistence",
        ConfigRevisionId::new(),
        Utc.with_ymd_and_hms(2026, 7, 17, 14, 0, 0).unwrap(),
    )
    .unwrap()
}

fn config() -> SilentSessionConfig {
    SilentSessionConfig::new(
        IdentityConfig {
            display_name: "proof".into(),
            project_root: "/repo/focusa".into(),
            continuity_id: "continuity-1".into(),
            work_item_ref: None,
            mission: "prove persistence".into(),
            agent_identity_ref: "agent:pi".into(),
            role_profile_ref: None,
        },
        HarnessConfig {
            kind: HarnessKind::Pi,
            adapter_version: "1".into(),
            native_resume_policy: NativeResumePolicy::Prefer,
        },
        ModelConfig {
            provider: "provider".into(),
            model: "model".into(),
            thinking: None,
            selection_policy: ModelSelectionPolicy::Exact,
            fallback_policy: ModelFallbackPolicy::Disabled,
            allowed_fallbacks: Vec::new(),
            auth_profile_ref: "operator".into(),
            require_entitlement_preflight: true,
            require_runtime_model_confirmation: true,
        },
    )
}

fn event(
    projection: &SilentSession,
    sequence: u64,
    previous: Option<String>,
) -> SilentSessionEvent {
    SilentSessionEvent {
        event_schema_version: EVENT_SCHEMA_VERSION,
        id: SilentSessionEventId::new(),
        silent_session_id: projection.id,
        run_id: None,
        sequence,
        kind: "session_drafted".into(),
        payload: json!({"sequence": sequence}),
        idempotency_key: format!("draft-{sequence}"),
        previous_event_hash: previous,
        event_hash: String::new(),
        occurred_at: Utc
            .with_ymd_and_hms(2026, 7, 17, 14, 0, sequence as u32)
            .unwrap(),
    }
}

#[test]
fn migration_creates_required_schema_and_backup() {
    let (dir, _persistence) = persistence();
    assert!(
        dir.join("focusa.sqlite.pre-silent-session-v5.backup")
            .is_file()
    );
    let connection = Connection::open(dir.join("focusa.sqlite")).unwrap();
    let table_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND (name LIKE 'silent_session_control%' OR name='silent_session_daemon_runs')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(table_count, 17);
}

#[test]
fn dry_run_rolls_back_all_schema_changes() {
    let (_dir, persistence) = persistence();
    persistence
        .with_connection_mut(|connection| {
            connection.execute_batch(
                r#"PRAGMA foreign_keys=OFF;
                DROP TABLE silent_session_control_backend_bindings;
                DROP TABLE silent_session_control_completion_evaluations;
                DROP TABLE silent_session_control_notifications;
                DROP TABLE silent_session_control_leases;
                DROP TABLE silent_session_control_checkpoints;
                DROP TABLE silent_session_control_stream_indexes;
                DROP TABLE silent_session_control_events;
                DROP TABLE silent_session_control_config_revisions;
                DROP TABLE silent_session_daemon_runs;
                DROP TABLE silent_session_controls;
                DROP TABLE silent_session_control_schema_meta;
                PRAGMA foreign_keys=ON;"#,
            )?;
            Ok(())
        })
        .unwrap();

    let outcome = migrate_silent_session_schema(&persistence, MigrationMode::DryRun).unwrap();
    assert!(!outcome.applied);
    persistence
        .with_connection_mut(|connection| {
            let count: i64 = connection.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='silent_session_controls'",
                [],
                |row| row.get(0),
            )?;
            assert_eq!(count, 0);
            Ok(())
        })
        .unwrap();
}

#[test]
fn schema_v1_upgrades_to_v4_stream_record_and_authorization_tables() {
    let (_dir, persistence) = persistence();
    persistence
        .with_connection_mut(|connection| {
            connection.execute_batch(
                r#"PRAGMA foreign_keys=OFF;
                DROP TABLE silent_session_daemon_runs;
                DROP TABLE silent_session_control_leases;
                DROP TABLE silent_session_control_stream_indexes;
                CREATE TABLE silent_session_daemon_runs (
                  run_id TEXT PRIMARY KEY, silent_session_id TEXT NOT NULL,
                  run_generation INTEGER NOT NULL, actor_instance_id TEXT NOT NULL,
                  config_revision_id TEXT NOT NULL, protocol_versions_json TEXT NOT NULL,
                  started_at TEXT NOT NULL, ended_at TEXT
                );
                CREATE TABLE silent_session_control_leases (
                  lease_id TEXT PRIMARY KEY, silent_session_id TEXT NOT NULL, run_id TEXT NOT NULL,
                  owner_actor_instance_id TEXT NOT NULL, fencing_token INTEGER NOT NULL,
                  status TEXT NOT NULL, issued_at TEXT NOT NULL, expires_at TEXT NOT NULL
                );
                CREATE TABLE silent_session_control_stream_indexes (
                  silent_session_id TEXT NOT NULL, run_id TEXT NOT NULL, stream_name TEXT NOT NULL,
                  chunk_sequence INTEGER NOT NULL, chunk_ref TEXT NOT NULL,
                  byte_start INTEGER NOT NULL, byte_end INTEGER NOT NULL,
                  chunk_hash TEXT NOT NULL, created_at TEXT NOT NULL,
                  PRIMARY KEY(silent_session_id, run_id, stream_name, chunk_sequence)
                );
                UPDATE silent_session_control_schema_meta SET version=1;
                PRAGMA foreign_keys=ON;"#,
            )?;
            Ok(())
        })
        .unwrap();

    let outcome = migrate_silent_session_schema(&persistence, MigrationMode::Apply).unwrap();
    assert_eq!(outcome.previous_version, 1);
    assert_eq!(outcome.target_version, 5);
    persistence
        .with_connection_mut(|connection| {
            for (table, column) in [
                ("silent_session_daemon_runs", "run_json"),
                ("silent_session_control_leases", "lease_json"),
                (
                    "silent_session_control_stream_indexes",
                    "last_event_sequence",
                ),
                ("silent_session_control_stream_indexes", "redaction_applied"),
            ] {
                let present: i64 = connection.query_row(
                    "SELECT COUNT(*) FROM pragma_table_info(?1) WHERE name=?2",
                    rusqlite::params![table, column],
                    |row| row.get(0),
                )?;
                assert_eq!(present, 1, "{table}.{column}");
            }
            Ok(())
        })
        .unwrap();
}

#[test]
fn reducer_event_and_projection_are_atomic_and_idempotent() {
    let (_dir, persistence) = persistence();
    let projection = session();
    let mut first = event(&projection, 1, None);
    assert_eq!(
        append_reducer_event_and_project(&persistence, &mut first, &projection).unwrap(),
        AppendOutcome::Appended
    );
    let expected_hash = first.event_hash.clone();
    assert_eq!(
        append_reducer_event_and_project(&persistence, &mut first, &projection).unwrap(),
        AppendOutcome::Replayed
    );
    assert_eq!(first.event_hash, expected_hash);

    persistence
        .with_connection_mut(|connection| {
            let events: i64 = connection.query_row(
                "SELECT COUNT(*) FROM silent_session_control_events WHERE silent_session_id=?1",
                [projection.id.to_string()],
                |row| row.get(0),
            )?;
            let projections: i64 = connection.query_row(
                "SELECT COUNT(*) FROM silent_session_controls WHERE silent_session_id=?1",
                [projection.id.to_string()],
                |row| row.get(0),
            )?;
            assert_eq!((events, projections), (1, 1));
            Ok(())
        })
        .unwrap();
}

#[test]
fn create_projection_revision_and_event_commit_atomically() {
    let (_dir, persistence) = persistence();
    let mut projection = session();
    let revision = SilentSessionConfigRevision {
        config_schema_version: 1,
        id: projection.active_config_revision_id,
        silent_session_id: projection.id,
        revision: 1,
        config: config(),
        redacted_config_hash: "hash".into(),
        created_by: ActorInstanceId::new(),
        created_at: projection.created_at,
    };
    let run = SilentSessionRun {
        silent_session_schema_version: 1,
        id: SilentSessionRunId::new(),
        silent_session_id: projection.id,
        generation: projection.current_run_generation,
        actor_instance_id: revision.created_by,
        config_revision_id: revision.id,
        protocol_versions: ProtocolVersions::default(),
        started_at: projection.created_at,
        ended_at: None,
    };
    let mut first = event(&projection, 1, None);
    first.run_id = Some(run.id);
    assert_eq!(
        append_create_event_and_project(&persistence, &mut first, &projection, &revision, &run,)
            .unwrap(),
        AppendOutcome::Appended
    );
    assert_eq!(
        load_session(&persistence, projection.id).unwrap(),
        Some(projection.clone())
    );
    assert_eq!(
        load_config_revision(&persistence, revision.id).unwrap(),
        Some(revision)
    );
    assert_eq!(load_run(&persistence, run.id).unwrap(), Some(run.clone()));
    assert_eq!(
        load_session_by_idempotency_key(&persistence, &first.idempotency_key)
            .unwrap()
            .unwrap(),
        (projection.clone(), first.payload.clone())
    );

    let now = projection.updated_at + chrono::Duration::seconds(1);
    let mut previous_run = run.clone();
    previous_run.ended_at = Some(now);
    let next_run = SilentSessionRun {
        silent_session_schema_version: 1,
        id: SilentSessionRunId::new(),
        silent_session_id: projection.id,
        generation: run.generation.next().unwrap(),
        actor_instance_id: ActorInstanceId::new(),
        config_revision_id: run.config_revision_id,
        protocol_versions: run.protocol_versions.clone(),
        started_at: now,
        ended_at: None,
    };
    projection.lifecycle = SilentSessionLifecycle::Draft;
    projection.current_run_generation = next_run.generation;
    projection.updated_at = now;
    let mut restart_event = event(&projection, 2, Some(first.event_hash.clone()));
    restart_event.idempotency_key = "restart-1".into();
    restart_event.kind = "restart_requested".into();
    restart_event.run_id = Some(next_run.id);
    assert_eq!(
        append_restart_event_and_project(
            &persistence,
            &mut restart_event,
            &projection,
            &previous_run,
            &next_run,
        )
        .unwrap(),
        AppendOutcome::Appended
    );
    assert_eq!(load_run(&persistence, run.id).unwrap(), Some(previous_run));
    assert_eq!(load_run(&persistence, next_run.id).unwrap(), Some(next_run));
    assert_eq!(
        load_session(&persistence, projection.id).unwrap(),
        Some(projection)
    );

    let rejected = session();
    let rejected_revision = SilentSessionConfigRevision {
        config_schema_version: 1,
        id: rejected.active_config_revision_id,
        silent_session_id: rejected.id,
        revision: 1,
        config: config(),
        redacted_config_hash: "other-hash".into(),
        created_by: ActorInstanceId::new(),
        created_at: rejected.created_at,
    };
    let rejected_run = SilentSessionRun {
        silent_session_schema_version: 1,
        id: SilentSessionRunId::new(),
        silent_session_id: rejected.id,
        generation: rejected.current_run_generation,
        actor_instance_id: rejected_revision.created_by,
        config_revision_id: rejected_revision.id,
        protocol_versions: ProtocolVersions::default(),
        started_at: rejected.created_at,
        ended_at: None,
    };
    let mut invalid_event = event(&rejected, 1, None);
    invalid_event.id = first.id;
    invalid_event.run_id = Some(rejected_run.id);
    assert!(
        append_create_event_and_project(
            &persistence,
            &mut invalid_event,
            &rejected,
            &rejected_revision,
            &rejected_run,
        )
        .is_err()
    );
    assert_eq!(load_session(&persistence, rejected.id).unwrap(), None);
    assert_eq!(
        load_config_revision(&persistence, rejected_revision.id).unwrap(),
        None
    );
    assert_eq!(load_run(&persistence, rejected_run.id).unwrap(), None);
}

#[test]
fn all_canonical_records_save_and_reload() {
    let (_dir, persistence) = persistence();
    let projection = session();
    let mut first = event(&projection, 1, None);
    append_reducer_event_and_project(&persistence, &mut first, &projection).unwrap();

    let actor = ActorInstanceId::new();
    let run_id = SilentSessionRunId::new();
    save_run(
        &persistence,
        &SilentSessionRun {
            silent_session_schema_version: 1,
            id: run_id,
            silent_session_id: projection.id,
            generation: RunGeneration::first(),
            actor_instance_id: actor,
            config_revision_id: projection.active_config_revision_id,
            protocol_versions: ProtocolVersions::default(),
            started_at: projection.created_at,
            ended_at: None,
        },
    )
    .unwrap();
    let revision = SilentSessionConfigRevision {
        config_schema_version: 1,
        id: projection.active_config_revision_id,
        silent_session_id: projection.id,
        revision: 1,
        config: config(),
        redacted_config_hash: "sha256:config".into(),
        created_by: actor,
        created_at: projection.created_at,
    };
    save_config_revision(&persistence, &revision).unwrap();
    let runtime_checkpoint_id = RuntimeCheckpointId::new();
    save_runtime_checkpoint(
        &persistence,
        &RuntimeCheckpoint {
            schema_version: 1,
            id: runtime_checkpoint_id,
            silent_session_id: projection.id,
            run_id,
            run_generation: RunGeneration::first(),
            event_sequence: 1,
            stream_cursor: "stdout:10".into(),
            runtime_state_hash: "sha256:runtime".into(),
            created_at: projection.created_at,
        },
    )
    .unwrap();
    let workpoint_checkpoint_id = WorkpointCheckpointId::new();
    save_workpoint_checkpoint(
        &persistence,
        &SilentSessionWorkpointCheckpoint {
            schema_version: 1,
            id: workpoint_checkpoint_id,
            silent_session_id: projection.id,
            workpoint_id: "workpoint-1".into(),
            mission: projection.mission.clone(),
            current_action: "persist".into(),
            next_action: "verify".into(),
            evidence_refs: vec!["test:proof".into()],
            created_at: projection.created_at,
        },
    )
    .unwrap();
    let lease_id = SilentSessionLeaseId::new();
    save_lease(
        &persistence,
        &SilentSessionLease {
            schema_version: 1,
            id: lease_id,
            silent_session_id: projection.id,
            run_id,
            owner_actor_instance_id: actor,
            fencing_token: 1,
            status: LeaseStatus::Active,
            issued_at: projection.created_at,
            expires_at: projection.updated_at + chrono::Duration::minutes(5),
        },
    )
    .unwrap();
    let completion_id = CompletionEvaluationId::new();
    save_completion_evaluation(
        &persistence,
        &CompletionEvaluation {
            schema_version: 1,
            id: completion_id,
            silent_session_id: projection.id,
            run_id,
            decision: CompletionDecision::Incomplete,
            reason: "verification pending".into(),
            required_evidence_refs: vec!["test:proof".into()],
            verified_evidence_refs: Vec::new(),
            receipt_ready: false,
            evaluated_by: actor,
            evaluated_at: projection.created_at,
        },
    )
    .unwrap();

    assert_eq!(
        load_session(&persistence, projection.id).unwrap(),
        Some(projection.clone())
    );
    assert_eq!(
        list_sessions(&persistence).unwrap(),
        vec![projection.clone()]
    );
    assert_eq!(
        load_session_events(&persistence, projection.id).unwrap(),
        vec![first]
    );
    assert_eq!(
        load_config_revision(&persistence, revision.id).unwrap(),
        Some(revision)
    );
    assert!(load_run(&persistence, run_id).unwrap().is_some());
    assert!(
        load_runtime_checkpoint(&persistence, runtime_checkpoint_id)
            .unwrap()
            .is_some()
    );
    assert!(
        load_workpoint_checkpoint(&persistence, workpoint_checkpoint_id)
            .unwrap()
            .is_some()
    );
    assert!(load_lease(&persistence, lease_id).unwrap().is_some());
    assert!(
        load_completion_evaluation(&persistence, completion_id)
            .unwrap()
            .is_some()
    );
    let usage = load_usage_summary(&persistence, projection.id, run_id).unwrap();
    assert_eq!(usage.lifecycle_event_count, 0);
    assert_eq!(usage.stream_event_count, 0);
    assert_eq!(
        list_checkpoint_values(&persistence, projection.id, run_id)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        list_completion_evaluations(&persistence, projection.id, run_id)
            .unwrap()
            .len(),
        1
    );
    persistence
        .with_connection_mut(|connection| {
            for (table, expected) in [
                ("silent_session_daemon_runs", 1),
                ("silent_session_control_checkpoints", 2),
                ("silent_session_control_leases", 1),
                ("silent_session_control_completion_evaluations", 1),
            ] {
                let count: i64 =
                    connection.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                        row.get(0)
                    })?;
                assert_eq!(count, expected, "table {table}");
            }
            Ok(())
        })
        .unwrap();
}

#[test]
fn config_revision_event_and_record_commit_atomically() {
    let (_dir, persistence) = persistence();
    let projection = session();
    let mut first = event(&projection, 1, None);
    append_reducer_event_and_project(&persistence, &mut first, &projection).unwrap();
    let revision = SilentSessionConfigRevision {
        config_schema_version: 1,
        id: ConfigRevisionId::new(),
        silent_session_id: projection.id,
        revision: 2,
        config: config(),
        redacted_config_hash: "revision-hash".into(),
        created_by: ActorInstanceId::new(),
        created_at: projection.updated_at,
    };
    let mut proposed = event(&projection, 2, Some(first.event_hash));
    proposed.kind = "config.revision_proposed".into();
    append_config_revision_event_and_project(&persistence, &mut proposed, &projection, &revision)
        .unwrap();
    assert_eq!(
        load_config_revision(&persistence, revision.id).unwrap(),
        Some(revision)
    );
    assert_eq!(
        load_session_events(&persistence, projection.id)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn chain_mismatch_and_scope_mismatch_roll_back() {
    let (_dir, persistence) = persistence();
    let projection = session();
    let mut first = event(&projection, 1, None);
    append_reducer_event_and_project(&persistence, &mut first, &projection).unwrap();

    let mut wrong_chain = event(&projection, 3, Some(first.event_hash.clone()));
    assert!(append_reducer_event_and_project(&persistence, &mut wrong_chain, &projection).is_err());

    let mut wrong_scope = projection.clone();
    wrong_scope.authority = SilentSessionAuthority::new("/repo/other", "continuity-1").unwrap();
    let mut second = event(&projection, 2, Some(first.event_hash));
    assert!(append_reducer_event_and_project(&persistence, &mut second, &wrong_scope).is_err());

    persistence
        .with_connection_mut(|connection| {
            let count: i64 = connection.query_row(
                "SELECT COUNT(*) FROM silent_session_control_events WHERE silent_session_id=?1",
                [projection.id.to_string()],
                |row| row.get(0),
            )?;
            assert_eq!(count, 1);
            Ok(())
        })
        .unwrap();
}
