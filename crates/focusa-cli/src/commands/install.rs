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

use anyhow::{Context, Result, anyhow};
use clap::Args;
use serde::Serialize;

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

    /// License key (commercial install). Eval mode is selected by absence.
    #[arg(long, value_name = "KEY")]
    pub license_key: Option<String>,

    /// Eval mode: skip license validation, write `eval: true` to license.json.
    #[arg(long)]
    pub eval: bool,

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
}

#[derive(Debug, Serialize)]
pub struct AssetPlan {
    pub name: String,
    pub version: String,
    pub triple: String,
    pub install_path: String,
}

pub async fn run(args: InstallArgs) -> Result<()> {
    let target = resolve_target(args.target)?;
    let channel = args.channel;
    let dry_run = args.dry_run;
    let install_root = std::env::var_os("HOME")
        .map(|h| std::path::PathBuf::from(h).join(".focusa"))
        .unwrap_or_else(|| std::path::PathBuf::from("/opt/focusa"));

    if dry_run {
        let plan = build_plan(&args, target, &install_root)?;
        if args.json {
            println!("{}", serde_json::to_string_pretty(&plan)?);
        } else {
            print_plan_human(&plan);
        }
        return Ok(());
    }

    // Wire implementation phases (delegate to spec 112 sub-beads):
    //   1. license::registry_validate (focusa-112-license-revalidate)
    //   2. asset download + sha256 (focusa-112-asset-download, focusa-112-checksum)
    //   3. symlink placement (focusa-112-symlinks)
    //   4. service delegation (focusa-112-service-delegate)
    //   5. PATH automation (focusa-112-path-automation)
    //   6. first install walkthrough (focusa-112-first-walkthrough)
    //   7. atomicity (focusa-112-atomicity)
    // Until each sub-bead lands, the wiring stubs out to a structured
    // "not yet wired" error so the CLI surface is stable.
    Err(anyhow!(
        "focusa install is the canonical orchestrator per Spec 112 §15A. \
         Sub-beads are tracked under focusa-112-* in the bead graph; the \
         orchestrator currently wires only the CLI surface (--target/--dry-run/--channel) \
         and refuses to actually install until each capability lands. \
         Run with --dry-run to preview the install plan."
    ))
}

fn resolve_target(target: InstallTarget) -> Result<InstallTarget> {
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

fn build_plan(args: &InstallArgs, target: InstallTarget, root: &std::path::Path) -> Result<InstallPlan> {
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
        ],
        symlink_planned: format!("{}/.local/bin/focusa", std::env::var("HOME").unwrap_or_default()),
        service_manager_planned: match target {
            InstallTarget::Linux => "systemd --user".to_string(),
            InstallTarget::Darwin => "launchd user agent".to_string(),
            InstallTarget::WindowsX64 | InstallTarget::WindowsArm64 => "sc.exe (Phase 2.0)".to_string(),
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
        InstallTarget::Darwin => "aarch64-apple-darwin".to_string(),
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
        assert_eq!(triple_for(InstallTarget::Darwin), "aarch64-apple-darwin");
        assert_eq!(triple_for(InstallTarget::WindowsX64), "x86_64-pc-windows-msvc");
        assert_eq!(triple_for(InstallTarget::WindowsArm64), "aarch64-pc-windows-msvc");
    }

    #[test]
    fn dry_run_plan_lists_three_assets() {
        let args = InstallArgs {
            target: InstallTarget::Linux,
            channel: Channel::Stable,
            dry_run: true,
            license_key: None,
            eval: false,
            persist_path: false,
            no_persist_path: false,
            on_shell: ShellFamily::Auto,
            json: false,
            github_repo: None,
        };
        let plan = build_plan(&args, InstallTarget::Linux, std::path::Path::new("/tmp/.focusa")).unwrap();
        assert_eq!(plan.assets_planned.len(), 3);
        assert!(plan.assets_planned.iter().any(|a| a.name == "focusa"));
        assert!(plan.assets_planned.iter().any(|a| a.name == "focusa-daemon"));
        assert!(plan.assets_planned.iter().any(|a| a.name == "focusa-tui"));
        assert_eq!(plan.license_mode, "missing");
    }

    #[test]
    fn dry_run_plan_with_eval_flag_marks_eval_license() {
        let args = InstallArgs {
            target: InstallTarget::Darwin,
            channel: Channel::Stable,
            dry_run: true,
            license_key: None,
            eval: true,
            persist_path: false,
            no_persist_path: false,
            on_shell: ShellFamily::Auto,
            json: false,
            github_repo: None,
        };
        let plan = build_plan(&args, InstallTarget::Darwin, std::path::Path::new("/tmp/.focusa")).unwrap();
        assert_eq!(plan.license_mode, "eval");
        assert!(plan.service_manager_planned.contains("launchd"));
    }

    #[test]
    fn dry_run_plan_with_license_key_marks_commercial() {
        let args = InstallArgs {
            target: InstallTarget::Linux,
            channel: Channel::Stable,
            dry_run: true,
            license_key: Some("focusa_live_xxxxx".to_string()),
            eval: false,
            persist_path: false,
            no_persist_path: false,
            on_shell: ShellFamily::Auto,
            json: false,
            github_repo: None,
        };
        let plan = build_plan(&args, InstallTarget::Linux, std::path::Path::new("/tmp/.focusa")).unwrap();
        assert_eq!(plan.license_mode, "commercial");
    }
}
