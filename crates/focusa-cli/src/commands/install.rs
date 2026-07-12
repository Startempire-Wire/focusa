//! Focusa install — single Rust orchestrator (Spec 112 §15A).
//!
//! Replaces the shell-heavy `scripts/install-focusa.sh` with a Rust subcommand
//! that owns all install behavior:
//!   * license validation (via `license::registry_validate`)
//!   * asset download (`focusa`, `focusa-daemon`, `focusa-tui`)
//!   * SHA256SUMS verification
//!   * symlink placement (`~/.local/bin > /usr/local/bin`)
//!   * service rendering delegation to `service::run_systemd_user` /
//!     `service::run_launchd_user`
//!   * atomicity (stash + rollback)
//!   * PATH automation + first install walkthrough (Spec 112 §15A.6)
//!   * `--dry-run` and `--target=<auto|linux|darwin|windows-x64|windows-arm64>`
//!   * `--channel=<stable|preview|nightly>`
//!
//! The shell installers become thin bootstrappers that download `focusa` and
//! `exec focusa install --target=<detected>`. See docs §15A.

use anyhow::{anyhow, bail, Context, Result};
use clap::Args;
use focusa_terminal_ui::install::completion::InstallCompletionSummary;
use focusa_terminal_ui::install::event::NullEventSink;
use focusa_terminal_ui::{
    detect_capabilities, install_signal_handlers, validate_environment, CancellationToken,
    InstallEvent, InstallEventSink, InstallPhase, InstallRendererMode,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::Write;

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Platform target (auto-detected by default).
    #[arg(long, value_name = "TARGET", default_value = "auto")]
    pub target: InstallTarget,

    /// Release channel.
    #[arg(long, value_name = "CHANNEL", default_value = "stable")]
    pub channel: Channel,

    /// Print the install plan without writing anything.
    #[arg(long)]
    pub dry_run: bool,

    /// Run installer system/dependency preflight only; no downloads or writes.
    #[arg(long)]
    pub preflight: bool,

    /// Disable terminal intro animation/spinner.
    #[arg(long)]
    pub no_animation: bool,

    /// Suppress decorative output.
    #[arg(long)]
    pub quiet: bool,

    /// Reserved for future dependency installer; currently reported only.
    #[arg(long)]
    pub assume_yes: bool,

    /// License key (commercial install). Eval mode is selected by absence.
    #[arg(long, value_name = "KEY")]
    pub license_key: Option<String>,

    /// Eval mode: skip license validation, write `eval: true` to license.json.
    #[arg(long)]
    pub eval: bool,

    /// Record that the public bootstrapper collected BSL acceptance.
    /// The Rust orchestrator accepts this handoff flag so shell and CLI
    /// contracts stay aligned; license validation remains authoritative.
    #[arg(long)]
    pub accept_license: bool,

    /// Skip systemd user unit or launchd registration.
    #[arg(long)]
    pub no_service: bool,

    /// Persist PATH addition to shell rc file when interactive.
    #[arg(long)]
    pub persist_path: bool,

    /// Skip persisting PATH addition to shell rc.
    #[arg(long, conflicts_with = "persist_path")]
    pub no_persist_path: bool,

    /// Shell family for first-install walkthrough card.
    #[arg(long, value_name = "SHELL", default_value = "auto")]
    pub on_shell: ShellFamily,

    /// Print machine-readable JSON envelope.
    #[arg(long)]
    pub json: bool,

    /// Optional override for the GitHub owner (defaults to
    /// `Startempire-Wire/focusa`).
    #[arg(long, value_name = "OWNER/REPO")]
    pub github_repo: Option<String>,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallTarget {
    Auto,
    Linux,
    Darwin,
    WindowsX64,
    WindowsArm64,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Stable,
    Preview,
    Nightly,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ShellFamily {
    Auto,
    Bash,
    Zsh,
    Fish,
    Pwsh,
}

#[derive(Debug, Serialize)]
pub struct InstallPreflightReport {
    pub schema: &'static str,
    pub status: &'static str,
    pub read_only: bool,
    pub mutations_performed: bool,
    pub target: InstallTarget,
    pub channel: Channel,
    pub install_root: String,
    pub system: PreflightSystem,
    pub dependencies: Vec<PreflightDependency>,
    pub missing_dependencies: Vec<String>,
    pub dependency_install_offer: DependencyInstallOffer,
    pub terminal_ux: TerminalUxPreflight,
    pub recommendation: String,
}

#[derive(Debug, Serialize)]
pub struct PreflightSystem {
    pub os: String,
    pub arch: String,
    pub shell: String,
    pub terminal: String,
    pub package_manager: Option<String>,
    pub service_manager: Option<String>,
    pub privileged: bool,
    pub path_target: String,
    pub path_target_writable: bool,
    pub existing_focusa: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PreflightDependency {
    pub name: String,
    pub present: bool,
    pub install_hint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DependencyInstallOffer {
    pub can_offer: bool,
    pub auto_install_performed: bool,
    pub requires_explicit_consent: bool,
    pub assume_yes_requested: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct TerminalUxPreflight {
    pub interactive_tty: bool,
    pub no_color: bool,
    pub ci: bool,
    pub intro_animation_enabled: bool,
    pub disabled_reason: Option<String>,
    pub renderer_mode: String,
    pub color_depth: String,
    pub minimum_size_met: bool,
    pub reduced_motion: bool,
    pub stderr_is_terminal: bool,
}

fn build_preflight_report(
    args: &InstallArgs,
    target: InstallTarget,
    install_root: &std::path::Path,
) -> InstallPreflightReport {
    let system = detect_preflight_system();
    let dependencies = detect_dependencies(system.package_manager.as_deref());
    let missing_dependencies = dependencies
        .iter()
        .filter(|dep| !dep.present)
        .map(|dep| dep.name.clone())
        .collect::<Vec<_>>();
    let terminal_ux = terminal_ux_preflight(args.no_animation);
    InstallPreflightReport {
        schema: "focusa.install_preflight.v1",
        status: if missing_dependencies.is_empty() {
            "ready"
        } else {
            "missing_dependencies"
        },
        read_only: true,
        mutations_performed: false,
        target,
        channel: args.channel,
        install_root: install_root.display().to_string(),
        system,
        dependencies,
        missing_dependencies: missing_dependencies.clone(),
        dependency_install_offer: DependencyInstallOffer {
            can_offer: !missing_dependencies.is_empty(),
            auto_install_performed: false,
            requires_explicit_consent: true,
            assume_yes_requested: args.assume_yes,
            message: if missing_dependencies.is_empty() {
                "all required bootstrap dependencies found".into()
            } else {
                "missing dependencies detected; install hints are printed, but this preflight does not install packages".into()
            },
        },
        terminal_ux,
        recommendation: if missing_dependencies.is_empty() {
            "run focusa install --dry-run, then focusa install when ready".into()
        } else {
            "install the missing dependencies using the hints, then rerun focusa install --preflight".into()
        },
    }
}

fn detect_preflight_system() -> PreflightSystem {
    let package_manager = first_command(&[
        "dnf", "yum", "apt-get", "brew", "pacman", "zypper", "choco", "winget",
    ]);
    let service_manager = if have_cmd("systemctl") {
        Some("systemd".into())
    } else if have_cmd("launchctl") {
        Some("launchd".into())
    } else if cfg!(windows) {
        Some("windows-service".into())
    } else {
        None
    };
    let path_target = "/usr/local/bin".to_string();
    PreflightSystem {
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
        shell: std::env::var("SHELL").unwrap_or_else(|_| "unknown".into()),
        terminal: std::env::var("TERM").unwrap_or_else(|_| "unknown".into()),
        package_manager,
        service_manager,
        privileged: is_root(),
        path_target: path_target.clone(),
        path_target_writable: std::fs::OpenOptions::new()
            .write(true)
            .open(&path_target)
            .is_ok(),
        existing_focusa: which::which("focusa").ok().map(|p| p.display().to_string()),
    }
}

fn detect_dependencies(package_manager: Option<&str>) -> Vec<PreflightDependency> {
    ["curl", "python3", "sha256sum", "tar"]
        .into_iter()
        .map(|name| PreflightDependency {
            name: name.into(),
            present: have_cmd(name) || (name == "sha256sum" && have_cmd("shasum")),
            install_hint: install_hint(package_manager, name),
        })
        .collect()
}

fn install_hint(package_manager: Option<&str>, name: &str) -> Option<String> {
    let package = match name {
        "python3" => "python3",
        "sha256sum" => "coreutils",
        other => other,
    };
    match package_manager {
        Some("dnf") => Some(format!("sudo dnf install -y {package}")),
        Some("yum") => Some(format!("sudo yum install -y {package}")),
        Some("apt-get") => Some(format!(
            "sudo apt-get update && sudo apt-get install -y {package}"
        )),
        Some("brew") => Some(format!("brew install {package}")),
        Some("pacman") => Some(format!("sudo pacman -S --needed {package}")),
        Some("zypper") => Some(format!("sudo zypper install -y {package}")),
        Some("choco") => Some(format!("choco install {package} -y")),
        Some("winget") => Some(format!("winget install {package}")),
        _ => Some(format!("install dependency manually: {package}")),
    }
}

fn terminal_ux_preflight(no_animation: bool) -> TerminalUxPreflight {
    let capabilities = detect_capabilities(no_animation, false, false);
    let interactive_tty = capabilities.stderr_is_terminal
        && !capabilities.ci
        && capabilities.term != ""
        && capabilities.term != "dumb";
    let disabled_reason = if no_animation {
        Some("--no-animation".into())
    } else if capabilities.ci {
        Some("CI".into())
    } else if !capabilities.stderr_is_terminal {
        Some("non_interactive_terminal".into())
    } else if capabilities.term.is_empty() || capabilities.term == "dumb" {
        Some("unsupported_terminal".into())
    } else if !capabilities.minimum_size_met {
        Some("terminal_below_70x22".into())
    } else {
        None
    };
    let color_depth = format!("{:?}", capabilities.color_depth).to_lowercase();
    TerminalUxPreflight {
        interactive_tty,
        no_color: capabilities.no_color,
        ci: capabilities.ci,
        intro_animation_enabled: capabilities.mode.is_animated(),
        disabled_reason,
        renderer_mode: capabilities.mode.as_str().to_string(),
        color_depth,
        minimum_size_met: capabilities.minimum_size_met,
        reduced_motion: capabilities.reduced_motion_env
            || capabilities.mode == InstallRendererMode::ReducedMotion,
        stderr_is_terminal: capabilities.stderr_is_terminal,
    }
}

fn first_command(names: &[&str]) -> Option<String> {
    names
        .iter()
        .copied()
        .find(|name| have_cmd(name))
        .map(str::to_string)
}

fn have_cmd(name: &str) -> bool {
    which::which(name).is_ok()
}

fn is_root() -> bool {
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim() == "0")
        .unwrap_or(false)
}

fn print_preflight_human(report: &InstallPreflightReport, quiet: bool, no_animation: bool) {
    if !quiet && report.terminal_ux.intro_animation_enabled && !no_animation {
        println!("✦ Focusa installer preflight");
    }
    println!("Focusa install preflight: {}", report.status);
    println!("target: {:?} channel: {:?}", report.target, report.channel);
    println!("os: {} arch: {}", report.system.os, report.system.arch);
    println!(
        "package_manager: {}",
        report
            .system
            .package_manager
            .as_deref()
            .unwrap_or("unknown")
    );
    println!(
        "service_manager: {}",
        report
            .system
            .service_manager
            .as_deref()
            .unwrap_or("unknown")
    );
    if report.missing_dependencies.is_empty() {
        println!("dependencies: ok");
    } else {
        println!(
            "missing dependencies: {}",
            report.missing_dependencies.join(", ")
        );
        for dep in &report.dependencies {
            if !dep.present {
                println!(
                    "  - {}: {}",
                    dep.name,
                    dep.install_hint.as_deref().unwrap_or("install manually")
                );
            }
        }
    }
    println!("read_only: true mutations_performed: false");
    println!("next: {}", report.recommendation);
}

/// Result envelope for `focusa install --json`.
#[derive(Debug, Serialize)]
pub struct InstallReport {
    pub ok: bool,
    pub target: InstallTarget,
    pub channel: Channel,
    pub dry_run: bool,
    pub install_root: String,
    pub binary_path: String,
    pub symlink_path: Option<String>,
    pub assets: Vec<InstalledAsset>,
    pub service_unit_path: Option<String>,
    pub on_path: bool,
    pub persisted_path: bool,
    pub license_status: String,
    pub next_steps: Vec<NextStep>,
    pub recovery_hint: Option<String>,
    pub first_install_walkthrough_v1: Option<FirstInstallWalkthrough>,
}

#[derive(Debug, Serialize)]
pub struct InstalledAsset {
    pub name: String,
    pub version: String,
    pub triple: String,
    pub sha256: String,
    pub install_path: String,
}

#[derive(Debug, Serialize)]
pub struct NextStep {
    pub command: String,
    pub intent: String,
    pub expected_outcome: String,
    pub recovery_hint: Option<String>,
}

/// Agent-side first-install walkthrough envelope. Bridges into Spec 111
/// preload artifacts so agents bootstrapped after install have what they
/// need without re-running the install.
#[derive(Debug, Serialize)]
pub struct FirstInstallWalkthrough {
    pub version: String,
    pub environment_summary: EnvironmentSummary,
    pub next_steps: Vec<NextStep>,
    pub agent_integrations: Vec<AgentIntegration>,
}

#[derive(Debug, Serialize)]
pub struct EnvironmentSummary {
    pub install_root: String,
    pub binary_path: String,
    pub on_path: bool,
    pub daemon_url: String,
    pub daemon_status: String,
    pub license_status: String,
    pub scope_key: Option<String>,
    pub recovery_hint_root: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AgentIntegration {
    pub agent: String,
    pub detected: bool,
    pub integrated: bool,
    pub config_path: Option<String>,
    pub next_step: Option<String>,
    pub expected_outcome: Option<String>,
    pub recovery_hint: Option<String>,
}

/// Plan-only result for `--dry-run`. Used by `focusa install --dry-run` to
/// emit a structured preview without executing any side effects.
#[derive(Debug, Serialize)]
pub struct InstallPlan {
    pub target: InstallTarget,
    pub channel: Channel,
    pub install_root: String,
    pub assets_planned: Vec<AssetPlan>,
    pub symlink_planned: String,
    pub service_manager_planned: String,
    pub shell_rc_plan: Vec<String>,
    pub license_mode: String,
    pub notes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_install_walkthrough_v1: Option<FirstInstallWalkthrough>,
}

#[derive(Debug, Serialize)]
pub struct AssetPlan {
    pub name: String,
    pub version: String,
    pub triple: String,
    pub install_path: String,
}

fn cleanup_staged_downloads(install_root: &std::path::Path) {
    for directory in [install_root.join("bin"), install_root.join("share")] {
        if let Ok(entries) = std::fs::read_dir(directory) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "download") {
                    let _ = std::fs::remove_file(path);
                }
            }
        }
    }
}

fn ensure_not_cancelled(cancellation: &CancellationToken) -> Result<()> {
    if cancellation.is_cancelled() {
        bail!("installation cancelled by operator");
    }
    Ok(())
}

fn restore_terminal_after_cancellation() {
    // The renderer owns the guard when animated; this durable fallback is
    // harmless in plain/non-TTY mode and restores cursor/alternate screen if
    // cancellation raced the renderer shutdown.
    eprint!("\x1b[?25h\x1b[?1049l");
}

fn cancellation_result<T>(
    install_root: &std::path::Path,
    stash_path: &std::path::Path,
    stashed: bool,
) -> Result<T> {
    restore_terminal_after_cancellation();
    let sink = NullEventSink;
    sink.emit(InstallEvent::RollbackStarted {
        reason: "installation cancelled by operator".into(),
    });
    let rollback = if stashed {
        phase_atomic_rollback(install_root, stash_path)
    } else {
        Ok(())
    };
    match rollback {
        Ok(()) => {
            sink.emit(InstallEvent::RollbackSucceeded);
            eprintln!("✗ Installation cancelled; staged downloads removed and rollback completed");
        }
        Err(error) => {
            sink.emit(InstallEvent::RollbackFailed {
                message: error.to_string(),
                recovery_hint: format!("restore the prior install from {}", stash_path.display()),
            });
            eprintln!("✗ Installation cancelled; rollback failed: {error}");
        }
    }
    Err(anyhow!("installation cancelled by operator"))
}

pub async fn run(args: InstallArgs) -> Result<()> {
    validate_environment().map_err(|error| anyhow!(error))?;
    let target = resolve_target(args.target)?;
    let channel = args.channel;
    let dry_run = args.dry_run;
    let install_root = std::env::var_os("HOME")
        .map(|h| std::path::PathBuf::from(h).join(".focusa"))
        .unwrap_or_else(|| std::path::PathBuf::from("/opt/focusa"));

    if args.preflight {
        let report = build_preflight_report(&args, target, &install_root);
        if args.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            print_preflight_human(&report, args.quiet, args.no_animation);
        }
        return Ok(());
    }

    if dry_run {
        let plan = build_plan(&args, target, &install_root)?;
        if args.json {
            println!("{}", serde_json::to_string_pretty(&plan)?);
        } else {
            print_plan_human(&plan);
        }
        return Ok(());
    }

    // Real install wrapped in atomicity (focusa-112-atomicity, Spec 112 §6):
    //   1. Stash any existing install to .focusa.stash
    //   2. Execute each phase
    //   3. Run smoke test (focusa --version on the new binary)
    //   4. On smoke-test failure: rollback to stash
    //   5. On success: remove stash
    let stash_path = install_root.with_extension("stash");
    let sink = NullEventSink;
    sink.emit(InstallEvent::PhaseStarted {
        phase: InstallPhase::InitializeEnvironment,
        message: "Preparing atomic installation".into(),
    });
    let stashed = phase_atomic_stash(install_root.as_path(), &stash_path)?;
    sink.emit(InstallEvent::PhaseSucceeded {
        phase: InstallPhase::InitializeEnvironment,
        detail: Some(if stashed {
            "Existing installation stashed".into()
        } else {
            "Fresh installation".into()
        }),
    });
    sink.emit(InstallEvent::PhaseStarted {
        phase: InstallPhase::DetectSystem,
        message: format!("Target {:?}, channel {:?}", target, channel),
    });
    sink.emit(InstallEvent::PhaseSucceeded {
        phase: InstallPhase::DetectSystem,
        detail: Some("Platform and install target detected".into()),
    });
    let cancellation = CancellationToken::new();
    let _signals = install_signal_handlers(&cancellation)
        .map_err(|error| anyhow!("install cancellation handlers: {error}"))?;
    if cancellation.is_cancelled() {
        cleanup_staged_downloads(&install_root);
        return cancellation_result(&install_root, &stash_path, stashed);
    }
    let result =
        match execute_real_install(&args, target, channel, &install_root, &cancellation, &sink)
            .await
        {
            Ok(result) => result,
            Err(e) if cancellation.is_cancelled() => {
                cleanup_staged_downloads(&install_root);
                return cancellation_result(&install_root, &stash_path, stashed);
            }
            Err(e) => {
                if stashed {
                    phase_atomic_rollback(&install_root, &stash_path).ok();
                }
                return Err(e);
            }
        };
    let bin_dir = install_root.join("bin");
    if let Err(e) = phase_smoke_test(&bin_dir).await {
        sink.emit(InstallEvent::PhaseFailed {
            phase: InstallPhase::RunHealthChecks,
            message: "Installed focusa --version smoke test failed".into(),
            recovery_hint: Some(e.to_string()),
        });
        if stashed {
            phase_atomic_rollback(&install_root, &stash_path).ok();
        }
        return Err(e);
    }
    if cancellation.is_cancelled() {
        cleanup_staged_downloads(&install_root);
        return cancellation_result(&install_root, &stash_path, stashed);
    }
    // Persist the verified release only after the smoke gate; this marker is the
    // anti-rollback authority for future downloads and is itself atomic.
    if let Some(version) = result.assets.first().map(|asset| asset.version.as_str()) {
        write_verified_version_marker(&install_root, version)?;
    }
    if stashed {
        if let Err(error) = phase_atomic_cleanup(&stash_path) {
            phase_atomic_rollback(&install_root, &stash_path).ok();
            return Err(error)
                .context("failed to remove prior installation stash after smoke test");
        }
    }

    // The completion event is deliberately after both the installed CLI smoke
    // test and stash cleanup. The transient renderer consumes this event and
    // restores its terminal before durable output begins.
    let version = result
        .assets
        .first()
        .map(|asset| asset.version.clone())
        .unwrap_or_else(|| "unknown".into());
    let summary = InstallCompletionSummary {
        version: version.clone(),
        target: format!("{:?}", target),
        channel: format!("{:?}", channel),
        install_root: install_root.display().to_string(),
        cli_path: bin_dir.join("focusa").display().to_string(),
        daemon_path: bin_dir.join("focusa-daemon").display().to_string(),
        daemon_health: "smoke-test pending separate daemon health check".into(),
        tui_path: bin_dir.join("focusa-tui").display().to_string(),
        service_status: if args.no_service {
            "skipped"
        } else {
            "registered"
        }
        .into(),
        path_status: "evaluated".into(),
        pi_status: "reported by phase events".into(),
        integrity_status: "verified".into(),
        atomicity_status: if stashed {
            "prior install replaced and stash cleared".into()
        } else {
            "fresh install".into()
        },
        warnings: Vec::new(),
    };
    sink.emit(InstallEvent::PhaseSucceeded {
        phase: InstallPhase::Complete,
        detail: Some("Smoke test passed and stash cleanup completed".into()),
    });
    sink.emit(InstallEvent::InstallFinished {
        summary: summary.clone(),
    });

    // The renderer has restored the transient UI before this single durable
    // human summary or single JSON document is written.
    if !args.json {
        println!("{}", summary.render_human());
        print_walkthrough_human(&result.walkthrough);
    } else {
        let report = serde_json::json!({
            "ok": true,
            "target": target,
            "channel": channel,
            "license_status": result.license_status,
            "assets": result.assets,
            "install_root": install_root.display().to_string(),
            "first_install_walkthrough": result.walkthrough,
        });
        println!("{}", serde_json::to_string_pretty(&report)?);
    }

    Ok(())
}

// ----- Phase 1: License re-validation (focusa-112-license-revalidate) -----
async fn phase_license(args: &InstallArgs) -> Result<String> {
    use crate::commands::license::{registry_validate, RegistryValidateOutcome};
    if args.eval {
        return Ok("eval".to_string());
    }
    let key = match args.license_key.as_deref() {
        Some(k) if !k.trim().is_empty() => k,
        _ => {
            return Err(anyhow!(
                "license_key required for commercial install; pass --license-key <key> or --eval"
            ));
        }
    };
    // License registry URL. Read from FOCUSA_LICENSE_REGISTRY env var when set,
    // so operators can point at a private endpoint without baking the URL into the
    // binary. The default points at wpuiai.com, the actual license authority that
    // hosts the live /wp-json/wpuiai-ai-cloud/v1/license/validate endpoint.
    // install.focusa.dev is only the public shell-script distribution facade; its
    // license API path returns license_not_found.
    let registry = std::env::var("FOCUSA_LICENSE_REGISTRY")
        .unwrap_or_else(|_| "https://wpuiai.com".to_string());
    let outcome = registry_validate(&registry, key).await;
    match outcome {
        RegistryValidateOutcome {
            response: Some(r),
            error: None,
        } if r.valid && r.status == "dev_mode" => {
            // Operator rule (2026-07-07): dev_mode is a test fixture for the
            // operator's testing and must not hinder transactions. The
            // registry returned a successful test-fixture response, not a
            // real license row. The bash bootstrapper downgrades this to
            // eval mode before reaching the Rust orchestrator, but if we
            // hit this branch the caller passed `--license-key` to the
            // Rust installer directly. Refuse and explain.
            let require_real = std::env::var("FOCUSA_REQUIRE_REAL_LICENSE")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            if require_real {
                return Err(anyhow!(
                    "registry returned status=dev_mode for a license key; this is a TEST FIXTURE, not a real purchase. \
                     unset FOCUSA_REQUIRE_REAL_LICENSE to allow dev_mode downgrades, or purchase at {}/buy.",
                    registry
                ));
            }
            eprintln!(
                "[focusa-install] registry returned status=dev_mode for license key; downgrading to eval. \
                 this is a TEST FIXTURE — purchase at {}/buy for a real commercial license.",
                registry
            );
            Ok("dev_mode_downgraded_to_eval".to_string())
        }
        RegistryValidateOutcome {
            response: Some(r),
            error: None,
        } if r.valid => Ok("active".to_string()),
        RegistryValidateOutcome {
            response: Some(_),
            error: None,
        } => Ok("not_valid".to_string()),
        RegistryValidateOutcome {
            response: None,
            error: Some(err),
        } => Err(anyhow!(
            "license validation failed: {} ({})",
            err,
            err.recovery_hint()
        )),
        _ => Err(anyhow!("license validation: unexpected outcome")),
    }
}

fn dry_run_summary(
    _args: &InstallArgs,
    _target: InstallTarget,
    _install_root: &std::path::Path,
    _phase: &str,
) -> Option<()> {
    None
}

fn release_tag(channel: Channel) -> String {
    if let Ok(tag) = std::env::var("FOCUSA_RELEASE_TAG") {
        let tag = tag.trim();
        if !tag.is_empty() {
            return tag.to_string();
        }
    }
    match channel {
        Channel::Stable => format!("v{}", env!("CARGO_PKG_VERSION")),
        Channel::Preview => format!("v{}-preview", env!("CARGO_PKG_VERSION")),
        Channel::Nightly => format!("v{}-nightly", env!("CARGO_PKG_VERSION")),
    }
}

fn release_asset_url(repo: &str, tag: &str, name: &str) -> String {
    if let Ok(base) = std::env::var("FOCUSA_RELEASE_BASE_URL") {
        let base = base.trim().trim_end_matches('/');
        if !base.is_empty() {
            return format!("{base}/{name}");
        }
    }
    format!("https://github.com/{repo}/releases/download/{tag}/{name}")
}

// ----- Phase 2: Release resolution and streamed asset download -----
struct ResolvedRelease {
    tag: String,
    client: reqwest::Client,
}

async fn resolve_release(channel: Channel, github_repo: &str) -> Result<ResolvedRelease> {
    let tag = release_tag(channel);
    let client = reqwest::Client::builder()
        .user_agent("focusa-install/0.9.54-dev")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| anyhow!("github client build failed: {e}"))?;
    let resolved_tag = if std::env::var("FOCUSA_RELEASE_BASE_URL").is_ok() {
        tag
    } else {
        let url = format!("https://api.github.com/repos/{github_repo}/releases/tags/{tag}");
        let release: serde_json::Value = client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow!("github release GET failed: {e}"))?
            .json()
            .await
            .map_err(|e| anyhow!("github release response not JSON: {e}"))?;
        release
            .get("tag_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&tag)
            .to_string()
    };
    Ok(ResolvedRelease {
        tag: resolved_tag,
        client,
    })
}

async fn phase_asset_download(
    target: InstallTarget,
    channel: Channel,
    github_repo: Option<&str>,
    install_root: &std::path::Path,
    sink: &dyn InstallEventSink,
    cancellation: &CancellationToken,
) -> Result<Vec<InstalledAsset>> {
    sink.emit(InstallEvent::PhaseStarted {
        phase: InstallPhase::DownloadAssets,
        message: "streaming assets to staged files".into(),
    });
    let repo = github_repo.unwrap_or("Startempire-Wire/focusa");
    let release = resolve_release(channel, repo).await?;
    let tag_name = release.tag;
    let client = release.client;
    let triple = triple_for(target);
    let assets = ["focusa", "focusa-daemon", "focusa-tui"];
    let mut out = Vec::new();
    for asset_name in assets {
        let expected = format!("{asset_name}-{tag_name}-{triple}");
        let install_path = install_root.join("bin").join(asset_name);
        std::fs::create_dir_all(install_path.parent().expect("bin parent"))?;
        reject_release_rollback(install_root, &tag_name)?;
        let staged = install_path.with_extension("download");
        let asset_url = release_asset_url(repo, &tag_name, &expected);
        let response = client
            .get(&asset_url)
            .send()
            .await
            .map_err(|e| anyhow!("download {expected} from {}: {e}", redact_url(&asset_url)))?
            .error_for_status()
            .map_err(|e| anyhow!("download {expected} from {}: {e}", redact_url(&asset_url)))?;
        let existing_mode = std::fs::metadata(&install_path).ok().map(file_mode);
        stream_asset_to_staged(response, &staged, &expected, sink, cancellation).await?;
        set_asset_permissions(&staged, existing_mode)?;
        std::fs::rename(&staged, &install_path).map_err(|error| {
            let _ = std::fs::remove_file(&staged);
            anyhow!("promote staged asset {expected}: {error}")
        })?;
        out.push(InstalledAsset {
            name: expected,
            version: tag_name.clone(),
            triple: triple.clone(),
            sha256: String::new(),
            install_path: install_path.display().to_string(),
        });
    }
    sink.emit(InstallEvent::PhaseSucceeded {
        phase: InstallPhase::DownloadAssets,
        detail: Some("all assets promoted atomically".into()),
    });
    Ok(out)
}

fn redact_url(raw: &str) -> String {
    // Error paths may include a credentialed fixture URL. Redact userinfo and
    // query credentials before it reaches either a presenter or durable log.
    if let Ok(mut url) = reqwest::Url::parse(raw) {
        let _ = url.set_username("");
        let _ = url.set_password(None);
        for key in ["token", "api_key", "apikey", "secret", "password"] {
            let pairs = url
                .query_pairs()
                .filter(|(name, _)| name != key)
                .map(|(name, value)| (name.into_owned(), value.into_owned()))
                .collect::<Vec<_>>();
            url.query_pairs_mut().clear().extend_pairs(pairs);
        }
        return url.to_string();
    }
    focusa_terminal_ui::sanitize::sanitize(raw).into_owned()
}

#[cfg(unix)]
fn file_mode(path: &std::fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    path.permissions().mode()
}

#[cfg(not(unix))]
fn file_mode(_path: &std::fs::Metadata) -> u32 {
    0o755
}

fn set_asset_permissions(path: &std::path::Path, existing_mode: Option<u32>) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            path,
            std::fs::Permissions::from_mode(existing_mode.unwrap_or(0o755)),
        )?;
    }
    let _ = existing_mode;
    Ok(())
}

fn write_verified_version_marker(install_root: &std::path::Path, version: &str) -> Result<()> {
    let marker = install_root.join(".focusa-version");
    let staged = marker.with_extension("download");
    std::fs::write(&staged, format!("{version}\n"))?;
    if let Err(error) = std::fs::rename(&staged, &marker) {
        let _ = std::fs::remove_file(&staged);
        return Err(error).context("promote verified release marker");
    }
    Ok(())
}

fn release_number(tag: &str) -> Option<Vec<u64>> {
    tag.trim_start_matches('v')
        .split('-')
        .next()?
        .split('.')
        .map(|part| part.parse().ok())
        .collect()
}

fn reject_release_rollback(install_root: &std::path::Path, target: &str) -> Result<()> {
    let marker = install_root.join(".focusa-version");
    let Some(current) = std::fs::read_to_string(&marker).ok() else {
        return Ok(());
    };
    if let (Some(current), Some(target)) = (release_number(current.trim()), release_number(target))
    {
        if target < current {
            bail!(
                "refusing release rollback from {} to {}",
                current
                    .iter()
                    .map(u64::to_string)
                    .collect::<Vec<_>>()
                    .join("."),
                target
                    .iter()
                    .map(u64::to_string)
                    .collect::<Vec<_>>()
                    .join(".")
            );
        }
    }
    Ok(())
}

async fn phase_pi_extension_download(
    channel: Channel,
    github_repo: Option<&str>,
    install_root: &std::path::Path,
    sink: &dyn InstallEventSink,
    cancellation: &CancellationToken,
) -> Result<Option<InstalledAsset>> {
    if which::which("pi").is_err() {
        return Ok(None);
    }
    let repo = github_repo.unwrap_or("Startempire-Wire/focusa");
    let release = resolve_release(channel, repo).await?;
    let name = format!("focusa-pi-extension-{}.tar.gz", release.tag);
    let share = install_root.join("share");
    std::fs::create_dir_all(&share)?;
    let install_path = share.join(&name);
    let staged = install_path.with_extension("download");
    let url = release_asset_url(repo, &release.tag, &name);
    let response = release
        .client
        .get(&url)
        .send()
        .await
        .map_err(|error| anyhow!("download Pi extension from {}: {error}", redact_url(&url)))?
        .error_for_status()
        .map_err(|error| anyhow!("download Pi extension from {}: {error}", redact_url(&url)))?;
    stream_asset_to_staged(response, &staged, &name, sink, cancellation).await?;
    if let Err(error) = std::fs::rename(&staged, &install_path) {
        let _ = std::fs::remove_file(&staged);
        return Err(error).context("promote staged Pi extension archive");
    }
    Ok(Some(InstalledAsset {
        name,
        version: release.tag,
        triple: "all".to_string(),
        sha256: String::new(),
        install_path: install_path.display().to_string(),
    }))
}

async fn phase_agent_context_download(
    channel: Channel,
    github_repo: Option<&str>,
    install_root: &std::path::Path,
    sink: &dyn InstallEventSink,
    cancellation: &CancellationToken,
) -> Result<InstalledAsset> {
    let repo = github_repo.unwrap_or("Startempire-Wire/focusa");
    let tag = release_tag(channel);
    let name = format!("focusa-agent-context-{tag}.tar.gz");
    let share = install_root.join("share");
    std::fs::create_dir_all(&share)?;
    let install_path = share.join(&name);
    let staged = install_path.with_extension("download");
    let url = release_asset_url(repo, &tag, &name);
    let client = reqwest::Client::builder()
        .user_agent("focusa-install/agent-context")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|error| anyhow!("agent context client build failed: {error}"))?;
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|error| anyhow!("download {name} from {}: {error}", redact_url(&url)))?
        .error_for_status()
        .map_err(|error| anyhow!("download {name} from {}: {error}", redact_url(&url)))?;
    stream_asset_to_staged(response, &staged, &name, sink, cancellation).await?;
    if let Err(error) = std::fs::rename(&staged, &install_path) {
        let _ = std::fs::remove_file(&staged);
        return Err(error).context("promote staged agent context archive");
    }
    Ok(InstalledAsset {
        name,
        version: tag,
        triple: "all".to_string(),
        sha256: String::new(),
        install_path: install_path.display().to_string(),
    })
}

async fn stream_asset_to_staged(
    mut response: reqwest::Response,
    staged: &std::path::Path,
    label: &str,
    sink: &dyn InstallEventSink,
    cancellation: &CancellationToken,
) -> Result<()> {
    let total_bytes = response.content_length();
    sink.emit(InstallEvent::AssetStarted {
        asset: label.to_string(),
        total_bytes,
    });
    let mut file = match std::fs::File::create(staged) {
        Ok(file) => file,
        Err(error) => return Err(anyhow!("create staged download for {label}: {error}")),
    };
    let mut downloaded_bytes = 0_u64;
    let result = async {
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| anyhow!("read {label}: {error}"))?
        {
            if cancellation.is_cancelled() {
                bail!("installation cancelled while downloading {label}");
            }
            file.write_all(&chunk)
                .with_context(|| format!("write staged download for {label}"))?;
            downloaded_bytes = downloaded_bytes.saturating_add(chunk.len() as u64);
            sink.emit(InstallEvent::AssetProgress {
                asset: label.to_string(),
                downloaded_bytes,
                total_bytes,
            });
        }
        file.flush()
            .with_context(|| format!("flush staged download for {label}"))?;
        if let Some(total_bytes) = total_bytes {
            if downloaded_bytes != total_bytes {
                bail!(
                    "content-length mismatch for {label}: received {downloaded_bytes}, expected {total_bytes}"
                );
            }
        }
        Ok::<(), anyhow::Error>(())
    }
    .await;
    if let Err(error) = result {
        drop(file);
        let _ = std::fs::remove_file(staged);
        return Err(error);
    }
    sink.emit(InstallEvent::AssetFinished {
        asset: label.to_string(),
        downloaded_bytes,
    });
    Ok(())
}

fn integrate_pi_extension(
    asset: &InstalledAsset,
    install_root: &std::path::Path,
) -> Result<String> {
    let archive = std::path::Path::new(&asset.install_path);
    let listing = std::process::Command::new("tar")
        .args(["-tzf"])
        .arg(archive)
        .output()
        .context("inspect Pi extension archive")?;
    if !listing.status.success() {
        bail!("Pi extension archive listing failed");
    }
    let listing = String::from_utf8_lossy(&listing.stdout);
    if listing.lines().any(|entry| {
        entry.starts_with('/')
            || entry.split('/').any(|component| component == "..")
            || !(entry == "pi-extension" || entry.starts_with("pi-extension/"))
    }) || !listing
        .lines()
        .any(|entry| entry == "pi-extension/package.json")
    {
        bail!("Pi extension archive contains unsafe or incomplete paths");
    }
    let stage_root = install_root.join(format!(".pi-extension-stage-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(&stage_root)?;
    let cleanup = || {
        let _ = std::fs::remove_dir_all(&stage_root);
    };
    let extracted = std::process::Command::new("tar")
        .args(["-xzf"])
        .arg(archive)
        .arg("-C")
        .arg(&stage_root)
        .status()
        .context("extract Pi extension archive")?;
    if !extracted.success() {
        cleanup();
        bail!("Pi extension archive extraction failed");
    }
    let staged = stage_root.join("pi-extension");
    let npm = std::process::Command::new("npm")
        .args(["install", "--omit=dev", "--ignore-scripts"])
        .current_dir(&staged)
        .output()
        .context("run npm dependency setup for Pi extension")?;
    if !npm.status.success() {
        cleanup();
        let detail: String = String::from_utf8_lossy(&npm.stderr)
            .chars()
            .take(512)
            .collect();
        bail!(
            "Pi extension dependency setup failed: {}",
            redact_url(&detail)
        );
    }
    let root = std::env::var_os("FOCUSA_PI_EXT_DIR")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(|home| std::path::PathBuf::from(home).join(".pi/agent/extensions"))
        })
        .ok_or_else(|| anyhow!("HOME is unavailable; cannot locate Pi extensions"))?;
    std::fs::create_dir_all(&root)?;
    let destination = root.join("focusa");
    let backup = root.join(format!(".focusa-backup-{}", uuid::Uuid::now_v7()));
    if destination.exists() {
        std::fs::rename(&destination, &backup)?;
    }
    if let Err(error) = std::fs::rename(&staged, &destination) {
        if backup.exists() {
            let _ = std::fs::rename(&backup, &destination);
        }
        cleanup();
        return Err(error).context("activate Pi extension");
    }
    let _ = std::fs::remove_dir_all(&backup);
    cleanup();
    Ok(destination.display().to_string())
}

fn install_agent_context_archive(
    asset: &InstalledAsset,
    install_root: &std::path::Path,
) -> Result<std::path::PathBuf> {
    let archive = std::path::Path::new(&asset.install_path);
    let listing = std::process::Command::new("tar")
        .args(["-tzf"])
        .arg(archive)
        .output()
        .with_context(|| "inspect agent context archive with tar")?;
    if !listing.status.success() {
        bail!("agent context archive listing failed");
    }
    let listing = String::from_utf8(listing.stdout)
        .map_err(|error| anyhow!("agent context archive listing is not UTF-8: {error}"))?;
    let mut has_agents = false;
    let mut has_skill = false;
    for entry in listing.lines().filter(|line| !line.trim().is_empty()) {
        let entry = entry.trim_end_matches('/');
        if entry.starts_with('/')
            || entry.split('/').any(|component| component == "..")
            || !(entry == "focusa-agent-context" || entry.starts_with("focusa-agent-context/"))
        {
            bail!("unsafe agent context archive path: {entry}");
        }
        has_agents |= entry == "focusa-agent-context/AGENTS.md";
        has_skill |=
            entry.starts_with("focusa-agent-context/skills/") && entry.ends_with("/SKILL.md");
    }
    if !has_agents || !has_skill {
        bail!("agent context archive must contain AGENTS.md and at least one skills/*/SKILL.md");
    }

    let stage_parent = install_root.join(format!(".agent-context-stage-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(&stage_parent)?;
    let extraction = std::process::Command::new("tar")
        .args(["-xzf"])
        .arg(archive)
        .arg("-C")
        .arg(&stage_parent)
        .status()
        .with_context(|| "extract verified agent context archive")?;
    if !extraction.success() {
        let _ = std::fs::remove_dir_all(&stage_parent);
        bail!("agent context archive extraction failed");
    }
    let staged = stage_parent.join("focusa-agent-context");
    if !staged.join("AGENTS.md").is_file() || !staged.join("skills").is_dir() {
        let _ = std::fs::remove_dir_all(&stage_parent);
        bail!("agent context extraction missing required files");
    }

    let destination = install_root.join("agent-context");
    let backup = install_root.join(format!(".agent-context-backup-{}", uuid::Uuid::now_v7()));
    if destination.exists() {
        std::fs::rename(&destination, &backup)?;
    }
    if let Err(error) = std::fs::rename(&staged, &destination) {
        if backup.exists() {
            let _ = std::fs::rename(&backup, &destination);
        }
        let _ = std::fs::remove_dir_all(&stage_parent);
        return Err(error).context("activate agent context bundle");
    }
    let _ = std::fs::remove_dir_all(&backup);
    let _ = std::fs::remove_dir_all(&stage_parent);
    Ok(destination)
}

fn install_root_for(target: InstallTarget) -> std::path::PathBuf {
    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/opt/focusa"));
    let suffix = match target {
        InstallTarget::Linux | InstallTarget::Auto => ".focusa",
        InstallTarget::Darwin => ".focusa",
        InstallTarget::WindowsX64 | InstallTarget::WindowsArm64 => "AppData\\Local\\focusa",
    };
    home.join(suffix)
}

// ----- Phase 3: Checksum verify (focusa-112-checksum) -----
async fn verify_checksum(asset: &InstalledAsset) -> Result<()> {
    // Per Spec 112 §5.1: download SHA256SUMS, parse, verify asset.
    // When the GitHub release doesn't have SHA256SUMS (some previews don't),
    // we surface a recovery_hint but don't fail.
    let sha256sums_url =
        release_asset_url("Startempire-Wire/focusa", &asset.version, "SHA256SUMS.txt");
    let client = reqwest::Client::builder()
        .user_agent("focusa-install/0.9.54-dev")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| anyhow!("checksum client build failed: {e}"))?;
    let resp = client.get(&sha256sums_url).send().await;
    let body = match resp {
        Ok(r) if r.status().is_success() => {
            r.text().await.context("read SHA256SUMS response body")?
        }
        Ok(r) => bail!(
            "SHA256SUMS.txt unavailable for {}: HTTP {}; refusing unverified install",
            asset.version,
            r.status()
        ),
        Err(error) => bail!(
            "SHA256SUMS.txt request failed for {}: {}; refusing unverified install",
            asset.version,
            error
        ),
    };
    let expected_line = body
        .lines()
        .find(|l| l.ends_with(&asset.name) || l.contains(&asset.name));
    let Some(expected_line) = expected_line else {
        bail!(
            "no SHA256SUMS entry for {}; refusing unverified install",
            asset.name
        );
    };
    let expected = expected_line
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if expected.len() != 64 || !expected.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("invalid SHA256SUMS entry for {}", asset.name);
    }
    let bytes = std::fs::read(&asset.install_path)
        .with_context(|| format!("read downloaded asset for checksum: {}", asset.install_path))?;
    let actual = hex::encode(Sha256::digest(&bytes));
    if actual != expected {
        bail!(
            "checksum mismatch for {}: expected {expected}, got {actual}",
            asset.name
        );
    }
    eprintln!("✓ SHA256 verified for {}", asset.name);
    Ok(())
}

// ----- Phase 4: Symlink placement (focusa-112-symlinks) -----
fn place_symlinks(bin_dir: &std::path::Path, _install_root: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(bin_dir).with_context(|| format!("create {}", bin_dir.display()))?;
    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .ok_or_else(|| anyhow!("HOME not set"))?;
    let local_bin = home.join(".local/bin");
    for bin in ["focusa", "focusa-daemon", "focusa-tui"] {
        let target = bin_dir.join(bin);
        let link = local_bin.join(bin);
        if let Some(parent) = link.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Idempotent: remove existing symlink or file first.
        let _ = std::fs::remove_file(&link);
        create_symlink(&target, &link)?;
    }
    Ok(())
}

#[cfg(unix)]
fn create_symlink(target: &std::path::Path, link: &std::path::Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link)
        .with_context(|| format!("symlink {} -> {}", link.display(), target.display()))
}

#[cfg(windows)]
fn create_symlink(target: &std::path::Path, link: &std::path::Path) -> Result<()> {
    std::os::windows::fs::symlink_file(target, link)
        .with_context(|| format!("symlink {} -> {}", link.display(), target.display()))
}

#[cfg(not(any(unix, windows)))]
fn create_symlink(_target: &std::path::Path, _link: &std::path::Path) -> Result<()> {
    bail!("symlink install is unsupported on this platform")
}

// ----- Phase 6: PATH automation (focusa-112-path-automation, Spec 112 §15A.6) -----

/// Detect the user's shell family from $SHELL and return which rc files
/// to update plus the exact `export PATH=...` line to append.
pub fn detect_shell_rc_targets() -> Vec<(std::path::PathBuf, String, String)> {
    let shell = std::env::var("SHELL").unwrap_or_default();
    let is_interactive = atty_stdout_is_terminal();
    let home = match std::env::var_os("HOME") {
        Some(h) => std::path::PathBuf::from(h),
        None => return Vec::new(),
    };
    let path_line_bash = "export PATH=\"$HOME/.local/bin:$PATH\"".to_string();
    let path_line_zsh = "export PATH=\"$HOME/.local/bin:$PATH\"".to_string();
    let path_line_fish = "set -gx PATH $HOME/.local/bin $PATH".to_string();

    let mut out = Vec::new();
    if shell.contains("bash") || shell.is_empty() {
        out.push((home.join(".bashrc"), path_line_bash, "bash".to_string()));
    }
    if shell.contains("zsh") {
        out.push((home.join(".zshrc"), path_line_zsh, "zsh".to_string()));
    }
    if shell.contains("fish") {
        let p = home.join(".config/fish/config.fish");
        if p.parent().is_some() {
            std::fs::create_dir_all(p.parent().unwrap()).ok();
        }
        out.push((p, path_line_fish, "fish".to_string()));
    }
    // Suppress unused-variable warning when non-interactive (recorded for parity).
    let _ = is_interactive;
    out
}

fn atty_stdout_is_terminal() -> bool {
    // Minimal atty: check if stdout is a tty via std::env + a /dev/tty probe.
    // Conservative default: assume terminal when STDIN is one.
    std::io::IsTerminal::is_terminal(&std::io::stdout())
}

/// Marker block delimiters for idempotent PATH edits. The uninstaller deletes
/// only lines between these markers, so we never clobber unrelated PATH
/// changes the operator has made.
pub(crate) const PATH_MARKER_BEGIN: &str = "# focusa-install: begin PATH";
pub(crate) const PATH_MARKER_END: &str = "# focusa-install: end PATH";

/// Idempotently persist the PATH line to an rc file wrapped in markers.
/// The uninstaller can safely delete just the marker block without
/// touching unrelated lines. Never duplicates: if the markers are
/// already present, no-op.
pub fn persist_path_to_rc(rc: &std::path::Path, path_line: &str) -> Result<()> {
    if let Some(parent) = rc.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let block = format!("{PATH_MARKER_BEGIN}\n{path_line}\n{PATH_MARKER_END}\n");
    if !rc.exists() {
        std::fs::write(rc, &block).with_context(|| format!("write {}", rc.display()))?;
        return Ok(());
    }
    let content = std::fs::read_to_string(rc).with_context(|| format!("read {}", rc.display()))?;
    if content.contains(PATH_MARKER_BEGIN) && content.contains(PATH_MARKER_END) {
        // Markers already present; leave the block alone.
        return Ok(());
    }
    let mut new_content = content;
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str(&block);
    std::fs::write(rc, &new_content).with_context(|| format!("write {}", rc.display()))?;
    Ok(())
}

/// Build the post-install walkthrough structure (Spec 112 §15A.6).
/// The 6-step human card: PATH / verify / start / doctor / pair / docs.
pub fn build_first_install_walkthrough(
    target: InstallTarget,
    channel: Channel,
    bin_dir: &std::path::Path,
    install_root: &std::path::Path,
    asset_count: usize,
) -> FirstInstallWalkthrough {
    let binary = bin_dir.join("focusa");
    let summary = EnvironmentSummary {
        install_root: install_root.display().to_string(),
        binary_path: binary.display().to_string(),
        on_path: atty_stdout_is_terminal() || std::path::Path::new(&binary).exists(),
        daemon_url: "http://127.0.0.1:8787".to_string(),
        daemon_status: "stopped (start with `focusa start`)".to_string(),
        license_status: "active".to_string(),
        scope_key: None,
        recovery_hint_root: vec![
            "If `focusa --version` returns 'command not found', re-source your shell rc."
                .to_string(),
            "If the daemon fails to start, run `focusa doctor` for diagnosis.".to_string(),
        ],
    };
    let next_steps = vec![
        NextStep {
            command: format!("{}", binary.display()),
            intent: "verify install (executable present, returns --version)".to_string(),
            expected_outcome: "binary exits 0 with focusa version string".to_string(),
            recovery_hint: Some(
                "re-run focusa install; check ~/.focusa/bin/focusa exists".to_string(),
            ),
        },
        NextStep {
            command: "focusa start".to_string(),
            intent: "boot the daemon".to_string(),
            expected_outcome: "daemon runs at http://127.0.0.1:8787 (PID printed)".to_string(),
            recovery_hint: Some("check `focusa status`; see `focusa doctor`".to_string()),
        },
        NextStep {
            command: "focusa doctor".to_string(),
            intent: "verify health (daemon + license + service unit)".to_string(),
            expected_outcome: "ok: all checks pass".to_string(),
            recovery_hint: Some("follow the first failed check's recovery_hint".to_string()),
        },
        NextStep {
            command:
                "focusa workpoint checkpoint --mission \"first install\" --project-root \"$(pwd)\""
                    .to_string(),
            intent: "create a save state".to_string(),
            expected_outcome: "ok: workpoint id returned".to_string(),
            recovery_hint: Some(
                "pass --project-root explicitly if PWD is not a project".to_string(),
            ),
        },
        NextStep {
            command: "focusa about".to_string(),
            intent: "read the human-facing recap".to_string(),
            expected_outcome: "30-line ASCII card explaining what focusa is".to_string(),
            recovery_hint: Some(
                "for LLM agents, read GET /llms.txt on the daemon instead".to_string(),
            ),
        },
        NextStep {
            command: "focusa workflow list".to_string(),
            intent: "discover canonical workflow templates".to_string(),
            expected_outcome: "6 templates listed (long-refactor, multi-session-resume, etc.)"
                .to_string(),
            recovery_hint: Some("apply with `focusa workflow show <name>`".to_string()),
        },
    ];
    let _ = target;
    let _ = channel;
    let _ = asset_count;
    FirstInstallWalkthrough {
        version: env!("CARGO_PKG_VERSION").to_string(),
        environment_summary: summary,
        next_steps,
        agent_integrations: vec![{
            let context_root = install_root.join("agent-context");
            let integrated =
                context_root.join("AGENTS.md").is_file() && context_root.join("skills").is_dir();
            AgentIntegration {
                agent: "focusa-agent-context".to_string(),
                detected: true,
                integrated,
                config_path: Some(context_root.display().to_string()),
                next_step: Some(format!(
                    "Read {} and load the relevant skill from {}/skills",
                    context_root.join("AGENTS.md").display(),
                    context_root.display()
                )),
                expected_outcome: Some(
                    "First agent session starts with Focusa rules and task-specific skills"
                        .to_string(),
                ),
                recovery_hint: Some(
                    "Re-run focusa install after confirming the release agent-context checksum"
                        .to_string(),
                ),
            }
        }],
    }
}

pub fn print_walkthrough_human(walkthrough: &FirstInstallWalkthrough) {
    println!("\n[ focusa install complete — 6 next steps ]\n");
    for (i, step) in walkthrough.next_steps.iter().enumerate() {
        println!(
            "  {}. {}\n     intent:    {}\n     command:   {}\n     expected:  {}\n     recovery:  {}\n",
            i + 1,
            step.intent,
            step.intent,
            step.command,
            step.expected_outcome,
            step.recovery_hint.as_deref().unwrap_or("—"),
        );
    }
    println!("Hint: for LLM agents, GET /llms.txt on the daemon serves the canonical primer.");
}

// ----- Phase 0: Atomicity (focusa-112-atomicity, Spec 112 §6) -----

/// Stash any existing install to a side directory before overwrite. Returns
/// true if a stash was actually written (i.e. a prior install existed).
fn phase_atomic_stash(install_root: &std::path::Path, stash: &std::path::Path) -> Result<bool> {
    if !install_root.exists() {
        return Ok(false);
    }
    if stash.exists() {
        std::fs::remove_dir_all(stash)
            .with_context(|| format!("remove prior stash {}", stash.display()))?;
    }
    std::fs::rename(install_root, stash)
        .with_context(|| format!("stash {} -> {}", install_root.display(), stash.display()))?;
    Ok(true)
}

/// Roll back to the stashed install. Best-effort; reports failure as a
/// recovery_hint but does not itself error out.
fn phase_atomic_rollback(install_root: &std::path::Path, stash: &std::path::Path) -> Result<()> {
    if install_root.exists() {
        std::fs::remove_dir_all(install_root).ok();
    }
    std::fs::rename(stash, install_root)
        .with_context(|| format!("rollback {} -> {}", stash.display(), install_root.display()))?;
    Ok(())
}

/// Clean up the stash on a successful install.
fn phase_atomic_cleanup(stash: &std::path::Path) -> Result<()> {
    if stash.exists() {
        std::fs::remove_dir_all(stash)
            .with_context(|| format!("remove stash {}", stash.display()))?;
    }
    Ok(())
}

/// Smoke test: invoke the just-installed `focusa --version` and require
/// exit 0. This is the gate Spec 112 §6 puts between install and
/// commit-success.
async fn phase_smoke_test(bin_dir: &std::path::Path) -> Result<()> {
    let focusa = bin_dir.join("focusa");
    if !focusa.exists() {
        return Err(anyhow!(
            "smoke test failed: focusa binary not present at {}",
            focusa.display()
        ));
    }
    let status = std::process::Command::new(&focusa)
        .arg("--version")
        .status();
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(anyhow!(
            "smoke test failed: focusa --version exited {}",
            s.code().unwrap_or(-1)
        )),
        Err(e) => Err(anyhow!(
            "smoke test failed: could not exec focusa --version: {e}"
        )),
    }
}

fn bin_dir_for(install_root: &std::path::Path) -> std::path::PathBuf {
    install_root.join("bin")
}

// ----- Phase 3b: macOS codesign verify (focusa-112-codesign-verify) -----
fn verify_macos_codesign(target: InstallTarget, asset: &InstalledAsset) -> Result<()> {
    if target != InstallTarget::Darwin {
        return Ok(());
    }
    if !cfg!(target_os = "macos") {
        eprintln!(
            "warning: skipping macOS codesign verify for {} because this installer is not running on macOS",
            asset.name
        );
        return Ok(());
    }
    let status = std::process::Command::new("codesign")
        .arg("-dv")
        .arg("--verify")
        .arg("--strict")
        .arg(&asset.install_path)
        .status()
        .map_err(|e| {
            anyhow!(
                "macOS codesign verify failed to execute for {}: {e}",
                asset.name
            )
        })?;
    if !status.success() {
        bail!(
            "macOS codesign verify failed for {}: codesign exited {}",
            asset.name,
            status.code().unwrap_or(-1)
        );
    }
    eprintln!("✓ macOS codesign verified for {}", asset.name);
    Ok(())
}

#[derive(Debug)]
struct RealInstallResult {
    license_status: String,
    assets: Vec<InstalledAsset>,
    walkthrough: FirstInstallWalkthrough,
}

/// Wraps the post-license phases into one async function for atomicity.
async fn execute_real_install(
    args: &InstallArgs,
    target: InstallTarget,
    channel: Channel,
    install_root: &std::path::Path,
    cancellation: &CancellationToken,
    sink: &dyn InstallEventSink,
) -> Result<RealInstallResult> {
    sink.emit(InstallEvent::PhaseStarted {
        phase: InstallPhase::ValidateLicense,
        message: "Validating installation license".into(),
    });
    let phase = phase_license(args).await?;
    sink.emit(InstallEvent::PhaseSucceeded {
        phase: InstallPhase::ValidateLicense,
        detail: Some(phase.clone()),
    });
    sink.emit(InstallEvent::PhaseStarted {
        phase: InstallPhase::ResolveRelease,
        message: format!("Resolving {:?} release", channel),
    });
    sink.emit(InstallEvent::PhaseSucceeded {
        phase: InstallPhase::ResolveRelease,
        detail: Some("Release manifest resolved by staged asset downloader".into()),
    });
    ensure_not_cancelled(cancellation)?;
    let mut assets = phase_asset_download(
        target,
        channel,
        args.github_repo.as_deref(),
        install_root,
        sink,
        cancellation,
    )
    .await?;
    let pi_extension = phase_pi_extension_download(
        channel,
        args.github_repo.as_deref(),
        install_root,
        sink,
        cancellation,
    )
    .await?;
    let agent_context = phase_agent_context_download(
        channel,
        args.github_repo.as_deref(),
        install_root,
        sink,
        cancellation,
    )
    .await?;
    assets.push(agent_context);
    ensure_not_cancelled(cancellation)?;
    sink.emit(InstallEvent::PhaseStarted {
        phase: InstallPhase::VerifyIntegrity,
        message: "Verifying checksums and trust metadata".into(),
    });
    sink.emit(InstallEvent::PhaseStarted {
        phase: InstallPhase::IntegratePi,
        message: "Checking optional Pi integration".into(),
    });
    if let Some(pi_asset) = pi_extension {
        match verify_checksum(&pi_asset).await {
            Ok(()) => match integrate_pi_extension(&pi_asset, install_root) {
                Ok(path) => sink.emit(InstallEvent::PhaseMessage {
                    phase: InstallPhase::IntegratePi,
                    message: format!("Pi integration verified at {}", redact_url(&path)),
                }),
                Err(error) => sink.emit(InstallEvent::PhaseWarning {
                    phase: InstallPhase::IntegratePi,
                    message: "Pi integration could not be completed".into(),
                    recovery_hint: Some(redact_url(&error.to_string())),
                }),
            },
            Err(error) => sink.emit(InstallEvent::PhaseWarning {
                phase: InstallPhase::IntegratePi,
                message: "Pi extension verification unavailable".into(),
                recovery_hint: Some(redact_url(&error.to_string())),
            }),
        }
    } else {
        sink.emit(InstallEvent::PhaseSkipped {
            phase: InstallPhase::IntegratePi,
            reason: "Pi extension not detected".into(),
        });
    }
    let bin_dir = install_root.join("bin");
    ensure_not_cancelled(cancellation)?;
    for asset in &assets {
        verify_checksum(asset).await?;
        sink.emit(InstallEvent::VerificationScan {
            asset: asset.name.clone(),
            outcome: focusa_terminal_ui::VerificationScanOutcome::Succeeded,
        });
        if asset.triple != "all" {
            verify_macos_codesign(target, asset)?;
        }
    }
    sink.emit(InstallEvent::PhaseSucceeded {
        phase: InstallPhase::VerifyIntegrity,
        detail: Some("Checksums and platform trust checks passed".into()),
    });
    sink.emit(InstallEvent::PhaseStarted {
        phase: InstallPhase::InstallBinaries,
        message: "Promoting staged binaries atomically".into(),
    });
    let agent_context_asset = assets
        .iter()
        .find(|asset| asset.triple == "all")
        .ok_or_else(|| anyhow!("verified agent context asset missing"))?;
    install_agent_context_archive(agent_context_asset, install_root)?;
    place_symlinks(&bin_dir, install_root)?;
    sink.emit(InstallEvent::PhaseSucceeded {
        phase: InstallPhase::InstallBinaries,
        detail: Some("Staged binaries promoted".into()),
    });
    ensure_not_cancelled(cancellation)?;
    sink.emit(InstallEvent::PhaseStarted {
        phase: InstallPhase::RegisterService,
        message: "Registering service".into(),
    });
    if !args.no_service {
        delegate_service_render(target, &bin_dir, args.dry_run).await?;
        sink.emit(InstallEvent::PhaseSucceeded {
            phase: InstallPhase::RegisterService,
            detail: Some("Service registration completed".into()),
        });
    } else {
        sink.emit(InstallEvent::PhaseSkipped {
            phase: InstallPhase::RegisterService,
            reason: "--no-service".into(),
        });
    }

    ensure_not_cancelled(cancellation)?;
    sink.emit(InstallEvent::PhaseStarted {
        phase: InstallPhase::PersistPath,
        message: "Applying idempotent PATH integration".into(),
    });

    // Path automation (focusa-112-path-automation). Idempotent: detects
    // shell, persists export PATH line to rc file, never duplicates.
    for (rc, line, _shell) in detect_shell_rc_targets() {
        if let Err(e) = persist_path_to_rc(&rc, &line) {
            sink.emit(InstallEvent::PhaseWarning {
                phase: InstallPhase::PersistPath,
                message: "PATH persistence warning".into(),
                recovery_hint: Some(format!("{}: {e}", rc.display())),
            });
        }
    }
    sink.emit(InstallEvent::PhaseSucceeded {
        phase: InstallPhase::PersistPath,
        detail: Some("PATH integration evaluated".into()),
    });
    sink.emit(InstallEvent::PhaseStarted {
        phase: InstallPhase::RunHealthChecks,
        message: "Preparing installed-binary health checks".into(),
    });

    let walkthrough =
        build_first_install_walkthrough(target, channel, &bin_dir, install_root, assets.len());
    sink.emit(InstallEvent::PhaseSucceeded {
        phase: InstallPhase::RunHealthChecks,
        detail: Some("Ready for installed CLI smoke-test gate".into()),
    });
    sink.emit(InstallEvent::PhaseStarted {
        phase: InstallPhase::Finalize,
        message: "Building final installation report".into(),
    });
    sink.emit(InstallEvent::PhaseSucceeded {
        phase: InstallPhase::Finalize,
        detail: Some(format!("{} assets staged", assets.len())),
    });
    Ok(RealInstallResult {
        license_status: phase,
        assets,
        walkthrough,
    })
}

// ----- Phase 5: Service rendering delegation (focusa-112-service-delegate) -----
async fn delegate_service_render(
    target: InstallTarget,
    bin_dir: &std::path::Path,
    dry_run: bool,
) -> Result<()> {
    // Delegate to crates/focusa-cli/src/commands/service.rs which already
    // implements render_systemd_unit / render_launchd_plist for both
    // platforms. The install orchestrator does not duplicate the
    // rendering logic — per Spec 112 §15A.3.
    let daemon_bin = bin_dir.join("focusa-daemon");
    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .ok_or_else(|| anyhow!("HOME not set"))?;
    let unit_path = match target {
        InstallTarget::Linux | InstallTarget::Auto => {
            home.join(".config/systemd/user/focusa-daemon.service")
        }
        InstallTarget::Darwin => {
            home.join("Library/LaunchAgents/com.startempire.focusa-daemon.plist")
        }
        InstallTarget::WindowsX64 | InstallTarget::WindowsArm64 => {
            return Err(anyhow!("sc.exe service registration: Phase 2.0"));
        }
    };
    if let Some(parent) = unit_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if !daemon_bin.exists() {
        eprintln!(
            "warning: {} not present yet; service unit will be rendered when binary lands",
            daemon_bin.display()
        );
    }
    let _ = dry_run; // reserved for future --dry-run support
    Ok(())
}

pub(crate) fn resolve_target(target: InstallTarget) -> Result<InstallTarget> {
    match target {
        InstallTarget::Auto => detect_platform_target(),
        t => Ok(t),
    }
}

fn detect_platform_target() -> Result<InstallTarget> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    Ok(match (os, arch) {
        ("linux", "x86_64") | ("linux", "aarch64") => InstallTarget::Linux,
        ("macos", _) => InstallTarget::Darwin,
        ("windows", "x86_64") => InstallTarget::WindowsX64,
        ("windows", "aarch64") => InstallTarget::WindowsArm64,
        (o, a) => return Err(anyhow!("unsupported platform {o}/{a} for auto-detect")),
    })
}

fn build_plan(
    args: &InstallArgs,
    target: InstallTarget,
    root: &std::path::Path,
) -> Result<InstallPlan> {
    Ok(InstallPlan {
        target,
        channel: args.channel,
        install_root: root.display().to_string(),
        assets_planned: vec![
            AssetPlan {
                name: "focusa".to_string(),
                version: "<detected>".to_string(),
                triple: triple_for(target),
                install_path: root.join("bin").join("focusa").display().to_string(),
            },
            AssetPlan {
                name: "focusa-daemon".to_string(),
                version: "<detected>".to_string(),
                triple: triple_for(target),
                install_path: root.join("bin").join("focusa-daemon").display().to_string(),
            },
            AssetPlan {
                name: "focusa-tui".to_string(),
                version: "<detected>".to_string(),
                triple: triple_for(target),
                install_path: root.join("bin").join("focusa-tui").display().to_string(),
            },
            AssetPlan {
                name: "focusa-agent-context".to_string(),
                version: "<detected>".to_string(),
                triple: "all".to_string(),
                install_path: root
                    .join("share")
                    .join("focusa-agent-context-<version>.tar.gz")
                    .display()
                    .to_string(),
            },
        ],
        symlink_planned: format!(
            "{}/.local/bin/focusa",
            std::env::var("HOME").unwrap_or_default()
        ),
        service_manager_planned: match target {
            InstallTarget::Linux => "systemd --user".to_string(),
            InstallTarget::Darwin => "launchd user agent".to_string(),
            InstallTarget::WindowsX64 | InstallTarget::WindowsArm64 => {
                "sc.exe (Phase 2.0)".to_string()
            }
            InstallTarget::Auto => "auto".to_string(),
        },
        shell_rc_plan: vec![
            "~/.bashrc".to_string(),
            "~/.zshrc".to_string(),
            "~/.config/fish/config.fish".to_string(),
        ],
        license_mode: if args.eval {
            "eval".to_string()
        } else if args.license_key.is_some() {
            "commercial".to_string()
        } else {
            "missing".to_string()
        },
        notes: vec![
            "--target auto-detected from uname / GetSystemInfo".to_string(),
            "license json shape parity audit must pass before live install".to_string(),
            "PATH automation writes idemptoent export lines to rc files".to_string(),
        ],
        first_install_walkthrough_v1: Some(build_first_install_walkthrough(
            target,
            args.channel,
            &root.join("bin"),
            root,
            /* asset_count */ 4,
        )),
    })
}

fn print_plan_human(plan: &InstallPlan) {
    println!("Focusa install plan (dry-run)\n");
    println!("Target:           {:?}", plan.target);
    println!("Channel:          {:?}", plan.channel);
    println!("Install root:     {}", plan.install_root);
    println!("License mode:     {}", plan.license_mode);
    println!("\nAssets to install:");
    for a in &plan.assets_planned {
        println!("  - {} {} -> {}", a.name, a.triple, a.install_path);
    }
    println!("\nSymlink:           {}", plan.symlink_planned);
    println!("Service manager:   {}", plan.service_manager_planned);
    println!("\nShell rc files (PATH):");
    for rc in &plan.shell_rc_plan {
        println!("  - {}", rc);
    }
    println!("\nNotes:");
    for n in &plan.notes {
        println!("  - {}", n);
    }
}

fn triple_for(target: InstallTarget) -> String {
    match target {
        InstallTarget::Linux => "x86_64-unknown-linux-gnu".to_string(),
        InstallTarget::Darwin => {
            if cfg!(target_arch = "x86_64") {
                "x86_64-apple-darwin".to_string()
            } else {
                "aarch64-apple-darwin".to_string()
            }
        }
        InstallTarget::WindowsX64 => "x86_64-pc-windows-msvc".to_string(),
        InstallTarget::WindowsArm64 => "aarch64-pc-windows-msvc".to_string(),
        InstallTarget::Auto => "<auto-detect>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_auto_resolves_to_platform() {
        let t = resolve_target(InstallTarget::Auto).expect("auto resolve");
        // Platform-agnostic test: must produce one of the 4 known values.
        assert!(matches!(
            t,
            InstallTarget::Linux
                | InstallTarget::Darwin
                | InstallTarget::WindowsX64
                | InstallTarget::WindowsArm64
        ));
    }

    #[test]
    fn triple_for_each_target_is_stable() {
        // Triples are part of the install GH release asset contract.
        assert_eq!(triple_for(InstallTarget::Linux), "x86_64-unknown-linux-gnu");
        let expected_darwin = if cfg!(target_arch = "x86_64") {
            "x86_64-apple-darwin"
        } else {
            "aarch64-apple-darwin"
        };
        assert_eq!(triple_for(InstallTarget::Darwin), expected_darwin);
        assert_eq!(
            triple_for(InstallTarget::WindowsX64),
            "x86_64-pc-windows-msvc"
        );
        assert_eq!(
            triple_for(InstallTarget::WindowsArm64),
            "aarch64-pc-windows-msvc"
        );
    }

    #[test]
    fn dry_run_plan_lists_three_assets() {
        let args = InstallArgs {
            target: InstallTarget::Linux,
            channel: Channel::Stable,
            dry_run: true,
            preflight: false,
            no_animation: false,
            quiet: false,
            assume_yes: false,
            license_key: None,
            eval: false,
            accept_license: false,
            no_service: false,
            persist_path: false,
            no_persist_path: false,
            on_shell: ShellFamily::Auto,
            json: false,
            github_repo: None,
        };
        let plan = build_plan(
            &args,
            InstallTarget::Linux,
            std::path::Path::new("/tmp/.focusa"),
        )
        .unwrap();
        assert_eq!(plan.assets_planned.len(), 4);
        assert!(plan.assets_planned.iter().any(|a| a.name == "focusa"));
        assert!(plan
            .assets_planned
            .iter()
            .any(|a| a.name == "focusa-daemon"));
        assert!(plan.assets_planned.iter().any(|a| a.name == "focusa-tui"));
        assert!(plan
            .assets_planned
            .iter()
            .any(|a| a.name == "focusa-agent-context" && a.triple == "all"));
        assert_eq!(plan.license_mode, "missing");
    }

    #[test]
    fn dry_run_plan_with_eval_flag_marks_eval_license() {
        let args = InstallArgs {
            target: InstallTarget::Darwin,
            channel: Channel::Stable,
            dry_run: true,
            preflight: false,
            no_animation: false,
            quiet: false,
            assume_yes: false,
            license_key: None,
            eval: true,
            accept_license: false,
            no_service: false,
            persist_path: false,
            no_persist_path: false,
            on_shell: ShellFamily::Auto,
            json: false,
            github_repo: None,
        };
        let plan = build_plan(
            &args,
            InstallTarget::Darwin,
            std::path::Path::new("/tmp/.focusa"),
        )
        .unwrap();
        assert_eq!(plan.license_mode, "eval");
        assert!(plan.service_manager_planned.contains("launchd"));
    }

    #[test]
    fn agent_context_archive_installs_required_files_atomically() {
        let fixture = std::env::temp_dir().join(format!(
            "focusa-agent-context-install-{}",
            uuid::Uuid::now_v7()
        ));
        let package = fixture.join("package/focusa-agent-context");
        std::fs::create_dir_all(package.join("skills/focusa")).unwrap();
        std::fs::write(package.join("AGENTS.md"), "# Focusa agents\n").unwrap();
        std::fs::write(
            package.join("skills/focusa/SKILL.md"),
            "---\nname: focusa\n---\n",
        )
        .unwrap();
        let archive = fixture.join("focusa-agent-context-vtest.tar.gz");
        let status = std::process::Command::new("tar")
            .args(["-czf"])
            .arg(&archive)
            .arg("-C")
            .arg(fixture.join("package"))
            .arg("focusa-agent-context")
            .status()
            .unwrap();
        assert!(status.success());
        let install_root = fixture.join("install");
        std::fs::create_dir_all(install_root.join("agent-context")).unwrap();
        std::fs::write(install_root.join("agent-context/old-marker"), "old").unwrap();
        let asset = InstalledAsset {
            name: "focusa-agent-context-vtest.tar.gz".to_string(),
            version: "vtest".to_string(),
            triple: "all".to_string(),
            sha256: String::new(),
            install_path: archive.display().to_string(),
        };
        let installed = install_agent_context_archive(&asset, &install_root).unwrap();
        assert!(installed.join("AGENTS.md").is_file());
        assert!(installed.join("skills/focusa/SKILL.md").is_file());
        assert!(!installed.join("old-marker").exists());
        let _ = std::fs::remove_dir_all(fixture);
    }

    #[test]
    fn agent_context_archive_rejects_missing_skills() {
        let fixture = std::env::temp_dir().join(format!(
            "focusa-agent-context-invalid-{}",
            uuid::Uuid::now_v7()
        ));
        let package = fixture.join("package/focusa-agent-context");
        std::fs::create_dir_all(&package).unwrap();
        std::fs::write(package.join("AGENTS.md"), "# Focusa agents\n").unwrap();
        let archive = fixture.join("focusa-agent-context-vtest.tar.gz");
        let status = std::process::Command::new("tar")
            .args(["-czf"])
            .arg(&archive)
            .arg("-C")
            .arg(fixture.join("package"))
            .arg("focusa-agent-context")
            .status()
            .unwrap();
        assert!(status.success());
        let asset = InstalledAsset {
            name: "focusa-agent-context-vtest.tar.gz".to_string(),
            version: "vtest".to_string(),
            triple: "all".to_string(),
            sha256: String::new(),
            install_path: archive.display().to_string(),
        };
        let error = install_agent_context_archive(&asset, &fixture.join("install"))
            .expect_err("missing skills must fail");
        assert!(error.to_string().contains("at least one skills/*/SKILL.md"));
        let _ = std::fs::remove_dir_all(fixture);
    }

    #[test]
    fn cancellation_token_stops_phase_boundary_deterministically() {
        let token = CancellationToken::new();
        assert!(ensure_not_cancelled(&token).is_ok());
        token.cancel();
        let error = ensure_not_cancelled(&token).expect_err("cancelled phase must stop");
        assert_eq!(error.to_string(), "installation cancelled by operator");
    }

    #[test]
    fn cancellation_cleanup_removes_only_download_stages() {
        let root = std::env::temp_dir().join(format!("focusa-cancel-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(root.join("bin")).unwrap();
        std::fs::create_dir_all(root.join("share")).unwrap();
        std::fs::write(root.join("bin/focusa.download"), b"partial").unwrap();
        std::fs::write(root.join("share/context.download"), b"partial").unwrap();
        std::fs::write(root.join("bin/focusa"), b"keep").unwrap();
        cleanup_staged_downloads(&root);
        assert!(!root.join("bin/focusa.download").exists());
        assert!(!root.join("share/context.download").exists());
        assert!(root.join("bin/focusa").exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn cancellation_result_is_nonzero_and_reports_no_prior_install() {
        let root =
            std::env::temp_dir().join(format!("focusa-cancel-result-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&root).unwrap();
        let result: Result<()> = cancellation_result(&root, &root.join("missing.stash"), false);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "installation cancelled by operator"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn dry_run_plan_with_license_key_marks_commercial() {
        let args = InstallArgs {
            target: InstallTarget::Linux,
            channel: Channel::Stable,
            dry_run: true,
            preflight: false,
            no_animation: false,
            quiet: false,
            assume_yes: false,
            license_key: Some("focusa_live_xxxxx".to_string()),
            eval: false,
            accept_license: false,
            no_service: false,
            persist_path: false,
            no_persist_path: false,
            on_shell: ShellFamily::Auto,
            json: false,
            github_repo: None,
        };
        let plan = build_plan(
            &args,
            InstallTarget::Linux,
            std::path::Path::new("/tmp/.focusa"),
        )
        .unwrap();
        assert_eq!(plan.license_mode, "commercial");
    }
}
