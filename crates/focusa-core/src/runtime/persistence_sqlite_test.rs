#![allow(clippy::field_reassign_with_default)]

use crate::runtime::persistence_sqlite::SqlitePersistence;
use crate::types::{EventLogEntry, FocusaConfig, FocusaEvent, SignalOrigin};
use chrono::Utc;
use rusqlite::Connection;
use uuid::Uuid;

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
fn sqlite_schema_migrates_v1_to_crdt_schema_v2() {
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
    assert_eq!(version, "2");
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
