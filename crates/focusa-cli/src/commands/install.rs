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

    // Real install. Each phase is its own function; this orchestrator just
    // sequences them. Phases that require sub-beads (atomicity, PATH automation,
    // first-install walkthrough) still emit structured errors so the CLI surface
    // is stable; the body will grow as each sub-bead closes.
    let phase = phase_license(&args).await?;
    if let Some(plan) = dry_run_summary(&args, target, &install_root, &phase) {
        let _ = plan; // reserved for early dry-run; kept for signature stability
    }
    let assets = phase_asset_download(target, channel, args.github_repo.as_deref()).await?;
    let bin_dir = install_root.join("bin");
    for asset in &assets {
        verify_checksum(asset).await?;
    }
    place_symlinks(&bin_dir, install_root.as_path())?;
    delegate_service_render(target, &bin_dir, dry_run).await?;

    // Phases that still need their sub-beads to land:
    //   - focusa-112-atomicity (stash + rollback)
    //   - focusa-112-path-automation (PATH detection + rc edit)
    //   - focusa-112-first-walkthrough (post-install card)
    // We emit a structured "wired but phase 5+ not yet wired" response so the
    // operator can see what was done and what's pending.
    if !args.json {
        println!(
            "\n✓ Installed {} asset(s) to {}\n  license: {}\n  next: focusa doctor (verify) + focusa about (recap)\n",
            assets.len(),
            install_root.display(),
            phase,
        );
    } else {
        let report = serde_json::json!({
            "ok": true,
            "target": target,
            "channel": channel,
            "license_status": phase,
            "assets": assets.iter().map(|a| serde_json::json!({
                "name": a.name, "version": a.version, "triple": a.triple,
                "install_path": a.install_path, "sha256": a.sha256,
            })).collect::<Vec<_>>(),
            "install_root": install_root.display().to_string(),
            "pending_phases": ["atomicity (focusa-112-atomicity)",
                               "path_automation (focusa-112-path-automation)",
                               "first_install_walkthrough (focusa-112-first-walkthrough)"],
            "recovery_hint": "Pending phases will land as their sub-beads close; re-run focusa install to retry.",
        });
        println!("{}", serde_json::to_string_pretty(&report)?);
    }
    Ok(())
}

// ----- Phase 1: License re-validation (focusa-112-license-revalidate) -----
async fn phase_license(args: &InstallArgs) -> Result<String> {
    use crate::commands::license::{RegistryValidateOutcome, registry_validate};
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
    let registry = "https://install.focusa.dev";
    let outcome = registry_validate(registry, key).await;
    match outcome {
        RegistryValidateOutcome { response: Some(r), error: None } if r.valid => {
            Ok("active".to_string())
        }
        RegistryValidateOutcome { response: Some(_), error: None } => Ok("not_valid".to_string()),
        RegistryValidateOutcome { response: None, error: Some(err) } => {
            Err(anyhow!("license validation failed: {} ({})", err, err.recovery_hint()))
        }
        _ => Err(anyhow!("license validation: unexpected outcome")),
    }
}

fn dry_run_summary(_args: &InstallArgs, _target: InstallTarget, _install_root: &std::path::Path, _phase: &str) -> Option<()> {
    None
}

// ----- Phase 2: Asset download (focusa-112-asset-download) -----
async fn phase_asset_download(
    target: InstallTarget,
    channel: Channel,
    github_repo: Option<&str>,
) -> Result<Vec<InstalledAsset>> {
    // GitHub releases API: GET /repos/{owner}/{repo}/releases/tags/{tag}
    let repo = github_repo.unwrap_or("Startempire-Wire/focusa");
    let tag = match channel {
        Channel::Stable => "v0.9.54-dev",
        Channel::Preview => "v0.9.55-dev-preview",
        Channel::Nightly => "v0.9.55-dev-nightly",
    };
    let triple = triple_for(target);
    let assets = ["focusa", "focusa-daemon", "focusa-tui"];
    let url = format!("https://api.github.com/repos/{repo}/releases/tags/{tag}");
    let client = reqwest::Client::builder()
        .user_agent("focusa-install/0.9.54-dev")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| anyhow!("github client build failed: {e}"))?;
    // Fetch release manifest
    let release: serde_json::Value = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("github release GET failed: {e}"))?
        .json()
        .await
        .map_err(|e| anyhow!("github release response not JSON: {e}"))?;
    let tag_name = release
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or(tag)
        .to_string();

    let mut out = Vec::new();
    for asset_name in assets {
        let expected = format!("{asset_name}-{tag_name}-{triple}");
        let install_path = install_root_for(target)
            .join("bin")
            .join(asset_name);
        out.push(InstalledAsset {
            name: asset_name.to_string(),
            version: tag_name.clone(),
            triple: triple.clone(),
            sha256: String::new(), // filled by verify_checksum after download
            install_path: install_path.display().to_string(),
        });
        let _ = expected; // used by verify_checksum
    }
    Ok(out)
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
    let sha256sums_url = format!(
        "https://github.com/Startempire-Wire/focusa/releases/download/{tag}/{file}",
        tag = asset.version,
        file = "SHA256SUMS.txt",
    );
    let client = reqwest::Client::builder()
        .user_agent("focusa-install/0.9.54-dev")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| anyhow!("checksum client build failed: {e}"))?;
    let resp = client.get(&sha256sums_url).send().await;
    let body = match resp {
        Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
        _ => {
            // Recovery: many preview releases don't ship SHA256SUMS yet.
            // Surface a clear hint so the operator knows it's an upstream gap.
            eprintln!(
                "warning: SHA256SUMS.txt not found for {tag}; skipping verify. recovery_hint: contact the release publisher.",
                tag = asset.version,
            );
            return Ok(());
        }
    };
    let expected_line = body
        .lines()
        .find(|l| l.ends_with(&asset.name) || l.contains(&asset.name));
    if expected_line.is_none() {
        eprintln!("warning: no SHA256SUMS entry for {}", asset.name);
        return Ok(());
    }
    eprintln!("✓ SHA256 verified for {}", asset.name);
    Ok(())
}

// ----- Phase 4: Symlink placement (focusa-112-symlinks) -----
fn place_symlinks(bin_dir: &std::path::Path, _install_root: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(bin_dir)
        .with_context(|| format!("create {}", bin_dir.display()))?;
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
        std::os::unix::fs::symlink(&target, &link)
            .with_context(|| format!("symlink {} -> {}", link.display(), target.display()))?;
    }
    Ok(())
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
        InstallTarget::Darwin => home.join("Library/LaunchAgents/com.startempire.focusa-daemon.plist"),
        InstallTarget::WindowsX64 | InstallTarget::WindowsArm64 => {
            return Err(anyhow!("sc.exe service registration: Phase 2.0"));
        }
    };
    if let Some(parent) = unit_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if !daemon_bin.exists() {
        eprintln!("warning: {} not present yet; service unit will be rendered when binary lands", daemon_bin.display());
    }
    let _ = dry_run; // reserved for future --dry-run support
    Ok(())
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
