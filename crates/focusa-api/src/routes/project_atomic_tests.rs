//! Consumer regressions for the platform-owned atomic publication boundary.

#[test]
fn genesis_atomic_json_can_create_and_replace() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("packet.json");
    crate::routes::project_genesis_support::write_json_atomic(
        &path,
        &serde_json::json!({"sequence": 1}),
    )
    .unwrap();
    crate::routes::project_genesis_support::write_json_atomic(
        &path,
        &serde_json::json!({"sequence": 2}),
    )
    .unwrap();
    let actual: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(actual["sequence"], 2);
}

#[test]
fn marker_atomic_json_can_create_and_replace() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("synthetic-marker.json");
    super::write_json_file_atomic(&path, &serde_json::json!({"sequence": 1})).unwrap();
    super::write_json_file_atomic(&path, &serde_json::json!({"sequence": 2})).unwrap();
    let actual: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(actual["sequence"], 2);
}

#[test]
fn marker_atomic_json_rejects_nonregular_destination() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("synthetic-marker");
    std::fs::create_dir(&path).unwrap();
    assert!(super::write_json_file_atomic(&path, &serde_json::json!({"sequence": 1})).is_err());
    assert!(path.is_dir());
}

#[cfg(unix)]
#[test]
fn marker_atomic_json_does_not_follow_a_destination_symlink() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("untouched.json");
    let marker = root.path().join("synthetic-marker.json");
    std::fs::write(&target, b"original").unwrap();
    std::os::unix::fs::symlink(&target, &marker).unwrap();
    assert!(super::write_json_file_atomic(&marker, &serde_json::json!({"sequence": 2})).is_err());
    assert_eq!(std::fs::read(target).unwrap(), b"original");
}
