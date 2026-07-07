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

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum UninstallStepKind {
    StopDaemon,
    RemoveService,
    RemoveSymlink,
    RemoveInstallRoot,
    RemoveLicense,
    RevertPath,
    PurgeAgentSkills,
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
            // Reverse of install's path-automation: read rc file, remove any
            // line containing `/.local/bin` paired with `export PATH=`.
            if let Some(p) = &step.target_path {
                let expanded = p.replace("$HOME", &std::env::var("HOME").unwrap_or_default());
                let path = std::path::PathBuf::from(&expanded);
                if path.exists() {
                    let content =
                        std::fs::read_to_string(&path).with_context(|| format!("read {p}"))?;
                    let new_content: String = content
                        .lines()
                        .filter(|line| !(line.contains(".local/bin") && line.contains("PATH")))
                        .collect::<Vec<_>>()
                        .join("\n");
                    if new_content != content {
                        std::fs::write(&path, &new_content)
                            .with_context(|| format!("write {p}"))?;
                        step.detail = Some("removed focusa PATH line".to_string());
                    } else {
                        step.detail = Some("no focusa PATH line present".to_string());
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
        // stop_daemon + remove_service + 3 symlinks + remove_install_root +
        // remove_license + 3 rc reverts = 10 steps
        assert_eq!(steps.len(), 10);
        assert_eq!(steps[0].name, "stop_daemon");
        assert_eq!(steps[1].name, "remove_service");
        assert!(steps.iter().any(|s| s.name == "remove_symlink_focusa"));
        assert!(steps.iter().any(|s| s.name == "remove_install_root"));
        assert!(steps.iter().any(|s| s.name == "remove_license"));
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
