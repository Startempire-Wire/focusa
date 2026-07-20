//! POSIX process-tree ownership for the per-user session runner.
//!
//! The runner can spawn only after the verified project-owner context is
//! revalidated. Every run receives a dedicated process group and an unguessable
//! process-instance identity, allowing signed heartbeats and fail-closed daemon
//! adoption without trusting a reused PID.

use crate::identity::{IdentityError, OsIdentity, VerifiedExecutionContext};
use crate::protocol::{
    ActiveRunRecord, AdoptionDecision, AdoptionExpectation, AdoptionRejection, ProcessTreeIdentity,
    ProtocolError, RunnerHeartbeat,
};
use chrono::{DateTime, Utc};
use focusa_core::silent_session::{SilentSessionId, SilentSessionRunId};
use nix::errno::Errno;
use nix::sys::signal::{Signal, killpg};
use nix::unistd::{Pid, getpgid, getsid};
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use thiserror::Error;
use tokio::process::{Child, Command};
use uuid::Uuid;

const RESERVED_RUNNER_ENV: [&str; 3] = [
    "FOCUSA_RUNNER_PROCESS_INSTANCE_ID",
    "FOCUSA_SILENT_SESSION_ID",
    "FOCUSA_SILENT_SESSION_RUN_ID",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosixSpawnRequest {
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub executable: PathBuf,
    pub argv: Vec<OsString>,
    pub env: BTreeMap<OsString, OsString>,
    pub launch_manifest_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitedRunRecord {
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub process_instance_id: String,
    pub exit_code: Option<i32>,
    pub signal: Option<i32>,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeartbeatSnapshot {
    pub heartbeat: RunnerHeartbeat,
    pub exited_runs: Vec<ExitedRunRecord>,
}

struct OwnedProcess {
    child: Child,
    record: ActiveRunRecord,
}

/// One runner instance bound to exactly one effective OS user.
pub struct PosixProcessSupervisor {
    runner_id: String,
    identity: OsIdentity,
    heartbeat_sequence: u64,
    last_heartbeat_at: Option<DateTime<Utc>>,
    runs: BTreeMap<SilentSessionRunId, OwnedProcess>,
}

impl PosixProcessSupervisor {
    pub fn for_current_user(runner_id: impl Into<String>) -> Result<Self, SupervisorError> {
        let runner_id = runner_id.into();
        if runner_id.trim().is_empty() {
            return Err(SupervisorError::InvalidRunnerId);
        }
        Ok(Self {
            runner_id,
            identity: OsIdentity::current()?,
            heartbeat_sequence: 0,
            last_heartbeat_at: None,
            runs: BTreeMap::new(),
        })
    }

    pub fn runner_id(&self) -> &str {
        &self.runner_id
    }

    pub fn identity(&self) -> &OsIdentity {
        &self.identity
    }

    pub fn active_run_count(&self) -> usize {
        self.runs.len()
    }

    /// Spawn as the already-verified project owner. This deliberately never
    /// calls setuid or composes an `as-user` shell command: cross-user daemon
    /// requests must reach the project owner's runner process first.
    pub fn spawn(
        &mut self,
        context: &VerifiedExecutionContext,
        request: PosixSpawnRequest,
        spawned_at: DateTime<Utc>,
    ) -> Result<ActiveRunRecord, SupervisorError> {
        validate_spawn_request(&request)?;
        if context.owner() != &self.identity {
            return Err(SupervisorError::ContextUserMismatch {
                context_user: context.owner().user_name.clone(),
                context_uid: context.owner().uid,
                runner_user: self.identity.user_name.clone(),
                runner_uid: self.identity.uid,
            });
        }
        context.revalidate()?;
        if self.runs.contains_key(&request.run_id) {
            return Err(SupervisorError::RunAlreadyOwned(request.run_id));
        }
        if self
            .runs
            .values()
            .any(|owned| owned.record.session_id == request.session_id)
        {
            return Err(SupervisorError::SessionAlreadyHasActiveRun(
                request.session_id,
            ));
        }

        let executable = canonical_executable(&request.executable)?;
        let executable_ref = executable
            .to_str()
            .ok_or_else(|| SupervisorError::ExecutablePathNotUtf8(executable.clone()))?
            .to_owned();
        let process_instance_id = format!("process:{}", Uuid::now_v7());

        let os_session_id = getsid(None).map_err(os_error)?;
        let mut command = Command::new(&executable);
        command
            .args(&request.argv)
            .current_dir(context.workspace_root())
            .env_clear()
            .envs(&request.env)
            .env(RESERVED_RUNNER_ENV[0], process_instance_id.as_str())
            .env(RESERVED_RUNNER_ENV[1], request.session_id.to_string())
            .env(RESERVED_RUNNER_ENV[2], request.run_id.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(false)
            .process_group(0);
        let child = command.spawn().map_err(io_error)?;
        let pid = child.id().ok_or(SupervisorError::MissingChildPid)?;
        let raw_pid = i32::try_from(pid).map_err(|_| SupervisorError::InvalidChildPid(pid))?;
        // `process_group(0)` is applied atomically by the spawn implementation;
        // deriving the group from the returned child PID avoids a false launch
        // failure when a very short process exits before a parent-side getpgid.
        let process_group_id = Pid::from_raw(raw_pid);

        let process_tree = ProcessTreeIdentity {
            process_instance_id,
            runner_id: self.runner_id.clone(),
            session_id: request.session_id,
            run_id: request.run_id,
            generation: request.generation,
            pid,
            process_group_id: i64::from(process_group_id.as_raw()),
            os_session_id: i64::from(os_session_id.as_raw()),
            execution_user: self.identity.user_name.clone(),
            execution_uid: self.identity.uid,
            executable_ref: executable_ref.clone(),
            spawned_at,
        };
        let record = ActiveRunRecord {
            runner_id: self.runner_id.clone(),
            session_id: request.session_id,
            run_id: request.run_id,
            generation: request.generation,
            project_root: context.project_root().to_path_buf(),
            project_identity_ref: context.project_identity_ref().to_owned(),
            workspace_root: context.workspace_root().to_path_buf(),
            execution_user: self.identity.user_name.clone(),
            execution_uid: self.identity.uid,
            executable_ref,
            launch_manifest_sha256: request.launch_manifest_sha256,
            process_tree,
            heartbeat_at: spawned_at,
        };
        self.runs.insert(
            request.run_id,
            OwnedProcess {
                child,
                record: record.clone(),
            },
        );
        Ok(record)
    }

    /// Reap finished direct children and produce the exact active-run set for
    /// a signed heartbeat. A regressing clock is rejected rather than making a
    /// stale process record appear fresh.
    pub fn heartbeat(
        &mut self,
        observed_at: DateTime<Utc>,
    ) -> Result<HeartbeatSnapshot, SupervisorError> {
        if self
            .last_heartbeat_at
            .is_some_and(|last| observed_at < last)
        {
            return Err(SupervisorError::HeartbeatClockRegressed);
        }

        let mut active_runs = Vec::new();
        let mut exited_runs = Vec::new();
        let mut finished_ids = Vec::new();
        for (run_id, owned) in &mut self.runs {
            match owned.child.try_wait().map_err(io_error)? {
                Some(status) => {
                    exited_runs.push(exit_record(&owned.record, status, observed_at));
                    finished_ids.push(*run_id);
                }
                None => match validate_live_process_tree(&owned.record) {
                    Ok(()) => {
                        owned.record.heartbeat_at = observed_at;
                        active_runs.push(owned.record.clone());
                    }
                    Err(SupervisorError::ProcessNoLongerExists) => {
                        // The OS may report ESRCH in the narrow interval before
                        // Tokio can reap the child. Never claim it active; keep
                        // the handle for the next heartbeat if no status is
                        // available yet.
                        if let Some(status) = owned.child.try_wait().map_err(io_error)? {
                            exited_runs.push(exit_record(&owned.record, status, observed_at));
                            finished_ids.push(*run_id);
                        }
                    }
                    Err(error) => return Err(error),
                },
            }
        }
        for run_id in finished_ids {
            self.runs.remove(&run_id);
        }

        self.heartbeat_sequence = self
            .heartbeat_sequence
            .checked_add(1)
            .ok_or(SupervisorError::HeartbeatSequenceExhausted)?;
        self.last_heartbeat_at = Some(observed_at);
        Ok(HeartbeatSnapshot {
            heartbeat: RunnerHeartbeat {
                runner_id: self.runner_id.clone(),
                os_user: self.identity.user_name.clone(),
                uid: self.identity.uid,
                sequence: self.heartbeat_sequence,
                observed_at,
                active_runs,
            },
            exited_runs,
        })
    }

    /// Adopt only a still-live process owned by this runner. The daemon's
    /// expectation is compared against the refreshed runner record field by
    /// field; an unknown or exited run fails closed.
    pub fn evaluate_adoption(
        &mut self,
        expectation: &AdoptionExpectation,
        observed_at: DateTime<Utc>,
    ) -> Result<AdoptionDecision, SupervisorError> {
        if expectation.runner_id != self.runner_id {
            return Ok(expectation.rejected(AdoptionRejection::RunnerMismatch));
        }

        let Some(owned) = self.runs.get_mut(&expectation.run_id) else {
            return Ok(expectation.rejected(AdoptionRejection::ProcessNotOwned));
        };
        if owned.child.try_wait().map_err(io_error)?.is_some() {
            self.runs.remove(&expectation.run_id);
            return Ok(expectation.rejected(AdoptionRejection::ProcessNotOwned));
        }
        if validate_live_process_tree(&owned.record).is_err() {
            return Ok(expectation.rejected(AdoptionRejection::ProcessIdentityMismatch));
        }
        owned.record.heartbeat_at = observed_at;
        expectation
            .evaluate(&owned.record)
            .map_err(SupervisorError::Protocol)
    }

    /// Force-kill the entire owned process group before reaping its leader.
    /// This ordering prevents the leader PID from being reused before the
    /// process-tree signal is delivered.
    pub async fn force_terminate(
        &mut self,
        run_id: SilentSessionRunId,
        observed_at: DateTime<Utc>,
    ) -> Result<ExitedRunRecord, SupervisorError> {
        let mut owned = self
            .runs
            .remove(&run_id)
            .ok_or(SupervisorError::RunNotOwned(run_id))?;
        let process_group = process_group_pid(&owned.record)?;
        match killpg(process_group, Signal::SIGKILL) {
            Ok(()) | Err(Errno::ESRCH) => {}
            Err(error) => return Err(os_error(error)),
        }
        let status = owned.child.wait().await.map_err(io_error)?;
        Ok(exit_record(&owned.record, status, observed_at))
    }
}

impl Drop for PosixProcessSupervisor {
    fn drop(&mut self) {
        for owned in self.runs.values() {
            if let Ok(process_group) = process_group_pid(&owned.record) {
                let _ = killpg(process_group, Signal::SIGKILL);
            }
        }
    }
}

fn validate_spawn_request(request: &PosixSpawnRequest) -> Result<(), SupervisorError> {
    if !request.session_id.is_uuid_v7() || !request.run_id.is_uuid_v7() || request.generation == 0 {
        return Err(SupervisorError::InvalidRunIdentity);
    }
    if request.launch_manifest_sha256.len() != 64
        || !request
            .launch_manifest_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(SupervisorError::InvalidLaunchManifestDigest);
    }
    if request.env.keys().any(|key| {
        RESERVED_RUNNER_ENV
            .iter()
            .any(|reserved| key == OsStr::new(reserved))
    }) {
        return Err(SupervisorError::ReservedEnvironmentOverride);
    }
    Ok(())
}

fn canonical_executable(path: &Path) -> Result<PathBuf, SupervisorError> {
    if !path.is_absolute() {
        return Err(SupervisorError::ExecutableMustBeAbsolute(
            path.to_path_buf(),
        ));
    }
    let canonical = fs::canonicalize(path).map_err(io_error)?;
    let metadata = fs::metadata(&canonical).map_err(io_error)?;
    if !metadata.is_file() || metadata.mode() & 0o111 == 0 {
        return Err(SupervisorError::ExecutableInvalid(canonical));
    }
    Ok(canonical)
}

fn validate_live_process_tree(record: &ActiveRunRecord) -> Result<(), SupervisorError> {
    let pid = process_pid(record)?;
    let process_group = getpgid(Some(pid)).map_err(os_error)?;
    let session = getsid(Some(pid)).map_err(os_error)?;
    if i64::from(process_group.as_raw()) != record.process_tree.process_group_id
        || i64::from(session.as_raw()) != record.process_tree.os_session_id
    {
        return Err(SupervisorError::ProcessIdentityChanged);
    }
    Ok(())
}

fn process_pid(record: &ActiveRunRecord) -> Result<Pid, SupervisorError> {
    let raw = i32::try_from(record.process_tree.pid)
        .map_err(|_| SupervisorError::InvalidChildPid(record.process_tree.pid))?;
    Ok(Pid::from_raw(raw))
}

fn process_group_pid(record: &ActiveRunRecord) -> Result<Pid, SupervisorError> {
    let raw = i32::try_from(record.process_tree.process_group_id)
        .map_err(|_| SupervisorError::ProcessIdentityChanged)?;
    if raw <= 0 {
        return Err(SupervisorError::ProcessIdentityChanged);
    }
    Ok(Pid::from_raw(raw))
}

fn exit_record(
    record: &ActiveRunRecord,
    status: ExitStatus,
    observed_at: DateTime<Utc>,
) -> ExitedRunRecord {
    ExitedRunRecord {
        session_id: record.session_id,
        run_id: record.run_id,
        generation: record.generation,
        process_instance_id: record.process_tree.process_instance_id.clone(),
        exit_code: status.code(),
        signal: status.signal(),
        observed_at,
    }
}

fn io_error(error: std::io::Error) -> SupervisorError {
    SupervisorError::Io(error.to_string())
}

fn os_error(error: Errno) -> SupervisorError {
    if error == Errno::ESRCH {
        SupervisorError::ProcessNoLongerExists
    } else {
        SupervisorError::Os(error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SupervisorError {
    #[error("runner id is required")]
    InvalidRunnerId,
    #[error("silent session and run identities must be UUIDv7 with a positive generation")]
    InvalidRunIdentity,
    #[error("launch manifest digest must be a 64-character SHA-256 value")]
    InvalidLaunchManifestDigest,
    #[error("runner-owned identity environment cannot be overridden")]
    ReservedEnvironmentOverride,
    #[error(
        "verified context user {context_user} ({context_uid}) does not match runner user {runner_user} ({runner_uid})"
    )]
    ContextUserMismatch {
        context_user: String,
        context_uid: u32,
        runner_user: String,
        runner_uid: u32,
    },
    #[error("run is already owned by this runner: {0}")]
    RunAlreadyOwned(SilentSessionRunId),
    #[error("session already has an active run: {0}")]
    SessionAlreadyHasActiveRun(SilentSessionId),
    #[error("run is not owned by this runner: {0}")]
    RunNotOwned(SilentSessionRunId),
    #[error("runner executable must be absolute: {0}")]
    ExecutableMustBeAbsolute(PathBuf),
    #[error("runner executable is not a regular executable file: {0}")]
    ExecutableInvalid(PathBuf),
    #[error("runner executable path is not valid UTF-8: {0}")]
    ExecutablePathNotUtf8(PathBuf),
    #[error("spawned process did not expose a PID")]
    MissingChildPid,
    #[error("spawned process PID is outside the POSIX range: {0}")]
    InvalidChildPid(u32),
    #[error("owned process no longer exists")]
    ProcessNoLongerExists,
    #[error("owned process changed its process-group or OS-session identity")]
    ProcessIdentityChanged,
    #[error("runner heartbeat clock regressed")]
    HeartbeatClockRegressed,
    #[error("runner heartbeat sequence is exhausted")]
    HeartbeatSequenceExhausted,
    #[error("execution identity verification failed: {0}")]
    Identity(#[from] IdentityError),
    #[error("runner protocol operation failed: {0}")]
    Protocol(ProtocolError),
    #[error("runner process OS operation failed: {0}")]
    Os(String),
    #[error("runner process I/O failed: {0}")]
    Io(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{ExecutionIdentityRequest, ExecutionMode};
    use chrono::Duration;
    use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
    use std::sync::atomic::{AtomicU64, Ordering};
    use tokio::time::{Duration as TokioDuration, sleep};

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

    struct TestProject {
        root: PathBuf,
        workspace: PathBuf,
    }

    impl TestProject {
        fn new() -> Self {
            let sequence = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "focusa-posix-runner-{}-{sequence}",
                std::process::id()
            ));
            let workspace = root.join("worktree");
            fs::create_dir_all(&workspace).expect("runner fixture should be created");
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
                .expect("fixture root should be private");
            fs::set_permissions(&workspace, fs::Permissions::from_mode(0o700))
                .expect("fixture workspace should be private");
            Self { root, workspace }
        }

        fn context(&self, daemon_uid: u32) -> VerifiedExecutionContext {
            let current = OsIdentity::current().expect("current identity should resolve");
            VerifiedExecutionContext::verify(&ExecutionIdentityRequest {
                daemon_uid,
                execution_user: current.user_name,
                execution_uid: current.uid,
                project_root: self.root.clone(),
                project_identity_ref: "project:runner-test".into(),
                workspace_root: self.workspace.clone(),
            })
            .expect("test project owner should verify")
        }
    }

    impl Drop for TestProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn spawn_request(script: &str) -> PosixSpawnRequest {
        PosixSpawnRequest {
            session_id: SilentSessionId::new(),
            run_id: SilentSessionRunId::new(),
            generation: 1,
            executable: PathBuf::from("/bin/sh"),
            argv: vec![OsString::from("-c"), OsString::from(script)],
            env: BTreeMap::new(),
            launch_manifest_sha256: "a".repeat(64),
        }
    }

    fn adoption_expectation(record: &ActiveRunRecord, now: DateTime<Utc>) -> AdoptionExpectation {
        AdoptionExpectation {
            daemon_id: "daemon:test".into(),
            runner_id: record.runner_id.clone(),
            session_id: record.session_id,
            run_id: record.run_id,
            generation: record.generation,
            project_root: record.project_root.clone(),
            project_identity_ref: record.project_identity_ref.clone(),
            workspace_root: record.workspace_root.clone(),
            execution_user: record.execution_user.clone(),
            execution_uid: record.execution_uid,
            executable_ref: record.executable_ref.clone(),
            launch_manifest_sha256: record.launch_manifest_sha256.clone(),
            expected_process_instance_id: Some(record.process_tree.process_instance_id.clone()),
            heartbeat_fresh_after: now - Duration::seconds(1),
        }
    }

    async fn wait_for_file(path: &Path) {
        for _ in 0..100 {
            if path.exists() {
                return;
            }
            sleep(TokioDuration::from_millis(10)).await;
        }
        panic!("runner child did not create {}", path.display());
    }

    #[tokio::test]
    async fn owner_runner_spawns_owned_process_group_and_supports_live_adoption() {
        let project = TestProject::new();
        let current = OsIdentity::current().expect("current identity should resolve");
        let context = project.context(current.uid.wrapping_add(1));
        assert_eq!(context.mode(), ExecutionMode::PerUserRunner);
        let proof = context
            .authorize_mutation_path("owner-proof")
            .expect("proof path should be workspace-scoped");
        let script = "printf '%s\\n%s\\n' \"$(/usr/bin/id -u)\" \"$PWD\" > owner-proof; /bin/sleep 30 & wait";
        let request = spawn_request(script);
        let run_id = request.run_id;
        let now = Utc::now();
        let mut supervisor = PosixProcessSupervisor::for_current_user("runner:test")
            .expect("current-user runner should initialize");
        let record = supervisor
            .spawn(&context, request, now)
            .expect("verified owner runner should spawn");

        wait_for_file(proof.as_path()).await;
        let proof_text = fs::read_to_string(proof.as_path()).expect("proof should be readable");
        let mut proof_lines = proof_text.lines();
        assert_eq!(proof_lines.next(), Some(current.uid.to_string().as_str()));
        assert_eq!(proof_lines.next(), context.workspace_root().to_str());
        assert_eq!(
            fs::metadata(proof.as_path())
                .expect("proof metadata should exist")
                .uid(),
            current.uid
        );
        assert_eq!(
            record.process_tree.process_group_id,
            i64::from(record.process_tree.pid)
        );

        let heartbeat_at = now + Duration::seconds(1);
        let heartbeat = supervisor
            .heartbeat(heartbeat_at)
            .expect("live process should heartbeat");
        assert_eq!(heartbeat.heartbeat.sequence, 1);
        assert_eq!(
            heartbeat.heartbeat.active_runs,
            vec![{
                let mut refreshed = record.clone();
                refreshed.heartbeat_at = heartbeat_at;
                refreshed
            }]
        );
        assert!(heartbeat.exited_runs.is_empty());

        let expectation = adoption_expectation(&record, heartbeat_at);
        let decision = supervisor
            .evaluate_adoption(&expectation, heartbeat_at)
            .expect("adoption evaluation should run");
        assert!(decision.accepted);
        assert!(decision.signed_runner_record_ref.is_some());

        let mut wrong_workspace = expectation;
        wrong_workspace.workspace_root = project.root.clone();
        let decision = supervisor
            .evaluate_adoption(&wrong_workspace, heartbeat_at)
            .expect("mismatched adoption should be typed");
        assert_eq!(
            decision.rejection,
            Some(AdoptionRejection::WorkspaceMismatch)
        );

        let exit = supervisor
            .force_terminate(run_id, Utc::now())
            .await
            .expect("owned process group should terminate");
        assert_eq!(exit.run_id, run_id);
        assert_eq!(exit.signal, Some(Signal::SIGKILL as i32));
        assert_eq!(supervisor.active_run_count(), 0);
    }

    #[tokio::test]
    async fn embedded_runner_reaps_exit_and_heartbeat_never_claims_it_active() {
        let project = TestProject::new();
        let current = OsIdentity::current().expect("current identity should resolve");
        let context = project.context(current.uid);
        assert_eq!(context.mode(), ExecutionMode::EmbeddedSameUser);
        let request = spawn_request("exit 7");
        let run_id = request.run_id;
        let mut supervisor = PosixProcessSupervisor::for_current_user("runner:embedded")
            .expect("embedded runner should initialize");
        supervisor
            .spawn(&context, request, Utc::now())
            .expect("embedded process should spawn");

        let mut observed_exit = None;
        for sequence in 1..=100 {
            let snapshot = supervisor
                .heartbeat(Utc::now() + Duration::milliseconds(sequence))
                .expect("heartbeat should sample process");
            if let Some(exit) = snapshot.exited_runs.into_iter().next() {
                assert!(snapshot.heartbeat.active_runs.is_empty());
                observed_exit = Some(exit);
                break;
            }
            sleep(TokioDuration::from_millis(10)).await;
        }
        let exit = observed_exit.expect("short process should be reaped");
        assert_eq!(exit.run_id, run_id);
        assert_eq!(exit.exit_code, Some(7));
        assert_eq!(supervisor.active_run_count(), 0);
    }

    #[tokio::test]
    async fn workspace_swap_is_rejected_at_spawn_revalidation() {
        let project = TestProject::new();
        let current = OsIdentity::current().expect("current identity should resolve");
        let context = project.context(current.uid);
        fs::remove_dir(&project.workspace).expect("empty workspace should be removable");
        symlink(&project.root, &project.workspace).expect("workspace swap should be created");
        let mut supervisor = PosixProcessSupervisor::for_current_user("runner:swap-test")
            .expect("runner should initialize");
        let error = supervisor
            .spawn(&context, spawn_request("exit 0"), Utc::now())
            .expect_err("symlink swap must fail closed before spawn");
        assert!(matches!(
            error,
            SupervisorError::Identity(IdentityError::SymlinkRejected(_))
        ));
        assert_eq!(supervisor.active_run_count(), 0);
    }
}
