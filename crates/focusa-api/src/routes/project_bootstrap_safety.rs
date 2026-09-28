//! Filesystem safety for the existing Project Bootstrap receipt transaction.
//! Never recursively delete a project directory or follow receipt-supplied links.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions, TryLockError},
    io::{self, Read},
    path::{Component, Path},
};

const MAX_ENTRIES: usize = 16_384;
const MAX_BYTES: u64 = 64 * 1024 * 1024;
const ARTIFACTS: &[&str] = &[
    ".focusa-project.json",
    ".focusa/settings.json",
    "docs",
    ".git",
    ".beads",
    ".focusa/genesis",
];
pub(super) type Snapshot = BTreeMap<String, Value>;

/// The kernel owns the lock lifetime, including process death. Never unlink a
/// locked inode: another caller could otherwise acquire a different inode.
pub(super) struct BootstrapLock {
    _file: File,
}
impl BootstrapLock {
    pub(super) fn acquire(root: &Path) -> io::Result<Self> {
        let path = root.join(".focusa-bootstrap.lock");
        if let Ok(meta) = fs::symlink_metadata(&path)
            && !meta.is_file()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "bootstrap lock is not a regular file",
            ));
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;
        if !file.metadata()?.is_file() || fs::symlink_metadata(&path)?.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "bootstrap lock identity changed",
            ));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let opened = file.metadata()?;
            let named = fs::symlink_metadata(&path)?;
            if (opened.dev(), opened.ino()) != (named.dev(), named.ino()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "bootstrap lock identity changed",
                ));
            }
        }
        file.try_lock().map_err(|error| match error {
            TryLockError::WouldBlock => {
                io::Error::new(io::ErrorKind::WouldBlock, "bootstrap transaction is active")
            }
            TryLockError::Error(error) => error,
        })?;
        Ok(Self { _file: file })
    }
}

/// Read-only canonical v1 adoption check shared by preview and apply.
/// A legacy v2 bootstrap marker requires explicit migration, not a rewrite.
pub(super) fn validate_project_marker(
    root: &Path,
    project_id: &str,
    canonical_name: &str,
) -> Result<(), (&'static str, String)> {
    let path = root.join(".focusa-project.json");
    let malformed = |message: String| ("malformed_project_marker", message);
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(malformed(error.to_string())),
        Ok(metadata) if !metadata.file_type().is_file() => {
            return Err(malformed("existing marker is not a regular file".into()));
        }
        Ok(_) => {}
    }
    let bytes = fs::read(&path).map_err(|error| malformed(error.to_string()))?;
    let marker: Value = serde_json::from_slice(&bytes).map_err(|_| {
        malformed("existing marker is invalid JSON; repair it explicitly before bootstrap".into())
    })?;
    if marker["schema"] != focusa_core::project_marker::MARKER_SCHEMA {
        return Err((
            "unsupported_project_marker",
            "existing marker must use the canonical focusa.project.v1 schema; migrate older bootstrap markers explicitly".into(),
        ));
    }
    match focusa_core::project_marker::read_marker(root) {
        focusa_core::project_marker::MarkerReadOutcome::Valid
        | focusa_core::project_marker::MarkerReadOutcome::LegacyMinimal { .. } => {}
        focusa_core::project_marker::MarkerReadOutcome::Missing => {
            return Err(malformed("marker disappeared during validation".into()));
        }
        focusa_core::project_marker::MarkerReadOutcome::Corrupted { error } => {
            return Err(malformed(error));
        }
    }
    let typed: focusa_core::project_marker::ProjectMarker =
        serde_json::from_value(marker).map_err(|error| malformed(error.to_string()))?;
    if typed.project_id != project_id
        || typed.canonical_name != canonical_name
        || typed.canonical_project_root().as_deref() != Some(root)
    {
        return Err((
            "cross_project_marker_conflict",
            "existing marker belongs to a different project, name or root; verify scope before continuing".into(),
        ));
    }
    Ok(())
}

pub(super) fn request_digest(mut request: Value, root: &Path) -> String {
    if let Some(object) = request.as_object_mut() {
        for key in ["confirm", "repair_action", "idempotency_key"] {
            object.remove(key);
        }
        object.insert("project_root".into(), json!(root));
    }
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&request).expect("JSON value serializes"))
    )
}

pub(super) fn read_receipt(path: &Path) -> Result<Option<Value>, String> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("cannot read bootstrap receipt: {error}")),
    };
    let receipt: Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("malformed bootstrap receipt: {e}"))?;
    if receipt["schema"] != "focusa.project_bootstrap_receipt.v1"
        || !matches!(
            receipt["status"].as_str(),
            Some("ready" | "onboarding_required" | "rolling_back" | "rolled_back")
        )
        || !receipt["idempotency_key"].is_string()
        || !receipt["project_root"].is_string()
    {
        return Err("invalid bootstrap receipt contract; explicit reconciliation required".into());
    }
    Ok(Some(receipt))
}

pub(super) fn status_next_action(status: &str) -> &'static str {
    match status {
        "ready" => "continue from Project Genesis readiness",
        "onboarding_required" => "complete the bounded Genesis next action",
        "rolling_back" => "resume rollback using the original idempotency key",
        "rolled_back" => "preview a new bootstrap transaction with a new idempotency key",
        "not_started" => "preview bootstrap",
        _ => "inspect the bootstrap receipt before retrying",
    }
}

pub(super) fn validate_apply_receipt(
    receipt: &Value,
    root: &Path,
    key: &str,
    digest: &str,
) -> Result<(), String> {
    if receipt["project_root"] != json!(root) {
        return Err("receipt project root differs from request".into());
    }
    if receipt["idempotency_key"] == key {
        if receipt["request_digest"] != digest {
            return Err("idempotency payload changed or legacy request proof is missing".into());
        }
        if !matches!(
            receipt["status"].as_str(),
            Some("ready" | "onboarding_required")
        ) {
            return Err(
                "transaction was rolled back or needs recovery; old success cannot be replayed"
                    .into(),
            );
        }
    } else if receipt["status"] != "rolled_back" {
        return Err("existing bootstrap receipt belongs to another transaction; reconcile it before applying".into());
    }
    Ok(())
}

pub(super) fn validate_artifact_paths(root: &Path) -> Result<(), String> {
    for relative in ARTIFACTS {
        checked_path(root, relative)?;
    }
    Ok(())
}

fn checked_path(root: &Path, relative: &str) -> Result<std::path::PathBuf, String> {
    let path = Path::new(relative);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(format!("unsafe receipt path: {relative}"));
    }
    if !ARTIFACTS
        .iter()
        .any(|allowed| path == Path::new(allowed) || path.starts_with(allowed))
    {
        return Err(format!("unowned receipt path: {relative}"));
    }
    let mut current = root.to_path_buf();
    for part in path.components() {
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(format!("symlink in receipt path: {relative}"));
            }
            Ok(_) => (),
            Err(error) if error.kind() == io::ErrorKind::NotFound => (),
            Err(error) => return Err(format!("inspect {relative}: {error}")),
        }
    }
    Ok(current)
}

fn describe(path: &Path, bytes: &mut u64) -> Result<Value, String> {
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    let mut value = if meta.is_dir() {
        json!({"kind":"directory"})
    } else if meta.is_file() {
        let remaining = MAX_BYTES.saturating_sub(*bytes);
        if meta.len() > remaining {
            return Err("bootstrap snapshot byte limit exceeded".into());
        }
        let mut data = Vec::new();
        File::open(path)
            .map_err(|e| e.to_string())?
            .take(remaining + 1)
            .read_to_end(&mut data)
            .map_err(|e| e.to_string())?;
        *bytes += data.len() as u64;
        if *bytes > MAX_BYTES {
            return Err("bootstrap snapshot byte limit exceeded".into());
        }
        json!({"kind":"file", "sha256":format!("{:x}", Sha256::digest(&data)), "length":data.len()})
    } else {
        return Err("bootstrap snapshot contains a symlink or special file".into());
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        value["device"] = json!(meta.dev());
        value["inode"] = json!(meta.ino());
        value["mode"] = json!(meta.mode());
        value["uid"] = json!(meta.uid());
        value["gid"] = json!(meta.gid());
        if meta.is_file() {
            value["mtime"] = json!([meta.mtime(), meta.mtime_nsec()]);
        }
    }
    Ok(value)
}

pub(super) fn snapshot(root: &Path, created: &[String]) -> Result<Snapshot, String> {
    let mut result = Snapshot::new();
    let mut pending = Vec::new();
    let mut bytes = 0;
    for item in created {
        if item == "project_root" {
            continue;
        }
        if !ARTIFACTS.contains(&item.as_str()) {
            return Err(format!("unowned created artifact: {item}"));
        }
        pending.push(item.clone());
    }
    while let Some(relative) = pending.pop() {
        if result.len() >= MAX_ENTRIES {
            return Err("bootstrap snapshot entry limit exceeded".into());
        }
        let path = checked_path(root, &relative)?;
        if !path.try_exists().map_err(|e| e.to_string())? {
            continue;
        }
        let value = describe(&path, &mut bytes)?;
        if value["kind"] == "directory" {
            for entry in fs::read_dir(&path).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let name = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| "non-UTF8 bootstrap artifact")?;
                if pending.len() + result.len() >= MAX_ENTRIES {
                    return Err("bootstrap snapshot entry limit exceeded".into());
                }
                pending.push(format!("{relative}/{name}"));
            }
        }
        result.insert(relative, value);
    }
    Ok(result)
}

/// Preflight the entire rollback before the first deletion. A retry may tolerate
/// absent entries only after the durable receipt entered rolling_back.
pub(super) fn rollback_plan(root: &Path, receipt: &Value) -> Result<Vec<(String, Value)>, String> {
    let expected: Snapshot =
        serde_json::from_value(receipt.get("created_artifact_snapshot").cloned().ok_or(
            "legacy receipt has no artifact ownership proof; explicit reconciliation required",
        )?)
        .map_err(|e| format!("invalid artifact snapshot: {e}"))?;
    let created: Vec<String> =
        serde_json::from_value(receipt["created_by_this_transaction"].clone())
            .map_err(|e| format!("invalid created-artifact list: {e}"))?;
    let observed = snapshot(root, &created)?;
    let resume = receipt["status"] == "rolling_back";
    for (path, proof) in &expected {
        checked_path(root, path)?;
        if !created.iter().any(|item| {
            item != "project_root" && (path == item || Path::new(path).starts_with(item))
        }) {
            return Err(format!(
                "snapshot path is outside created artifacts: {path}"
            ));
        }
        match observed.get(path) {
            Some(value) if value == proof => (),
            None if resume => (),
            _ => return Err(format!("artifact changed since bootstrap: {path}")),
        }
    }
    if let Some(path) = observed.keys().find(|path| !expected.contains_key(*path)) {
        return Err(format!("foreign artifact would be affected: {path}"));
    }
    let mut plan = observed.into_iter().collect::<Vec<_>>();
    plan.sort_by(|a, b| {
        Path::new(&b.0)
            .components()
            .count()
            .cmp(&Path::new(&a.0).components().count())
            .then_with(|| a.0.cmp(&b.0))
    });
    Ok(plan)
}

pub(super) fn remove_owned_entry(
    root: &Path,
    relative: &str,
    expected: &Value,
) -> Result<(), String> {
    let path = checked_path(root, relative)?;
    if describe(&path, &mut 0)? != *expected {
        return Err(format!("artifact changed during rollback: {relative}"));
    }
    if expected["kind"] == "directory" {
        // A late foreign child prevents removal; it is never recursively deleted.
        fs::remove_dir(path)
            .map_err(|e| format!("preserved nonempty/changed directory {relative}: {e}"))
    } else {
        fs::remove_file(path).map_err(|e| format!("remove owned file {relative}: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_preflight_is_read_only_and_rejects_unproven_identity() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("missing");
        validate_project_marker(&missing, "project-a", "Project A").unwrap();
        assert!(!missing.exists());
        let root = temp.path();
        let marker_path = root.join(".focusa-project.json");
        let valid = json!({"schema":"focusa.project.v1","project_id":"project-a","canonical_name":"Project A","project_root":root,"created_at":"2026-09-28T00:00:00Z"});
        for (marker, code) in [
            (
                json!({"schema":"focusa.project.v2"}),
                "unsupported_project_marker",
            ),
            (
                json!({"schema":"focusa.project.v1"}),
                "malformed_project_marker",
            ),
        ] {
            let before = serde_json::to_vec(&marker).unwrap();
            fs::write(&marker_path, &before).unwrap();
            assert_eq!(
                validate_project_marker(root, "project-a", "Project A")
                    .unwrap_err()
                    .0,
                code
            );
            assert_eq!(fs::read(&marker_path).unwrap(), before);
            assert!(!root.join(".focusa-bootstrap.lock").exists());
            assert!(!root.join(".focusa").exists());
        }
        for (field, replacement) in [
            ("project_id", json!("other")),
            ("project_root", json!("/some/other/project")),
            ("canonical_name", json!("Other name")),
        ] {
            let mut marker = valid.clone();
            marker[field] = replacement;
            fs::write(&marker_path, serde_json::to_vec(&marker).unwrap()).unwrap();
            assert_eq!(
                validate_project_marker(root, "project-a", "Project A")
                    .unwrap_err()
                    .0,
                "cross_project_marker_conflict"
            );
        }
        fs::write(&marker_path, b"broken JSON").unwrap();
        assert_eq!(
            validate_project_marker(root, "project-a", "Project A")
                .unwrap_err()
                .0,
            "malformed_project_marker"
        );
        let bytes = serde_json::to_vec(&valid).unwrap();
        fs::write(&marker_path, &bytes).unwrap();
        validate_project_marker(root, "project-a", "Project A").unwrap();
        assert_eq!(fs::read(&marker_path).unwrap(), bytes);
    }

    #[cfg(unix)]
    #[test]
    fn marker_preflight_rejects_symlinks_without_touching_the_target() {
        let temp = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let target = external.path().join("marker");
        fs::write(&target, b"preserve this file").unwrap();
        std::os::unix::fs::symlink(&target, temp.path().join(".focusa-project.json")).unwrap();
        assert_eq!(
            validate_project_marker(temp.path(), "project-a", "Project A")
                .unwrap_err()
                .0,
            "malformed_project_marker"
        );
        assert_eq!(fs::read(&target).unwrap(), b"preserve this file");
    }
    fn fixture() -> (tempfile::TempDir, Value) {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("docs")).unwrap();
        fs::create_dir(root.path().join(".git")).unwrap();
        fs::write(root.path().join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
        fs::write(root.path().join(".focusa-project.json"), "original").unwrap();
        let created = vec![
            "project_root".into(),
            "docs".into(),
            ".git".into(),
            ".focusa-project.json".into(),
        ];
        let proof = snapshot(root.path(), &created).unwrap();
        (
            root,
            json!({"status":"ready","created_by_this_transaction":created,"created_artifact_snapshot":proof}),
        )
    }
    #[test]
    fn pristine_rollback_removes_only_manifest_entries_and_preserves_root() {
        let (root, receipt) = fixture();
        fs::write(root.path().join("adopted.txt"), "preserve").unwrap();
        for (path, proof) in rollback_plan(root.path(), &receipt).unwrap() {
            remove_owned_entry(root.path(), &path, &proof).unwrap();
        }
        assert!(root.path().is_dir());
        assert_eq!(
            fs::read_to_string(root.path().join("adopted.txt")).unwrap(),
            "preserve"
        );
        assert!(!root.path().join(".git").exists());
    }
    #[test]
    fn foreign_documents_block_before_any_deletion() {
        let (root, receipt) = fixture();
        fs::write(root.path().join("docs/later.md"), "operator work").unwrap();
        assert!(
            rollback_plan(root.path(), &receipt)
                .unwrap_err()
                .contains("foreign artifact")
        );
        assert!(root.path().join(".focusa-project.json").is_file());
        assert!(root.path().join(".git/HEAD").is_file());
    }
    #[test]
    fn later_git_state_and_modified_marker_are_preserved() {
        for path in [".git/HEAD", ".focusa-project.json"] {
            let (root, receipt) = fixture();
            fs::write(root.path().join(path), "later work").unwrap();
            assert!(rollback_plan(root.path(), &receipt).is_err());
            assert_eq!(
                fs::read_to_string(root.path().join(path)).unwrap(),
                "later work"
            );
        }
    }
    #[test]
    fn late_child_is_not_recursively_deleted() {
        let (root, receipt) = fixture();
        let plan = rollback_plan(root.path(), &receipt).unwrap();
        fs::write(root.path().join("docs/late.txt"), "keep").unwrap();
        let (_, proof) = plan.iter().find(|(path, _)| path == "docs").unwrap();
        assert!(remove_owned_entry(root.path(), "docs", proof).is_err());
        assert!(root.path().join("docs/late.txt").is_file());
    }
    #[test]
    fn legacy_and_escaping_receipts_fail_closed() {
        let (root, mut receipt) = fixture();
        receipt
            .as_object_mut()
            .unwrap()
            .remove("created_artifact_snapshot");
        assert!(rollback_plan(root.path(), &receipt).is_err());
        for path in ["../outside", "/tmp/foreign", "docs/../foreign", "unowned"] {
            assert!(checked_path(root.path(), path).is_err());
        }
    }
    #[test]
    fn interrupted_rollback_requires_durable_in_progress_state() {
        let (root, mut receipt) = fixture();
        fs::remove_file(root.path().join(".git/HEAD")).unwrap();
        assert!(rollback_plan(root.path(), &receipt).is_err());
        receipt["status"] = json!("rolling_back");
        assert!(rollback_plan(root.path(), &receipt).is_ok());
    }
    #[cfg(unix)]
    #[test]
    fn symlink_substitution_cannot_escape_project() {
        let (root, receipt) = fixture();
        let foreign = tempfile::tempdir().unwrap();
        fs::write(foreign.path().join("keep"), "foreign").unwrap();
        fs::remove_dir(root.path().join("docs")).unwrap();
        std::os::unix::fs::symlink(foreign.path(), root.path().join("docs")).unwrap();
        assert!(rollback_plan(root.path(), &receipt).is_err());
        assert!(foreign.path().join("keep").is_file());
    }
    #[cfg(unix)]
    #[test]
    fn write_paths_reject_symlinked_settings_parent_before_mutation() {
        let root = tempfile::tempdir().unwrap();
        let foreign = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(foreign.path(), root.path().join(".focusa")).unwrap();
        assert!(validate_artifact_paths(root.path()).is_err());
        assert!(!foreign.path().join("settings.json").exists());
    }

    #[test]
    fn lock_conflict_is_distinct_from_io_failure_and_drop_releases() {
        let root = tempfile::tempdir().unwrap();
        let lock = BootstrapLock::acquire(root.path()).unwrap();
        assert!(
            matches!(BootstrapLock::acquire(root.path()), Err(e) if e.kind() == io::ErrorKind::WouldBlock)
        );
        drop(lock);
        assert!(BootstrapLock::acquire(root.path()).is_ok());
        assert!(
            matches!(BootstrapLock::acquire(&root.path().join("absent")), Err(e) if e.kind() == io::ErrorKind::NotFound)
        );
    }
    #[test]
    fn rolled_back_receipt_never_replays_ready() {
        let root = Path::new("/projects/example");
        let mut receipt = json!({"status":"ready","project_root":root,"idempotency_key":"same","request_digest":"original"});
        assert!(validate_apply_receipt(&receipt, root, "same", "original").is_ok());
        assert!(validate_apply_receipt(&receipt, root, "same", "changed").is_err());
        assert!(validate_apply_receipt(&receipt, root, "other", "original").is_err());
        receipt["status"] = json!("rolled_back");
        assert!(status_next_action("rolled_back").contains("new idempotency key"));
        assert!(status_next_action("rolling_back").contains("resume rollback"));
        assert!(validate_apply_receipt(&receipt, root, "same", "original").is_err());
        assert!(validate_apply_receipt(&receipt, root, "other", "original").is_ok());
        assert!(
            validate_apply_receipt(
                &receipt,
                Path::new("/projects/foreign"),
                "other",
                "original"
            )
            .is_err()
        );
    }
    #[test]
    fn corrupt_or_unreadable_receipt_is_not_not_started() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("receipt.json");
        assert!(read_receipt(&path).unwrap().is_none());
        fs::write(&path, "{").unwrap();
        assert!(read_receipt(&path).is_err());
        fs::write(&path, "null").unwrap();
        assert!(read_receipt(&path).is_err());
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(read_receipt(&path).is_err());
    }
    #[test]
    fn digest_distinguishes_changed_payload_but_not_confirmation() {
        let root = Path::new("/projects/example");
        let a = json!({"project_id":"same","confirm":false});
        let b = json!({"project_id":"same","confirm":true,"repair_action":"retry"});
        assert_eq!(request_digest(a.clone(), root), request_digest(b, root));
        assert_ne!(
            request_digest(a, root),
            request_digest(json!({"project_id":"other"}), root)
        );
    }
}
