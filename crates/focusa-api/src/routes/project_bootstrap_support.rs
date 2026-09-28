//! Project bootstrap request, inspection, receipt, and provider helpers.

use super::project_bootstrap_safety as safety;
use axum::{Json, http::StatusCode};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub(super) struct ProjectBootstrapRequest {
    pub project_root: String,
    pub project_id: String,
    pub canonical_name: String,
    pub continuity_id: String,
    pub idempotency_key: String,
    pub discipline_profile: Option<String>,
    pub initialize_git: Option<bool>,
    pub initialize_task_provider: Option<bool>,
    pub task_provider: Option<String>,
    pub hlt: Option<String>,
    pub hlt_confirmed: Option<bool>,
    pub desired_end_state: Option<String>,
    pub current_state: Option<String>,
    pub specification_ref: Option<String>,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    pub confirm: Option<bool>,
    pub repair_action: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(super) struct ProjectBootstrapStatusQuery {
    pub project_root: String,
}

pub(super) fn reject(
    status: StatusCode,
    code: &str,
    message: impl Into<String>,
) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({
            "status": "blocked",
            "failure_class": code,
            "message": message.into(),
            "next_action": "inspect the bootstrap preview and exact recovery before retrying",
        })),
    )
}

pub(super) fn validate_root(
    raw: &str,
    allow_missing: bool,
) -> Result<PathBuf, (StatusCode, Json<Value>)> {
    let path = PathBuf::from(raw);
    if !path.is_absolute()
        || !focusa_core::scope_safety::classify_project_root(raw).is_safe()
        || path
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err(reject(
            StatusCode::BAD_REQUEST,
            "unsafe_project_root",
            "project_root must be an explicit safe absolute child path",
        ));
    }
    let resolved = if path.try_exists().map_err(|error| {
        reject(
            StatusCode::BAD_REQUEST,
            "project_root_unavailable",
            error.to_string(),
        )
    })? {
        fs::canonicalize(path).map_err(|error| {
            reject(
                StatusCode::BAD_REQUEST,
                "project_root_unavailable",
                format!("cannot resolve project_root: {error}"),
            )
        })
    } else if allow_missing {
        // Bind a not-yet-created leaf to its real existing ancestor rather than
        // persisting a symlink alias as a second project identity.
        let mut ancestor = path.as_path();
        let mut missing = Vec::new();
        while !ancestor.try_exists().map_err(|error| {
            reject(
                StatusCode::BAD_REQUEST,
                "project_root_unavailable",
                error.to_string(),
            )
        })? {
            missing.push(
                ancestor
                    .file_name()
                    .ok_or_else(|| {
                        reject(
                            StatusCode::BAD_REQUEST,
                            "unsafe_project_root",
                            "missing safe parent",
                        )
                    })?
                    .to_os_string(),
            );
            ancestor = ancestor.parent().ok_or_else(|| {
                reject(
                    StatusCode::BAD_REQUEST,
                    "unsafe_project_root",
                    "missing safe parent",
                )
            })?;
        }
        let mut resolved = fs::canonicalize(ancestor).map_err(|error| {
            reject(
                StatusCode::BAD_REQUEST,
                "project_root_unavailable",
                error.to_string(),
            )
        })?;
        if !resolved.is_dir() {
            return Err(reject(
                StatusCode::BAD_REQUEST,
                "project_root_not_directory",
                "project root ancestor is not a directory",
            ));
        }
        for component in missing.into_iter().rev() {
            resolved.push(component);
        }
        Ok(resolved)
    } else {
        Err(reject(
            StatusCode::NOT_FOUND,
            "project_root_missing",
            "project root does not exist; preview/apply can create it",
        ))
    }?;
    if resolved.is_file() {
        return Err(reject(
            StatusCode::BAD_REQUEST,
            "project_root_not_directory",
            "project root is a file",
        ));
    }
    let rendered = resolved.to_string_lossy();
    let safety = focusa_core::scope_safety::classify_project_root(&rendered);
    if resolved.parent().is_none() || !safety.is_safe() {
        return Err(reject(
            StatusCode::BAD_REQUEST,
            "unsafe_project_root",
            format!("{}; {}", safety.human_kind(), safety.next_step_hint()),
        ));
    }
    Ok(resolved)
}

pub(super) fn receipt_path(root: &Path) -> PathBuf {
    root.join(".focusa").join("bootstrap").join("receipt.json")
}

pub(super) use super::project_bootstrap_fs::{
    artifact_write_rejection, check_write_access, create_json_atomic, require_owner_context,
    write_json_atomic,
};

pub(super) fn read_json(path: &Path) -> Option<Value> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

pub(super) fn read_receipt(root: &Path) -> Result<Option<Value>, (StatusCode, Json<Value>)> {
    let receipt = safety::read_receipt(&receipt_path(root))
        .map_err(|error| reject(StatusCode::CONFLICT, "bootstrap_receipt_unreadable", error))?;
    if receipt
        .as_ref()
        .is_some_and(|value| value["project_root"] != json!(root))
    {
        return Err(reject(
            StatusCode::CONFLICT,
            "receipt_scope_mismatch",
            "receipt does not own this project root",
        ));
    }
    Ok(receipt)
}

pub(super) fn stable_receipt_id(root: &Path, key: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(root.to_string_lossy().as_bytes());
    hash.update([0]);
    hash.update(key.as_bytes());
    format!("bootstrap-{}", &hex::encode(hash.finalize())[..20])
}

pub(super) fn executable(names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        Command::new("sh")
            .args(["-c", &format!("command -v {name}")])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

pub(super) fn inspection(root: &Path, req: &ProjectBootstrapRequest) -> Value {
    let standard = req
        .discipline_profile
        .as_deref()
        .unwrap_or("standard_software_project")
        == "standard_software_project";
    let wants_git = req.initialize_git.unwrap_or(standard);
    let wants_tasks = req.initialize_task_provider.unwrap_or(standard);
    let provider = req.task_provider.as_deref().unwrap_or("beads");
    let root_exists = root.exists();
    let marker_exists = root.join(".focusa-project.json").is_file();
    let git_exists = root.join(".git").is_dir();
    let docs_exists = root.join("docs").is_dir();
    let beads_exists = root.join(".beads").is_dir();
    let provider_binary = executable(&["bd", "br"]);
    let mut blockers = Vec::new();
    if wants_tasks && provider == "beads" && provider_binary.is_none() {
        blockers.push("task_provider_unavailable: install or select an approved provider");
    }
    let planned_changes = [
        (!root_exists).then_some("create project folder"),
        (!marker_exists).then_some("create Focusa project marker and settings"),
        (!docs_exists).then_some("create docs folder"),
        (wants_git && !git_exists).then_some("initialize local Git repository without a remote"),
        (wants_tasks && provider == "beads" && !beads_exists)
            .then_some("initialize project-local Beads task provider"),
        Some("stage and commit Project Genesis when authority is complete"),
    ]
    .into_iter()
    .flatten()
    .map(str::to_string)
    .collect::<Vec<_>>();
    json!({
        "schema": "focusa.project_bootstrap_preview.v1",
        "status": if blockers.is_empty() { "preview_ready" } else { "blocked" },
        "project_root": root,
        "discipline_profile": req.discipline_profile.as_deref().unwrap_or("standard_software_project"),
        "observed": {
            "root": root_exists,
            "marker": marker_exists,
            "local_git": git_exists,
            "docs": docs_exists,
            "beads": beads_exists,
            "task_provider_binary": provider_binary,
        },
        "planned_changes": planned_changes,
        "preserved_choices": ["programming language", "framework", "remote", "deployment target", "domain"],
        "blockers": blockers,
        "rollback": "remove only objects listed as created_by_this_transaction; never remove adopted project state",
        "verification": ["marker guard", "local git has no remotes", "task provider health", "Genesis readiness", "first Workpoint"],
        "next_action": if wants_tasks && provider != "beads" { "supply an approved provider adapter" } else { "apply with confirm=true" },
    })
}

#[cfg(test)]
mod safety_tests {
    use super::*;

    #[test]
    fn artifact_errors_preserve_distinct_http_failure_classes() {
        for (kind, status, code) in [
            (
                std::io::ErrorKind::AlreadyExists,
                StatusCode::CONFLICT,
                "bootstrap_artifact_already_exists",
            ),
            (
                std::io::ErrorKind::PermissionDenied,
                StatusCode::FORBIDDEN,
                "bootstrap_permission_denied",
            ),
            (
                std::io::ErrorKind::ReadOnlyFilesystem,
                StatusCode::FORBIDDEN,
                "bootstrap_read_only_filesystem",
            ),
            (
                std::io::ErrorKind::StorageFull,
                StatusCode::INSUFFICIENT_STORAGE,
                "bootstrap_no_space",
            ),
        ] {
            let (actual_status, Json(body)) =
                artifact_write_rejection("marker_create", std::io::Error::from(kind));
            assert_eq!(actual_status, status);
            assert_eq!(body["failure_class"], code);
        }
        #[cfg(unix)]
        {
            let (status, Json(body)) = artifact_write_rejection(
                "settings_create",
                std::io::Error::from_raw_os_error(nix::errno::Errno::EDQUOT as i32),
            );
            assert_eq!(status, StatusCode::INSUFFICIENT_STORAGE);
            assert_eq!(body["failure_class"], "bootstrap_quota_exceeded");
        }
    }

    #[cfg(unix)]
    #[test]
    fn preview_write_access_reports_the_same_read_only_parent_apply_would_hit() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let settings_parent = root.path().join(".focusa");
        fs::create_dir(&settings_parent).unwrap();
        fs::set_permissions(&settings_parent, fs::Permissions::from_mode(0o500)).unwrap();
        let checked = check_write_access(root.path());
        fs::set_permissions(&settings_parent, fs::Permissions::from_mode(0o700)).unwrap();
        if !nix::unistd::geteuid().is_root() {
            let (status, Json(body)) =
                artifact_write_rejection("preview_write_access", checked.unwrap_err());
            assert_eq!(status, StatusCode::FORBIDDEN);
            assert_eq!(body["failure_class"], "bootstrap_permission_denied");
        }
    }

    #[cfg(unix)]
    #[test]
    fn owner_context_fails_closed_for_foreign_ancestor_before_creation() {
        use std::os::unix::fs::MetadataExt;
        let own = tempfile::tempdir().unwrap();
        require_owner_context(&own.path().join("future-project")).unwrap();
        let foreign = [Path::new("/tmp"), Path::new("/home")]
            .into_iter()
            .find(|path| {
                path.is_dir()
                    && fs::metadata(path).unwrap().uid() != nix::unistd::geteuid().as_raw()
            });
        if let Some(parent) = foreign {
            let path = parent.join(format!(
                "focusa-foreign-owner-test-{}",
                uuid::Uuid::now_v7()
            ));
            assert!(
                require_owner_context(&path)
                    .unwrap_err()
                    .starts_with("owner_runner_required:")
            );
            assert!(!path.exists());
        }
    }

    #[test]
    fn new_json_artifact_publication_never_clobbers_existing_content() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("settings.json");
        create_json_atomic(&path, &json!({"owner":"first"})).unwrap();
        let before = fs::read(&path).unwrap();
        assert!(create_json_atomic(&path, &json!({"owner":"second"})).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
        write_json_atomic(&path, &json!({"owner":"updated receipt"})).unwrap();
        assert_eq!(read_json(&path).unwrap()["owner"], "updated receipt");
    }

    #[test]
    fn concurrent_json_creators_publish_exactly_one_winner() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("settings.json");
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
        let handles: Vec<_> = (0..2)
            .map(|id| {
                let path = path.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    (id, create_json_atomic(&path, &json!({"creator":id})))
                })
            })
            .collect();
        barrier.wait();
        let results: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        let winners: Vec<_> = results
            .iter()
            .filter(|(_, result)| result.is_ok())
            .collect();
        assert_eq!(winners.len(), 1);
        assert_eq!(read_json(&path).unwrap()["creator"], json!(winners[0].0));
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn new_json_artifact_publication_preserves_destination_symlink() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("user-data");
        let path = root.path().join("settings.json");
        fs::write(&target, b"preserve user data").unwrap();
        std::os::unix::fs::symlink(&target, &path).unwrap();
        assert!(create_json_atomic(&path, &json!({"replacement":true})).is_err());
        assert!(
            fs::symlink_metadata(&path)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read(&target).unwrap(), b"preserve user data");
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
    }

    #[test]
    fn bootstrap_uses_shared_root_safety_for_existing_and_missing_paths() {
        for path in [
            "/",
            "/root",
            "/home",
            "/home/example-owner",
            "/usr/local/bin",
            "/tmp/../",
        ] {
            assert!(
                validate_root(path, true).is_err(),
                "accepted unsafe root {path}"
            );
        }
        let root = tempfile::tempdir().unwrap();
        let canonical = fs::canonicalize(root.path()).unwrap();
        assert_eq!(
            validate_root(root.path().to_str().unwrap(), false).unwrap(),
            canonical
        );
        let child = root.path().join("new-project");
        assert_eq!(
            validate_root(child.to_str().unwrap(), true).unwrap(),
            canonical.join("new-project")
        );
        assert!(!child.exists(), "root inspection must be read-only");
    }

    #[cfg(unix)]
    #[test]
    fn aliases_cannot_bypass_shared_unsafe_root_classification() {
        let root = tempfile::tempdir().unwrap();
        let alias = root.path().join("alias");
        std::os::unix::fs::symlink("/", &alias).unwrap();
        assert!(validate_root(alias.to_str().unwrap(), false).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn missing_leaf_uses_canonical_parent_without_creating_it() {
        let parent = tempfile::tempdir().unwrap();
        let links = tempfile::tempdir().unwrap();
        let alias = links.path().join("alias");
        std::os::unix::fs::symlink(parent.path(), &alias).unwrap();
        assert_eq!(
            validate_root(alias.join("project").to_str().unwrap(), true).unwrap(),
            parent.path().join("project")
        );
        assert!(!parent.path().join("project").exists());
        let file = parent.path().join("file");
        fs::write(&file, "not a directory").unwrap();
        assert!(validate_root(file.to_str().unwrap(), false).is_err());
    }
}
