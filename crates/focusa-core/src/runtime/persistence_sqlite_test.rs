#![allow(clippy::field_reassign_with_default)]

use crate::runtime::persistence_sqlite::SqlitePersistence;
use crate::silent_session::{
    ModelBinding, ObservationProvenance, RedactionMetadata, SILENT_SESSION_EVENT_SCHEMA,
    SILENT_SESSION_RUN_SCHEMA, SILENT_SESSION_SCHEMA, SilentSession, SilentSessionConfigRevisionId,
    SilentSessionEvent, SilentSessionEventId, SilentSessionHealth, SilentSessionId,
    SilentSessionLeaseId, SilentSessionLifecycleState, SilentSessionRun, SilentSessionRunId,
    SilentSessionVersions, WorkspaceBinding, WorkspaceStrategy,
};
use crate::silent_session_authorization::{
    SILENT_SESSION_APPROVAL_SCHEMA, SilentSessionApproval, SilentSessionPrincipal,
    SilentSessionRole, SilentSessionScope,
};
use crate::types::{EventLogEntry, FocusaConfig, FocusaEvent, FocusaState, SignalOrigin};
use chrono::Utc;
use rusqlite::Connection;
use uuid::Uuid;

fn sample_silent_session(dir: &std::path::Path) -> (SilentSession, SilentSessionEvent) {
    let session_id = SilentSessionId::new();
    let run_id = SilentSessionRunId::new();
    let now = Utc::now();
    let session = SilentSession {
        schema: SILENT_SESSION_SCHEMA.into(),
        versions: SilentSessionVersions::default(),
        session_id,
        display_name: "persistence canary".into(),
        created_at: now,
        created_by_actor_ref: "test".into(),
        operator_principal_ref: "operator:test".into(),
        os_execution_user: "test".into(),
        project_root: dir.to_path_buf(),
        project_identity_ref: "project:test".into(),
        continuity_id: "continuity:test".into(),
        trajectory_ref: None,
        workpoint_ref: None,
        work_item_ref: Some("item:test".into()),
        operator_ask: crate::silent_session::OperatorAskBinding::capture(
            "ask:persistence-test",
            "prove durable replay",
            1,
            now,
        ),
        mission: "prove durable replay".into(),
        lifecycle_state: SilentSessionLifecycleState::Running,
        health: SilentSessionHealth::Healthy,
        semantic_observation: None,
        active_run_id: Some(run_id),
        config_revision_id: SilentSessionConfigRevisionId::new(),
        writer_lease_ref: Some(SilentSessionLeaseId::new()),
        retention_policy_ref: "retention:test".into(),
        receipt_refs: vec![],
    };
    let event = SilentSessionEvent {
        schema: SILENT_SESSION_EVENT_SCHEMA.into(),
        event_id: SilentSessionEventId::new(),
        session_id,
        run_id,
        seq: 1,
        occurred_at: now,
        observed_at: now,
        kind: "run_started".into(),
        source: "test".into(),
        provenance: ObservationProvenance::RuntimeObserved,
        canonical: true,
        payload: serde_json::json!({"pid": 42}),
        artifact_refs: vec![],
        correlation_id: Uuid::now_v7(),
        redaction: RedactionMetadata {
            applied: false,
            classes: vec![],
        },
    };
    (session, event)
}

fn sample_silent_run(session: &SilentSession) -> SilentSessionRun {
    SilentSessionRun {
        schema: SILENT_SESSION_RUN_SCHEMA.into(),
        versions: SilentSessionVersions::default(),
        run_id: session
            .active_run_id
            .expect("sample session has active run"),
        session_id: session.session_id,
        generation: 1,
        runner_id: "runner:test".into(),
        adapter_id: "adapter:test".into(),
        process_backend_id: "process:test".into(),
        requested_model_binding: ModelBinding {
            provider: "test".into(),
            model: "test-model".into(),
            thinking: None,
        },
        effective_model_binding: None,
        observed_model_binding: None,
        workspace_binding: WorkspaceBinding {
            workspace_id: "workspace:test".into(),
            root: session.project_root.clone(),
            strategy: WorkspaceStrategy::ExclusiveExisting,
            branch_ref: None,
        },
        process_identity: None,
        harness_native_session_ref: None,
        started_at: Some(Utc::now()),
        ended_at: None,
        exit_status: None,
        current_event_seq: 1,
        output_stream_refs: vec![],
        runtime_checkpoint_refs: vec![],
        workpoint_checkpoint_refs: vec![],
    }
}

fn temp_dir() -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("focusa-test-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn test_event(turn_id: &str) -> EventLogEntry {
    EventLogEntry {
        id: Uuid::now_v7(),
        timestamp: Utc::now(),
        event: FocusaEvent::TurnCompleted {
            turn_id: turn_id.to_string(),
            harness_name: "test".to_string(),
            raw_user_input: None,
            assistant_output: Some("ok".to_string()),
            artifacts_used: Vec::new(),
            errors: Vec::new(),
            prompt_tokens: Some(1),
            completion_tokens: Some(1),
        },
        correlation_id: Some("test-correlation".to_string()),
        origin: SignalOrigin::Daemon,
        machine_id: None,
        instance_id: None,
        session_id: None,
        thread_id: None,
        is_observation: false,
    }
}

#[test]
fn sqlite_event_hash_chain_links_appended_events() {
    let dir = temp_dir();
    let mut cfg = FocusaConfig::default();
    cfg.data_dir = dir.to_string_lossy().to_string();

    let p = SqlitePersistence::new(&cfg).unwrap();
    let first = test_event("t1");
    let second = test_event("t2");
    p.append_event(&first).unwrap();
    p.append_event(&second).unwrap();

    let db_path = dir.join("focusa.sqlite");
    let conn = Connection::open(db_path).unwrap();
    let rows: Vec<(i64, String, String)> = conn
        .prepare(
            "SELECT chain_index, previous_hash, event_hash FROM event_hash_chain ORDER BY chain_index",
        )
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].0, 0);
    assert_eq!(rows[0].1, "GENESIS");
    assert_eq!(rows[1].0, 1);
    assert_eq!(rows[1].1, rows[0].2);
    assert_ne!(rows[0].2, rows[1].2);
}

#[test]
fn durable_sequence_cursor_replays_after_restart_without_duplicates() {
    let dir = temp_dir();
    let mut cfg = FocusaConfig::default();
    cfg.data_dir = dir.to_string_lossy().to_string();

    let first = test_event("cursor-1");
    let second = test_event("cursor-2");
    let third = test_event("cursor-3");
    let second_id = second.id.to_string();
    {
        let persistence = SqlitePersistence::new(&cfg).unwrap();
        persistence.append_event(&first).unwrap();
        persistence.append_event(&second).unwrap();
        persistence.append_event(&third).unwrap();
        let all = persistence.durable_events_after(0, 10).unwrap();
        assert_eq!(
            all.iter().map(|event| event.sequence).collect::<Vec<_>>(),
            [1, 2, 3]
        );
        assert_eq!(
            persistence.durable_event_sequence(&second_id).unwrap(),
            Some(2)
        );
        assert_eq!(persistence.latest_durable_event_sequence().unwrap(), 3);
    }

    let reopened = SqlitePersistence::new(&cfg).unwrap();
    let replay = reopened.durable_events_after(1, 10).unwrap();
    assert_eq!(
        replay
            .iter()
            .map(|event| event.sequence)
            .collect::<Vec<_>>(),
        [2, 3]
    );
    let tail = reopened.durable_events_after(2, 10).unwrap();
    assert_eq!(tail.len(), 1);
    assert_eq!(tail[0].sequence, 3);
}

#[test]
fn sqlite_persistence_creates_machine_id() {
    let dir = temp_dir();

    let mut cfg = FocusaConfig::default();
    cfg.data_dir = dir.to_string_lossy().to_string();

    let p = SqlitePersistence::new(&cfg).unwrap();
    let mid = p.machine_id().unwrap();
    assert!(!mid.trim().is_empty());

    // Re-open should preserve machine_id.
    let p2 = SqlitePersistence::new(&cfg).unwrap();
    let mid2 = p2.machine_id().unwrap();
    assert_eq!(mid, mid2);
}

#[test]
fn sqlite_persistence_rejects_incompatible_schema_version() {
    let dir = temp_dir();

    let mut cfg = FocusaConfig::default();
    cfg.data_dir = dir.to_string_lossy().to_string();

    {
        let _p = SqlitePersistence::new(&cfg).unwrap();
    }

    let db_path = dir.join("focusa.sqlite");
    let conn = Connection::open(db_path).unwrap();
    conn.execute(
        "UPDATE meta SET value = '999' WHERE key = 'schema_version'",
        [],
    )
    .unwrap();

    let err = match SqlitePersistence::new(&cfg) {
        Ok(_) => panic!("expected incompatible schema version error"),
        Err(err) => err,
    };
    let msg = format!("{err:#}");
    assert!(
        msg.contains("unsupported schema_version"),
        "expected unsupported schema_version error, got: {msg}"
    );
}

#[test]
fn sqlite_persistence_rolls_back_to_fresh_state_on_incompatible_snapshot() {
    let dir = temp_dir();

    let mut cfg = FocusaConfig::default();
    cfg.data_dir = dir.to_string_lossy().to_string();

    {
        let _p = SqlitePersistence::new(&cfg).unwrap();
    }

    let db_path = dir.join("focusa.sqlite");
    let conn = Connection::open(db_path).unwrap();
    conn.execute(
        r#"
        INSERT INTO snapshots(name, version, ts, state_json)
        VALUES('focusa', 1, '2026-01-01T00:00:00Z', '{"legacy_only":true}')
        ON CONFLICT(name) DO UPDATE SET
          version=excluded.version,
          ts=excluded.ts,
          state_json=excluded.state_json
        "#,
        [],
    )
    .unwrap();

    let p = SqlitePersistence::new(&cfg).unwrap();
    let state = p
        .load_state()
        .expect("load_state should not fail for incompatible legacy snapshot");
    assert!(
        state.is_none(),
        "incompatible snapshot should trigger fresh-state fallback"
    );
}

#[test]
fn sqlite_crdt_import_is_scoped_and_idempotent() {
    use crate::sync::CrdtLog;

    let dir = temp_dir();
    let mut cfg = FocusaConfig::default();
    cfg.data_dir = dir.to_string_lossy().to_string();
    let p = SqlitePersistence::new(&cfg).unwrap();

    let mut log = CrdtLog::new();
    let mut in_scope = test_event("crdt-in-scope");
    in_scope.correlation_id =
        Some("project_root=/tmp/focusa-portable-fixture/project|continuity_id=main".into());
    in_scope.machine_id = Some("machine-a".into());
    let event = log.add_local_event(in_scope, "machine-a");

    let imported = p
        .import_crdt_events_same_root(
            "peer-a",
            "/tmp/focusa-portable-fixture/project",
            "main",
            std::slice::from_ref(&event),
        )
        .unwrap();
    assert_eq!(imported, 1);
    let imported_again = p
        .import_crdt_events_same_root(
            "peer-a",
            "/tmp/focusa-portable-fixture/project",
            "main",
            std::slice::from_ref(&event),
        )
        .unwrap();
    assert_eq!(imported_again, 0);
    let scoped = p
        .crdt_events_for_scope("/tmp/focusa-portable-fixture/project", "main", 10)
        .unwrap();
    assert_eq!(scoped.len(), 1);
    assert_eq!(scoped[0].entry.id, event.entry.id);

    let wrong_scope = p
        .import_crdt_events_same_root("peer-a", "/other/project", "main", &[event])
        .unwrap();
    assert_eq!(wrong_scope, 0);
}

#[test]
fn sqlite_schema_migrates_v1_through_silent_session_config_schema_v6() {
    let dir = temp_dir();
    let mut cfg = FocusaConfig::default();
    cfg.data_dir = dir.to_string_lossy().to_string();
    let p = SqlitePersistence::new(&cfg).unwrap();
    drop(p);

    let db_path = dir.join("focusa.sqlite");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute(
        "UPDATE meta SET value = '1' WHERE key = 'schema_version'",
        [],
    )
    .unwrap();
    drop(conn);

    let p2 = SqlitePersistence::new(&cfg).unwrap();
    let conn = Connection::open(db_path).unwrap();
    let version: String = conn
        .query_row(
            "SELECT value FROM meta WHERE key='schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(version, "6");
    let table_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='crdt_events'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(table_count, 1);
    drop(p2);
}

#[test]
fn sqlite_crdt_same_root_two_daemon_reconciliation_converges() {
    use crate::sync::CrdtLog;

    let dir_a = temp_dir();
    let dir_b = temp_dir();
    let mut cfg_a = FocusaConfig::default();
    cfg_a.data_dir = dir_a.to_string_lossy().to_string();
    let mut cfg_b = FocusaConfig::default();
    cfg_b.data_dir = dir_b.to_string_lossy().to_string();
    let a = SqlitePersistence::new(&cfg_a).unwrap();
    let b = SqlitePersistence::new(&cfg_b).unwrap();

    let mut log_a = CrdtLog::new();
    let mut log_b = CrdtLog::new();
    let mut event_a = test_event("daemon-a-turn");
    event_a.correlation_id =
        Some("project_root=/tmp/focusa-portable-fixture/project|continuity_id=main".into());
    event_a.machine_id = Some("daemon-a".into());
    let mut event_b = test_event("daemon-b-turn");
    event_b.correlation_id =
        Some("project_root=/tmp/focusa-portable-fixture/project|continuity_id=main".into());
    event_b.machine_id = Some("daemon-b".into());

    let crdt_a = log_a.add_local_event(event_a, "daemon-a");
    let crdt_b = log_b.add_local_event(event_b, "daemon-b");
    a.append_crdt_event(&crdt_a, None).unwrap();
    b.append_crdt_event(&crdt_b, None).unwrap();

    let a_events = a
        .crdt_events_for_scope("/tmp/focusa-portable-fixture/project", "main", 100)
        .unwrap();
    let b_events = b
        .crdt_events_for_scope("/tmp/focusa-portable-fixture/project", "main", 100)
        .unwrap();
    assert_eq!(
        a.import_crdt_events_same_root(
            "daemon-b",
            "/tmp/focusa-portable-fixture/project",
            "main",
            &b_events
        )
        .unwrap(),
        1
    );
    assert_eq!(
        b.import_crdt_events_same_root(
            "daemon-a",
            "/tmp/focusa-portable-fixture/project",
            "main",
            &a_events
        )
        .unwrap(),
        1
    );

    let final_a = a
        .crdt_events_for_scope("/tmp/focusa-portable-fixture/project", "main", 100)
        .unwrap();
    let final_b = b
        .crdt_events_for_scope("/tmp/focusa-portable-fixture/project", "main", 100)
        .unwrap();
    let ids_a: std::collections::BTreeSet<_> = final_a.iter().map(|e| e.entry.id).collect();
    let ids_b: std::collections::BTreeSet<_> = final_b.iter().map(|e| e.entry.id).collect();
    assert_eq!(ids_a, ids_b);
    assert_eq!(ids_a.len(), 2);
}

#[test]
fn silent_session_projection_and_hash_chain_are_atomic_replay_safe_and_tamper_evident() {
    let dir = temp_dir();
    let mut config = FocusaConfig::default();
    config.data_dir = dir.to_string_lossy().to_string();
    let persistence = SqlitePersistence::new(&config).unwrap();
    let (session, first) = sample_silent_session(&dir);

    persistence
        .persist_silent_session_event(&session, &first)
        .unwrap();
    persistence
        .persist_silent_session_event(&session, &first)
        .unwrap();
    assert_eq!(
        persistence.load_silent_session(session.session_id).unwrap(),
        Some(session.clone())
    );
    assert_eq!(
        persistence
            .load_silent_session_events(session.session_id)
            .unwrap(),
        vec![first.clone()]
    );
    persistence
        .verify_silent_session_event_chain(session.session_id)
        .unwrap();

    let mut conflicting = first.clone();
    conflicting.payload = serde_json::json!({"pid": 99});
    assert!(
        persistence
            .persist_silent_session_event(&session, &conflicting)
            .is_err()
    );

    let mut second = first.clone();
    second.event_id = SilentSessionEventId::new();
    second.seq = 2;
    second.kind = "activity_observed".into();
    persistence
        .persist_silent_session_event(&session, &second)
        .unwrap();
    persistence
        .verify_silent_session_event_chain(session.session_id)
        .unwrap();

    persistence
        .corrupt_silent_session_event_hash_for_test(&second.event_id.to_string())
        .unwrap();
    assert!(
        persistence
            .verify_silent_session_event_chain(session.session_id)
            .is_err()
    );
}

#[test]
fn silent_session_run_event_cursor_resumes_exactly_and_rejects_cross_run_replay() {
    let dir = temp_dir();
    let mut config = FocusaConfig::default();
    config.data_dir = dir.to_string_lossy().to_string();
    let persistence = SqlitePersistence::new(&config).unwrap();
    let (session, first) = sample_silent_session(&dir);
    persistence
        .persist_silent_session_event(&session, &first)
        .unwrap();
    let mut second = first.clone();
    second.event_id = SilentSessionEventId::new();
    second.seq = 2;
    second.kind = "output_observed".into();
    persistence
        .persist_silent_session_event(&session, &second)
        .unwrap();

    assert_eq!(
        persistence
            .load_silent_session_run_events_after(
                session.session_id,
                first.run_id,
                Some(first.event_id),
            )
            .unwrap(),
        vec![second]
    );
    assert!(
        persistence
            .load_silent_session_run_events_after(
                session.session_id,
                SilentSessionRunId::new(),
                Some(first.event_id),
            )
            .is_err()
    );
}

#[test]
fn silent_session_run_projection_survives_restart_and_fences_identity_and_generation() {
    let dir = temp_dir();
    let mut config = FocusaConfig::default();
    config.data_dir = dir.to_string_lossy().to_string();
    let (session, first) = sample_silent_session(&dir);
    let run = sample_silent_run(&session);

    {
        let persistence = SqlitePersistence::new(&config).unwrap();
        persistence
            .persist_silent_session_event(&session, &first)
            .unwrap();
        persistence.put_silent_session_run(&session, &run).unwrap();

        let mut rebound = run.clone();
        rebound.generation = 2;
        assert!(
            persistence
                .put_silent_session_run(&session, &rebound)
                .is_err()
        );
    }

    let reopened = SqlitePersistence::new(&config).unwrap();
    assert_eq!(
        reopened
            .load_silent_session_run(session.session_id, run.run_id)
            .unwrap(),
        Some(run.clone())
    );
    assert_eq!(
        reopened
            .load_silent_session_run(SilentSessionId::new(), run.run_id)
            .unwrap(),
        None
    );
}

#[test]
fn silent_session_lifecycle_cas_is_atomic_and_rejects_stale_state_generation_and_cursor() {
    let dir = temp_dir();
    let mut config = FocusaConfig::default();
    config.data_dir = dir.to_string_lossy().to_string();
    let persistence = SqlitePersistence::new(&config).unwrap();
    let (session, first) = sample_silent_session(&dir);
    let run = sample_silent_run(&session);
    persistence
        .persist_silent_session_event(&session, &first)
        .unwrap();
    persistence.put_silent_session_run(&session, &run).unwrap();
    let approval = |approval_id: &str, action_digest: &str| SilentSessionApproval {
        schema: SILENT_SESSION_APPROVAL_SCHEMA.into(),
        approval_id: approval_id.into(),
        operator_actor_ref: "operator:test".into(),
        action: "pause".into(),
        project_identity_ref: session.project_identity_ref.clone(),
        continuity_id: session.continuity_id.clone(),
        workpoint_ref: None,
        session_id: Some(session.session_id),
        run_id: Some(run.run_id),
        config_hash: "config:test".into(),
        action_digest: action_digest.into(),
        model_binding: "test/test-model".into(),
        workspace_ref: "workspace:test".into(),
        risk_class: "controlled".into(),
        expires_at: Utc::now() + chrono::Duration::minutes(5),
        permitted_side_effects: vec!["lifecycle:pausing".into()],
    };
    let accepted_approval = approval("approval:pause", "digest:pause");
    let stale_approval = approval("approval:stale", "digest:stale");
    persistence
        .put_silent_session_approval(&accepted_approval)
        .unwrap();
    persistence
        .put_silent_session_approval(&stale_approval)
        .unwrap();

    let mut pausing = session.clone();
    pausing.lifecycle_state = SilentSessionLifecycleState::Pausing;
    let mut advanced_run = run.clone();
    advanced_run.current_event_seq = 2;
    let mut transition = first.clone();
    transition.event_id = SilentSessionEventId::new();
    transition.seq = 2;
    transition.kind = "lifecycle.pausing".into();
    transition.payload = serde_json::json!({"reason_code": "operator_pause"});

    persistence
        .persist_silent_session_lifecycle_cas(
            SilentSessionLifecycleState::Running,
            1,
            run.run_id,
            &accepted_approval.approval_id,
            &accepted_approval.action_digest,
            Utc::now(),
            &pausing,
            &advanced_run,
            &transition,
        )
        .unwrap();
    assert_eq!(
        persistence
            .load_silent_session(session.session_id)
            .unwrap()
            .unwrap()
            .lifecycle_state,
        SilentSessionLifecycleState::Pausing
    );
    assert_eq!(
        persistence
            .load_silent_session_run(session.session_id, run.run_id)
            .unwrap()
            .unwrap()
            .current_event_seq,
        2
    );
    assert_eq!(
        persistence
            .load_silent_session_approval(&accepted_approval.approval_id)
            .unwrap(),
        None
    );

    let mut stale_event = transition.clone();
    stale_event.event_id = SilentSessionEventId::new();
    assert!(
        persistence
            .persist_silent_session_lifecycle_cas(
                SilentSessionLifecycleState::Running,
                1,
                run.run_id,
                &stale_approval.approval_id,
                &stale_approval.action_digest,
                Utc::now(),
                &pausing,
                &advanced_run,
                &stale_event,
            )
            .is_err()
    );
    assert!(
        persistence
            .persist_silent_session_lifecycle_cas(
                SilentSessionLifecycleState::Pausing,
                2,
                run.run_id,
                &stale_approval.approval_id,
                &stale_approval.action_digest,
                Utc::now(),
                &pausing,
                &advanced_run,
                &stale_event,
            )
            .is_err()
    );
    assert_eq!(
        persistence
            .load_silent_session_events(session.session_id)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        persistence
            .load_silent_session_approval(&stale_approval.approval_id)
            .unwrap(),
        Some(stale_approval.clone())
    );

    let mut newer_run = run.clone();
    newer_run.run_id = SilentSessionRunId::new();
    newer_run.generation = 2;
    newer_run.current_event_seq = 0;
    persistence
        .put_silent_session_run(&pausing, &newer_run)
        .unwrap();
    let mut delayed_run = advanced_run.clone();
    delayed_run.current_event_seq = 3;
    let mut delayed_event = transition.clone();
    delayed_event.event_id = SilentSessionEventId::new();
    delayed_event.seq = 3;
    let delayed_error = persistence
        .persist_silent_session_lifecycle_cas(
            SilentSessionLifecycleState::Pausing,
            1,
            run.run_id,
            &stale_approval.approval_id,
            &stale_approval.action_digest,
            Utc::now(),
            &pausing,
            &delayed_run,
            &delayed_event,
        )
        .unwrap_err();
    assert!(
        delayed_error
            .to_string()
            .contains("silent-session generation conflict"),
        "unexpected delayed-control error: {delayed_error:#}"
    );
    assert_eq!(
        persistence
            .load_silent_session_approval(&stale_approval.approval_id)
            .unwrap(),
        Some(stale_approval.clone())
    );
    assert_eq!(
        persistence
            .redeem_silent_session_approval(
                &stale_approval.approval_id,
                &stale_approval.action_digest,
                Utc::now(),
            )
            .unwrap(),
        stale_approval
    );
    persistence
        .verify_silent_session_event_chain(session.session_id)
        .unwrap();
}

#[test]
fn silent_session_principals_approvals_and_runner_keys_survive_restart_and_replay_fails() {
    let dir = temp_dir();
    let mut config = FocusaConfig::default();
    config.data_dir = dir.to_string_lossy().to_string();
    let principal = SilentSessionPrincipal {
        actor_ref: "operator:test".into(),
        actor_instance_ref: "operator:test:instance".into(),
        role: SilentSessionRole::Operator,
        os_user: "test".into(),
        project_root: dir.join("project"),
        project_identity_ref: "project:test".into(),
        continuity_id: "main".into(),
        workpoint_ref: Some("workpoint:test".into()),
        scopes: [SilentSessionScope::Control, SilentSessionScope::Read]
            .into_iter()
            .collect(),
    };
    let approval = SilentSessionApproval {
        schema: SILENT_SESSION_APPROVAL_SCHEMA.into(),
        approval_id: "approval:test".into(),
        operator_actor_ref: principal.actor_ref.clone(),
        action: "start".into(),
        project_identity_ref: principal.project_identity_ref.clone(),
        continuity_id: principal.continuity_id.clone(),
        workpoint_ref: principal.workpoint_ref.clone(),
        session_id: None,
        run_id: None,
        config_hash: "config-hash".into(),
        action_digest: "digest:test".into(),
        model_binding: "openai/model".into(),
        workspace_ref: "workspace:test".into(),
        risk_class: "controlled".into(),
        expires_at: Utc::now() + chrono::Duration::minutes(5),
        permitted_side_effects: vec!["write_project".into()],
    };
    {
        let persistence = SqlitePersistence::new(&config).unwrap();
        persistence
            .put_silent_session_principal(&principal)
            .unwrap();
        persistence.put_silent_session_approval(&approval).unwrap();
        persistence
            .put_silent_session_runner_identity(
                "runner:test",
                "base64-public-key",
                "test",
                "project:test",
            )
            .unwrap();
    }
    let persistence = SqlitePersistence::new(&config).unwrap();
    assert_eq!(
        persistence
            .load_silent_session_principal(&principal.actor_instance_ref)
            .unwrap(),
        Some(principal)
    );
    assert_eq!(
        persistence
            .load_silent_session_runner_identity("runner:test")
            .unwrap(),
        Some((
            "base64-public-key".into(),
            "test".into(),
            "project:test".into()
        ))
    );
    assert_eq!(
        persistence
            .redeem_silent_session_approval(
                &approval.approval_id,
                &approval.action_digest,
                Utc::now(),
            )
            .unwrap(),
        approval
    );
    assert!(
        persistence
            .redeem_silent_session_approval("approval:test", "digest:test", Utc::now())
            .is_err()
    );
    assert!(persistence.put_silent_session_approval(&approval).is_err());
}

#[test]
fn silent_session_writer_lease_registry_survives_restart_and_rejects_stale_cas() {
    let dir = temp_dir();
    let mut config = FocusaConfig::default();
    config.data_dir = dir.to_string_lossy().to_string();
    {
        let persistence = SqlitePersistence::new(&config).unwrap();
        let (revision, mut registry) = persistence
            .load_silent_session_writer_lease_registry()
            .unwrap();
        assert_eq!(revision, 0);
        registry.next_fencing_token = 42;
        assert_eq!(
            persistence
                .persist_silent_session_writer_lease_registry_cas(revision, &registry)
                .unwrap(),
            1
        );
        assert!(
            persistence
                .persist_silent_session_writer_lease_registry_cas(0, &registry)
                .is_err()
        );
    }
    let persistence = SqlitePersistence::new(&config).unwrap();
    let (revision, registry) = persistence
        .load_silent_session_writer_lease_registry()
        .unwrap();
    assert_eq!(revision, 1);
    assert_eq!(
        registry,
        crate::silent_session_writer::WriterLeaseRegistry {
            next_fencing_token: 42,
            ..crate::silent_session_writer::WriterLeaseRegistry::default()
        }
    );
}
