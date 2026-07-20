//! OS-backed project-owner and mutation-scope verification.
//!
//! Cross-user daemon execution is never implemented with shell composition or
//! an ambient user name. The project root's numeric owner is authoritative. A
//! process may execute only when the current process already has that UID; this
//! yields embedded mode for a same-user daemon and requires a per-user runner
//! otherwise.

use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsIdentity {
    pub user_name: String,
    pub uid: u32,
    pub gid: u32,
}

impl OsIdentity {
    #[cfg(unix)]
    pub fn current() -> Result<Self, IdentityError> {
        use nix::unistd::{Gid, Uid, User};

        let uid = Uid::effective();
        let gid = Gid::effective();
        let user = User::from_uid(uid)
            .map_err(|error| IdentityError::OsIdentityLookup(error.to_string()))?
            .ok_or(IdentityError::UnknownUid(uid.as_raw()))?;
        Ok(Self {
            user_name: user.name,
            uid: uid.as_raw(),
            gid: gid.as_raw(),
        })
    }

    #[cfg(not(unix))]
    pub fn current() -> Result<Self, IdentityError> {
        Err(IdentityError::PlatformUnsupported)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// The daemon already runs as the verified project owner and may embed the
    /// runner library without crossing an OS-user boundary.
    EmbeddedSameUser,
    /// The daemon has another UID; this process is the verified owner's runner.
    PerUserRunner,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionIdentityRequest {
    pub daemon_uid: u32,
    pub execution_user: String,
    pub execution_uid: u32,
    pub project_root: PathBuf,
    pub project_identity_ref: String,
    pub workspace_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedExecutionContext {
    mode: ExecutionMode,
    owner: OsIdentity,
    project_root: PathBuf,
    project_identity_ref: String,
    workspace_root: PathBuf,
}

impl VerifiedExecutionContext {
    /// Verify that both the project and effective workspace are real,
    /// non-symlink directories owned by the requested OS identity.
    pub fn verify(request: &ExecutionIdentityRequest) -> Result<Self, IdentityError> {
        Self::verify_as(request, OsIdentity::current()?)
    }

    fn verify_as(
        request: &ExecutionIdentityRequest,
        current: OsIdentity,
    ) -> Result<Self, IdentityError> {
        if request.execution_user.trim().is_empty()
            || request.project_identity_ref.trim().is_empty()
        {
            return Err(IdentityError::InvalidRequest);
        }
        if current.uid != request.execution_uid || current.user_name != request.execution_user {
            return Err(IdentityError::RunnerUserMismatch {
                expected_user: request.execution_user.clone(),
                expected_uid: request.execution_uid,
                actual_user: current.user_name,
                actual_uid: current.uid,
            });
        }

        let project_root = verify_owned_directory(&request.project_root, request.execution_uid)?;
        let workspace_root =
            verify_owned_directory(&request.workspace_root, request.execution_uid)?;
        let mode = if request.daemon_uid == request.execution_uid {
            ExecutionMode::EmbeddedSameUser
        } else {
            ExecutionMode::PerUserRunner
        };

        Ok(Self {
            mode,
            owner: current,
            project_root,
            project_identity_ref: request.project_identity_ref.clone(),
            workspace_root,
        })
    }

    pub fn mode(&self) -> ExecutionMode {
        self.mode
    }

    pub fn owner(&self) -> &OsIdentity {
        &self.owner
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn project_identity_ref(&self) -> &str {
        &self.project_identity_ref
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Re-check the effective OS identity and both authority-bearing roots at
    /// the last responsible moment before a process or mutation starts.
    pub fn revalidate(&self) -> Result<(), IdentityError> {
        let current = OsIdentity::current()?;
        if current.uid != self.owner.uid || current.user_name != self.owner.user_name {
            return Err(IdentityError::RunnerUserMismatch {
                expected_user: self.owner.user_name.clone(),
                expected_uid: self.owner.uid,
                actual_user: current.user_name,
                actual_uid: current.uid,
            });
        }
        if current.gid != self.owner.gid {
            return Err(IdentityError::RunnerGroupMismatch {
                expected_gid: self.owner.gid,
                actual_gid: current.gid,
            });
        }
        verify_owned_directory(&self.project_root, self.owner.uid)?;
        verify_owned_directory(&self.workspace_root, self.owner.uid)?;
        Ok(())
    }

    /// Resolve one workspace-relative mutation path without accepting absolute
    /// paths, `..`, or any existing symlink. Existing path components must
    /// remain owned by the verified execution UID.
    pub fn authorize_mutation_path(
        &self,
        relative_path: impl AsRef<Path>,
    ) -> Result<VerifiedMutationPath, IdentityError> {
        let relative_path = relative_path.as_ref();
        if relative_path.as_os_str().is_empty() || relative_path.is_absolute() {
            return Err(IdentityError::UnsafeMutationPath);
        }

        let mut components = Vec::<OsString>::new();
        for component in relative_path.components() {
            match component {
                Component::Normal(value) => components.push(value.to_os_string()),
                Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(IdentityError::UnsafeMutationPath);
                }
            }
        }
        if components.is_empty() {
            return Err(IdentityError::UnsafeMutationPath);
        }

        let mut candidate = self.workspace_root.clone();
        for component in components {
            candidate.push(component);
            match fs::symlink_metadata(&candidate) {
                Ok(metadata) => {
                    if metadata.file_type().is_symlink() {
                        return Err(IdentityError::SymlinkRejected(candidate));
                    }
                    verify_metadata_owner(&candidate, &metadata, self.owner.uid)?;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(IdentityError::Io(error.to_string())),
            }
        }

        Ok(VerifiedMutationPath { path: candidate })
    }
}

/// A path proven relative to one verified, same-owner workspace at preflight.
/// Callers must still use no-follow file creation where the platform supports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedMutationPath {
    path: PathBuf,
}

impl VerifiedMutationPath {
    pub fn as_path(&self) -> &Path {
        &self.path
    }
}

#[cfg(unix)]
fn verify_owned_directory(path: &Path, owner_uid: u32) -> Result<PathBuf, IdentityError> {
    let original =
        fs::symlink_metadata(path).map_err(|error| IdentityError::Io(error.to_string()))?;
    if original.file_type().is_symlink() {
        return Err(IdentityError::SymlinkRejected(path.to_path_buf()));
    }
    if !original.is_dir() {
        return Err(IdentityError::NotDirectory(path.to_path_buf()));
    }
    verify_metadata_owner(path, &original, owner_uid)?;
    verify_not_shared_writable(path, &original)?;

    let canonical = fs::canonicalize(path).map_err(|error| IdentityError::Io(error.to_string()))?;
    let canonical_metadata =
        fs::metadata(&canonical).map_err(|error| IdentityError::Io(error.to_string()))?;
    verify_metadata_owner(&canonical, &canonical_metadata, owner_uid)?;
    verify_not_shared_writable(&canonical, &canonical_metadata)?;
    Ok(canonical)
}

#[cfg(not(unix))]
fn verify_owned_directory(_path: &Path, _owner_uid: u32) -> Result<PathBuf, IdentityError> {
    Err(IdentityError::PlatformUnsupported)
}

#[cfg(unix)]
fn verify_metadata_owner(
    path: &Path,
    metadata: &fs::Metadata,
    expected_uid: u32,
) -> Result<(), IdentityError> {
    use std::os::unix::fs::MetadataExt;

    let actual_uid = metadata.uid();
    if actual_uid != expected_uid {
        return Err(IdentityError::OwnerMismatch {
            path: path.to_path_buf(),
            expected_uid,
            actual_uid,
        });
    }
    Ok(())
}

#[cfg(not(unix))]
fn verify_metadata_owner(
    _path: &Path,
    _metadata: &fs::Metadata,
    _expected_uid: u32,
) -> Result<(), IdentityError> {
    Err(IdentityError::PlatformUnsupported)
}

#[cfg(unix)]
fn verify_not_shared_writable(path: &Path, metadata: &fs::Metadata) -> Result<(), IdentityError> {
    use std::os::unix::fs::MetadataExt;

    let mode = metadata.mode();
    if mode & 0o022 != 0 {
        return Err(IdentityError::SharedWritableDirectory {
            path: path.to_path_buf(),
            mode: mode & 0o777,
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IdentityError {
    #[error("runner execution identity request is incomplete")]
    InvalidRequest,
    #[error("runner identity platform is unsupported")]
    PlatformUnsupported,
    #[error("OS identity lookup failed: {0}")]
    OsIdentityLookup(String),
    #[error("OS user for uid {0} does not exist")]
    UnknownUid(u32),
    #[error(
        "runner user mismatch: expected {expected_user} ({expected_uid}), got {actual_user} ({actual_uid})"
    )]
    RunnerUserMismatch {
        expected_user: String,
        expected_uid: u32,
        actual_user: String,
        actual_uid: u32,
    },
    #[error("runner primary group mismatch: expected gid {expected_gid}, got {actual_gid}")]
    RunnerGroupMismatch { expected_gid: u32, actual_gid: u32 },
    #[error("path is not a directory: {0}")]
    NotDirectory(PathBuf),
    #[error("symlink path is not allowed for project execution: {0}")]
    SymlinkRejected(PathBuf),
    #[error("path owner mismatch for {path}: expected uid {expected_uid}, got {actual_uid}")]
    OwnerMismatch {
        path: PathBuf,
        expected_uid: u32,
        actual_uid: u32,
    },
    #[error("project directory is group/world writable: {path} ({mode:o})")]
    SharedWritableDirectory { path: PathBuf, mode: u32 },
    #[error("mutation path must remain relative and symlink-free")]
    UnsafeMutationPath,
    #[error("project identity I/O failed: {0}")]
    Io(String),
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

    struct TestProject {
        root: PathBuf,
        workspace: PathBuf,
    }

    impl TestProject {
        fn new() -> Self {
            let sequence = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "focusa-runner-identity-{}-{sequence}",
                std::process::id()
            ));
            let workspace = root.join("worktree");
            fs::create_dir_all(&workspace).expect("test project should be created");
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
                .expect("test root should be private");
            fs::set_permissions(&workspace, fs::Permissions::from_mode(0o700))
                .expect("test workspace should be private");
            Self { root, workspace }
        }

        fn request(&self, daemon_uid: u32) -> ExecutionIdentityRequest {
            let current = OsIdentity::current().expect("current Unix user should resolve");
            ExecutionIdentityRequest {
                daemon_uid,
                execution_user: current.user_name,
                execution_uid: current.uid,
                project_root: self.root.clone(),
                project_identity_ref: "project:test".into(),
                workspace_root: self.workspace.clone(),
            }
        }
    }

    impl Drop for TestProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn same_user_daemon_selects_embedded_execution() {
        let project = TestProject::new();
        let current = OsIdentity::current().expect("current identity should resolve");
        let context = VerifiedExecutionContext::verify(&project.request(current.uid))
            .expect("owned project should verify");
        assert_eq!(context.mode, ExecutionMode::EmbeddedSameUser);
        assert_eq!(context.owner, current);
        assert_eq!(
            context.project_root,
            fs::canonicalize(&project.root).expect("root should canonicalize")
        );
    }

    #[test]
    fn cross_user_daemon_requires_the_project_owners_runner() {
        let project = TestProject::new();
        let current = OsIdentity::current().expect("current identity should resolve");
        let daemon_uid = current.uid.wrapping_add(1);
        let context = VerifiedExecutionContext::verify(&project.request(daemon_uid))
            .expect("owner runner should verify");
        assert_eq!(context.mode, ExecutionMode::PerUserRunner);
        assert_eq!(context.owner.uid, current.uid);
    }

    #[test]
    fn wrong_execution_identity_fails_before_project_access() {
        let project = TestProject::new();
        let current = OsIdentity::current().expect("current identity should resolve");
        let mut request = project.request(current.uid);
        request.execution_uid = current.uid.wrapping_add(1);
        let error = VerifiedExecutionContext::verify(&request)
            .expect_err("another user's request must be denied");
        assert!(matches!(error, IdentityError::RunnerUserMismatch { .. }));
    }

    #[test]
    fn project_and_workspace_symlinks_are_rejected() {
        let project = TestProject::new();
        let project_link = project.root.with_extension("link");
        symlink(&project.root, &project_link).expect("test symlink should be created");
        let mut request = project.request(
            OsIdentity::current()
                .expect("current identity should resolve")
                .uid,
        );
        request.project_root = project_link.clone();
        assert_eq!(
            VerifiedExecutionContext::verify(&request),
            Err(IdentityError::SymlinkRejected(project_link.clone()))
        );
        fs::remove_file(project_link).expect("test symlink should be removed");

        let workspace_link = project.root.join("workspace-link");
        symlink(&project.workspace, &workspace_link).expect("workspace symlink should be created");
        request.project_root = project.root.clone();
        request.workspace_root = workspace_link.clone();
        assert_eq!(
            VerifiedExecutionContext::verify(&request),
            Err(IdentityError::SymlinkRejected(workspace_link))
        );
    }

    #[test]
    fn mutation_paths_reject_traversal_and_existing_symlinks() {
        let project = TestProject::new();
        let current = OsIdentity::current().expect("current identity should resolve");
        let context = VerifiedExecutionContext::verify(&project.request(current.uid))
            .expect("owned project should verify");

        let safe = context
            .authorize_mutation_path("src/generated/output.rs")
            .expect("owned relative path should be authorized");
        assert_eq!(
            safe.as_path(),
            context.workspace_root.join("src/generated/output.rs")
        );
        assert_eq!(
            context.authorize_mutation_path("../outside"),
            Err(IdentityError::UnsafeMutationPath)
        );
        assert_eq!(
            context.authorize_mutation_path("/tmp/outside"),
            Err(IdentityError::UnsafeMutationPath)
        );

        let outside = project.root.join("outside");
        fs::create_dir(&outside).expect("outside fixture should be created");
        let escape = project.workspace.join("escape");
        symlink(&outside, &escape).expect("escape symlink should be created");
        assert_eq!(
            context.authorize_mutation_path("escape/file"),
            Err(IdentityError::SymlinkRejected(
                context.workspace_root.join("escape")
            ))
        );
    }

    #[test]
    fn shared_writable_project_root_is_rejected() {
        let project = TestProject::new();
        fs::set_permissions(&project.root, fs::Permissions::from_mode(0o777))
            .expect("fixture permissions should change");
        let current = OsIdentity::current().expect("current identity should resolve");
        let error = VerifiedExecutionContext::verify(&project.request(current.uid))
            .expect_err("shared writable project must fail closed");
        assert!(matches!(
            error,
            IdentityError::SharedWritableDirectory { .. }
        ));
    }
}
