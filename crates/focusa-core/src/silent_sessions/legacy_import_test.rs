use std::fs;

use serde_json::json;

use crate::silent_sessions::{LegacySilentSessionImporter, StreamStorageError};

fn temp_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("focusa-legacy-import-{}", uuid::Uuid::now_v7()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[cfg(unix)]
fn current_uid(path: &std::path::Path) -> u32 {
    use std::os::unix::fs::MetadataExt;
    fs::metadata(path).unwrap().uid()
}

#[cfg(not(unix))]
fn current_uid(_path: &std::path::Path) -> u32 {
    0
}

#[test]
fn imports_untrusted_metadata_and_logs_without_executing_commands() {
    let dir = temp_dir();
    let logs = dir.join("logs");
    fs::create_dir(&logs).unwrap();
    let log = logs.join("legacy.log");
    fs::write(&log, b"bounded legacy output\n").unwrap();
    let command_marker = dir.join("must-not-exist");
    let registry = dir.join("focusa-silent-registry.json");
    fs::write(
        &registry,
        serde_json::to_vec_pretty(&json!({
            "sessions": {
                "focusa-silent-proof": {
                    "log_path": log,
                    "command": format!("touch {}", command_marker.display()),
                    "root_dir": dir
                }
            }
        }))
        .unwrap(),
    )
    .unwrap();
    let destination = temp_dir();
    let importer =
        LegacySilentSessionImporter::new(&destination, vec![logs.clone()], current_uid(&registry))
            .unwrap();
    let first = importer.import_registry(&registry).unwrap();
    assert_eq!(first.len(), 1);
    assert!(first[0].legacy_unverified);
    assert_eq!(first[0].aliases, vec!["focusa-silent-proof"]);
    assert!(!command_marker.exists());
    let copied = destination.join(first[0].copied_log_ref.as_ref().unwrap());
    assert_eq!(fs::read(copied).unwrap(), b"bounded legacy output\n");

    let second = importer.import_registry(&registry).unwrap();
    assert_eq!(second[0].session_id, first[0].session_id);
    assert_eq!(second[0].run_id, first[0].run_id);
}

#[cfg(unix)]
#[test]
fn rejects_symlinked_legacy_registry_and_log_paths() {
    use std::os::unix::fs::symlink;

    let dir = temp_dir();
    let real = dir.join("real.json");
    fs::write(&real, b"{}").unwrap();
    let linked = dir.join("focusa-silent-registry.json");
    symlink(&real, &linked).unwrap();
    let importer =
        LegacySilentSessionImporter::new(temp_dir(), vec![dir], current_uid(&real)).unwrap();
    assert!(matches!(
        importer.import_registry(&linked),
        Err(StreamStorageError::UnsafePath(_))
    ));
}
