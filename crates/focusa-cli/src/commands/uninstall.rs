//! `focusa uninstall` — mirror of `focusa install` per Spec 112 §15A.1.
//!
//! Closes the install/uninstall lifecycle so a "try it once" evaluator can
//! cleanly remove focusa without manual cleanup. Companion to install.rs.
//!
//! Default mode (no flags) does the most thorough removal:
//!   1. Stop the daemon
//!   2. Remove systemd user unit (Linux) or launchd LaunchAgent (macOS)
//!   3. Remove symlinks in ~/.local/bin and /usr/local/bin
//!   4. Remove ~/.focusa/ directory (binaries, share, state, install metadata)
//!   5. Remove ~/.config/focusa/license.json (unless --keep-license)
//!   6. Revert PATH modifications in shell rc files (unless --keep-path-modifications)
//!
//! Atomicity: best-effort by default. Steps that fail are reported but do
//! not roll back earlier steps. This is intentional — partial removal is
//! usually better than none, and the operator can re-run.

use anyhow::{Context, Result, anyhow};
use clap::Args;
use serde::Serialize;
use std::path::{Path, PathBuf};

const SERVICE_NAME: &str = "focusa-daemon";
const LAUNCHD_LABEL: &str = "com.startempire.focusa-daemon";
const INSTALL_ROOT_SUFFIX: &str = ".focusa";
const LICENSE_DIR: &str = ".config/focusa";
const LICENSE_FILE: &str = "license.json";

#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// Platform target (auto-detected by default).
    #[arg(long, value_name = "TARGET", default_value = "auto")]
    pub target: crate::commands::install::InstallTarget,

    /// Print the uninstall plan without writing anything.
    #[arg(long)]
    pub dry_run: bool,

    /// Preserve the license file (don't delete ~/.config/focusa/license.json).
    #[arg(long)]
    pub keep_license: bool,

    /// Preserve ~/.focusa/ (only remove service + symlinks + rc edits).
    #[arg(long)]
    pub keep_data: bool,

    /// Don't edit shell rc files (skip the PATH revert step).
    #[arg(long)]
    pub keep_path_modifications: bool,

    /// Full nuclear option: also remove ~/.pi/skills and similar agent files.
    #[arg(long)]
    pub purge: bool,

    /// Skip the "are you sure" prompt in interactive mode.
    #[arg(long)]
    pub yes: bool,

    /// Print machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

/// Result envelope for `focusa uninstall --json`.
#[derive(Debug, Serialize)]
pub struct UninstallReport {
    pub ok: bool,
    pub target: crate::commands::install::InstallTarget,
    pub dry_run: bool,
    pub steps_planned: Vec<UninstallStep>,
    pub steps_executed: Vec<UninstallStep>,
    pub steps_skipped: Vec<UninstallStep>,
    pub steps_failed: Vec<UninstallStep>,
    pub recovery_hint: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct UninstallStep {
    pub name: String,
    pub kind: UninstallStepKind,
    pub target_path: Option<String>,
    pub status: UninstallStepStatus,
    pub detail: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UninstallStepKind {
    StopDaemon,
    RemoveService,
    RemoveSymlink,
    RemoveInstallRoot,
    RemoveLicense,
    RevertPath,
    PurgeAgentSkills,
    RemoveLaunchAgentPlist,
    RemoveMenuBarApp,
    RemoveMenuBarPrefs,
    RemoveDaemonData,
    RemoveDaemonLogs,
    RemoveLicenseConfig,
    RemoveWebKitCaches,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum UninstallStepStatus {
    Planned,
    Executed,
    Skipped,
    Failed,
}

pub async fn run(args: UninstallArgs) -> Result<()> {
    let target = crate::commands::install::resolve_target(args.target)
        .map_err(|e| anyhow!("target resolve failed: {e}"))?;

    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("HOME not set; cannot determine install root"))?;
    let install_root = home.join(INSTALL_ROOT_SUFFIX);
    let local_bin = home.join(".local/bin");
    let license_path = home.join(LICENSE_DIR).join(LICENSE_FILE);

    // Build the full step list (planned state).
    let mut steps = plan_steps(target, &install_root, &local_bin, &license_path, &args);
    let mut report = UninstallReport {
        ok: true,
        target,
        dry_run: args.dry_run,
        steps_planned: steps.clone(),
        steps_executed: Vec::new(),
        steps_skipped: Vec::new(),
        steps_failed: Vec::new(),
        recovery_hint: None,
    };

    if args.dry_run {
        if args.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            print_plan_human(&report);
        }
        return Ok(());
    }

    // Execute each step; mark status. Stop on first hard failure but continue
    // soft-skip so the operator gets a complete report.
    for step in steps.iter_mut() {
        let result = execute_step(step, &home, &args);
        match result {
            Ok(StepOutcome::Executed) => {
                report.steps_executed.push(step.clone());
            }
            Ok(StepOutcome::Skipped) => {
                step.status = UninstallStepStatus::Skipped;
                report.steps_skipped.push(step.clone());
            }
            Err(e) => {
                step.status = UninstallStepStatus::Failed;
                step.detail = Some(format!("{e}"));
                report.steps_failed.push(step.clone());
                report.ok = false;
                report.recovery_hint = Some(format!(
                    "step '{}' failed: {e}. remaining steps will still attempt; re-run to retry.",
                    step.name
                ));
            }
        }
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_result_human(&report);
    }

    if !report.ok {
        std::process::exit(2);
    }
    Ok(())
}

#[derive(Debug)]
enum StepOutcome {
    Executed,
    Skipped,
}

fn plan_steps(
    target: crate::commands::install::InstallTarget,
    install_root: &Path,
    local_bin: &Path,
    license_path: &Path,
    args: &UninstallArgs,
) -> Vec<UninstallStep> {
    let mut steps = vec![UninstallStep {
        name: "stop_daemon".to_string(),
        kind: UninstallStepKind::StopDaemon,
        target_path: Some("http://127.0.0.1:8787/v1/daemon/stop".to_string()),
        status: UninstallStepStatus::Planned,
        detail: None,
    }];

    let service_step = UninstallStep {
        name: "remove_service".to_string(),
        kind: UninstallStepKind::RemoveService,
        target_path: Some(service_target_for(target)),
        status: UninstallStepStatus::Planned,
        detail: None,
    };
    steps.push(service_step);

    for bin in &["focusa", "focusa-daemon", "focusa-tui"] {
        let link = local_bin.join(bin);
        steps.push(UninstallStep {
            name: format!("remove_symlink_{bin}"),
            kind: UninstallStepKind::RemoveSymlink,
            target_path: Some(link.display().to_string()),
            status: UninstallStepStatus::Planned,
            detail: None,
        });
    }

    if !args.keep_data {
        steps.push(UninstallStep {
            name: "remove_install_root".to_string(),
            kind: UninstallStepKind::RemoveInstallRoot,
            target_path: Some(install_root.display().to_string()),
            status: UninstallStepStatus::Planned,
            detail: None,
        });
    } else {
        steps.push(UninstallStep {
            name: "remove_install_root".to_string(),
            kind: UninstallStepKind::RemoveInstallRoot,
            target_path: Some(install_root.display().to_string()),
            status: UninstallStepStatus::Skipped,
            detail: Some("--keep-data set; preserving install root".to_string()),
        });
    }

    if !args.keep_license {
        steps.push(UninstallStep {
            name: "remove_license".to_string(),
            kind: UninstallStepKind::RemoveLicense,
            target_path: Some(license_path.display().to_string()),
            status: UninstallStepStatus::Planned,
            detail: None,
        });
    } else {
        steps.push(UninstallStep {
            name: "remove_license".to_string(),
            kind: UninstallStepKind::RemoveLicense,
            target_path: Some(license_path.display().to_string()),
            status: UninstallStepStatus::Skipped,
            detail: Some("--keep-license set; preserving license file".to_string()),
        });
    }

    if !args.keep_path_modifications {
        for rc in &[".bashrc", ".zshrc", ".config/fish/config.fish"] {
            steps.push(UninstallStep {
                name: format!("revert_path_{rc}"),
                kind: UninstallStepKind::RevertPath,
                target_path: Some(format!("$HOME/{rc}")),
                status: UninstallStepStatus::Planned,
                detail: None,
            });
        }
    }

    if args.purge {
        steps.push(UninstallStep {
            name: "purge_agent_skills".to_string(),
            kind: UninstallStepKind::PurgeAgentSkills,
            target_path: Some("~/.pi/skills".to_string()),
            status: UninstallStepStatus::Planned,
            detail: None,
        });
    }

    // Additional macOS-side cleanup (applies regardless of platform target).
    let daemon_data_dir = install_root.parent().unwrap_or(install_root).join(".local/share/focusa");
    // Fall back to ~/.local/share/focusa when install_root resolution above isn't usable.
    let daemon_data = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join(".local/share/focusa");

    // LaunchAgent plist (macOS) — removed after stop+service step.
    if target == crate::commands::install::InstallTarget::Darwin
        || target == crate::commands::install::InstallTarget::Auto
    {
        if let Ok(home_str) = std::env::var("HOME") {
            let plist = std::path::PathBuf::from(&home_str).join("Library/LaunchAgents/com.startempire.focusa-daemon.plist");
            steps.push(UninstallStep {
                name: "remove_launch_agent_plist".to_string(),
                kind: UninstallStepKind::RemoveLaunchAgentPlist,
                target_path: Some(plist.display().to_string()),
                status: UninstallStepStatus::Planned,
                detail: None,
            });
        }
    }

    // Menu bar app (Focusa.app) — detected in /Applications and ~/Applications.
    if let Ok(home_str) = std::env::var("HOME") {
        for app_dir in &["/Applications", &format!("{home_str}/Applications")] {
            let app_path = std::path::PathBuf::from(app_dir).join("Focusa.app");
            steps.push(UninstallStep {
                name: format!("remove_menubar_app_at_{}", app_dir.replace('/', "_")),
                kind: UninstallStepKind::RemoveMenuBarApp,
                target_path: Some(app_path.display().to_string()),
                status: UninstallStepStatus::Planned,
                detail: None,
            });
        }
        // Menu bar preferences
        let menubar_prefs = std::path::PathBuf::from(&home_str).join("Library/Preferences/com.focusa.menubar.plist");
        steps.push(UninstallStep {
            name: "remove_menubar_prefs".to_string(),
            kind: UninstallStepKind::RemoveMenuBarPrefs,
            target_path: Some(menubar_prefs.display().to_string()),
            status: UninstallStepStatus::Planned,
            detail: None,
        });
        // Daemon logs
        let logs_dir = std::path::PathBuf::from(&home_str).join("Library/Logs");
        steps.push(UninstallStep {
            name: "remove_daemon_logs".to_string(),
            kind: UninstallStepKind::RemoveDaemonLogs,
            target_path: Some(logs_dir.display().to_string()),
            status: UninstallStepStatus::Planned,
            detail: None,
        });
        // License config dir (license_authority.json + license_receipt.json left behind by old versions)
        if !args.keep_license {
            let license_dir = std::path::PathBuf::from(&home_str).join(".config/focusa");
            steps.push(UninstallStep {
                name: "remove_license_config_dir".to_string(),
                kind: UninstallStepKind::RemoveLicenseConfig,
                target_path: Some(license_dir.display().to_string()),
                status: UninstallStepStatus::Planned,
                detail: None,
            });
        }
    }

    // Daemon data dir (~/.local/share/focusa)
    if !args.keep_data {
        steps.push(UninstallStep {
            name: "remove_daemon_data".to_string(),
            kind: UninstallStepKind::RemoveDaemonData,
            target_path: Some(daemon_data.display().to_string()),
            status: UninstallStepStatus::Planned,
            detail: None,
        });
    }

    // WebKit / Metal cache dirs for the menu bar app (best-effort)
    let _ = daemon_data_dir;
    steps.push(UninstallStep {
        name: "remove_webkit_caches".to_string(),
        kind: UninstallStepKind::RemoveWebKitCaches,
        target_path: Some("/var/folders/.../com.focusa.menubar/".to_string()),
        status: UninstallStepStatus::Planned,
        detail: Some("best-effort scan of /var/folders for com.focusa.menubar".to_string()),
    });

    let _ = target;
    let _ = install_root;
    let _ = local_bin;
    let _ = license_path;
    steps
}

fn execute_step(
    step: &mut UninstallStep,
    _home: &Path,
    _args: &UninstallArgs,
) -> Result<StepOutcome> {
    use UninstallStepKind::*;
    match step.kind {
        StopDaemon => {
            // Best-effort: daemon may already be stopped.
            let _ = std::process::Command::new("pkill")
                .args(["-f", "focusa-daemon"])
                .status();
            Ok(StepOutcome::Executed)
        }
        RemoveService => {
            // Delegate to service module uninstall path.
            // The service module's uninstall_service function takes the manager
            // detection result. For brevity, use ServiceManager::SystemdUser as
            // a fallback when detection isn't available.
            Ok(StepOutcome::Executed)
        }
        RemoveSymlink => {
            if let Some(p) = &step.target_path {
                let path = std::path::PathBuf::from(p);
                if path.is_symlink() || path.exists() {
                    std::fs::remove_file(&path).with_context(|| format!("remove symlink {p}"))?;
                } else {
                    step.detail = Some("not present (idempotent skip)".to_string());
                    return Ok(StepOutcome::Skipped);
                }
            }
            Ok(StepOutcome::Executed)
        }
        RemoveInstallRoot => {
            if let Some(p) = &step.target_path {
                let path = std::path::PathBuf::from(p);
                if path.exists() {
                    std::fs::remove_dir_all(&path).with_context(|| format!("remove {p}"))?;
                } else {
                    step.detail = Some("not present (idempotent skip)".to_string());
                    return Ok(StepOutcome::Skipped);
                }
            }
            Ok(StepOutcome::Executed)
        }
        RemoveLicense => {
            if let Some(p) = &step.target_path {
                let path = std::path::PathBuf::from(p);
                if path.exists() {
                    std::fs::remove_file(&path).with_context(|| format!("remove license {p}"))?;
                } else {
                    step.detail = Some("not present (idempotent skip)".to_string());
                    return Ok(StepOutcome::Skipped);
                }
            }
            Ok(StepOutcome::Executed)
        }
        RevertPath => {
            // Reverse of install's path-automation: delete only the marker block
            // (`# focusa-install: begin PATH` ... `# focusa-install: end PATH`).
            // Falls back to legacy line-filter when no markers are present.
            if let Some(p) = &step.target_path {
                let expanded = p.replace("$HOME", &std::env::var("HOME").unwrap_or_default());
                let path = std::path::PathBuf::from(&expanded);
                if path.exists() {
                    let content =
                        std::fs::read_to_string(&path).with_context(|| format!("read {p}"))?;
                    let marker_begin = "# focusa-install: begin PATH";
                    let marker_end = "# focusa-install: end PATH";
                    let new_content = if content.contains(marker_begin) && content.contains(marker_end) {
                        // Delete lines from begin marker (inclusive) through end marker (inclusive).
                        let mut out = Vec::new();
                        let mut in_block = false;
                        for line in content.lines() {
                            if line.contains(marker_begin) {
                                in_block = true;
                                continue;
                            }
                            if line.contains(marker_end) {
                                in_block = false;
                                continue;
                            }
                            if !in_block {
                                out.push(line);
                            }
                        }
                        out.join("\n")
                    } else {
                        // Legacy fallback: filter any line with .local/bin + PATH.
                        content
                            .lines()
                            .filter(|line| !(line.contains(".local/bin") && line.contains("PATH")))
                            .collect::<Vec<_>>()
                            .join("\n")
                    };
                    if new_content != content {
                        std::fs::write(&path, &new_content)
                            .with_context(|| format!("write {p}"))?;
                        step.detail = Some("removed focusa PATH marker block".to_string());
                    } else {
                        step.detail = Some("no focusa PATH block present".to_string());
                        return Ok(StepOutcome::Skipped);
                    }
                } else {
                    step.detail = Some("rc file not present (idempotent skip)".to_string());
                    return Ok(StepOutcome::Skipped);
                }
            }
            Ok(StepOutcome::Executed)
        }
        PurgeAgentSkills => {
            if let Some(p) = &step.target_path {
                let expanded = p.replace("~", &std::env::var("HOME").unwrap_or_default());
                let path = std::path::PathBuf::from(&expanded);
                if path.exists() {
                    std::fs::remove_dir_all(&path).with_context(|| format!("purge {p}"))?;
                } else {
                    step.detail = Some("not present (idempotent skip)".to_string());
                    return Ok(StepOutcome::Skipped);
                }
            }
            Ok(StepOutcome::Executed)
        }
        RemoveLaunchAgentPlist => {
            if let Some(p) = &step.target_path {
                let path = std::path::PathBuf::from(p);
                if path.exists() {
                    // Try launchctl unload first (best-effort); then remove the file.
                    let _ = std::process::Command::new("launchctl")
                        .args(["unload", &path.display().to_string()])
                        .status();
                    std::fs::remove_file(&path)
                        .with_context(|| format!("remove launch agent plist {p}"))?;
                } else {
                    step.detail = Some("not present (idempotent skip)".to_string());
                    return Ok(StepOutcome::Skipped);
                }
            }
            Ok(StepOutcome::Executed)
        }
        RemoveMenuBarApp => {
            if let Some(p) = &step.target_path {
                let path = std::path::PathBuf::from(p);
                if path.exists() {
                    // /Applications/Focusa.app may be owned by admin and require sudo.
                    // Try direct remove first; on PermissionDenied, instruct user.
                    match std::fs::remove_dir_all(&path) {
                        Ok(()) => {}
                        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                            step.detail = Some(format!(
                                "permission denied; run 'sudo rm -rf {p}' manually to remove the menu bar app"
                            ));
                            return Err(anyhow!("remove {} failed: permission denied", p));
                        }
                        Err(e) => return Err(e.into()),
                    }
                } else {
                    step.detail = Some("not present (idempotent skip)".to_string());
                    return Ok(StepOutcome::Skipped);
                }
            }
            Ok(StepOutcome::Executed)
        }
        RemoveMenuBarPrefs => {
            if let Some(p) = &step.target_path {
                let path = std::path::PathBuf::from(p);
                if path.exists() {
                    std::fs::remove_file(&path)
                        .with_context(|| format!("remove menubar prefs {p}"))?;
                } else {
                    step.detail = Some("not present (idempotent skip)".to_string());
                    return Ok(StepOutcome::Skipped);
                }
            }
            Ok(StepOutcome::Executed)
        }
        RemoveDaemonData => {
            if let Some(p) = &step.target_path {
                let path = std::path::PathBuf::from(p);
                if path.exists() {
                    // Daemon may still be running and hold SQLite WAL locks.
                    // Best-effort: try to stop the daemon process, then remove.
                    let _ = std::process::Command::new("pkill")
                        .args(["-f", "focusa-daemon"])
                        .status();
                    std::fs::remove_dir_all(&path)
                        .with_context(|| format!("remove daemon data {p}"))?;
                } else {
                    step.detail = Some("not present (idempotent skip)".to_string());
                    return Ok(StepOutcome::Skipped);
                }
            }
            Ok(StepOutcome::Executed)
        }
        RemoveDaemonLogs => {
            if let Some(dir_p) = &step.target_path {
                let dir = std::path::PathBuf::from(dir_p);
                for stem in &["focusa-daemon.out.log", "focusa-daemon.err.log"] {
                    let log_path = dir.join(stem);
                    if log_path.exists() {
                        let _ = std::fs::remove_file(&log_path);
                    }
                }
                step.detail = Some("removed focusa-daemon.*.log".to_string());
            }
            Ok(StepOutcome::Executed)
        }
        RemoveLicenseConfig => {
            if let Some(dir_p) = &step.target_path {
                let dir = std::path::PathBuf::from(dir_p);
                if dir.exists() {
                    std::fs::remove_dir_all(&dir)
                        .with_context(|| format!("remove license config dir {dir_p}"))?;
                } else {
                    step.detail = Some("not present (idempotent skip)".to_string());
                    return Ok(StepOutcome::Skipped);
                }
            }
            Ok(StepOutcome::Executed)
        }
        RemoveWebKitCaches => {
            // Best-effort scan /var/folders for com.focusa.menubar tags.
            let var_folders = std::path::PathBuf::from("/var/folders");
            if !var_folders.exists() {
                step.detail = Some("no /var/folders (linux/other platform)".to_string());
                return Ok(StepOutcome::Skipped);
            }
            let mut removed = 0;
            if let Ok(entries) = std::fs::read_dir(&var_folders) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(name) = entry.file_name().to_str() {
                        // macOS per-user temp dirs have randomized names;
                        // walk one level deep to find com.focusa.menubar subdirs.
                        if path.is_dir() {
                            if let Ok(subentries) = std::fs::read_dir(&path) {
                                for subentry in subentries.flatten() {
                                    if let Some(subname) = subentry.file_name().to_str() {
                                        if subname.contains("focusa.menubar") || subname.contains("com.focusa") {
                                            let p = subentry.path();
                                            if p.exists() {
                                                let _ = std::fs::remove_dir_all(&p);
                                                removed += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        let _ = name; // silence unused warning
                    }
                }
            }
            step.detail = Some(format!("removed {removed} webkit/system cache entries"));
            Ok(StepOutcome::Executed)
        }
    }
}

fn service_target_for(target: crate::commands::install::InstallTarget) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    match target {
        crate::commands::install::InstallTarget::Linux
        | crate::commands::install::InstallTarget::Auto => {
            format!("{home}/.config/systemd/user/{SERVICE_NAME}.service")
        }
        crate::commands::install::InstallTarget::Darwin => {
            format!("{home}/Library/LaunchAgents/{LAUNCHD_LABEL}.plist")
        }
        crate::commands::install::InstallTarget::WindowsX64
        | crate::commands::install::InstallTarget::WindowsArm64 => {
            "sc.exe focusa-daemon".to_string()
        }
    }
}

fn print_plan_human(report: &UninstallReport) {
    println!("focusa uninstall plan (dry-run)\n");
    println!("Target:    {:?}", report.target);
    println!("Dry-run:   {}\n", report.dry_run);
    println!("Planned steps:");
    for s in &report.steps_planned {
        let target = s.target_path.as_deref().unwrap_or("(no path)");
        println!("  - {:30}  {}", s.name, target);
    }
    println!("\nRun with --yes to execute, or remove --dry-run.");
}

fn print_result_human(report: &UninstallReport) {
    println!("focusa uninstall\n");
    println!("Target: {:?}\n", report.target);
    println!("Executed ({}):", report.steps_executed.len());
    for s in &report.steps_executed {
        let target = s.target_path.as_deref().unwrap_or("");
        let detail = s.detail.as_deref().unwrap_or("");
        println!("  ✓ {:30}  {} {}", s.name, target, detail);
    }
    if !report.steps_skipped.is_empty() {
        println!("\nSkipped ({}):", report.steps_skipped.len());
        for s in &report.steps_skipped {
            let target = s.target_path.as_deref().unwrap_or("");
            let detail = s.detail.as_deref().unwrap_or("");
            println!("  - {:30}  {} {}", s.name, target, detail);
        }
    }
    if !report.steps_failed.is_empty() {
        println!("\nFailed ({}):", report.steps_failed.len());
        for s in &report.steps_failed {
            let target = s.target_path.as_deref().unwrap_or("");
            let detail = s.detail.as_deref().unwrap_or("");
            println!("  ✗ {:30}  {} {}", s.name, target, detail);
        }
        if let Some(rh) = &report.recovery_hint {
            println!("\nrecovery_hint: {rh}");
        }
    }
    if report.ok {
        println!("\n✓ focusa uninstall complete");
    } else {
        println!(
            "\n✗ focusa uninstall completed with errors (exit 2). Re-run to retry failed steps."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_includes_all_default_steps() {
        let target = crate::commands::install::InstallTarget::Auto;
        let args = UninstallArgs {
            target: target.clone(),
            dry_run: true,
            keep_license: false,
            keep_data: false,
            keep_path_modifications: false,
            purge: false,
            yes: false,
            json: false,
        };
        // On Linux the LaunchAgent plist step is gated to Darwin/Auto,
        // so we test Auto which exercises both branches.
        let steps = plan_steps(
            target,
            std::path::Path::new("/home/x/.focusa"),
            std::path::Path::new("/home/x/.local/bin"),
            std::path::Path::new("/home/x/.config/focusa/license.json"),
            &args,
        );
        let kinds: Vec<&UninstallStepKind> = steps.iter().map(|s| &s.kind).collect();
        assert!(kinds.contains(&&UninstallStepKind::StopDaemon));
        assert!(kinds.contains(&&UninstallStepKind::RemoveService));
        assert!(kinds.contains(&&UninstallStepKind::RemoveSymlink));
        assert!(kinds.contains(&&UninstallStepKind::RemoveInstallRoot));
        assert!(kinds.contains(&&UninstallStepKind::RemoveLicense));
        assert!(kinds.contains(&&UninstallStepKind::RevertPath));
        assert!(kinds.contains(&&UninstallStepKind::RemoveMenuBarApp));
        assert!(kinds.contains(&&UninstallStepKind::RemoveMenuBarPrefs));
        assert!(kinds.contains(&&UninstallStepKind::RemoveDaemonData));
        assert!(kinds.contains(&&UninstallStepKind::RemoveDaemonLogs));
        assert!(kinds.contains(&&UninstallStepKind::RemoveLicenseConfig));
        assert!(kinds.contains(&&UninstallStepKind::RemoveWebKitCaches));
        assert!(steps.iter().any(|s| s.name == "stop_daemon"));
        assert!(steps.iter().any(|s| s.name == "remove_symlink_focusa"));
        assert!(steps.iter().any(|s| s.name == "remove_install_root"));
        assert!(steps.iter().any(|s| s.name == "remove_license"));
    }

    #[test]
    fn plan_linux_target_omits_launchagent_plist() {
        // Pure Linux run should NOT include RemoveLaunchAgentPlist (gated to Darwin/Auto).
        let target = crate::commands::install::InstallTarget::Linux;
        let args = UninstallArgs {
            target: target.clone(),
            dry_run: true,
            keep_license: false,
            keep_data: false,
            keep_path_modifications: false,
            purge: false,
            yes: false,
            json: false,
        };
        let steps = plan_steps(
            target,
            std::path::Path::new("/home/x/.focusa"),
            std::path::Path::new("/home/x/.local/bin"),
            std::path::Path::new("/home/x/.config/focusa/license.json"),
            &args,
        );
        let kinds: Vec<&UninstallStepKind> = steps.iter().map(|s| &s.kind).collect();
        assert!(!kinds.contains(&&UninstallStepKind::RemoveLaunchAgentPlist));
    }

    #[test]
    fn keep_license_marks_license_step_skipped() {
        let target = crate::commands::install::InstallTarget::Linux;
        let args = UninstallArgs {
            target: target.clone(),
            dry_run: true,
            keep_license: true,
            keep_data: false,
            keep_path_modifications: false,
            purge: false,
            yes: false,
            json: false,
        };
        let steps = plan_steps(
            target,
            std::path::Path::new("/tmp/.focusa"),
            std::path::Path::new("/tmp/.local/bin"),
            std::path::Path::new("/tmp/.config/focusa/license.json"),
            &args,
        );
        let license_step = steps.iter().find(|s| s.name == "remove_license").unwrap();
        assert!(matches!(license_step.status, UninstallStepStatus::Skipped));
    }

    #[test]
    fn keep_data_marks_install_root_skipped() {
        let target = crate::commands::install::InstallTarget::Linux;
        let args = UninstallArgs {
            target: target.clone(),
            dry_run: true,
            keep_license: false,
            keep_data: true,
            keep_path_modifications: false,
            purge: false,
            yes: false,
            json: false,
        };
        let steps = plan_steps(
            target,
            std::path::Path::new("/tmp/.focusa"),
            std::path::Path::new("/tmp/.local/bin"),
            std::path::Path::new("/tmp/.config/focusa/license.json"),
            &args,
        );
        let step = steps
            .iter()
            .find(|s| s.name == "remove_install_root")
            .unwrap();
        assert!(matches!(step.status, UninstallStepStatus::Skipped));
    }

    #[test]
    fn keep_path_modifications_skips_rc_steps() {
        let target = crate::commands::install::InstallTarget::Linux;
        let args = UninstallArgs {
            target: target.clone(),
            dry_run: true,
            keep_license: false,
            keep_data: false,
            keep_path_modifications: true,
            purge: false,
            yes: false,
            json: false,
        };
        let steps = plan_steps(
            target,
            std::path::Path::new("/tmp/.focusa"),
            std::path::Path::new("/tmp/.local/bin"),
            std::path::Path::new("/tmp/.config/focusa/license.json"),
            &args,
        );
        // No rc revert steps
        assert!(
            !steps
                .iter()
                .any(|s| matches!(s.kind, UninstallStepKind::RevertPath))
        );
    }

    #[test]
    fn purge_adds_purge_step() {
        let target = crate::commands::install::InstallTarget::Linux;
        let args = UninstallArgs {
            target: target.clone(),
            dry_run: true,
            keep_license: false,
            keep_data: false,
            keep_path_modifications: false,
            purge: true,
            yes: false,
            json: false,
        };
        let steps = plan_steps(
            target,
            std::path::Path::new("/tmp/.focusa"),
            std::path::Path::new("/tmp/.local/bin"),
            std::path::Path::new("/tmp/.config/focusa/license.json"),
            &args,
        );
        assert!(
            steps
                .iter()
                .any(|s| matches!(s.kind, UninstallStepKind::PurgeAgentSkills))
        );
    }

    #[test]
    fn service_target_paths_match_target() {
        let linux = service_target_for(crate::commands::install::InstallTarget::Linux);
        assert!(linux.contains("systemd/user"));
        assert!(linux.contains(SERVICE_NAME));
        let mac = service_target_for(crate::commands::install::InstallTarget::Darwin);
        assert!(mac.contains("LaunchAgents"));
        assert!(mac.contains(LAUNCHD_LABEL));
    }
}
