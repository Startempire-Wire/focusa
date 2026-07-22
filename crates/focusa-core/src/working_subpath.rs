use std::{
    path::{Path, PathBuf},
    process::Command,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceKind {
    Primary,
    FeatureBranch,
    LinkedWorktree,
    Detached,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkingSubpathLifecycle {
    Active,
    Merging,
    Merged,
    Stale,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkingSubpath {
    pub working_subpath_id: String,
    pub workspace_root: String,
    pub workspace_kind: WorkspaceKind,
    pub git_common_dir_id: String,
    pub branch_or_ref: String,
    pub head_commit: String,
    pub beads_root: Option<String>,
    pub beads_prefix: Option<String>,
    pub lifecycle: WorkingSubpathLifecycle,
    pub created_from_ref: Option<String>,
    pub merge_target_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitWorkingContext {
    pub canonical_parent_root: String,
    pub active_worktree_root: String,
    pub git_common_dir: String,
    pub git_dir: String,
    pub working_subpath: WorkingSubpath,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WorkingSubpathError {
    #[error("working path does not exist: {0}")]
    MissingPath(String),
    #[error("git context command failed: {0}")]
    GitCommand(String),
    #[error("git common directory has no canonical parent: {0}")]
    MissingCanonicalParent(String),
}

pub fn resolve_git_working_context(
    start: &Path,
) -> Result<Option<GitWorkingContext>, WorkingSubpathError> {
    if !start.exists() {
        return Err(WorkingSubpathError::MissingPath(
            start.to_string_lossy().into_owned(),
        ));
    }
    let start = if start.is_file() {
        start.parent().unwrap_or(start)
    } else {
        start
    };
    let Some(workspace_root) = git_optional(start, &["rev-parse", "--show-toplevel"])? else {
        return Ok(None);
    };
    let workspace_root = canonical(Path::new(&workspace_root));
    let common_raw = git_required(
        start,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?;
    let git_dir_raw = git_required(start, &["rev-parse", "--path-format=absolute", "--git-dir"])?;
    let common_dir = canonical(&resolve_git_path(start, &common_raw));
    let git_dir = canonical(&resolve_git_path(start, &git_dir_raw));
    let canonical_parent = canonical_parent_for_common_dir(&common_dir)?;
    let branch = git_required(start, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let head = git_required(start, &["rev-parse", "HEAD"])?;
    let common_id = stable_id("git-common", &[path_text(&common_dir)]);
    let workspace_kind = classify_workspace(&workspace_root, &canonical_parent, &branch);
    let subpath_id = if workspace_kind == WorkspaceKind::Primary {
        "primary".into()
    } else {
        stable_id(
            "working-subpath",
            &[
                common_id.clone(),
                path_text(&workspace_root),
                branch.clone(),
            ],
        )
    };
    let beads_root_path = canonical_parent.join(".beads");
    let beads_root = beads_root_path
        .is_dir()
        .then(|| path_text(&beads_root_path));
    let parent_prefix = beads_root
        .as_deref()
        .and_then(|_| read_parent_beads_prefix(&beads_root_path));
    let beads_prefix = parent_prefix.map(|prefix| {
        if subpath_id == "primary" {
            prefix
        } else {
            format!("{prefix}-wt-{}", short_id(&subpath_id))
        }
    });
    Ok(Some(GitWorkingContext {
        canonical_parent_root: path_text(&canonical_parent),
        active_worktree_root: path_text(&workspace_root),
        git_common_dir: path_text(&common_dir),
        git_dir: path_text(&git_dir),
        working_subpath: WorkingSubpath {
            working_subpath_id: subpath_id,
            workspace_root: path_text(&workspace_root),
            workspace_kind,
            git_common_dir_id: common_id,
            branch_or_ref: branch,
            head_commit: head,
            beads_root,
            beads_prefix,
            lifecycle: WorkingSubpathLifecycle::Active,
            created_from_ref: None,
            merge_target_ref: None,
        },
    }))
}

fn classify_workspace(workspace: &Path, parent: &Path, branch: &str) -> WorkspaceKind {
    if branch == "HEAD" {
        WorkspaceKind::Detached
    } else if workspace != parent {
        WorkspaceKind::LinkedWorktree
    } else if matches!(branch, "main" | "master") {
        WorkspaceKind::Primary
    } else {
        WorkspaceKind::FeatureBranch
    }
}

fn canonical_parent_for_common_dir(common_dir: &Path) -> Result<PathBuf, WorkingSubpathError> {
    if common_dir.file_name().and_then(|value| value.to_str()) == Some(".git") {
        return common_dir
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| WorkingSubpathError::MissingCanonicalParent(path_text(common_dir)));
    }
    Err(WorkingSubpathError::MissingCanonicalParent(path_text(
        common_dir,
    )))
}

fn resolve_git_path(start: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        start.join(path)
    }
}

fn git_optional(start: &Path, args: &[&str]) -> Result<Option<String>, WorkingSubpathError> {
    let output = Command::new("git").arg("-C").arg(start).args(args).output();
    let Ok(output) = output else {
        return Ok(None);
    };
    if !output.status.success() {
        return Ok(None);
    }
    Ok(nonempty_stdout(output.stdout))
}

fn git_required(start: &Path, args: &[&str]) -> Result<String, WorkingSubpathError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(start)
        .args(args)
        .output()
        .map_err(|error| WorkingSubpathError::GitCommand(error.to_string()))?;
    if !output.status.success() {
        return Err(WorkingSubpathError::GitCommand(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    nonempty_stdout(output.stdout).ok_or_else(|| {
        WorkingSubpathError::GitCommand(format!("empty output for {}", args.join(" ")))
    })
}

fn nonempty_stdout(bytes: Vec<u8>) -> Option<String> {
    let value = String::from_utf8_lossy(&bytes).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn path_text(path: &Path) -> String {
    path.to_string_lossy().trim_end_matches('/').to_string()
}

fn stable_id(namespace: &str, parts: &[String]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(namespace.as_bytes());
    for part in parts {
        hasher.update([0]);
        hasher.update(part.as_bytes());
    }
    format!("{namespace}:{}", hex::encode(hasher.finalize()))
}

fn short_id(id: &str) -> &str {
    id.rsplit(':').next().unwrap_or(id).get(..8).unwrap_or(id)
}

fn read_parent_beads_prefix(beads_root: &Path) -> Option<String> {
    let issues = std::fs::read_to_string(beads_root.join("issues.jsonl")).ok()?;
    for line in issues.lines() {
        let id = serde_json::from_str::<serde_json::Value>(line)
            .ok()?
            .get("id")?
            .as_str()?
            .to_string();
        if let Some((prefix, _)) = id.split_once('-') {
            return Some(prefix.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_classification_separates_parent_folder_from_branch_context() {
        let parent = Path::new("/repo/focusa");
        assert_eq!(
            classify_workspace(parent, parent, "main"),
            WorkspaceKind::Primary
        );
        assert_eq!(
            classify_workspace(parent, parent, "feature/worktree"),
            WorkspaceKind::FeatureBranch
        );
        assert_eq!(
            classify_workspace(Path::new("/repo/focusa-wt"), parent, "feature/worktree"),
            WorkspaceKind::LinkedWorktree
        );
        assert_eq!(
            classify_workspace(parent, parent, "HEAD"),
            WorkspaceKind::Detached
        );
    }

    #[test]
    fn working_context_ids_are_deterministic_and_workspace_specific() {
        let common = "git-common:abc".to_string();
        let first = stable_id(
            "working-subpath",
            &[common.clone(), "/repo/wt-a".into(), "feature/a".into()],
        );
        let repeated = stable_id(
            "working-subpath",
            &[common.clone(), "/repo/wt-a".into(), "feature/a".into()],
        );
        let second = stable_id(
            "working-subpath",
            &[common, "/repo/wt-b".into(), "feature/b".into()],
        );
        assert_eq!(first, repeated);
        assert_ne!(first, second);
    }
}
