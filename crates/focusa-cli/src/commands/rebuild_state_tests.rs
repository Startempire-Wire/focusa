use super::*;
use focusa_core::types::{FocusaConfig, FocusaEvent, SignalOrigin};

fn database() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE snapshots(name TEXT, version INTEGER, ts TEXT, state_json TEXT);
         CREATE TABLE event_hash_chain(chain_index INTEGER);
         CREATE TABLE snapshot_event_cursors(name TEXT PRIMARY KEY,event_sequence INTEGER,
            snapshot_version INTEGER,updated_at TEXT);",
    )
    .unwrap();
    conn
}

#[test]
fn missing_snapshot_fails_without_creating_one() {
    let mut conn = database();
    assert!(write_snapshot(&mut conn, &FocusaState::new(), 0).is_err());
    let count: i64 = conn
        .query_row("SELECT count(*) FROM snapshots", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn ambiguous_snapshot_rolls_back_all_changes() {
    let mut conn = database();
    conn.execute_batch("INSERT INTO snapshots VALUES ('focusa',7,'original','original'),('focusa',7,'original','original');").unwrap();
    assert!(write_snapshot(&mut conn, &FocusaState::new(), 0).is_err());
    let unchanged: i64 = conn.query_row("SELECT count(*) FROM snapshots WHERE version=7 AND ts='original' AND state_json='original'", [], |r| r.get(0)).unwrap();
    assert_eq!(unchanged, 2);
}

#[test]
fn existing_snapshot_is_written_and_other_rows_unchanged() {
    let mut conn = database();
    conn.execute_batch(
        "INSERT INTO snapshots VALUES ('focusa',7,'old','old'),('other',8,'other','other');
         CREATE UNIQUE INDEX snapshot_names ON snapshots(name);",
    )
    .unwrap();
    let state = FocusaState::new();
    write_snapshot(&mut conn, &state, 0).unwrap();
    let stored: String = conn
        .query_row(
            "SELECT state_json FROM snapshots WHERE name='focusa'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stored).unwrap(),
        serde_json::to_value(&state).unwrap()
    );
    let cursor: (i64, i64) = conn.query_row(
        "SELECT event_sequence,snapshot_version FROM snapshot_event_cursors WHERE name='focusa'", [],
        |r| Ok((r.get(0)?,r.get(1)?)),
    ).unwrap();
    assert_eq!(cursor, (0, state.version as i64));
    let other: String = conn
        .query_row(
            "SELECT state_json FROM snapshots WHERE name='other'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(other, "other");
    // A failed cursor write must roll the snapshot back too.
    conn.execute_batch("DROP TABLE snapshot_event_cursors;")
        .unwrap();
    let mut newer = state;
    newer.version += 1;
    assert!(write_snapshot(&mut conn, &newer, 0).is_err());
    let after: String = conn
        .query_row(
            "SELECT state_json FROM snapshots WHERE name='focusa'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(after, stored);
}

fn rejected_event() -> EventLogEntry {
    EventLogEntry {
        id: uuid::Uuid::now_v7(),
        timestamp: chrono::Utc::now(),
        temporal: Default::default(),
        event: FocusaEvent::TurnCompleted {
            turn_id: "rebuild-regression".into(),
            harness_name: "test".into(),
            raw_user_input: None,
            assistant_output: None,
            artifacts_used: vec![],
            errors: vec![],
            prompt_tokens: None,
            completion_tokens: None,
        },
        correlation_id: None,
        origin: SignalOrigin::Cli,
        machine_id: None,
        instance_id: None,
        session_id: None,
        project_root: None,
        continuity_id: None,
        thread_id: Some(uuid::Uuid::now_v7()),
        is_observation: false,
    }
}

#[tokio::test]
async fn dry_run_never_creates_missing_input_database() {
    let path = std::env::temp_dir().join(format!(
        "focusa-rebuild-missing-{}.sqlite",
        uuid::Uuid::now_v7()
    ));
    let result = run(
        RebuildStateArgs {
            snapshot_db: path.to_string_lossy().into_owned(),
            events_db: path.to_string_lossy().into_owned(),
            since: None,
            dry_run: true,
            confirm: false,
        },
        true,
    )
    .await;
    assert!(result.is_err());
    assert!(!path.exists());
}

#[tokio::test]
async fn command_rejected_event_preserves_target_snapshot() {
    let root = std::env::temp_dir().join(format!("focusa-rebuild-test-{}", uuid::Uuid::now_v7()));
    let source = SqlitePersistence::new(&FocusaConfig {
        data_dir: root.join("source").to_string_lossy().into_owned(),
        ..Default::default()
    })
    .unwrap();
    let target = SqlitePersistence::new(&FocusaConfig {
        data_dir: root.join("target").to_string_lossy().into_owned(),
        ..Default::default()
    })
    .unwrap();
    let initial = FocusaState::new();
    source.save_state(&initial).unwrap();
    target.save_state(&initial).unwrap();
    let source_path = root.join("source/focusa.sqlite");
    let target_path = root.join("target/focusa.sqlite");
    target.append_event(&rejected_event()).unwrap();
    let conn = Connection::open(&target_path).unwrap();
    let original: (String, String) = conn
        .query_row(
            "SELECT ts,state_json FROM snapshots WHERE name='focusa'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    let result = run(
        RebuildStateArgs {
            snapshot_db: source_path.to_string_lossy().into_owned(),
            events_db: target_path.to_string_lossy().into_owned(),
            since: None,
            dry_run: false,
            confirm: true,
        },
        true,
    )
    .await;
    assert!(format!("{:#}", result.unwrap_err()).contains("snapshot not written"));
    let stored: (String, String) = conn
        .query_row(
            "SELECT ts,state_json FROM snapshots WHERE name='focusa'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(stored, original);

    // The same command uses the real producer's text UUIDs and cursor schema.
    let valid_target = SqlitePersistence::new(&FocusaConfig {
        data_dir: root.join("valid").to_string_lossy().into_owned(),
        ..Default::default()
    })
    .unwrap();
    valid_target.save_state(&initial).unwrap();
    let mut valid = rejected_event();
    valid.thread_id = None;
    valid.instance_id = Some(uuid::Uuid::now_v7());
    valid.session_id = Some(uuid::Uuid::now_v7());
    valid.timestamp = "2000-01-01T00:00:00Z".parse().unwrap();
    valid.event = FocusaEvent::ContinuousWorkModeEnabled {
        project_run_id: uuid::Uuid::now_v7(),
        policy: Default::default(),
        scope: None,
        work_item_id: None,
        workpoint_id: None,
    };
    valid_target.append_event(&valid).unwrap();
    let valid_path = root.join("valid/focusa.sqlite");
    run(
        RebuildStateArgs {
            snapshot_db: source_path.to_string_lossy().into_owned(),
            events_db: valid_path.to_string_lossy().into_owned(),
            since: Some("2050-01-01T00:00:00Z".into()),
            dry_run: false,
            confirm: true,
        },
        true,
    )
    .await
    .unwrap();
    let (rebuilt, cursor) = valid_target
        .load_state_with_event_sequence()
        .unwrap()
        .unwrap();
    assert_eq!(cursor, 1);
    assert_eq!(rebuilt.work_loop.budget_epoch_id, Some(valid.id));
    assert_eq!(
        rebuilt.work_loop.budget_epoch_started_at,
        Some(valid.timestamp)
    );
    assert!(
        valid_target
            .durable_events_after(cursor, 10)
            .unwrap()
            .is_empty()
    );
    let mut valid_conn = Connection::open(&valid_path).unwrap();
    assert!(
        write_snapshot(&mut valid_conn, &initial, 0)
            .unwrap_err()
            .to_string()
            .contains("advanced")
    );
    let after = valid_target
        .load_state_with_event_sequence()
        .unwrap()
        .unwrap();
    assert_eq!(after.0.version, rebuilt.version);
    assert_eq!(after.1, cursor);

    let peer = SqlitePersistence::new(&FocusaConfig {
        data_dir: root.join("peer").to_string_lossy().into_owned(),
        ..Default::default()
    })
    .unwrap();
    valid.id = uuid::Uuid::now_v7();
    peer.append_event(&valid).unwrap();
    let peer_state = replay_events(initial, std::slice::from_ref(&valid)).unwrap();
    peer.save_state(&peer_state).unwrap();
    let mismatch = run(
        RebuildStateArgs {
            snapshot_db: valid_path.to_string_lossy().into_owned(),
            events_db: root
                .join("peer/focusa.sqlite")
                .to_string_lossy()
                .into_owned(),
            since: None,
            dry_run: false,
            confirm: true,
        },
        true,
    )
    .await
    .unwrap_err();
    assert!(mismatch.to_string().contains("different event prefix"));
    assert_eq!(peer.load_state_with_event_sequence().unwrap().unwrap().1, 1);
    drop((conn, valid_conn, source, target, valid_target, peer));
    std::fs::remove_dir_all(root).unwrap();
}
