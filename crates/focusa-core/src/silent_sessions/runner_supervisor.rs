//! Spec133 §13.1/§13.2 runner supervision (daemon side).
//!
//! The daemon owns exactly one runner process per identity and launches it from
//! a typed argument vector, never a shell string (§13.3). Provisioning lives in
//! `runner_provisioning`; this module only resolves, launches, probes and
//! terminates the process tree.

use anyhow::{bail, Context};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use super::runner_provisioning::RunnerPaths;
use super::RunnerIdentity;

pub const RUNNER_BINARY_NAME: &str = "focusa-session-runner";
const DEFAULT_READY_TIMEOUT: Duration = Duration::from_secs(10);
const TERMINATE_GRACE: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct RunnerLaunchConfig {
    pub paths: RunnerPaths,
    pub identity: RunnerIdentity,
    pub owner_uid: u32,
    pub binary: Option<PathBuf>,
    pub ready_timeout: Duration,
}

impl RunnerLaunchConfig {
    pub fn new(paths: RunnerPaths, identity: RunnerIdentity, owner_uid: u32) -> Self {
        Self {
            paths,
            identity,
            owner_uid,
            binary: None,
            ready_timeout: DEFAULT_READY_TIMEOUT,
        }
    }
}

/// Resolve the runner executable: explicit override, else the sibling of the
/// running daemon (install layout puts both in the same directory).
pub fn resolve_binary(explicit: Option<&Path>) -> anyhow::Result<PathBuf> {
    if let Some(path) = explicit {
        if !path.is_file() {
            bail!("runner executable does not exist: {}", path.display());
        }
        return Ok(path.to_path_buf());
    }
    let exe = std::env::current_exe().context("resolve current executable")?;
    let dir = exe
        .parent()
        .context("current executable has no parent directory")?;
    let candidate = dir.join(format!(
        "{RUNNER_BINARY_NAME}{}",
        std::env::consts::EXE_SUFFIX
    ));
    if !candidate.is_file() {
        bail!(
            "runner executable not found next to daemon: {}",
            candidate.display()
        );
    }
    Ok(candidate)
}

/// Exact argument vector passed to the runner. Kept pure so it is unit-testable
/// without spawning a process (§13.3: typed argv, never shell composition).
pub fn launch_argv(binary: &Path, config: &RunnerLaunchConfig) -> Vec<String> {
    vec![
        binary.display().to_string(),
        "--socket".into(),
        config.paths.socket.display().to_string(),
        "--key_file".into(),
        config.paths.key_file.display().to_string(),
        "--nonce_ledger".into(),
        config.paths.nonce_ledger.display().to_string(),
        "--secret_dir".into(),
        config.paths.secret_dir.display().to_string(),
        "--principal_id".into(),
        config.identity.principal_id.clone(),
        "--owner_os_user".into(),
        config.identity.os_user.clone(),
        "--socket_scope".into(),
        config.identity.socket_scope.clone(),
    ]
}

pub struct RunnerProcess {
    child: Child,
    pub pid: u32,
    pub socket: PathBuf,
}

impl RunnerProcess {
    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Ask the runner to exit, then escalate. Signals the process group on
    /// POSIX so no orphaned children survive (§13.1/§13.5).
    pub fn terminate(&mut self) -> anyhow::Result<()> {
        #[cfg(unix)]
        unsafe {
            libc::kill(-(self.pid as i32), libc::SIGTERM);
        }
        #[cfg(windows)]
        {
            let _ = self.child.kill();
        }
        let deadline = Instant::now() + TERMINATE_GRACE;
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => return Ok(()),
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(50))
                }
                Ok(None) => break,
                Err(error) => return Err(error).context("poll runner exit"),
            }
        }
        #[cfg(unix)]
        unsafe {
            libc::kill(-(self.pid as i32), libc::SIGKILL);
        }
        self.child.kill().context("force kill runner")?;
        self.child.wait().context("reap runner")?;
        Ok(())
    }
}

/// Provision material, launch the runner and wait until its socket is ready.
pub fn launch(config: &RunnerLaunchConfig) -> anyhow::Result<RunnerProcess> {
    config
        .paths
        .ensure_material(config.owner_uid)
        .context("provision runner material")?;
    let binary = resolve_binary(config.binary.as_deref())?;
    let argv = launch_argv(&binary, config);
    let mut command = Command::new(&argv[0]);
    command.args(&argv[1..]);
    command.current_dir(&config.paths.root);
    command.stdin(Stdio::null());
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let child = command
        .spawn()
        .with_context(|| format!("spawn runner {}", binary.display()))?;
    let pid = child.id();
    let mut process = RunnerProcess {
        child,
        pid,
        socket: config.paths.socket.clone(),
    };
    wait_ready(&mut process, config.ready_timeout)?;
    Ok(process)
}

/// Readiness is defined by the transport the runner actually serves.
fn wait_ready(process: &mut RunnerProcess, timeout: Duration) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        let deadline = Instant::now() + timeout;
        loop {
            if let Ok(meta) = std::fs::metadata(&process.socket) {
                if meta.file_type().is_socket() {
                    return Ok(());
                }
            }
            if !process.is_alive() {
                bail!("runner exited before its socket became ready");
            }
            if Instant::now() >= deadline {
                bail!(
                    "runner socket not ready within {:?}: {}",
                    timeout,
                    process.socket.display()
                );
            }
            std::thread::sleep(Duration::from_millis(25));
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (process, timeout);
        bail!("runner readiness probe requires the named-pipe transport (Spec 133 P2)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> RunnerLaunchConfig {
        let root = std::env::temp_dir().join(format!(
            "focusa-runner-sup-{}",
            uuid::Uuid::now_v7()
        ));
        let paths = RunnerPaths::resolve(&root, "principal-1", "scope-1").unwrap();
        let identity = RunnerPaths::identity("verious", "principal-1", "scope-1");
        RunnerLaunchConfig::new(paths, identity, 0)
    }

    #[test]
    fn argv_is_typed_and_carries_all_seven_flags() {
        let cfg = config();
        let argv = launch_argv(Path::new("/opt/focusa/focusa-session-runner"), &cfg);
        for flag in [
            "--socket",
            "--key_file",
            "--nonce_ledger",
            "--secret_dir",
            "--principal_id",
            "--owner_os_user",
            "--socket_scope",
        ] {
            assert!(argv.iter().any(|arg| arg == flag), "missing {flag}");
        }
        assert_eq!(argv[0], "/opt/focusa/focusa-session-runner");
        assert!(!argv.iter().any(|arg| arg.contains("&&") || arg.contains(';')));
    }

    #[test]
    fn explicit_binary_must_exist() {
        let missing = std::env::temp_dir().join("focusa-no-such-runner");
        assert!(resolve_binary(Some(&missing)).is_err());
    }
}
