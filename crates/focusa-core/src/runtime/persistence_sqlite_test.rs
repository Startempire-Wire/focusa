use crate::runtime::persistence_sqlite::SqlitePersistence;
use crate::types::FocusaConfig;
use rusqlite::Connection;

fn temp_dir() -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("focusa-test-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
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
