//! Owner-bound project filesystem writes and typed failure translation.
use super::project_bootstrap_support::reject;
use axum::{Json, http::StatusCode};
use serde_json::Value;
use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};
use uuid::Uuid;

/// Cross-user writes require a separately approved per-user runner, never
/// ambient daemon/root privileges.
pub(super) fn require_owner_context(root: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let mut ancestor = root;
        while !ancestor.exists() {
            ancestor = ancestor
                .parent()
                .ok_or_else(|| "owner_context_unavailable: no existing ancestor".to_string())?;
        }
        let owner = fs::metadata(ancestor)
            .map_err(|e| format!("owner_context_unavailable: {e}"))?
            .uid();
        let current = nix::unistd::geteuid().as_raw();
        if owner != current {
            return Err(format!(
                "owner_runner_required: project ancestor owner uid {owner} differs from daemon uid {current}"
            ));
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = root;
        Err(
            "owner_context_unavailable: owner-equivalent runner not verified on this platform"
                .into(),
        )
    }
}

/// Check the existing parent of each declared write target without creating it.
/// Apply repeats this check under the transaction lock; post-check races still
/// return typed OS errors and never authorize a foreign-owner write.
pub(super) fn check_write_access(root: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use nix::unistd::{AccessFlags, access};
        for relative in [
            "",
            ".focusa/bootstrap/receipt.json",
            ".focusa/settings.json",
        ] {
            let target = root.join(relative);
            let mut existing = target.as_path();
            while !existing.exists() {
                existing = existing.parent().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "write target has no ancestor")
                })?;
            }
            let directory = if existing.is_dir() {
                existing
            } else {
                existing.parent().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "write target has no parent")
                })?
            };
            access(directory, AccessFlags::W_OK | AccessFlags::X_OK).map_err(io::Error::from)?;
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = root;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "owner-equivalent write preflight unavailable",
        ))
    }
}

pub(super) fn write_json_atomic(path: &Path, value: &Value) -> Result<(), String> {
    publish_json(path, value, true).map_err(|error| error.to_string())
}

/// Same-directory hard-link publication cannot replace a concurrently
/// created artifact; no artifact is attributed without a successful link.
pub(super) fn create_json_atomic(path: &Path, value: &Value) -> io::Result<()> {
    publish_json(path, value, false)
}

pub(super) fn artifact_write_rejection(stage: &str, error: io::Error) -> (StatusCode, Json<Value>) {
    let (status, code) = match error.kind() {
        io::ErrorKind::AlreadyExists => (StatusCode::CONFLICT, "bootstrap_artifact_already_exists"),
        io::ErrorKind::PermissionDenied => (StatusCode::FORBIDDEN, "bootstrap_permission_denied"),
        io::ErrorKind::ReadOnlyFilesystem => {
            (StatusCode::FORBIDDEN, "bootstrap_read_only_filesystem")
        }
        _ => {
            #[cfg(unix)]
            if error.raw_os_error() == Some(nix::errno::Errno::EDQUOT as i32) {
                return reject(
                    StatusCode::INSUFFICIENT_STORAGE,
                    "bootstrap_quota_exceeded",
                    format!("{stage}: {error}"),
                );
            }
            if error.kind() == io::ErrorKind::StorageFull {
                (StatusCode::INSUFFICIENT_STORAGE, "bootstrap_no_space")
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "bootstrap_artifact_io_failed",
                )
            }
        }
    };
    reject(status, code, format!("{stage}: {error}"))
}

fn publish_json(path: &Path, value: &Value, replace: bool) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing parent"))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".receipt-{}.tmp", Uuid::now_v7()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let result = (|| -> io::Result<()> {
        file.write_all(&bytes)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        if replace {
            fs::rename(&temporary, path)?;
        } else {
            fs::hard_link(&temporary, path)?;
        }
        Ok(())
    })();
    drop(file);
    if let Err(cleanup) = fs::remove_file(&temporary)
        && cleanup.kind() != io::ErrorKind::NotFound
    {
        return Err(io::Error::new(
            cleanup.kind(),
            format!(
                "{}; temporary cleanup: {cleanup}",
                result
                    .err()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "artifact published".into())
            ),
        ));
    }
    File::open(parent)?.sync_all()?;
    result
}
