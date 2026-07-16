//! First-class Focusa daemon service install (Spec112 §5A, focusa-foyr).
//!
//! Replaces the shell-only install-focusa.sh service units with a Rust core that
//! probes the host service manager, renders the correct unit/LaunchAgent file
//! from the same binary the CLI is running, and writes it to the conventional
//! per-user path. The shell installer is reduced to a thin bootstrap that
//! delegates to `focusa install-service`.

use anyhow::{Context, Result, anyhow};
use clap::Args;
use serde::Serialize;
use std::path::{Path, PathBuf};

const SERVICE_NAME: &str = "focusa-daemon";
const LAUNCHD_LABEL: &str = "com.startempire.focusa-daemon";

#[derive(Args, Debug)]
pub struct InstallServiceArgs {
    /// Skip actually enabling/loading the service.
    #[arg(long)]
    pub no_enable: bool,
    /// Print machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Serialize)]
pub struct InstallServiceReport {
    pub manager: String,
    pub service_name: String,
    pub binary_path: String,
    pub unit_path: Option<String>,
    pub enabled: bool,
    pub loaded: bool,
    pub dry_run: bool,
    pub notes: Vec<String>,
    pub recovery_hint: Option<String>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum ServiceManager {
    SystemdUser,
    LaunchdUser,
    None,
}

fn detect_manager() -> ServiceManager {
    if cfg!(target_os = "macos") {
        if which("launchctl").is_some() {
            return ServiceManager::LaunchdUser;
        }
    } else if which("systemctl").is_some() {
        return ServiceManager::SystemdUser;
    }
    ServiceManager::None
}

fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for entry in std::env::split_paths(&path) {
        let candidate = entry.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn find_self_binary() -> Result<PathBuf> {
    std::env::current_exe().context("could not resolve focusa binary path")
}

fn find_daemon_binary() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("could not resolve focusa binary path")?;
    let dir = exe.parent().unwrap_or_else(|| Path::new("."));
    let candidate = dir.join("focusa-daemon");
    if candidate.is_file() {
        return Ok(candidate);
    }
    which("focusa-daemon").ok_or_else(|| anyhow!(
        "focusa-daemon binary not found next to focusa CLI and not on PATH; install it from the same release before installing the service"
    ))
}

fn render_systemd_unit(binary: &Path) -> String {
    format!(
        "[Unit]
Description=Focusa Daemon
After=network-online.target

[Service]
ExecStart={}
Restart=on-failure
RestartSec=3
WorkingDirectory={}

[Install]
WantedBy=default.target
",
        binary.display(),
        binary
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("/usr/local/bin")
    )
}

fn render_launchd_plist(binary: &Path, log_dir: &Path) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>{label}</string>
  <key>ProgramArguments</key><array><string>{bin}</string></array>
  <key>RunAtLoad</key><true/><key>KeepAlive</key><true/>
  <key>StandardOutPath</key><string>{out}</string>
  <key>StandardErrorPath</key><string>{err}</string>
</dict></plist>
"#,
        label = LAUNCHD_LABEL,
        bin = binary.display(),
        out = log_dir.join("focusa-daemon.out.log").display(),
        err = log_dir.join("focusa-daemon.err.log").display(),
    )
}

fn run_systemd_user(
    unit_path: &Path,
    dry_run: bool,
    no_enable: bool,
) -> Result<(bool, Vec<String>)> {
    let mut notes = Vec::new();
    if let Some(parent) = unit_path.parent() {
        if !dry_run {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
    }
    let _ = unit_path; // unit content rendered by caller
    if dry_run || no_enable {
        notes.push("service file written but not enabled (dry-run or --no-enable)".to_string());
        return Ok((false, notes));
    }
    let status = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            return Err(anyhow!(
                "systemctl --user daemon-reload failed: exit={}",
                s.code().unwrap_or(-1)
            ));
        }
        Err(e) => return Err(anyhow!("systemctl --user daemon-reload not runnable: {e}")),
    }
    let status = std::process::Command::new("systemctl")
        .args(["--user", "enable", "--now", SERVICE_NAME])
        .status();
    match status {
        Ok(s) if s.success() => Ok((true, notes)),
        Ok(s) => Err(anyhow!(
            "systemctl --user enable --now focusa-daemon failed: exit={}",
            s.code().unwrap_or(-1)
        )),
        Err(e) => Err(anyhow!(
            "systemctl --user enable --now focusa-daemon not runnable: {e}"
        )),
    }
}

pub(crate) fn restart_launchd_after_commit() -> Result<()> {
    let uid = std::process::Command::new("id")
        .arg("-u")
        .output()
        .context("resolve uid for launchctl kickstart")?;
    if !uid.status.success() {
        return Err(anyhow!("id -u failed while restarting LaunchAgent"));
    }
    let uid = String::from_utf8_lossy(&uid.stdout).trim().to_string();
    let service_target = format!("gui/{uid}/{LAUNCHD_LABEL}");
    let restart = std::process::Command::new("launchctl")
        .args(["kickstart", "-k", &service_target])
        .status()
        .context("launchctl kickstart -k not runnable")?;
    if !restart.success() {
        return Err(anyhow!(
            "launchctl kickstart -k failed: exit={}",
            restart.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

fn run_launchd_user(
    plist_path: &Path,
    dry_run: bool,
    no_enable: bool,
) -> Result<(bool, Vec<String>)> {
    let mut notes = Vec::new();
    if !dry_run {
        // A missing/not-yet-loaded agent returns a nonzero unload status.
        // Replacement remains safe; suppress that benign launchctl noise.
        let _ = std::process::Command::new("launchctl")
            .args(["unload", &plist_path.to_string_lossy()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        notes.push("replaced existing LaunchAgent when present".to_string());
    }
    if dry_run || no_enable {
        notes.push("LaunchAgent written but not loaded (dry-run or --no-enable)".to_string());
        return Ok((false, notes));
    }
    let status = std::process::Command::new("launchctl")
        .args(["load", "-w", &plist_path.to_string_lossy()])
        .status();
    match status {
        Ok(s) if s.success() => {
            // `load -w` may leave an already-running process mapped to the
            // pre-upgrade inode after the install root was stashed. Force a
            // launchd restart so health reflects the newly promoted binary.
            restart_launchd_after_commit()?;
            notes.push("LaunchAgent restarted on promoted binary".to_string());
            Ok((true, notes))
        }
        Ok(s) => Err(anyhow!(
            "launchctl load -w failed: exit={}",
            s.code().unwrap_or(-1)
        )),
        Err(e) => Err(anyhow!("launchctl load -w not runnable: {e}")),
    }
}

/// Uninstall the Focusa daemon service from the user's session.
/// Mirror of `run_systemd_user` and `run_launchd_user` for the install path.
/// Idempotent: re-running on an already-removed service is a no-op success.
pub fn uninstall_service(manager: ServiceManager, dry_run: bool) -> Result<(bool, Vec<String>)> {
    let mut notes = Vec::new();
    match manager {
        ServiceManager::SystemdUser => {
            // Best-effort: stop the unit, then remove the file.
            if !dry_run {
                let _ = std::process::Command::new("systemctl")
                    .args(["--user", "disable", "--now", SERVICE_NAME])
                    .status();
            }
            let unit_path = PathBuf::from(format!(
                "{}/.config/systemd/user/{}.service",
                std::env::var("HOME").unwrap_or_default(),
                SERVICE_NAME
            ));
            if unit_path.exists() {
                if !dry_run {
                    std::fs::remove_file(&unit_path)
                        .with_context(|| format!("remove {}", unit_path.display()))?;
                }
                notes.push(format!("removed {}", unit_path.display()));
            } else {
                notes.push(format!(
                    "{} not present (idempotent skip)",
                    unit_path.display()
                ));
            }
            if !dry_run {
                let _ = std::process::Command::new("systemctl")
                    .args(["--user", "daemon-reload"])
                    .status();
            }
            Ok((true, notes))
        }
        ServiceManager::LaunchdUser => {
            let plist_path = PathBuf::from(format!(
                "{}/Library/LaunchAgents/{}.plist",
                std::env::var("HOME").unwrap_or_default(),
                LAUNCHD_LABEL
            ));
            if !dry_run {
                // launchctl bootout before removing file (matches Apple docs).
                let _ = std::process::Command::new("launchctl")
                    .args(["bootout", &plist_path.to_string_lossy()])
                    .status();
            }
            if plist_path.exists() {
                if !dry_run {
                    std::fs::remove_file(&plist_path)
                        .with_context(|| format!("remove {}", plist_path.display()))?;
                }
                notes.push(format!("removed {}", plist_path.display()));
            } else {
                notes.push(format!(
                    "{} not present (idempotent skip)",
                    plist_path.display()
                ));
            }
            Ok((true, notes))
        }
        ServiceManager::None => {
            notes.push("no service manager detected; nothing to remove".to_string());
            Ok((true, notes))
        }
    }
}

pub async fn run(args: InstallServiceArgs, dry_run: bool) -> Result<()> {
    let manager = detect_manager();
    let binary = find_daemon_binary()?;
    let self_bin = find_self_binary()?;
    let mut notes = Vec::new();
    let unit_path: Option<String>;

    let report: InstallServiceReport = match manager {
        ServiceManager::SystemdUser => {
            let home = std::env::var("HOME").context("HOME not set")?;
            let path = PathBuf::from(&home)
                .join(".config/systemd/user")
                .join(format!("{SERVICE_NAME}.service"));
            let unit = render_systemd_unit(&binary);
            if !dry_run {
                std::fs::write(&path, &unit)
                    .with_context(|| format!("write {}", path.display()))?;
            }
            unit_path = Some(path.display().to_string());
            let (ok, extra) = run_systemd_user(&path, dry_run, args.no_enable)?;
            let enabled = ok;
            notes.extend(extra);
            notes.push(format!("binary: {}", binary.display()));
            notes.push(format!("cli: {}", self_bin.display()));
            InstallServiceReport {
                manager: "systemd_user".into(),
                service_name: SERVICE_NAME.into(),
                binary_path: binary.display().to_string(),
                unit_path,
                enabled,
                loaded: enabled,
                dry_run,
                notes,
                recovery_hint: None,
            }
        }
        ServiceManager::LaunchdUser => {
            let home = std::env::var("HOME").context("HOME not set")?;
            let agents = PathBuf::from(&home).join("Library/LaunchAgents");
            let log_dir = PathBuf::from(&home).join("Library/Logs");
            if !dry_run {
                std::fs::create_dir_all(&agents)?;
                std::fs::create_dir_all(&log_dir)?;
            }
            let plist = agents.join(format!("{LAUNCHD_LABEL}.plist"));
            let body = render_launchd_plist(&binary, &log_dir);
            if !dry_run {
                std::fs::write(&plist, &body)
                    .with_context(|| format!("write {}", plist.display()))?;
            }
            unit_path = Some(plist.display().to_string());
            let (ok, extra) = run_launchd_user(&plist, dry_run, args.no_enable)?;
            let enabled = ok;
            notes.extend(extra);
            notes.push(format!("binary: {}", binary.display()));
            notes.push(format!("cli: {}", self_bin.display()));
            InstallServiceReport {
                manager: "launchd_user".into(),
                service_name: LAUNCHD_LABEL.into(),
                binary_path: binary.display().to_string(),
                unit_path,
                enabled,
                loaded: enabled,
                dry_run,
                notes,
                recovery_hint: None,
            }
        }
        ServiceManager::None => InstallServiceReport {
            manager: "none".into(),
            service_name: SERVICE_NAME.into(),
            binary_path: binary.display().to_string(),
            unit_path: None,
            enabled: false,
            loaded: false,
            dry_run,
            notes: vec![
                "no supported service manager detected on this host".into(),
                format!("binary: {}", binary.display()),
            ],
            recovery_hint: Some(
                "install systemd (Linux) or launchd (macOS) and rerun focusa install-service"
                    .into(),
            ),
        },
    };

    if args.json || dry_run {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("focusa install-service");
        println!("  manager:       {}", report.manager);
        println!("  service_name:  {}", report.service_name);
        println!("  binary:        {}", report.binary_path);
        if let Some(p) = report.unit_path {
            println!("  unit_path:     {p}");
        }
        println!("  enabled:       {}", report.enabled);
        println!("  dry_run:       {}", report.dry_run);
        for n in &report.notes {
            println!("  note: {n}");
        }
        if let Some(h) = report.recovery_hint {
            println!("  recovery_hint: {h}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_systemd_unit_mentions_binary() {
        let body = render_systemd_unit(Path::new("/opt/focusa/bin/focusa-daemon"));
        assert!(body.contains("ExecStart=/opt/focusa/bin/focusa-daemon"));
        assert!(body.contains("Restart=on-failure"));
    }

    #[test]
    fn render_launchd_plist_mentions_binary() {
        let body = render_launchd_plist(
            Path::new("/opt/focusa/bin/focusa-daemon"),
            Path::new("/tmp/logs"),
        );
        assert!(body.contains("/opt/focusa/bin/focusa-daemon"));
        assert!(body.contains(LAUNCHD_LABEL));
        assert!(body.contains("/tmp/logs/focusa-daemon.out.log"));
    }
}
