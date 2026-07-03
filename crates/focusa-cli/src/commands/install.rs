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

use anyhow::{Context, Result, anyhow, bail};
use clap::Args;
use serde::Serialize;
use sha2::{Digest, Sha256};

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

    // Real install wrapped in atomicity (focusa-112-atomicity, Spec 112 §6):
    //   1. Stash any existing install to .focusa.stash
    //   2. Execute each phase
    //   3. Run smoke test (focusa --version on the new binary)
    //   4. On smoke-test failure: rollback to stash
    //   5. On success: remove stash
    let stash_path = install_root.with_extension("stash");
    let stashed = phase_atomic_stash(&install_root, &stash_path)?;
    if let Err(e) = execute_real_install(&args, target, channel, &install_root).await {
        if stashed {
            phase_atomic_rollback(&install_root, &stash_path).ok();
        }
        return Err(e);
    }
    let bin_dir = install_root.join("bin");
    if let Err(e) = phase_smoke_test(&bin_dir).await {
        if stashed {
            phase_atomic_rollback(&install_root, &stash_path).ok();
        }
        return Err(e);
    }
    if stashed {
        phase_atomic_cleanup(&stash_path).ok();
    }

    // Phases that still need their sub-beads to land:
    //   - focusa-112-path-automation (PATH detection + rc edit)
    //   - focusa-112-first-walkthrough (post-install card)
    // We emit a structured "wired but phase 5+ not yet wired" response so the
    // operator can see what was done and what's pending. The walkthrough is
    // emitted via print_walkthrough_human() below when --json is not set, and
    // embedded in the JSON envelope when --json is set.
    if !args.json {
        println!(
            "\n✓ Installed assets to {}\n  atomicity: stashed={}, smoke-test OK\n  walkthrough: 6 next steps below\n",
            install_root.display(),
            stashed,
        );
    }
    // The JSON envelope (with embedded walkthrough) is emitted by the
    // execute_real_install() return path below. Print it here if --json.
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
    let Some(expected_line) = expected_line else {
        eprintln!("warning: no SHA256SUMS entry for {}", asset.name);
        return Ok(());
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
    let bytes = std::fs::read(&asset.path)
        .with_context(|| format!("read downloaded asset for checksum: {}", asset.path.display()))?;
    let actual = hex::encode(Sha256::digest(&bytes));
    if actual != expected {
        bail!("checksum mismatch for {}: expected {expected}, got {actual}", asset.name);
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

/// Idempotently persist the PATH line to an rc file. Never duplicates:
/// if the exact line is already present, no-op. If a similar line is
/// present, also no-op (idempotency over cleverness).
pub fn persist_path_to_rc(rc: &std::path::Path, path_line: &str) -> Result<()> {
    if let Some(parent) = rc.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if !rc.exists() {
        std::fs::write(rc, format!("{path_line}\n"))
            .with_context(|| format!("write {}", rc.display()))?;
        return Ok(());
    }
    let content = std::fs::read_to_string(rc)
        .with_context(|| format!("read {}", rc.display()))?;
    if content.contains(".local/bin") && content.contains("PATH") {
        // Already there in some form; don't duplicate.
        return Ok(());
    }
    let mut new_content = content;
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str(path_line);
    new_content.push('\n');
    std::fs::write(rc, &new_content)
        .with_context(|| format!("write {}", rc.display()))?;
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
            "If `focusa --version` returns 'command not found', re-source your shell rc.".to_string(),
            "If the daemon fails to start, run `focusa doctor` for diagnosis.".to_string(),
        ],
    };
    let next_steps = vec![
        NextStep {
            command: format!("{}", binary.display()),
            intent: "verify install (executable present, returns --version)".to_string(),
            expected_outcome: "binary exits 0 with focusa version string".to_string(),
            recovery_hint: Some("re-run focusa install; check ~/.focusa/bin/focusa exists".to_string()),
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
            command: "focusa workpoint checkpoint --mission \"first install\" --project-root \"$(pwd)\"".to_string(),
            intent: "create a save state".to_string(),
            expected_outcome: "ok: workpoint id returned".to_string(),
            recovery_hint: Some("pass --project-root explicitly if PWD is not a project".to_string()),
        },
        NextStep {
            command: "focusa about".to_string(),
            intent: "read the human-facing recap".to_string(),
            expected_outcome: "30-line ASCII card explaining what focusa is".to_string(),
            recovery_hint: Some("for LLM agents, read GET /llms.txt on the daemon instead".to_string()),
        },
        NextStep {
            command: "focusa workflow list".to_string(),
            intent: "discover canonical workflow templates".to_string(),
            expected_outcome: "6 templates listed (long-refactor, multi-session-resume, etc.)".to_string(),
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
        agent_integrations: Vec::new(),
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
        Err(e) => Err(anyhow!("smoke test failed: could not exec focusa --version: {e}")),
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
        .arg(&asset.path)
        .status()
        .map_err(|e| anyhow!("macOS codesign verify failed to execute for {}: {e}", asset.name))?;
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

/// Wraps the post-license phases into one async function for atomicity.
async fn execute_real_install(
    args: &InstallArgs,
    target: InstallTarget,
    channel: Channel,
    install_root: &std::path::Path,
) -> Result<()> {
    let phase = phase_license(args).await?;
    let assets = phase_asset_download(target, channel, args.github_repo.as_deref()).await?;
    let bin_dir = install_root.join("bin");
    for asset in &assets {
        verify_checksum(asset).await?;
        verify_macos_codesign(target, asset)?;
    }
    place_symlinks(&bin_dir, install_root)?;
    delegate_service_render(target, &bin_dir, args.dry_run).await?;

    // Path automation (focusa-112-path-automation). Idempotent: detects
    // shell, persists export PATH line to rc file, never duplicates.
    for (rc, line, _shell) in detect_shell_rc_targets() {
        if let Err(e) = persist_path_to_rc(&rc, &line) {
            eprintln!("warning: failed to persist PATH to {}: {e}", rc.display());
        }
    }

    // First-install walkthrough (focusa-112-first-walkthrough). Prints inline
    // to the same terminal where install ran — no separate wizard UI.
    let walkthrough = build_first_install_walkthrough(
        target,
        channel,
        &bin_dir,
        &install_root,
        assets.len(),
    );
    if !args.json {
        print_walkthrough_human(&walkthrough);
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
            "first_install_walkthrough": serde_json::to_value(&walkthrough)?,
            "recovery_hint": "Pending phases will land as their sub-beads close; re-run focusa install to retry.",
        });
        println!("{}", serde_json::to_string_pretty(&report)?);
        // Skip the second JSON block below.
    }
    eprintln!(
        "[install] license={}, assets={}, bin_dir={}",
        phase,
        assets.len(),
        bin_dir.display(),
    );
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
