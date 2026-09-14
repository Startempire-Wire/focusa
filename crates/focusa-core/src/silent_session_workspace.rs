//! Workspace strategy planning and isolated git worktree materialization.

use crate::silent_session::{
    IntegrationPolicy, SilentSessionId, WorkspaceBinding, WorkspaceStrategy,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use thiserror::Error;

pub const WORKSPACE_PLAN_SCHEMA: &str = "focusa.silent_session_workspace_plan.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspacePlanningRequest {
    pub session_id: SilentSessionId,
    pub project_root: PathBuf,
    pub project_slug: String,
    pub work_item_ref: Option<String>,
    pub requested_strategy: Option<WorkspaceStrategy>,
    pub background_mutation: bool,
    pub worktree_root: Option<PathBuf>,
    pub base_ref: Option<String>,
    pub lease_acquired: bool,
    pub competing_writer: bool,
    pub explicit_shared_approval_ref: Option<String>,
    pub path_intents: Vec<PathBuf>,
    pub existing_branch_refs: BTreeSet<String>,
    pub existing_workspace_paths: BTreeSet<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentSessionWorkspacePlan {
    pub schema: String,
    pub session_id: SilentSessionId,
    pub strategy: WorkspaceStrategy,
    pub source_root: PathBuf,
    pub workspace_root: PathBuf,
    pub workspace_id: String,
    pub branch_ref: Option<String>,
    pub base_ref: Option<String>,
    pub integration_policy: IntegrationPolicy,
    pub read_only: bool,
    pub explicit_shared_approval_ref: Option<String>,
    pub path_intents: Vec<PathBuf>,
    pub collision_suffix: Option<String>,
    pub warnings: Vec<String>,
}

impl WorkspacePlanningRequest {
    pub fn plan(self) -> Result<SilentSessionWorkspacePlan, WorkspacePlanError> {
        if !self.session_id.is_uuid_v7()
            || !self.project_root.is_absolute()
            || self.project_slug.trim().is_empty()
            || !safe_path_intents(&self.path_intents)
        {
            return Err(WorkspacePlanError::InvalidScope);
        }
        let strategy = self
            .requested_strategy
            .unwrap_or(if self.background_mutation {
                WorkspaceStrategy::IsolatedWorktree
            } else {
                WorkspaceStrategy::ExclusiveExisting
            });
        let explicit_shared_approval_ref = self
            .explicit_shared_approval_ref
            .filter(|value| !value.trim().is_empty());
        match strategy {
            WorkspaceStrategy::ExclusiveExisting => {
                if !self.lease_acquired || self.competing_writer {
                    return Err(WorkspacePlanError::ExclusiveWorkspaceUnavailable);
                }
            }
            WorkspaceStrategy::ExplicitShared => {
                if explicit_shared_approval_ref.is_none() || self.path_intents.is_empty() {
                    return Err(WorkspacePlanError::ExplicitSharedApprovalRequired);
                }
            }
            WorkspaceStrategy::ReadOnlyShared | WorkspaceStrategy::IsolatedWorktree => {}
        }

        let project_slug = sanitize_segment(&self.project_slug)?;
        let work_item_slug =
            sanitize_segment(self.work_item_ref.as_deref().unwrap_or("unbound-work-item"))?;
        let session_short = short_session_id(self.session_id);
        let (workspace_root, branch_ref, collision_suffix) =
            if strategy == WorkspaceStrategy::IsolatedWorktree {
                let configured_root = self
                    .worktree_root
                    .filter(|path| path.is_absolute())
                    .ok_or(WorkspacePlanError::WorktreeRootRequired)?;
                let root = subprocess_compatible_path(
                    fs::canonicalize(&configured_root).unwrap_or(configured_root),
                );
                let mut branch = format!("focusa/silent/{session_short}/{work_item_slug}");
                let mut path = root.join(&project_slug).join(&session_short);
                let collision = self.existing_branch_refs.contains(&branch)
                    || self.existing_workspace_paths.contains(&path)
                    || path.exists();
                let suffix = collision.then(|| collision_suffix(self.session_id, &work_item_slug));
                if let Some(suffix) = &suffix {
                    branch.push('-');
                    branch.push_str(suffix);
                    path = root
                        .join(&project_slug)
                        .join(format!("{session_short}-{suffix}"));
                }
                (path, Some(branch), suffix)
            } else {
                (self.project_root.clone(), None, None)
            };
        let warnings = if strategy == WorkspaceStrategy::ExplicitShared {
            vec!["explicit shared writer mode requires visible conflict monitoring".into()]
        } else {
            Vec::new()
        };
        Ok(SilentSessionWorkspacePlan {
            schema: WORKSPACE_PLAN_SCHEMA.into(),
            session_id: self.session_id,
            strategy: strategy.clone(),
            source_root: self.project_root,
            workspace_root,
            workspace_id: format!("workspace:{}", self.session_id),
            branch_ref,
            base_ref: self.base_ref,
            integration_policy: if strategy == WorkspaceStrategy::ReadOnlyShared {
                IntegrationPolicy::Manual
            } else {
                IntegrationPolicy::GovernedMerge
            },
            read_only: strategy == WorkspaceStrategy::ReadOnlyShared,
            explicit_shared_approval_ref,
            path_intents: self.path_intents,
            collision_suffix,
            warnings,
        })
    }
}

pub fn materialize_isolated_worktree(
    plan: &SilentSessionWorkspacePlan,
) -> Result<WorkspaceBinding, WorkspacePlanError> {
    if plan.schema != WORKSPACE_PLAN_SCHEMA
        || plan.strategy != WorkspaceStrategy::IsolatedWorktree
        || plan.branch_ref.as_deref().is_none_or(str::is_empty)
        || plan.base_ref.as_deref().is_none_or(str::is_empty)
    {
        return Err(WorkspacePlanError::InvalidPlan);
    }
    let source_root =
        subprocess_compatible_path(fs::canonicalize(&plan.source_root).map_err(io_error)?);
    if !source_root.join(".git").exists() {
        return Err(WorkspacePlanError::SourceIsNotGitRepository);
    }
    if plan.workspace_root.exists() {
        return Err(WorkspacePlanError::WorkspaceCollision);
    }
    let parent = plan
        .workspace_root
        .parent()
        .ok_or(WorkspacePlanError::UnsafeWorkspaceParent)?;
    let parent_existed = parent.exists();
    fs::create_dir_all(parent).map_err(io_error)?;
    #[cfg(unix)]
    if !parent_existed {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700)).map_err(io_error)?;
    }
    validate_private_directory(parent)?;

    let output = Command::new("git")
        .arg("-C")
        .arg(&source_root)
        .arg("worktree")
        .arg("add")
        .arg("-b")
        .arg(plan.branch_ref.as_deref().unwrap_or_default())
        .arg(&plan.workspace_root)
        .arg(plan.base_ref.as_deref().unwrap_or_default())
        .output()
        .map_err(io_error)?;
    if !output.status.success() {
        return Err(WorkspacePlanError::GitWorktreeFailed(bounded_stderr(
            &output.stderr,
        )));
    }
    let workspace_root =
        subprocess_compatible_path(fs::canonicalize(&plan.workspace_root).map_err(io_error)?);
    if workspace_root != plan.workspace_root
        || !workspace_root.starts_with(parent)
        || !workspace_root.join(".git").exists()
    {
        return Err(WorkspacePlanError::MaterializedWorkspaceMismatch);
    }
    Ok(WorkspaceBinding {
        workspace_id: plan.workspace_id.clone(),
        root: workspace_root,
        strategy: WorkspaceStrategy::IsolatedWorktree,
        branch_ref: plan.branch_ref.clone(),
    })
}

#[cfg(not(windows))]
fn subprocess_compatible_path(path: PathBuf) -> PathBuf {
    path
}

#[cfg(windows)]
fn subprocess_compatible_path(path: PathBuf) -> PathBuf {
    let rendered = path.to_string_lossy();
    if let Some(rest) = rendered.strip_prefix("\\\\?\\UNC\\") {
        PathBuf::from(format!("\\\\{rest}"))
    } else if let Some(rest) = rendered.strip_prefix("\\\\?\\") {
        PathBuf::from(rest)
    } else {
        path
    }
}

fn validate_private_directory(path: &Path) -> Result<(), WorkspacePlanError> {
    let metadata = fs::symlink_metadata(path).map_err(io_error)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(WorkspacePlanError::UnsafeWorkspaceParent);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.mode() & 0o022 != 0 {
            return Err(WorkspacePlanError::UnsafeWorkspaceParent);
        }
    }
    Ok(())
}

fn safe_path_intents(paths: &[PathBuf]) -> bool {
    paths.iter().all(|path| {
        !path.as_os_str().is_empty()
            && !path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
    })
}

fn sanitize_segment(value: &str) -> Result<String, WorkspacePlanError> {
    let mut output = String::new();
    let mut previous_dash = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            output.push(character);
            previous_dash = false;
        } else if !previous_dash && !output.is_empty() {
            output.push('-');
            previous_dash = true;
        }
        if output.len() >= 48 {
            break;
        }
    }
    while output.ends_with('-') {
        output.pop();
    }
    if output.is_empty() || output == "." || output == ".." {
        return Err(WorkspacePlanError::InvalidName);
    }
    Ok(output)
}

fn short_session_id(session_id: SilentSessionId) -> String {
    let compact = session_id.to_string().replace('-', "");
    format!("{}-{}", &compact[..8], &compact[compact.len() - 6..])
}

fn collision_suffix(session_id: SilentSessionId, work_item_slug: &str) -> String {
    let digest = Sha256::digest(format!("{session_id}:{work_item_slug}").as_bytes());
    format!("{digest:x}").chars().take(8).collect()
}

fn bounded_stderr(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .chars()
        .take(512)
        .collect::<String>()
}

fn io_error(error: std::io::Error) -> WorkspacePlanError {
    WorkspacePlanError::Io(error.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkspacePlanError {
    #[error("workspace planning scope is invalid")]
    InvalidScope,
    #[error("workspace name cannot be sanitized safely")]
    InvalidName,
    #[error("isolated worktree root must be absolute")]
    WorktreeRootRequired,
    #[error("exclusive existing workspace requires a lease and no competitor")]
    ExclusiveWorkspaceUnavailable,
    #[error("explicit shared mode requires approval and path intents")]
    ExplicitSharedApprovalRequired,
    #[error("isolated worktree plan is incomplete")]
    InvalidPlan,
    #[error("source root is not a git repository")]
    SourceIsNotGitRepository,
    #[error("workspace path or branch collides with existing state")]
    WorkspaceCollision,
    #[error("workspace parent is symlinked or shared-writable")]
    UnsafeWorkspaceParent,
    #[error("materialized workspace identity does not match the plan")]
    MaterializedWorkspaceMismatch,
    #[error("git worktree creation failed: {0}")]
    GitWorktreeFailed(String),
    #[error("workspace filesystem operation failed: {0}")]
    Io(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    fn root() -> PathBuf {
        std::env::temp_dir().join(format!("focusa-workspace-{}", uuid::Uuid::now_v7()))
    }

    fn request(source: PathBuf, worktree_root: PathBuf) -> WorkspacePlanningRequest {
        WorkspacePlanningRequest {
            session_id: SilentSessionId::new(),
            project_root: source,
            project_slug: "Focusa Main!!!".into(),
            work_item_ref: Some("focusa-a6yq6.6.2 / unsafe?".into()),
            requested_strategy: None,
            background_mutation: true,
            worktree_root: Some(worktree_root),
            base_ref: Some("HEAD".into()),
            lease_acquired: false,
            competing_writer: true,
            explicit_shared_approval_ref: None,
            path_intents: vec![PathBuf::from("crates/focusa-core")],
            existing_branch_refs: BTreeSet::new(),
            existing_workspace_paths: BTreeSet::new(),
        }
    }

    #[test]
    fn background_mutation_defaults_to_sanitized_collision_safe_isolation() {
        let source = crate::test_support::absolute_path("silent-workspace-project");
        let worktree_root = crate::test_support::absolute_path("silent-workspace-root");
        let first = request(source.clone(), worktree_root.clone())
            .plan()
            .unwrap();
        assert_eq!(first.strategy, WorkspaceStrategy::IsolatedWorktree);
        assert!(
            first
                .branch_ref
                .as_deref()
                .unwrap()
                .starts_with("focusa/silent/")
        );
        assert!(!first.branch_ref.as_deref().unwrap().contains(' '));
        assert!(first.workspace_root.starts_with(&worktree_root));

        let mut colliding = request(source, worktree_root);
        colliding.session_id = first.session_id;
        colliding
            .existing_branch_refs
            .insert(first.branch_ref.clone().unwrap());
        let second = colliding.plan().unwrap();
        assert!(second.collision_suffix.is_some());
        assert_ne!(second.branch_ref, first.branch_ref);
        assert_ne!(second.workspace_root, first.workspace_root);
    }

    #[test]
    fn existing_and_shared_strategies_enforce_lease_read_only_and_approval() {
        let source = crate::test_support::absolute_path("silent-workspace-project");
        let worktree_root = crate::test_support::absolute_path("silent-workspace-root");
        let mut exclusive = request(source.clone(), worktree_root.clone());
        exclusive.background_mutation = false;
        exclusive.requested_strategy = Some(WorkspaceStrategy::ExclusiveExisting);
        assert_eq!(
            exclusive.clone().plan(),
            Err(WorkspacePlanError::ExclusiveWorkspaceUnavailable)
        );
        exclusive.lease_acquired = true;
        exclusive.competing_writer = false;
        assert_eq!(
            exclusive.plan().unwrap().strategy,
            WorkspaceStrategy::ExclusiveExisting
        );

        let mut read_only = request(source.clone(), worktree_root.clone());
        read_only.requested_strategy = Some(WorkspaceStrategy::ReadOnlyShared);
        let plan = read_only.plan().unwrap();
        assert!(plan.read_only && plan.branch_ref.is_none());

        let mut shared = request(source, worktree_root);
        shared.requested_strategy = Some(WorkspaceStrategy::ExplicitShared);
        assert_eq!(
            shared.clone().plan(),
            Err(WorkspacePlanError::ExplicitSharedApprovalRequired)
        );
        shared.explicit_shared_approval_ref = Some("approval:shared".into());
        let plan = shared.plan().unwrap();
        assert_eq!(plan.strategy, WorkspaceStrategy::ExplicitShared);
        assert_eq!(plan.warnings.len(), 1);
    }

    #[test]
    fn two_isolated_sessions_commit_without_touching_dirty_primary_workspace() {
        let root = root();
        let source = root.join("source");
        let worktree_root = root.join("worktrees");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&worktree_root).unwrap();
        #[cfg(unix)]
        {
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
            fs::set_permissions(&worktree_root, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let git = |cwd: &Path, args: &[&str]| {
            let output = Command::new("git")
                .arg("-C")
                .arg(cwd)
                .args(args)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "git command failed: {args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8_lossy(&output.stdout).trim().to_owned()
        };
        git(&source, &["init"]);
        git(
            &source,
            &["config", "user.email", "focusa-test@example.invalid"],
        );
        git(&source, &["config", "user.name", "Focusa Test"]);
        fs::write(source.join("README.md"), "base\n").unwrap();
        git(&source, &["add", "README.md"]);
        git(&source, &["commit", "-m", "fixture"]);
        let primary_head = git(&source, &["rev-parse", "HEAD"]);
        fs::write(source.join("README.md"), "dirty primary\n").unwrap();

        let first = request(source.clone(), worktree_root.clone())
            .plan()
            .unwrap();
        let mut second_request = request(source.clone(), worktree_root);
        second_request.work_item_ref = Some("focusa-a6yq6.6.5-second".into());
        let second = second_request.plan().unwrap();
        let first_binding = materialize_isolated_worktree(&first).unwrap();
        let second_binding = materialize_isolated_worktree(&second).unwrap();
        for (binding, value) in [(&first_binding, "first\n"), (&second_binding, "second\n")] {
            fs::write(binding.root.join("session.txt"), value).unwrap();
            git(&binding.root, &["add", "session.txt"]);
            git(&binding.root, &["commit", "-m", value.trim()]);
        }

        assert_eq!(git(&source, &["rev-parse", "HEAD"]), primary_head);
        assert_eq!(
            fs::read_to_string(source.join("README.md")).unwrap(),
            "dirty primary\n"
        );
        let primary_status = git(&source, &["status", "--porcelain"]);
        assert!(primary_status.contains("README.md"));
        assert!(!primary_status.contains("session.txt"));
        assert_ne!(
            git(&first_binding.root, &["rev-parse", "HEAD"]),
            git(&second_binding.root, &["rev-parse", "HEAD"])
        );

        for binding in [&first_binding, &second_binding] {
            git(
                &source,
                &[
                    "worktree",
                    "remove",
                    "--force",
                    binding.root.to_str().unwrap(),
                ],
            );
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn real_git_worktree_is_created_with_verified_session_binding() {
        let root = root();
        let source = root.join("source");
        let worktree_root = root.join("worktrees");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&worktree_root).unwrap();
        #[cfg(unix)]
        {
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
            fs::set_permissions(&worktree_root, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let run = |args: &[&str]| {
            let status = Command::new("git")
                .arg("-C")
                .arg(&source)
                .args(args)
                .status()
                .unwrap();
            assert!(status.success(), "git command failed: {args:?}");
        };
        run(&["init"]);
        run(&["config", "user.email", "focusa-test@example.invalid"]);
        run(&["config", "user.name", "Focusa Test"]);
        let mut readme = fs::File::create(source.join("README.md")).unwrap();
        writeln!(readme, "worktree proof").unwrap();
        run(&["add", "README.md"]);
        run(&["commit", "-m", "fixture"]);

        let plan = request(source.clone(), worktree_root).plan().unwrap();
        let binding = materialize_isolated_worktree(&plan).unwrap();
        assert_eq!(binding.workspace_id, plan.workspace_id);
        assert_eq!(binding.root, plan.workspace_root);
        assert_eq!(binding.branch_ref, plan.branch_ref);
        assert_eq!(binding.strategy, WorkspaceStrategy::IsolatedWorktree);

        let status = Command::new("git")
            .arg("-C")
            .arg(&source)
            .args(["worktree", "remove", "--force"])
            .arg(&binding.root)
            .status()
            .unwrap();
        assert!(status.success());
        let _ = fs::remove_dir_all(root);
    }
}
