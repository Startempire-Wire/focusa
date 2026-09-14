//! Private Linux systemd/process inspection for the canonical system lifecycle.

use super::{SYSTEM_DAEMON_PATH, SYSTEM_SERVICE_NAME, SYSTEM_STATE_DIR};
use anyhow::{Context, Result, anyhow, bail};
use std::path::Path;
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ProcessInventory {
    pub(super) active: bool,
    main_pid: Option<u32>,
    named_pids: Vec<u32>,
}

pub(super) fn ensure_linux_root() -> Result<()> {
    let output = Command::new("id")
        .arg("-u")
        .output()
        .context("resolve installer uid")?;
    if !output.status.success() || String::from_utf8_lossy(&output.stdout).trim() != "0" {
        bail!("authoritative system installation requires Linux root");
    }
    Ok(())
}

pub(super) fn ensure_operator_start_allowed() -> Result<()> {
    let value = systemctl_property("RefuseManualStart")?.unwrap_or_default();
    if value.trim().eq_ignore_ascii_case("yes") {
        bail!(
            "operator halt is active (RefuseManualStart=yes); canonical installation will not alter or bypass it"
        );
    }
    Ok(())
}

pub(super) fn validate_effective_system_service() -> Result<()> {
    let expected_state = SYSTEM_STATE_DIR;
    let working_directory = systemctl_property("WorkingDirectory")?.unwrap_or_default();
    if working_directory.trim() != expected_state {
        bail!(
            "effective systemd WorkingDirectory is noncanonical: expected={expected_state} observed={}",
            working_directory.trim()
        );
    }
    let exec_start = systemctl_property("ExecStart")?.unwrap_or_default();
    if !exec_start.contains(SYSTEM_DAEMON_PATH) {
        bail!(
            "effective systemd ExecStart is noncanonical: expected={SYSTEM_DAEMON_PATH} observed={}",
            exec_start.trim()
        );
    }
    let environment = systemctl_property("Environment")?.unwrap_or_default();
    let entries = shlex::split(&environment)
        .ok_or_else(|| anyhow!("effective systemd Environment could not be parsed"))?;
    for key in ["FOCUSA_HOME", "FOCUSA_DATA_DIR"] {
        let values = entries
            .iter()
            .filter_map(|entry| entry.strip_prefix(&format!("{key}=")))
            .collect::<Vec<_>>();
        if values.as_slice() != [expected_state] {
            bail!(
                "effective systemd {key} must bind exactly one canonical state root: expected={expected_state} observed={values:?}"
            );
        }
    }
    if entries
        .iter()
        .any(|entry| entry == "FOCUSA_DEV_MODE=1" || entry == "FOCUSA_DEV_MODE=true")
    {
        bail!("effective production systemd service enables FOCUSA_DEV_MODE");
    }
    Ok(())
}

pub(super) fn inspect_processes(expected_executable: &Path) -> Result<ProcessInventory> {
    inspect_processes_inner(expected_executable, false)
}

pub(super) fn inspect_processes_before_update_restart(
    expected_executable: &Path,
) -> Result<ProcessInventory> {
    inspect_processes_inner(expected_executable, true)
}

fn inspect_processes_inner(
    expected_executable: &Path,
    allow_service_owned_replaced_inode: bool,
) -> Result<ProcessInventory> {
    let active = systemctl_status(&["is-active", "--quiet", SYSTEM_SERVICE_NAME])?;
    let main_pid = systemctl_property("MainPID")?
        .and_then(|value| value.trim().parse::<u32>().ok())
        .filter(|pid| *pid > 0);
    let mut named_pids = focusa_daemon_pids()?;
    named_pids.sort_unstable();
    let inventory = ProcessInventory {
        active,
        main_pid,
        named_pids,
    };
    if let Some(pid) = validate_process_inventory(&inventory)? {
        let running = std::fs::read_link(format!("/proc/{pid}/exe"))
            .with_context(|| format!("resolve executable for canonical daemon pid {pid}"))?;
        if !executable_matches(
            &running,
            expected_executable,
            allow_service_owned_replaced_inode,
        ) {
            bail!(
                "canonical daemon executable mismatch: pid={pid} expected={} observed={}",
                expected_executable.display(),
                running.display()
            );
        }
    }
    Ok(inventory)
}

fn executable_matches(running: &Path, expected: &Path, allow_replaced_inode: bool) -> bool {
    running == expected
        || (allow_replaced_inode
            && running.to_string_lossy() == format!("{} (deleted)", expected.display()))
}

fn validate_process_inventory(inventory: &ProcessInventory) -> Result<Option<u32>> {
    if !inventory.active && !inventory.named_pids.is_empty() {
        bail!(
            "unmanaged focusa-daemon process(es) detected while system service is inactive: {:?}",
            inventory.named_pids
        );
    }
    if !inventory.active {
        return Ok(None);
    }
    let pid = inventory
        .main_pid
        .ok_or_else(|| anyhow!("active system service has no MainPID"))?;
    if inventory.named_pids.as_slice() != [pid] {
        bail!(
            "canonical system service must own exactly one focusa-daemon process: main_pid={pid} observed={:?}",
            inventory.named_pids
        );
    }
    Ok(Some(pid))
}

fn focusa_daemon_pids() -> Result<Vec<u32>> {
    let mut pids = Vec::new();
    for entry in std::fs::read_dir("/proc").context("read /proc for daemon inventory")? {
        let entry = entry?;
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse().ok())
        else {
            continue;
        };
        let comm = match std::fs::read_to_string(entry.path().join("comm")) {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => continue,
            Err(error) => return Err(error).context(format!("read /proc/{pid}/comm")),
        };
        if comm.trim() == "focusa-daemon" {
            pids.push(pid);
        }
    }
    Ok(pids)
}

pub(super) fn systemctl(args: &[&str], operation: &str) -> Result<()> {
    let output = Command::new("systemctl")
        .args(args)
        .output()
        .with_context(|| operation.to_string())?;
    if output.status.success() {
        return Ok(());
    }
    let detail = String::from_utf8_lossy(&output.stderr)
        .trim()
        .chars()
        .take(240)
        .collect::<String>();
    bail!(
        "{operation} failed: exit={}{}",
        output.status.code().unwrap_or(-1),
        if detail.is_empty() {
            String::new()
        } else {
            format!(" ({detail})")
        }
    )
}

fn systemctl_property(property: &str) -> Result<Option<String>> {
    let output = Command::new("systemctl")
        .args([
            "show",
            SYSTEM_SERVICE_NAME,
            &format!("--property={property}"),
            "--value",
        ])
        .output()
        .context("query canonical system service")?;
    if output.status.success() {
        return Ok(Some(String::from_utf8_lossy(&output.stdout).into_owned()));
    }
    let detail = String::from_utf8_lossy(&output.stderr);
    if detail.contains("could not be found") || detail.contains("not-found") {
        return Ok(None);
    }
    bail!(
        "query canonical system service failed: exit={} ({})",
        output.status.code().unwrap_or(-1),
        detail.trim()
    )
}

fn systemctl_status(args: &[&str]) -> Result<bool> {
    let output = Command::new("systemctl")
        .args(args)
        .output()
        .context("inspect canonical system service status")?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(3 | 4) => Ok(false),
        _ => bail!(
            "inspect canonical system service status failed: exit={} ({})",
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stderr).trim()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_inventory_rejects_unmanaged_duplicate_and_missing_main_pid() {
        let unmanaged = ProcessInventory {
            active: false,
            main_pid: None,
            named_pids: vec![41],
        };
        assert!(validate_process_inventory(&unmanaged).is_err());

        let duplicate = ProcessInventory {
            active: true,
            main_pid: Some(41),
            named_pids: vec![41, 42],
        };
        assert!(validate_process_inventory(&duplicate).is_err());

        let missing = ProcessInventory {
            active: true,
            main_pid: None,
            named_pids: vec![41],
        };
        assert!(validate_process_inventory(&missing).is_err());

        let exact = ProcessInventory {
            active: true,
            main_pid: Some(41),
            named_pids: vec![41],
        };
        assert_eq!(validate_process_inventory(&exact).unwrap(), Some(41));
    }

    #[test]
    fn replaced_inode_is_allowed_only_for_the_owned_update_restart_boundary() {
        let expected = Path::new("/usr/local/bin/focusa-daemon");
        let replaced = Path::new("/usr/local/bin/focusa-daemon (deleted)");
        assert!(executable_matches(expected, expected, false));
        assert!(!executable_matches(replaced, expected, false));
        assert!(executable_matches(replaced, expected, true));
        assert!(!executable_matches(
            Path::new("/tmp/focusa-daemon (deleted)"),
            expected,
            true
        ));
    }
}
