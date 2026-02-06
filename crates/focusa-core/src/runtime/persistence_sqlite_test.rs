use crate::runtime::persistence_sqlite::SqlitePersistence;
use crate::types::FocusaConfig;

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
