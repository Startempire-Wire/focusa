//! Canonical Linux system-daemon lifecycle for Rust install and update paths.
//!
//! The lifecycle fails closed on an operator halt or any unmanaged/duplicate
//! `focusa-daemon` process. It atomically stages the systemd unit, keeps a
//! durable rollback copy until the new runtime passes process and HTTP health
//! checks, and never mutates the preserved SQLite/lease state directory.

#[path = "system_service_callgraph.rs"]
mod callgraph_probe;
#[path = "system_service_process.rs"]
mod process;

use anyhow::{Context, Result, anyhow, bail};
#[allow(deprecated)]
use nix::fcntl::{FlockArg, flock};
use serde_json::Value;
use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use process::{
    ensure_linux_root, ensure_operator_start_allowed, inspect_processes,
    inspect_processes_before_update_restart, systemctl, validate_effective_system_service,
};

pub(crate) const SYSTEM_SERVICE_NAME: &str = "focusa-daemon.service";
pub(crate) const SYSTEM_UNIT_PATH: &str = "/etc/systemd/system/focusa-daemon.service";
pub(crate) const SYSTEM_STATE_DIR: &str = "/usr/local/lib/focusa";
const SYSTEM_DAEMON_PATH: &str = "/usr/local/bin/focusa-daemon";
const DEPLOY_LOCK_PATH: &str = "/run/lock/focusa-daemon-install.lock";

pub(crate) struct SystemDeployLock {
    _file: File,
}

#[derive(Debug)]
pub(crate) struct SystemServiceTransaction {
    unit_path: PathBuf,
    staged_path: PathBuf,
    backup_path: PathBuf,
    prior_unit: Option<Vec<u8>>,
    prior_active: bool,
    settled: bool,
}

pub(crate) fn acquire_system_deploy_lock(system_bin: &Path) -> Result<SystemDeployLock> {
    let path = if system_bin == Path::new("/usr/local/bin") {
        PathBuf::from(DEPLOY_LOCK_PATH)
    } else {
        system_bin.join(".focusa-daemon-install.lock")
    };
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&path)
        .with_context(|| format!("open canonical daemon deploy lock {}", path.display()))?;
    #[allow(deprecated)]
    let lock_result = flock(file.as_raw_fd(), FlockArg::LockExclusiveNonblock);
    lock_result.map_err(|error| {
        anyhow!(
            "another canonical daemon install is active (lock={}): {error}",
            path.display()
        )
    })?;
    Ok(SystemDeployLock { _file: file })
}

pub(crate) fn render_system_unit(daemon_path: &Path, state_dir: &Path) -> String {
    format!(
        "[Unit]\n\
Description=Focusa Metacognition Daemon\n\
After=network-online.target\n\
Wants=network-online.target\n\
\n\
[Service]\n\
Type=simple\n\
ExecStart={}\n\
WorkingDirectory={}\n\
Environment=FOCUSA_HOME={}\n\
Environment=FOCUSA_DATA_DIR={}\n\
Restart=on-failure\n\
RestartSec=3\n\
TimeoutStartSec=20\n\
TimeoutStopSec=20\n\
MemoryHigh=2G\n\
MemoryMax=3G\n\
ProtectSystem=strict\n\
ReadWritePaths={}\n\
PrivateTmp=true\n\
NoNewPrivileges=true\n\
\n\
[Install]\n\
WantedBy=multi-user.target\n",
        daemon_path.display(),
        state_dir.display(),
        state_dir.display(),
        state_dir.display(),
        state_dir.display(),
    )
}

pub(crate) fn is_canonical_system_daemon(path: &Path) -> bool {
    path == Path::new(SYSTEM_DAEMON_PATH)
}

pub(crate) fn preflight_system_install() -> Result<()> {
    ensure_linux_root()?;
    ensure_operator_start_allowed()?;
    inspect_processes(Path::new(SYSTEM_DAEMON_PATH))?;
    Ok(())
}

pub(crate) fn restart_existing_system_service() -> Result<()> {
    ensure_linux_root()?;
    ensure_operator_start_allowed()?;
    inspect_processes_before_update_restart(Path::new(SYSTEM_DAEMON_PATH))?;
    systemctl(
        &["restart", SYSTEM_SERVICE_NAME],
        "restart canonical system service",
    )?;
    let inventory = inspect_processes(Path::new(SYSTEM_DAEMON_PATH))?;
    if !inventory.active {
        bail!("canonical system service is not active after restart");
    }
    Ok(())
}

pub(crate) fn prepare_system_service() -> Result<SystemServiceTransaction> {
    ensure_linux_root()?;
    ensure_operator_start_allowed()?;
    let prior_inventory = inspect_processes(Path::new(SYSTEM_DAEMON_PATH))?;
    let unit_path = PathBuf::from(SYSTEM_UNIT_PATH);
    let state_dir = PathBuf::from(SYSTEM_STATE_DIR);
    let transaction = std::process::id();
    let staged_path = unit_path.with_extension(format!("service.staged-{transaction}"));
    let backup_path = unit_path.with_extension(format!("service.rollback-{transaction}"));
    if staged_path.exists() || backup_path.exists() {
        bail!(
            "stale system service transaction exists: staged={} backup={}",
            staged_path.display(),
            backup_path.display()
        );
    }

    let prior_unit = match std::fs::read(&unit_path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error).context(format!("read {}", unit_path.display())),
    };
    if let Some(bytes) = &prior_unit {
        write_new_file(&backup_path, bytes)?;
    }
    std::fs::create_dir_all(&state_dir)
        .with_context(|| format!("create preserved state root {}", state_dir.display()))?;
    let unit = render_system_unit(Path::new(SYSTEM_DAEMON_PATH), &state_dir);
    if let Err(error) = write_new_file(&staged_path, unit.as_bytes()).and_then(|()| {
        std::fs::rename(&staged_path, &unit_path)
            .with_context(|| format!("promote canonical unit {}", unit_path.display()))
    }) {
        let _ = std::fs::remove_file(&staged_path);
        let _ = std::fs::remove_file(&backup_path);
        return Err(error);
    }
    if let Err(error) = systemctl(&["daemon-reload"], "reload canonical system service") {
        restore_unit_bytes(&unit_path, &staged_path, prior_unit.as_deref())?;
        let _ = std::fs::remove_file(&backup_path);
        return Err(error);
    }

    let transaction = SystemServiceTransaction {
        unit_path,
        staged_path,
        backup_path,
        prior_unit,
        prior_active: prior_inventory.active,
        settled: false,
    };
    validate_effective_system_service()?;
    Ok(transaction)
}

impl SystemServiceTransaction {
    pub(crate) fn activate_and_verify(&mut self, expected_version: &str) -> Result<()> {
        systemctl(
            &["enable", "--now", SYSTEM_SERVICE_NAME],
            "enable and start canonical system service",
        )?;
        if self.prior_active {
            systemctl(
                &["restart", SYSTEM_SERVICE_NAME],
                "restart canonical system service",
            )?;
        }
        let inventory = inspect_processes(Path::new(SYSTEM_DAEMON_PATH))?;
        if !inventory.active {
            bail!("canonical system service is not active after activation");
        }
        verify_health(expected_version)?;
        Ok(())
    }

    fn restore_prior_state(&mut self) -> Result<()> {
        restore_unit_bytes(
            &self.unit_path,
            &self.staged_path,
            self.prior_unit.as_deref(),
        )?;
        systemctl(&["daemon-reload"], "reload restored system service")?;
        if self.prior_active {
            systemctl(
                &["restart", SYSTEM_SERVICE_NAME],
                "restart restored system service",
            )?;
            let inventory = inspect_processes(Path::new(SYSTEM_DAEMON_PATH))?;
            if !inventory.active {
                bail!("restored canonical system service is not active");
            }
        } else {
            let _ = systemctl(
                &["stop", SYSTEM_SERVICE_NAME],
                "stop restored inactive service",
            );
        }
        let _ = std::fs::remove_file(&self.backup_path);
        self.settled = true;
        Ok(())
    }

    pub(crate) fn commit(mut self) {
        self.settled = true;
        if self.backup_path.exists()
            && let Err(error) = std::fs::remove_file(&self.backup_path)
        {
            eprintln!(
                "warning: canonical service committed but rollback cleanup {} failed: {error}",
                self.backup_path.display()
            );
        }
    }
}

impl Drop for SystemServiceTransaction {
    fn drop(&mut self) {
        if !self.settled
            && let Err(error) = self.restore_prior_state()
        {
            eprintln!(
                "warning: automatic system service rollback failed: {error}; retained rollback={}",
                self.backup_path.display()
            );
        }
    }
}

fn verify_health(expected_version: &str) -> Result<()> {
    let url = std::env::var("FOCUSA_DAEMON_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8787/v1/health".into());
    let mut last_error = String::new();
    for _ in 0..40 {
        let output = Command::new("curl")
            .args(["-fsS", "--max-time", "2", &url])
            .output()
            .context("run canonical daemon health probe")?;
        if output.status.success() {
            match serde_json::from_slice::<Value>(&output.stdout) {
                Ok(payload)
                    if payload.get("version").and_then(Value::as_str) == Some(expected_version) =>
                {
                    callgraph_probe::verify(&url)?;
                    return Ok(());
                }
                Ok(payload) => {
                    last_error = format!(
                        "health version mismatch: expected {expected_version}, got {}",
                        payload
                            .get("version")
                            .and_then(Value::as_str)
                            .unwrap_or("<missing>")
                    );
                }
                Err(error) => last_error = format!("health response is not JSON: {error}"),
            }
        } else {
            last_error = String::from_utf8_lossy(&output.stderr)
                .trim()
                .chars()
                .take(240)
                .collect();
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    bail!("canonical daemon health verification failed: {last_error}")
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .with_context(|| format!("create {}", path.display()))?;
    file.write_all(bytes)
        .with_context(|| format!("write {}", path.display()))?;
    file.sync_all()
        .with_context(|| format!("sync {}", path.display()))?;
    Ok(())
}

fn restore_unit_bytes(unit_path: &Path, staged_path: &Path, prior: Option<&[u8]>) -> Result<()> {
    let _ = std::fs::remove_file(staged_path);
    match prior {
        Some(bytes) => {
            let rollback_stage =
                unit_path.with_extension(format!("service.restore-{}", std::process::id()));
            let _ = std::fs::remove_file(&rollback_stage);
            write_new_file(&rollback_stage, bytes)?;
            std::fs::rename(&rollback_stage, unit_path)
                .with_context(|| format!("restore {}", unit_path.display()))?;
        }
        None => match std::fs::remove_file(unit_path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error).context(format!("remove {}", unit_path.display())),
        },
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_unit_binds_one_binary_and_one_state_root() {
        let unit = render_system_unit(
            Path::new("/usr/local/bin/focusa-daemon"),
            Path::new("/usr/local/lib/focusa"),
        );
        assert!(unit.contains("ExecStart=/usr/local/bin/focusa-daemon"));
        assert!(unit.contains("WorkingDirectory=/usr/local/lib/focusa"));
        assert!(unit.contains("Environment=FOCUSA_HOME=/usr/local/lib/focusa"));
        assert!(unit.contains("Environment=FOCUSA_DATA_DIR=/usr/local/lib/focusa"));
        assert_eq!(unit.matches("ExecStart=").count(), 1);
        assert!(!unit.contains("/home/"));
        assert!(unit.contains("MemoryHigh=2G"));
        assert!(unit.contains("MemoryMax=3G"));
    }
}
