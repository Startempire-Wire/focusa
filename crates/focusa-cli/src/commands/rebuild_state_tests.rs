use super::*;
use focusa_core::types::{FocusaEvent, SignalOrigin};

fn database() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE snapshots(name TEXT, version INTEGER, ts TEXT, state_json TEXT);",
    )
    .unwrap();
    conn
}

#[test]
fn missing_snapshot_fails_without_creating_one() {
    let mut conn = database();
    assert!(write_snapshot(&mut conn, &FocusaState::new()).is_err());
    let count: i64 = conn
        .query_row("SELECT count(*) FROM snapshots", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn ambiguous_snapshot_rolls_back_all_changes() {
    let mut conn = database();
    conn.execute_batch("INSERT INTO snapshots VALUES ('focusa',7,'original','original'),('focusa',7,'original','original');").unwrap();
    assert!(write_snapshot(&mut conn, &FocusaState::new()).is_err());
    let unchanged: i64 = conn.query_row("SELECT count(*) FROM snapshots WHERE version=7 AND ts='original' AND state_json='original'", [], |r| r.get(0)).unwrap();
    assert_eq!(unchanged, 2);
}

#[test]
fn existing_snapshot_is_written_and_other_rows_unchanged() {
    let mut conn = database();
    conn.execute_batch(
        "INSERT INTO snapshots VALUES ('focusa',7,'old','old'),('other',8,'other','other');",
    )
    .unwrap();
    let state = FocusaState::new();
    write_snapshot(&mut conn, &state).unwrap();
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
    let other: String = conn
        .query_row(
            "SELECT state_json FROM snapshots WHERE name='other'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(other, "other");
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
            since: "2020-01-01T00:00:00Z".into(),
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
    std::fs::create_dir(&root).unwrap();
    let source = root.join("source.sqlite");
    let target = root.join("target.sqlite");
    let state_json = serde_json::to_string(&FocusaState::new()).unwrap();
    for path in [&source, &target] {
        let conn = Connection::open(path).unwrap();
        conn.execute_batch("CREATE TABLE snapshots(name TEXT PRIMARY KEY, version INTEGER, ts TEXT, state_json TEXT);").unwrap();
        conn.execute(
            "INSERT INTO snapshots VALUES ('focusa',0,'original',?1)",
            [&state_json],
        )
        .unwrap();
    }
    let conn = Connection::open(&target).unwrap();
    conn.execute_batch("CREATE TABLE events(event_id TEXT, ts TEXT, origin TEXT, correlation_id TEXT, payload_json TEXT, machine_id TEXT, instance_id BLOB, session_id TEXT, thread_id BLOB, is_observation INTEGER);").unwrap();
    let event = rejected_event();
    conn.execute(
        "INSERT INTO events VALUES (?1,?2,'cli',NULL,?3,NULL,NULL,NULL,?4,0)",
        rusqlite::params![
            event.id.to_string(),
            event.timestamp.to_rfc3339(),
            serde_json::to_string(&event.event).unwrap(),
            event.thread_id
        ],
    )
    .unwrap();
    let result = run(
        RebuildStateArgs {
            snapshot_db: source.to_string_lossy().into_owned(),
            events_db: target.to_string_lossy().into_owned(),
            since: "2000-01-01T00:00:00Z".into(),
            dry_run: false,
            confirm: true,
        },
        true,
    )
    .await;
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("snapshot not written")
    );
    let stored: (String, String) = conn
        .query_row(
            "SELECT ts,state_json FROM snapshots WHERE name='focusa'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(stored, ("original".into(), state_json));
    drop(conn);
    std::fs::remove_dir_all(root).unwrap();
}
