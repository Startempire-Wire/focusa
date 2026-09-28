//! Durable reservation before creating a project root. The project-local
//! receipt cannot exist until that root exists; this sibling record closes
//! the otherwise invisible mkdir-to-first-receipt crash window.
use super::project_bootstrap_fs::{
    artifact_write_rejection, create_json_atomic, write_json_atomic,
};
use super::project_bootstrap_support::{read_receipt, reject};
use axum::{Json, http::StatusCode};
use chrono::Utc;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

type Rejection = (StatusCode, Json<Value>);

fn path(root: &Path) -> Result<PathBuf, Rejection> {
    let parent = root.parent().ok_or_else(|| {
        reject(
            StatusCode::BAD_REQUEST,
            "bootstrap_parent_unavailable",
            "project root has no parent",
        )
    })?;
    if !parent.is_dir() {
        return Err(reject(
            StatusCode::CONFLICT,
            "bootstrap_parent_unavailable",
            "project parent must exist before a root can be created",
        ));
    }
    let digest = Sha256::digest(root.to_string_lossy().as_bytes());
    Ok(parent.join(format!(".focusa-bootstrap-{:x}.pending.json", digest)))
}

pub(super) fn parent_ready(root: &Path) -> Result<(), Rejection> {
    path(root).map(|_| ())
}

pub(super) fn read(root: &Path) -> Result<Option<Value>, Rejection> {
    let file = path(root)?;
    let meta = match fs::symlink_metadata(&file) {
        Ok(meta) => meta,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(artifact_write_rejection("pre_root_read", error)),
    };
    if !meta.file_type().is_file() || meta.len() > 16384 {
        return Err(reject(
            StatusCode::CONFLICT,
            "bootstrap_pre_root_unreadable",
            "pre-root journal is not a bounded regular file",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta.uid() != nix::unistd::geteuid().as_raw() || meta.mode() & 0o077 != 0 {
            return Err(reject(
                StatusCode::CONFLICT,
                "bootstrap_pre_root_unreadable",
                "pre-root journal owner or mode differs",
            ));
        }
    }
    // Recheck the inode after opening so an exchanged path cannot be followed.
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(nix::libc::O_NOFOLLOW);
    }
    let opened = options
        .open(&file)
        .map_err(|error| artifact_write_rejection("pre_root_read", error))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let actual = opened
            .metadata()
            .map_err(|error| artifact_write_rejection("pre_root_read", error))?;
        if (actual.dev(), actual.ino()) != (meta.dev(), meta.ino()) {
            return Err(reject(
                StatusCode::CONFLICT,
                "bootstrap_pre_root_unreadable",
                "pre-root journal identity changed",
            ));
        }
    }
    use std::io::Read;
    let mut bytes = Vec::new();
    opened
        .take(16385)
        .read_to_end(&mut bytes)
        .map_err(|error| artifact_write_rejection("pre_root_read", error))?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| {
        reject(
            StatusCode::CONFLICT,
            "bootstrap_pre_root_unreadable",
            "pre-root journal is malformed",
        )
    })?;
    if bytes.len() > 16384
        || value["schema"] != "focusa.project_bootstrap_pre_root.v1"
        || value["project_root"] != json!(root)
        || !matches!(value["status"].as_str(), Some("reserved" | "rolled_back"))
        || value["idempotency_key"].as_str().is_none()
        || value["request_digest"].as_str().is_none()
    {
        return Err(reject(
            StatusCode::CONFLICT,
            "bootstrap_pre_root_unreadable",
            "pre-root journal does not bind this project",
        ));
    }
    Ok(Some(value))
}

pub(super) fn reserve(root: &Path, idempotency_key: &str, digest: &str) -> Result<(), Rejection> {
    parent_ready(root)?;
    if read(root)?.is_some() {
        return Err(reject(
            StatusCode::CONFLICT,
            "bootstrap_pre_root_interrupted",
            "prior pre-root reservation requires explicit recovery",
        ));
    }
    let record = json!({
        "schema":"focusa.project_bootstrap_pre_root.v1", "status":"reserved",
        "project_root":root, "idempotency_key":idempotency_key,
        "request_digest":digest, "reserved_at":Utc::now().to_rfc3339(),
    });
    create_json_atomic(&path(root)?, &record)
        .map_err(|error| artifact_write_rejection("pre_root_reserve", error))
}

pub(super) fn settle_after_receipt(root: &Path, digest: &str) -> Result<(), Rejection> {
    let Some(record) = read(root)? else {
        return Ok(());
    };
    let Some(receipt) = read_receipt(root)? else {
        return Err(reject(
            StatusCode::CONFLICT,
            "bootstrap_pre_root_interrupted",
            "receipt must be durable before reservation removal",
        ));
    };
    if receipt["request_digest"] != digest || record["request_digest"] != digest {
        return Err(reject(
            StatusCode::CONFLICT,
            "bootstrap_pre_root_interrupted",
            "receipt and reservation do not match",
        ));
    }
    fs::remove_file(path(root)?)
        .map_err(|error| artifact_write_rejection("pre_root_settle", error))?;
    fs::File::open(root.parent().unwrap())
        .and_then(|dir| dir.sync_all())
        .map_err(|error| artifact_write_rejection("pre_root_settle", error))
}

pub(super) fn rollback_without_root(
    root: &Path,
    idempotency_key: &str,
) -> Result<Option<Value>, Rejection> {
    let Some(mut record) = read(root)? else {
        return Ok(None);
    };
    if root.exists() {
        return Err(reject(
            StatusCode::CONFLICT,
            "bootstrap_root_creation_uncertain",
            "root exists without receipt; ownership cannot be proven for automatic removal",
        ));
    }
    if record["idempotency_key"] != idempotency_key {
        return Err(reject(
            StatusCode::CONFLICT,
            "receipt_mismatch",
            "idempotency key does not own pre-root reservation",
        ));
    }
    if record["status"] == "rolled_back" {
        return Ok(Some(record));
    }
    record["status"] = json!("rolled_back");
    record["rolled_back_at"] = json!(Utc::now().to_rfc3339());
    write_json_atomic(&path(root)?, &record).map_err(|error| {
        reject(
            StatusCode::INTERNAL_SERVER_ERROR,
            "bootstrap_pre_root_rollback_failed",
            error,
        )
    })?;
    Ok(Some(record))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reservation_precedes_mkdir_and_rollback_never_removes_an_unproven_root() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("new-project");
        reserve(&root, "attempt-1", "digest-1").unwrap();
        assert!(!root.exists());
        assert_eq!(read(&root).unwrap().unwrap()["status"], "reserved");
        assert!(reserve(&root, "attempt-2", "digest-2").is_err());
        fs::create_dir(&root).unwrap();
        assert!(rollback_without_root(&root, "attempt-1").is_err());
        assert!(root.is_dir());
        fs::remove_dir(&root).unwrap();
        let rolled_back = rollback_without_root(&root, "attempt-1").unwrap().unwrap();
        assert_eq!(rolled_back["status"], "rolled_back");
        assert_eq!(
            rollback_without_root(&root, "attempt-1").unwrap().unwrap(),
            rolled_back
        );
    }
}
