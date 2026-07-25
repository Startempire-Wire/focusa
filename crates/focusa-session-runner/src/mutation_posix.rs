//! Race-resistant project mutation primitives for the verified OS owner.
//!
//! Paths are resolved from an open workspace directory descriptor. Every
//! existing parent is opened with `O_NOFOLLOW`, checked for owner and mode, and
//! retained while a same-directory temporary file is atomically renamed. This
//! prevents path traversal and symlink swaps from redirecting runner-owned
//! writes outside the verified workspace.

use crate::identity::{IdentityError, VerifiedExecutionContext};
use chrono::Utc;
use focusa_core::silent_session_bootstrap::{
    AgentBootstrapBarrierError, VerifiedProjectMutationGrant,
};
use nix::errno::Errno;
use nix::fcntl::{AtFlags, OFlag, open, openat, renameat};
use nix::sys::stat::{Mode, SFlag, fstat, fstatat};
use nix::unistd::{UnlinkatFlags, fsync, unlinkat};
use sha2::{Digest, Sha256};
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::Write;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

const DIRECTORY_FLAGS: OFlag = OFlag::O_RDONLY
    .union(OFlag::O_DIRECTORY)
    .union(OFlag::O_NOFOLLOW)
    .union(OFlag::O_CLOEXEC);
const TEMP_FILE_FLAGS: OFlag = OFlag::O_WRONLY
    .union(OFlag::O_CREAT)
    .union(OFlag::O_EXCL)
    .union(OFlag::O_NOFOLLOW)
    .union(OFlag::O_CLOEXEC);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationReceipt {
    pub path: PathBuf,
    pub owner_uid: u32,
    pub mode: u32,
    pub byte_count: usize,
    pub sha256: String,
}

/// Atomically write one workspace-relative regular file as the verified
/// project owner. Parent directories must already exist and remain private
/// from group/world writers. Existing safe file permissions are preserved;
/// `new_file_mode` applies only when the target does not exist.
pub fn write_project_file_atomic(
    context: &VerifiedExecutionContext,
    mutation_grant: &VerifiedProjectMutationGrant,
    relative_path: impl AsRef<Path>,
    contents: &[u8],
    new_file_mode: u32,
) -> Result<MutationReceipt, MutationError> {
    mutation_grant.verify_execution_scope(
        context.project_root(),
        context.project_identity_ref(),
        context.workspace_root(),
        Utc::now(),
    )?;
    context.revalidate()?;
    validate_mode(new_file_mode)?;
    let (parents, file_name) = split_relative_path(relative_path.as_ref())?;
    let workspace = open_directory(context.workspace_root())?;
    verify_directory_fd(
        workspace.as_raw_fd(),
        context.owner().uid,
        context.workspace_root(),
    )?;

    let mut parent = workspace;
    let mut resolved_parent = context.workspace_root().to_path_buf();
    for component in parents {
        let next = openat(
            Some(parent.as_raw_fd()),
            Path::new(&component),
            DIRECTORY_FLAGS,
            Mode::empty(),
        )
        .map(owned_fd)
        .map_err(|error| MutationError::UnsafeParent {
            path: resolved_parent.join(&component),
            reason: error.to_string(),
        })?;
        resolved_parent.push(&component);
        verify_directory_fd(next.as_raw_fd(), context.owner().uid, &resolved_parent)?;
        parent = next;
    }

    let final_mode = target_mode(
        parent.as_raw_fd(),
        &file_name,
        context.owner().uid,
        new_file_mode,
        &resolved_parent,
    )?;
    let temp_name = OsString::from(format!(".focusa-mutation-{}.tmp", Uuid::now_v7()));
    let temp_raw = openat(
        Some(parent.as_raw_fd()),
        Path::new(&temp_name),
        TEMP_FILE_FLAGS,
        Mode::from_bits_truncate(0o600),
    )
    .map_err(os_error)?;
    let temp_fd = owned_fd(temp_raw);
    let result = persist_and_rename(
        &parent,
        temp_fd,
        &temp_name,
        &file_name,
        contents,
        final_mode,
        context.owner().uid,
    );
    if result.is_err() {
        let _ = unlinkat(
            Some(parent.as_raw_fd()),
            Path::new(&temp_name),
            UnlinkatFlags::NoRemoveDir,
        );
    }
    result?;

    Ok(MutationReceipt {
        path: resolved_parent.join(&file_name),
        owner_uid: context.owner().uid,
        mode: final_mode,
        byte_count: contents.len(),
        sha256: format!("sha256:{:x}", Sha256::digest(contents)),
    })
}

fn persist_and_rename(
    parent: &OwnedFd,
    temp_fd: OwnedFd,
    temp_name: &OsStr,
    file_name: &OsStr,
    contents: &[u8],
    final_mode: u32,
    owner_uid: u32,
) -> Result<(), MutationError> {
    let temp_stat = fstat(temp_fd.as_raw_fd()).map_err(os_error)?;
    if temp_stat.st_uid != owner_uid {
        return Err(MutationError::OwnerMismatch {
            path: PathBuf::from(temp_name),
            expected_uid: owner_uid,
            actual_uid: temp_stat.st_uid,
        });
    }

    let mut file = File::from(temp_fd);
    file.set_permissions(fs::Permissions::from_mode(final_mode))
        .map_err(io_error)?;
    file.write_all(contents).map_err(io_error)?;
    file.sync_all().map_err(io_error)?;
    drop(file);

    renameat(
        Some(parent.as_raw_fd()),
        Path::new(temp_name),
        Some(parent.as_raw_fd()),
        Path::new(file_name),
    )
    .map_err(os_error)?;
    fsync(parent.as_raw_fd()).map_err(os_error)
}

fn target_mode(
    parent_fd: RawFd,
    file_name: &OsStr,
    expected_uid: u32,
    new_file_mode: u32,
    parent_path: &Path,
) -> Result<u32, MutationError> {
    match fstatat(
        Some(parent_fd),
        Path::new(file_name),
        AtFlags::AT_SYMLINK_NOFOLLOW,
    ) {
        Ok(stat) => {
            let path = parent_path.join(file_name);
            let file_type = SFlag::from_bits_truncate(stat.st_mode) & SFlag::S_IFMT;
            if file_type != SFlag::S_IFREG {
                return Err(MutationError::TargetNotRegularFile(path));
            }
            if stat.st_uid != expected_uid {
                return Err(MutationError::OwnerMismatch {
                    path,
                    expected_uid,
                    actual_uid: stat.st_uid,
                });
            }
            let mode = stat.st_mode & 0o777;
            validate_mode(mode)?;
            Ok(mode)
        }
        Err(Errno::ENOENT) => Ok(new_file_mode),
        Err(error) => Err(os_error(error)),
    }
}

fn split_relative_path(path: &Path) -> Result<(Vec<OsString>, OsString), MutationError> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(MutationError::UnsafeRelativePath);
    }
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => components.push(value.to_os_string()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(MutationError::UnsafeRelativePath);
            }
        }
    }
    let file_name = components.pop().ok_or(MutationError::UnsafeRelativePath)?;
    Ok((components, file_name))
}

fn open_directory(path: &Path) -> Result<OwnedFd, MutationError> {
    open(path, DIRECTORY_FLAGS, Mode::empty())
        .map(owned_fd)
        .map_err(os_error)
}

fn verify_directory_fd(fd: RawFd, expected_uid: u32, path: &Path) -> Result<(), MutationError> {
    let stat = fstat(fd).map_err(os_error)?;
    let file_type = SFlag::from_bits_truncate(stat.st_mode) & SFlag::S_IFMT;
    if file_type != SFlag::S_IFDIR {
        return Err(MutationError::UnsafeParent {
            path: path.to_path_buf(),
            reason: "not a directory".into(),
        });
    }
    if stat.st_uid != expected_uid {
        return Err(MutationError::OwnerMismatch {
            path: path.to_path_buf(),
            expected_uid,
            actual_uid: stat.st_uid,
        });
    }
    let mode = stat.st_mode & 0o777;
    if mode & 0o022 != 0 {
        return Err(MutationError::UnsafeParent {
            path: path.to_path_buf(),
            reason: format!("group/world writable mode {mode:o}"),
        });
    }
    Ok(())
}

fn validate_mode(mode: u32) -> Result<(), MutationError> {
    if mode > 0o777 || mode & 0o022 != 0 {
        return Err(MutationError::UnsafeFileMode(mode));
    }
    Ok(())
}

fn owned_fd(raw: RawFd) -> OwnedFd {
    // SAFETY: `open`/`openat` returned a new descriptor on success, and this is
    // its sole conversion into an owning Rust value.
    unsafe { OwnedFd::from_raw_fd(raw) }
}

fn os_error(error: Errno) -> MutationError {
    MutationError::Os(error.to_string())
}

fn io_error(error: std::io::Error) -> MutationError {
    MutationError::Io(error.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MutationError {
    #[error("project mutation path must be a non-empty workspace-relative file path")]
    UnsafeRelativePath,
    #[error("project mutation file mode is unsafe: {0:o}")]
    UnsafeFileMode(u32),
    #[error("project mutation parent is unsafe at {path}: {reason}")]
    UnsafeParent { path: PathBuf, reason: String },
    #[error("project mutation target is not a regular file: {0}")]
    TargetNotRegularFile(PathBuf),
    #[error(
        "project mutation owner mismatch for {path}: expected uid {expected_uid}, got {actual_uid}"
    )]
    OwnerMismatch {
        path: PathBuf,
        expected_uid: u32,
        actual_uid: u32,
    },
    #[error("project mutation bootstrap barrier failed: {0}")]
    BootstrapBarrier(#[from] AgentBootstrapBarrierError),
    #[error("project mutation identity verification failed: {0}")]
    Identity(#[from] IdentityError),
    #[error("project mutation OS operation failed: {0}")]
    Os(String),
    #[error("project mutation I/O failed: {0}")]
    Io(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{ExecutionIdentityRequest, OsIdentity};
    use chrono::Duration;
    use focusa_core::silent_session::{
        ModelBinding, SILENT_SESSION_LEASE_SCHEMA, SilentSessionId, SilentSessionLease,
        SilentSessionLeaseId, SilentSessionRunId, WorkpointBinding,
    };
    use focusa_core::silent_session_authorization::{
        ContextAuthorityActionClass, ContextAuthorityGrant,
    };
    use focusa_core::silent_session_bootstrap::{
        AGENT_BOOTSTRAP_PACKET_SCHEMA, AgentBootstrapPacket, BootstrapWorkspaceBinding,
        ContextBootstrapBinding, ProjectIdentityBootstrapBinding, ProjectMutationBarrierRequest,
        TrajectoryBootstrapBinding, WorkpointBootstrapBinding, context_authority_action_digest,
        verify_agent_bootstrap_packet, verify_project_mutation_barrier,
    };
    use std::os::unix::fs::{MetadataExt, symlink};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

    struct TestProject {
        root: PathBuf,
        workspace: PathBuf,
        context: VerifiedExecutionContext,
    }

    impl TestProject {
        fn new() -> Self {
            let sequence = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "focusa-runner-mutation-{}-{sequence}",
                std::process::id()
            ));
            let workspace = root.join("worktree");
            fs::create_dir_all(workspace.join("src")).expect("fixture should be created");
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
                .expect("root should be private");
            fs::set_permissions(&workspace, fs::Permissions::from_mode(0o700))
                .expect("workspace should be private");
            fs::set_permissions(workspace.join("src"), fs::Permissions::from_mode(0o700))
                .expect("source directory should be private");
            let current = OsIdentity::current().expect("current identity should resolve");
            let context = VerifiedExecutionContext::verify(&ExecutionIdentityRequest {
                daemon_uid: current.uid,
                execution_user: current.user_name,
                execution_uid: current.uid,
                project_root: root.clone(),
                project_identity_ref: "project:mutation-test".into(),
                workspace_root: workspace.clone(),
            })
            .expect("fixture context should verify");
            Self {
                root,
                workspace,
                context,
            }
        }

        fn mutation_grant(&self) -> VerifiedProjectMutationGrant {
            let now = Utc::now();
            let fresh_until = now + Duration::minutes(5);
            let project_identity_ref = self.context.project_identity_ref().to_owned();
            let continuity_id = "continuity:runner-mutation-test".to_owned();
            let trajectory_ref = "trajectory:runner-mutation-test".to_owned();
            let workpoint_ref = WorkpointBinding {
                workpoint_id: "workpoint:runner-mutation-test".into(),
                revision: Some(1),
            };
            let model = ModelBinding {
                provider: "openai-codex".into(),
                model: "gpt-test".into(),
                thinking: Some("high".into()),
            };
            let packet = AgentBootstrapPacket {
                schema: AGENT_BOOTSTRAP_PACKET_SCHEMA.into(),
                packet_id: Uuid::now_v7(),
                session_id: SilentSessionId::new(),
                run_id: SilentSessionRunId::new(),
                generation: 1,
                generated_at: now,
                fresh_until,
                project_identity: ProjectIdentityBootstrapBinding {
                    project_identity_ref: project_identity_ref.clone(),
                    project_root: self.context.project_root().to_path_buf(),
                    fingerprint: "runner-mutation-fingerprint".into(),
                    snapshot_ref: "project-snapshot:runner-mutation".into(),
                    snapshot_sha256: "1".repeat(64),
                    verified_at: now,
                    fresh_until,
                },
                continuity_id: continuity_id.clone(),
                trajectory: TrajectoryBootstrapBinding {
                    trajectory_ref: trajectory_ref.clone(),
                    project_identity_ref: project_identity_ref.clone(),
                    continuity_id: continuity_id.clone(),
                    snapshot_ref: "trajectory-snapshot:runner-mutation".into(),
                    snapshot_sha256: "2".repeat(64),
                    generated_at: now,
                    fresh_until,
                    status: focusa_core::silent_session_bootstrap::TrajectoryBootstrapStatus::CanonicalAdvisory,
                    waypoints: vec!["prove scoped runner mutation".into()],
                    active_gap: "runner mutation is not yet verified".into(),
                },
                workpoint: WorkpointBootstrapBinding {
                    workpoint_ref: workpoint_ref.clone(),
                    project_identity_ref: project_identity_ref.clone(),
                    continuity_id: continuity_id.clone(),
                    snapshot_ref: "workpoint-snapshot:runner-mutation".into(),
                    snapshot_sha256: "3".repeat(64),
                    generated_at: now,
                    fresh_until,
                },
                operator_ask: focusa_core::silent_session::OperatorAskBinding::capture(
                    "ask:runner-mutation-test",
                    "prove guarded project mutation",
                    1,
                    now,
                ),
                context: ContextBootstrapBinding {
                    context_packet_ref: "context-packet:runner-mutation".into(),
                    project_identity_ref: project_identity_ref.clone(),
                    continuity_id: continuity_id.clone(),
                    trajectory_ref,
                    workpoint_ref: workpoint_ref.clone(),
                    source_snapshot_ref: "context-snapshot:runner-mutation".into(),
                    packet_sha256: "4".repeat(64),
                    generated_at: now,
                    fresh_until,
                    advisory: true,
                    canonical: false,
                    canonical_mutation_allowed: false,
                    selected_context: vec!["src/generated.rs".into()],
                    excluded_context: vec![],
                    risk_refs: vec!["risk:generated-code-overwrite".into()],
                    valid_next_tools: vec!["tool:verify-project-mutation".into()],
                },
                ontology: focusa_core::silent_session_bootstrap::SilentSessionOntologyBindings {
                    agent_identity_ref: "agent:runner-test".into(),
                    actor_instance_ref: "actor-instance:runner-mutation-test".into(),
                    role_profile_ref: "role:runner-test".into(),
                    capability_profile_ref: "capability:posix-mutation".into(),
                    permission_profile_ref: "permission:isolated-worktree".into(),
                    responsibility_ref: "responsibility:mutation-test".into(),
                    handoff_boundary_ref: "handoff:operator".into(),
                    execution_context_ref: "execution-context:runner-test".into(),
                    tool_surface_ref: "tool-surface:posix-runner".into(),
                    affordance_ref: "affordance:generated-write".into(),
                    resource_ref: "resource:test-project".into(),
                    cost_model_ref: "cost-model:test".into(),
                    reliability_profile_ref: "reliability:strict".into(),
                    reversibility_profile_ref: "reversibility:worktree".into(),
                    work_item_ref: "focusa-a6yq6.4.4".into(),
                    action_intent_ref: "action-intent:generated-write".into(),
                    blocker_ref: "blocker:none".into(),
                    verification_record_ref: "verification:runner-bootstrap".into(),
                    evidence_artifact_ref: "evidence:runner-mutation".into(),
                },
                work_item_ref: Some("focusa-a6yq6.4.4".into()),
                workspace: BootstrapWorkspaceBinding {
                    workspace_ref: "workspace:runner-mutation-test".into(),
                    workspace_root: self.context.workspace_root().to_path_buf(),
                },
                model: model.clone(),
                role_ref: "role:runner-test".into(),
                mission: "prove guarded project mutation".into(),
                exact_next_action: "write one scoped fixture".into(),
                active_object_refs: vec!["src/generated.rs".into()],
                hook_refs: vec!["hook:before-project-mutation".into()],
                blockers: vec![],
                do_not_drift: vec!["do not write outside the workspace".into()],
                evidence_refs: vec!["test:runner-bootstrap-barrier".into()],
                proof_gaps: vec![],
                completion_expectations: vec!["owner write is atomic".into()],
            };
            let verification = verify_agent_bootstrap_packet(&packet, now)
                .expect("runner bootstrap packet should verify");
            let actor_instance_ref = "actor-instance:runner-mutation-test";
            let lease = SilentSessionLease {
                schema: SILENT_SESSION_LEASE_SCHEMA.into(),
                lease_id: SilentSessionLeaseId::new(),
                session_id: packet.session_id,
                project_root: packet.project_identity.project_root.clone(),
                project_identity_ref: project_identity_ref.clone(),
                continuity_id: continuity_id.clone(),
                work_item_ref: packet.work_item_ref.clone(),
                workspace_ref: packet.workspace.workspace_ref.clone(),
                path_intents: vec![PathBuf::from("src")],
                mutation_mode: focusa_core::silent_session::WriterMutationMode::IsolatedWorktree,
                writer_role: "primary".into(),
                owner_actor_instance_ref: actor_instance_ref.into(),
                fencing_token: 9,
                acquired_at: now - Duration::seconds(1),
                heartbeat_at: now,
                expires_at: now + Duration::minutes(2),
                adoption_policy: "operator_only".into(),
            };
            let authority = ContextAuthorityGrant {
                verdict_ref: "context-authority:runner-mutation-test".into(),
                allowed: true,
                project_identity_ref,
                continuity_id,
                workpoint_ref: Some(workpoint_ref.workpoint_id),
                action_class: ContextAuthorityActionClass::GeneratedCodeOverwrite,
                action: "project_mutation".into(),
                action_digest: context_authority_action_digest(
                    &packet,
                    ContextAuthorityActionClass::GeneratedCodeOverwrite,
                    "project_mutation",
                )
                .expect("action digest should serialize"),
                issued_at: now,
                expires_at: now + Duration::minutes(1),
            };
            verify_project_mutation_barrier(&ProjectMutationBarrierRequest {
                packet: &packet,
                bootstrap_verification: &verification,
                lease: &lease,
                context_authority: &authority,
                action_class: ContextAuthorityActionClass::GeneratedCodeOverwrite,
                action: "project_mutation",
                actor_instance_ref,
                requested_model: &model,
                effective_model: Some(&model),
                observed_model: Some(&model),
                now,
            })
            .expect("all runner project-mutation barriers should pass")
        }
    }

    impl Drop for TestProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn atomic_write_creates_owner_file_and_preserves_existing_safe_mode() {
        let project = TestProject::new();
        let grant = project.mutation_grant();
        let current = OsIdentity::current().expect("current identity should resolve");
        let receipt = write_project_file_atomic(
            &project.context,
            &grant,
            "src/generated.rs",
            b"first\n",
            0o644,
        )
        .expect("new owner file should be written");
        assert_eq!(
            receipt.path,
            project.context.workspace_root().join("src/generated.rs")
        );
        assert_eq!(receipt.owner_uid, current.uid);
        assert_eq!(receipt.mode, 0o644);
        assert_eq!(receipt.byte_count, 6);
        assert_eq!(
            receipt.sha256,
            format!("sha256:{:x}", Sha256::digest(b"first\n"))
        );
        let metadata = fs::metadata(&receipt.path).expect("written file should exist");
        assert_eq!(metadata.uid(), current.uid);
        assert_eq!(metadata.permissions().mode() & 0o777, 0o644);

        fs::set_permissions(&receipt.path, fs::Permissions::from_mode(0o700))
            .expect("fixture mode should change");
        let replacement = write_project_file_atomic(
            &project.context,
            &grant,
            "src/generated.rs",
            b"second\n",
            0o600,
        )
        .expect("existing owner file should be replaced");
        assert_eq!(replacement.mode, 0o700);
        assert_eq!(
            fs::read(&replacement.path).expect("replacement should be readable"),
            b"second\n"
        );
        assert_eq!(
            fs::metadata(&replacement.path)
                .expect("replacement metadata should exist")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert!(
            fs::read_dir(project.workspace.join("src"))
                .expect("source directory should list")
                .all(|entry| !entry
                    .expect("directory entry should read")
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".focusa-mutation-"))
        );
    }

    #[test]
    fn bootstrap_grant_scope_is_checked_before_any_project_file_is_created() {
        let project = TestProject::new();
        let other_project = TestProject::new();
        let wrong_scope_grant = other_project.mutation_grant();
        let target = project.workspace.join("src/blocked.rs");

        assert!(matches!(
            write_project_file_atomic(
                &project.context,
                &wrong_scope_grant,
                "src/blocked.rs",
                b"must not exist",
                0o600,
            ),
            Err(MutationError::BootstrapBarrier(
                AgentBootstrapBarrierError::ScopeMismatch("execution_scope")
            ))
        ));
        assert!(!target.exists());
    }

    #[test]
    fn final_and_intermediate_symlinks_cannot_redirect_mutation() {
        let project = TestProject::new();
        let grant = project.mutation_grant();
        let outside = project.root.join("outside");
        fs::create_dir(&outside).expect("outside directory should exist");
        fs::set_permissions(&outside, fs::Permissions::from_mode(0o700))
            .expect("outside should be private");
        let outside_file = outside.join("protected.txt");
        fs::write(&outside_file, b"unchanged").expect("outside fixture should exist");

        let final_link = project.workspace.join("src/final-link");
        symlink(&outside_file, &final_link).expect("final symlink should exist");
        assert!(matches!(
            write_project_file_atomic(&project.context, &grant, "src/final-link", b"attack", 0o600),
            Err(MutationError::TargetNotRegularFile(_))
        ));
        assert_eq!(
            fs::read(&outside_file).expect("outside fixture should remain"),
            b"unchanged"
        );

        let parent_link = project.workspace.join("parent-link");
        symlink(&outside, &parent_link).expect("parent symlink should exist");
        assert!(matches!(
            write_project_file_atomic(
                &project.context,
                &grant,
                "parent-link/protected.txt",
                b"attack",
                0o600
            ),
            Err(MutationError::UnsafeParent { .. })
        ));
        assert_eq!(
            fs::read(&outside_file).expect("outside fixture should remain"),
            b"unchanged"
        );
    }

    #[test]
    fn traversal_shared_parent_and_unsafe_modes_fail_closed() {
        let project = TestProject::new();
        let grant = project.mutation_grant();
        assert_eq!(
            write_project_file_atomic(&project.context, &grant, "../outside", b"bad", 0o600),
            Err(MutationError::UnsafeRelativePath)
        );
        assert_eq!(
            write_project_file_atomic(&project.context, &grant, "/tmp/outside", b"bad", 0o600),
            Err(MutationError::UnsafeRelativePath)
        );
        assert_eq!(
            write_project_file_atomic(&project.context, &grant, "src/file", b"bad", 0o666),
            Err(MutationError::UnsafeFileMode(0o666))
        );

        fs::set_permissions(
            project.workspace.join("src"),
            fs::Permissions::from_mode(0o770),
        )
        .expect("source directory should become group writable");
        assert!(matches!(
            write_project_file_atomic(&project.context, &grant, "src/file", b"bad", 0o600),
            Err(MutationError::UnsafeParent { .. })
        ));
    }
}
