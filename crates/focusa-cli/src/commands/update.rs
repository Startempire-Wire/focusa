//! Spec 128 read-only update inventory/status/check/plan.
//!
//! This command intentionally performs no mutation: no downloads, no binary
//! replacement, no daemon restart. It only inventories local Focusa surfaces
//! and reports stale parts against an operator-supplied or environment-supplied
//! latest version placeholder until the release manifest resolver is wired.

use anyhow::Context;
use clap::{Args, Subcommand};
use focusa_core::license::load_license_status;
use focusa_core::update::{ReleaseChannel, UPDATE_POLICY_SCHEMA_V1, UpdateMode, UpdatePolicy};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Subcommand, Debug)]
pub enum UpdateCmd {
    /// Read-only installed-surface inventory and stale-part summary.
    Status(UpdateStatusArgs),
    /// Read-only update check. Same inventory as status plus channel/latest context.
    Check(UpdateStatusArgs),
    /// Read-only update plan. Shows what would change, prompts, compatibility gates, and restart impact.
    Plan(UpdateStatusArgs),
    /// Show or set the local update policy. Does not apply updates.
    #[command(subcommand)]
    Policy(UpdatePolicyCmd),
}

#[derive(Subcommand, Debug)]
pub enum UpdatePolicyCmd {
    /// Show effective policy, using license-derived defaults when no file exists.
    Show,
    /// Write update policy fields without applying updates.
    Set(UpdatePolicySetArgs),
}

#[derive(Args, Debug)]
pub struct UpdatePolicySetArgs {
    /// Enable/disable update checks/policy.
    #[arg(long)]
    pub enabled: Option<bool>,
    /// Channel: stable, preview, dev, nightly.
    #[arg(long)]
    pub channel: Option<String>,
    /// Mode: notify, prompt, scheduled, automatic, manual.
    #[arg(long)]
    pub mode: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct UpdateStatusArgs {
    /// Release channel to compare against.
    #[arg(long, default_value = "dev")]
    pub channel: String,

    /// Latest eligible version/tag override. Defaults to FOCUSA_LATEST_VERSION,
    /// then FOCUSA_UPDATE_LATEST_TAG, then this CLI package version.
    #[arg(long, value_name = "VERSION_OR_TAG")]
    pub latest_version: Option<String>,

    /// Daemon health URL used for safe daemon version probing.
    #[arg(long, default_value = "http://127.0.0.1:8787/v1/health")]
    pub daemon_health_url: String,
}

#[derive(Debug, Serialize)]
struct UpdateInventoryEnvelope {
    schema: &'static str,
    status: &'static str,
    command: &'static str,
    read_only: bool,
    mutations_performed: bool,
    channel: String,
    latest: LatestVersion,
    policy: UpdatePolicySummary,
    license: LicenseSummary,
    parts: Vec<InstalledPart>,
    stale_parts: Vec<String>,
    warnings: Vec<String>,
    next_actions: Vec<String>,
}

#[derive(Debug, Serialize)]
struct UpdatePlanEnvelope {
    schema: &'static str,
    status: &'static str,
    read_only: bool,
    mutations_performed: bool,
    apply_allowed: bool,
    apply_blocked_until: Vec<String>,
    channel: String,
    latest: LatestVersion,
    policy: UpdatePolicySummary,
    license: LicenseSummary,
    compatibility: CompatibilityPlan,
    safety: UpdateSafetyPlan,
    prompt: PromptPlan,
    install_order: Vec<&'static str>,
    parts: Vec<PartPlan>,
    warnings: Vec<String>,
    next_actions: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CompatibilityPlan {
    status: &'static str,
    daemon_api_contract: &'static str,
    pi_tool_contract: &'static str,
    data_schema: &'static str,
    requires_migration: bool,
    blockers: Vec<String>,
}

#[derive(Debug, Serialize)]
struct UpdateSafetyPlan {
    lock: LockPlan,
    staging: StagingPlan,
    atomic_install: AtomicInstallPlan,
    recovery: RecoveryPlan,
    preserves: Vec<&'static str>,
    no_half_written_executable_rule: &'static str,
}

#[derive(Debug, Serialize)]
struct LockPlan {
    path: String,
    mode: &'static str,
    stale_after_seconds: u64,
    behavior: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct StagingPlan {
    root: String,
    manifest_path: String,
    download_dir: String,
    verify_before_promote: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct AtomicInstallPlan {
    strategy: &'static str,
    sequence: Vec<&'static str>,
    daemon_policy: &'static str,
}

#[derive(Debug, Serialize)]
struct RecoveryPlan {
    journal_path: String,
    interrupted_states: Vec<&'static str>,
    recovery_actions: Vec<&'static str>,
    rollback_available: bool,
}

#[derive(Debug, Serialize)]
struct PromptPlan {
    mode: String,
    update_prompt_required: bool,
    daemon_restart_prompt_required: bool,
    copy: Vec<&'static str>,
    choices: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct PartPlan {
    part: &'static str,
    current_version: Option<String>,
    target_version: String,
    action: &'static str,
    reason: String,
    restart_required: bool,
    order: u8,
}

#[derive(Debug, Serialize)]
struct LatestVersion {
    version: String,
    source: String,
    release_manifest_required: bool,
    eligibility_status: &'static str,
}

#[derive(Debug, Serialize)]
struct UpdatePolicySummary {
    path: String,
    exists: bool,
    enabled: bool,
    channel: String,
    mode: String,
    auto_apply_allowed: bool,
    auto_apply_blocked_until: Vec<String>,
    note: String,
}

#[derive(Debug, Serialize)]
struct LicenseSummary {
    level: String,
    dev_mode: bool,
    features: Vec<String>,
    source: &'static str,
    note: &'static str,
}

#[derive(Debug, Serialize)]
struct InstalledPart {
    part: &'static str,
    expected_path: String,
    resolved_path: Option<String>,
    exists: bool,
    version: Option<String>,
    version_source: &'static str,
    version_probe_safe: bool,
    sha256: Option<String>,
    stale: Option<bool>,
    stale_reason: String,
    notes: Vec<String>,
}

pub async fn run(cmd: UpdateCmd, json_mode: bool) -> anyhow::Result<()> {
    match cmd {
        UpdateCmd::Status(args) => {
            let envelope = build_inventory("status", args).await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            } else {
                print_human(&envelope);
            }
        }
        UpdateCmd::Check(args) => {
            let envelope = build_inventory("check", args).await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            } else {
                print_human(&envelope);
            }
        }
        UpdateCmd::Plan(args) => {
            let envelope = build_inventory("plan", args).await?;
            let plan = build_update_plan(envelope);
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            } else {
                print_plan_human(&plan);
            }
        }
        UpdateCmd::Policy(cmd) => run_policy(cmd, json_mode)?,
    }
    Ok(())
}

async fn build_inventory(
    command_name: &'static str,
    args: UpdateStatusArgs,
) -> anyhow::Result<UpdateInventoryEnvelope> {
    let latest = resolve_latest(args.latest_version.as_deref());
    let daemon_health = probe_daemon_health(&args.daemon_health_url).await;
    let parts = vec![
        inspect_cli(&latest.version).await?,
        inspect_daemon(&latest.version, daemon_health).await?,
        inspect_tui(&latest.version).await?,
    ];
    let stale_parts = parts
        .iter()
        .filter(|part| part.stale == Some(true))
        .map(|part| part.part.to_string())
        .collect::<Vec<_>>();
    let mut warnings = vec![
        "read-only inventory only; no update apply, download, binary replacement, or daemon restart was attempted".to_string(),
        "release manifest eligibility/signature/provenance is required before trusted auto-apply".to_string(),
    ];
    for part in &parts {
        if part.stale == Some(true) {
            warnings.push(format!("{} is stale: {}", part.part, part.stale_reason));
        }
        if part.version.is_none() {
            warnings.push(format!(
                "{} version unknown: {}",
                part.part, part.stale_reason
            ));
        }
    }
    let next_actions = if stale_parts.is_empty() {
        vec!["Implement Spec128 policy/license/dev_mode defaults before auto-apply.".to_string()]
    } else {
        vec![
            "Use this stale-part report as input to focusa update plan once Spec128 planning is implemented.".to_string(),
            "Do not manually replace binaries from this command; it is read-only by design.".to_string(),
        ]
    };
    Ok(UpdateInventoryEnvelope {
        schema: "focusa.update_inventory.v1",
        status: "completed",
        command: command_name,
        read_only: true,
        mutations_performed: false,
        channel: args.channel,
        latest,
        policy: update_policy_summary(),
        license: license_summary(),
        parts,
        stale_parts,
        warnings,
        next_actions,
    })
}

fn build_update_plan(inventory: UpdateInventoryEnvelope) -> UpdatePlanEnvelope {
    let mut blockers = vec![
        "release_manifest_signature_verification_not_wired_to_plan".to_string(),
        "update_locking_not_implemented".to_string(),
        "atomic_install_not_implemented".to_string(),
        "rollback_apply_not_implemented".to_string(),
    ];
    if inventory.latest.source == "current_cli_package_version" {
        blockers.push("latest_release_manifest_resolver_not_wired".to_string());
    }
    let mut order = 1u8;
    let mut parts = Vec::new();
    for part in inventory.parts.iter().filter(|p| p.part != "daemon") {
        parts.push(part_plan(part, &inventory.latest.version, &mut order));
    }
    if let Some(daemon) = inventory.parts.iter().find(|p| p.part == "daemon") {
        parts.push(part_plan(daemon, &inventory.latest.version, &mut order));
    }
    let daemon_restart = parts
        .iter()
        .any(|p| p.part == "daemon" && p.restart_required);
    let prompt_mode = inventory.policy.mode.clone();
    let prompt_required = !matches!(prompt_mode.as_str(), "automatic");
    UpdatePlanEnvelope {
        schema: "focusa.update_plan.v1",
        status: "planned_read_only",
        read_only: true,
        mutations_performed: false,
        apply_allowed: false,
        apply_blocked_until: blockers.clone(),
        channel: inventory.channel,
        latest: inventory.latest,
        policy: inventory.policy,
        license: inventory.license,
        compatibility: CompatibilityPlan {
            status: if blockers.is_empty() {
                "ready"
            } else {
                "blocked_until_apply_gates"
            },
            daemon_api_contract: "focusa.api.v1",
            pi_tool_contract: "focusa.pi-tools.v1",
            data_schema: "focusa.data.v1",
            requires_migration: false,
            blockers,
        },
        safety: build_safety_plan(),
        prompt: PromptPlan {
            mode: prompt_mode,
            update_prompt_required: prompt_required,
            daemon_restart_prompt_required: daemon_restart,
            copy: vec![
                "Your Focusa data, projects, license, Workpoints, evidence, and .env files will not be overwritten by a valid update plan.",
                "Daemon restart is shown separately because it may interrupt active sessions.",
                "This command is read-only; it has not downloaded, installed, or restarted anything.",
            ],
            choices: vec![
                "show_details",
                "later",
                "skip_version",
                "disable_auto_update",
                "apply_when_available",
            ],
        },
        install_order: vec!["cli", "tui", "daemon_last"],
        parts,
        warnings: inventory.warnings,
        next_actions: vec![
            "Implement Spec128 locking/staging/atomic install before update apply.".into(),
            "Implement Spec128 rollback/history before update apply.".into(),
            "Keep using focusa update status/check/plan as read-only surfaces.".into(),
        ],
    }
}

fn build_safety_plan() -> UpdateSafetyPlan {
    let base = update_state_root();
    let staging_root = base.join("staging");
    UpdateSafetyPlan {
        lock: LockPlan {
            path: base.join("update.lock").display().to_string(),
            mode: "exclusive_create_new_with_pid_and_started_at",
            stale_after_seconds: 1800,
            behavior: vec![
                "only one update may stage or apply on a host at a time",
                "stale locks require process liveness check before takeover",
                "lock release happens after journaled success or rollback decision",
            ],
        },
        staging: StagingPlan {
            root: staging_root.display().to_string(),
            manifest_path: staging_root
                .join("release-manifest.json")
                .display()
                .to_string(),
            download_dir: staging_root.join("downloads").display().to_string(),
            verify_before_promote: vec![
                "release_manifest_signature",
                "asset_sha256",
                "asset_size",
                "version_eligibility",
                "platform_triple_match",
                "executable_smoke_test",
            ],
        },
        atomic_install: AtomicInstallPlan {
            strategy: "write_temp_fsync_rename_then_smoke_test",
            sequence: vec![
                "snapshot_existing_binary_metadata",
                "write_new_binary_to_same_filesystem_temp_path",
                "fsync_temp_file_and_parent_directory",
                "preserve_permissions_owner_xattrs_capabilities_when_supported",
                "rename_temp_over_target_atomically",
                "fsync_parent_directory_after_rename",
                "run_post_promote_smoke_test",
                "rollback_from_snapshot_on_smoke_failure",
            ],
            daemon_policy: "daemon binary is promoted last; restart is a separate explicit/policy-gated step",
        },
        recovery: RecoveryPlan {
            journal_path: base.join("update-journal.json").display().to_string(),
            interrupted_states: vec![
                "lock_acquired",
                "assets_staged",
                "verified",
                "promoting_cli",
                "promoting_tui",
                "promoting_daemon",
                "smoke_testing",
                "rollback_required",
            ],
            recovery_actions: vec![
                "resume_verification_for_fully_staged_assets",
                "rollback_promoted_part_from_snapshot_when_journal_marks_incomplete",
                "discard_unverified_stage_on_checksum_or_signature_mismatch",
                "preserve_user_data_license_env_projects_workpoints_evidence",
                "print_manual_recovery_commands_without_running_destructive_actions",
            ],
            rollback_available: true,
        },
        preserves: vec![
            "license.json",
            ".env",
            "projects",
            "workpoints",
            "evidence",
            "logs",
            "permissions",
            "owner",
            "xattrs_when_supported",
            "capabilities_when_supported",
        ],
        no_half_written_executable_rule: "never write directly to an executable target path; promote only by same-filesystem atomic rename after verification",
    }
}

fn update_state_root() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("focusa")
        .join("update")
}

fn part_plan(part: &InstalledPart, target_version: &str, order: &mut u8) -> PartPlan {
    let action = match part.stale {
        Some(true) => "would_update",
        Some(false) => "no_op",
        None => "probe_required",
    };
    let restart_required = part.part == "daemon" && part.stale == Some(true);
    let plan = PartPlan {
        part: part.part,
        current_version: part.version.clone(),
        target_version: target_version.to_string(),
        action,
        reason: part.stale_reason.clone(),
        restart_required,
        order: *order,
    };
    *order = order.saturating_add(1);
    plan
}

fn print_plan_human(plan: &UpdatePlanEnvelope) {
    println!("Focusa update plan (read-only)");
    println!("channel: {} target: {}", plan.channel, plan.latest.version);
    println!("apply_allowed: false");
    println!("compatibility: {}", plan.compatibility.status);
    if !plan.apply_blocked_until.is_empty() {
        println!("blocked_until: {}", plan.apply_blocked_until.join(", "));
    }
    for part in &plan.parts {
        println!(
            "  {}. {}: {} current={} target={} restart_required={}",
            part.order,
            part.part,
            part.action,
            part.current_version.as_deref().unwrap_or("unknown"),
            part.target_version,
            part.restart_required
        );
    }
    println!("lock: {}", plan.safety.lock.path);
    println!("staging: {}", plan.safety.staging.root);
    println!("atomic_install: {}", plan.safety.atomic_install.strategy);
    println!("recovery_journal: {}", plan.safety.recovery.journal_path);
    println!("prompt_mode: {}", plan.prompt.mode);
    for line in &plan.prompt.copy {
        println!("note: {line}");
    }
}

fn resolve_latest(override_value: Option<&str>) -> LatestVersion {
    if let Some(v) = override_value.filter(|s| !s.trim().is_empty()) {
        return LatestVersion {
            version: normalize_version(v),
            source: "--latest-version".into(),
            release_manifest_required: true,
            eligibility_status: "placeholder_until_manifest_resolver",
        };
    }
    for env_key in ["FOCUSA_LATEST_VERSION", "FOCUSA_UPDATE_LATEST_TAG"] {
        if let Ok(v) = std::env::var(env_key) {
            if !v.trim().is_empty() {
                return LatestVersion {
                    version: normalize_version(&v),
                    source: env_key.into(),
                    release_manifest_required: true,
                    eligibility_status: "placeholder_until_manifest_resolver",
                };
            }
        }
    }
    LatestVersion {
        version: env!("CARGO_PKG_VERSION").into(),
        source: "current_cli_package_version".into(),
        release_manifest_required: true,
        eligibility_status: "placeholder_until_manifest_resolver",
    }
}

fn update_policy_summary() -> UpdatePolicySummary {
    let path = update_policy_path();
    let exists = path.exists();
    let policy = read_update_policy().unwrap_or_else(|_| default_policy_from_license());
    UpdatePolicySummary {
        path: path.display().to_string(),
        exists,
        enabled: policy.enabled,
        channel: policy.channel.label().to_string(),
        mode: policy.mode.label().to_string(),
        auto_apply_allowed: policy.auto_apply_allowed,
        auto_apply_blocked_until: policy.auto_apply_blocked_until,
        note: if exists {
            "policy file loaded; auto-apply still requires later locking/rollback/apply gates"
                .into()
        } else {
            "license-derived default policy; no policy file exists yet".into()
        },
    }
}

fn license_summary() -> LicenseSummary {
    match load_license_status() {
        Ok(status) => {
            let dev_mode = status.tier == "dev_mode"
                || (status.features.iter().any(|f| f == "developer_channel")
                    && status.features.iter().any(|f| f == "ota_auto_update"));
            LicenseSummary {
                level: if dev_mode {
                    "dev_mode".into()
                } else {
                    status.tier
                },
                dev_mode,
                features: status.features,
                source: "local_license_file",
                note: "policy defaults are derived from license, but update apply remains disabled until safety gates exist",
            }
        }
        Err(_) => LicenseSummary {
            level: "evaluation".into(),
            dev_mode: false,
            features: vec![],
            source: "fallback_evaluation",
            note: "license unreadable; defaulting update policy posture to evaluation notify-only",
        },
    }
}

fn update_policy_path() -> PathBuf {
    std::env::var_os("FOCUSA_UPDATE_POLICY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/local/lib/focusa/update-policy.json"))
}

fn default_policy_from_license() -> UpdatePolicy {
    match load_license_status() {
        Ok(status) => {
            let dev_override = std::env::var("FOCUSA_DEV_MODE")
                .map(|v| matches!(v.as_str(), "1" | "true" | "yes"))
                .unwrap_or(false);
            UpdatePolicy::default_for_license(status.tier, &status.features, dev_override)
        }
        Err(_) => UpdatePolicy::default_for_license("evaluation", &[], false),
    }
}

fn read_update_policy() -> anyhow::Result<UpdatePolicy> {
    let path = update_policy_path();
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read update policy {}", path.display()))?;
    let policy: UpdatePolicy = serde_json::from_str(&raw)
        .with_context(|| format!("parse update policy {}", path.display()))?;
    if policy.schema != UPDATE_POLICY_SCHEMA_V1 {
        anyhow::bail!(
            "unsupported update policy schema: expected {}, got {}",
            UPDATE_POLICY_SCHEMA_V1,
            policy.schema
        );
    }
    Ok(policy)
}

fn write_update_policy(policy: &UpdatePolicy) -> anyhow::Result<PathBuf> {
    let path = update_policy_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create update policy dir {}", parent.display()))?;
    }
    let body = serde_json::to_string_pretty(policy)?;
    std::fs::write(&path, format!("{body}\n"))
        .with_context(|| format!("write update policy {}", path.display()))?;
    Ok(path)
}

fn run_policy(cmd: UpdatePolicyCmd, json_mode: bool) -> anyhow::Result<()> {
    match cmd {
        UpdatePolicyCmd::Show => {
            let path = update_policy_path();
            let exists = path.exists();
            let policy = read_update_policy().unwrap_or_else(|_| default_policy_from_license());
            let out = serde_json::json!({
                "schema": "focusa.update_policy_status.v1",
                "status": "completed",
                "path": path,
                "exists": exists,
                "policy": policy,
                "mutations_performed": false,
                "auto_apply_allowed": false,
            });
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("Focusa update policy");
                println!("path: {}", out["path"].as_str().unwrap_or("unknown"));
                println!("exists: {exists}");
                println!(
                    "mode: {}",
                    out["policy"]["mode"].as_str().unwrap_or("unknown")
                );
                println!(
                    "channel: {}",
                    out["policy"]["channel"].as_str().unwrap_or("unknown")
                );
                println!("auto_apply_allowed: false");
            }
        }
        UpdatePolicyCmd::Set(args) => {
            let mut policy = read_update_policy().unwrap_or_else(|_| default_policy_from_license());
            if let Some(enabled) = args.enabled {
                policy.enabled = enabled;
            }
            if let Some(channel) = args.channel {
                policy.channel = channel
                    .parse::<ReleaseChannel>()
                    .map_err(anyhow::Error::msg)?;
            }
            if let Some(mode) = args.mode {
                policy.mode = mode.parse::<UpdateMode>().map_err(anyhow::Error::msg)?;
            }
            // Setting policy still cannot unlock auto-apply in this slice.
            policy.auto_apply_allowed = false;
            if policy.auto_apply_blocked_until.is_empty() {
                policy.auto_apply_blocked_until = vec![
                    "update_locking".into(),
                    "atomic_install".into(),
                    "rollback_apply".into(),
                    "health_proof".into(),
                ];
            }
            let path = write_update_policy(&policy)?;
            let out = serde_json::json!({
                "schema": "focusa.update_policy_write.v1",
                "status": "completed",
                "path": path,
                "policy": policy,
                "mutations_performed": true,
                "mutation_scope": "update_policy_file_only",
                "auto_apply_allowed": false,
                "next_action": "focusa update status --json"
            });
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!(
                    "updated policy: {}",
                    out["path"].as_str().unwrap_or("unknown")
                );
                println!("auto_apply_allowed: false");
            }
        }
    }
    Ok(())
}

async fn inspect_cli(latest: &str) -> anyhow::Result<InstalledPart> {
    let path = resolve_path("focusa", "/usr/local/bin/focusa");
    inspect_executable_part("cli", "/usr/local/bin/focusa", path, latest, true).await
}

async fn inspect_tui(latest: &str) -> anyhow::Result<InstalledPart> {
    let path = resolve_path("focusa-tui", "/usr/local/bin/focusa-tui");
    inspect_executable_part("tui", "/usr/local/bin/focusa-tui", path, latest, true).await
}

async fn inspect_daemon(latest: &str, health: Option<String>) -> anyhow::Result<InstalledPart> {
    let path = resolve_path("focusa-daemon", "/usr/local/bin/focusa-daemon");
    let sha256 = path.as_deref().and_then(|p| sha256_file(Path::new(p)).ok());
    let exists = path.is_some();
    let version = health.as_deref().map(normalize_version);
    let stale = version.as_ref().map(|v| v != latest);
    let stale_reason = match (&version, stale) {
        (Some(v), Some(true)) => format!("running daemon health version {v} differs from latest {latest}"),
        (Some(v), Some(false)) => format!("running daemon health version {v} matches latest {latest}"),
        _ => "daemon version unknown; safe probe uses /v1/health because focusa-daemon --version starts the server".into(),
    };
    Ok(InstalledPart {
        part: "daemon",
        expected_path: "/usr/local/bin/focusa-daemon".into(),
        resolved_path: path,
        exists,
        version,
        version_source: "daemon_health_endpoint",
        version_probe_safe: true,
        sha256,
        stale,
        stale_reason,
        notes: vec!["binary --version intentionally not invoked; current daemon binary treats --version as startup input".into()],
    })
}

async fn inspect_executable_part(
    part: &'static str,
    expected_path: &str,
    path: Option<String>,
    latest: &str,
    probe_version: bool,
) -> anyhow::Result<InstalledPart> {
    let sha256 = path.as_deref().and_then(|p| sha256_file(Path::new(p)).ok());
    let version = if probe_version {
        match path.as_deref() {
            Some(p) => probe_version_command(p)
                .await
                .map(|s| normalize_version(&s)),
            None => None,
        }
    } else {
        None
    };
    let stale = version.as_ref().map(|v| v != latest);
    let stale_reason = match (&version, stale, &path) {
        (_, _, None) => format!("{part} binary not found"),
        (Some(v), Some(true), _) => {
            format!("installed {part} version {v} differs from latest {latest}")
        }
        (Some(v), Some(false), _) => {
            format!("installed {part} version {v} matches latest {latest}")
        }
        _ => format!("{part} version probe unavailable"),
    };
    Ok(InstalledPart {
        part,
        expected_path: expected_path.into(),
        resolved_path: path,
        exists: sha256.is_some(),
        version,
        version_source: "binary_--version",
        version_probe_safe: true,
        sha256,
        stale,
        stale_reason,
        notes: Vec::new(),
    })
}

fn resolve_path(command: &str, canonical: &str) -> Option<String> {
    if Path::new(canonical).exists() {
        return Some(canonical.into());
    }
    which::which(command)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

async fn probe_version_command(path: &str) -> Option<String> {
    let output = timeout(
        Duration::from_secs(3),
        Command::new(path).arg("--version").output(),
    )
    .await
    .ok()?
    .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        None
    } else {
        Some(stdout)
    }
}

async fn probe_daemon_health(url: &str) -> Option<String> {
    if let Some(version) = probe_daemon_health_reqwest(url).await {
        return Some(version);
    }
    if let Some(version) = probe_daemon_health_curl(url) {
        return Some(version);
    }
    probe_local_http_health(url)
}

async fn probe_daemon_health_reqwest(url: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .no_proxy()
        .build()
        .ok()?;
    let body: serde_json::Value = client.get(url).send().await.ok()?.json().await.ok()?;
    body.get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn probe_daemon_health_curl(url: &str) -> Option<String> {
    let output = std::process::Command::new("curl")
        .args(["-fsS", "--max-time", "3", url])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    json.get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn probe_local_http_health(url: &str) -> Option<String> {
    let rest = url.strip_prefix("http://")?;
    let (host_port, path) = rest.split_once('/')?;
    if !host_port.starts_with("127.0.0.1:") && !host_port.starts_with("localhost:") {
        return None;
    }
    let port = host_port.split_once(':')?.1.parse::<u16>().ok()?;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(1)).ok()?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
    let request = format!("GET /{path} HTTP/1.1\r\nHost: {host_port}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).ok()?;
    let mut response = String::new();
    stream.read_to_string(&mut response).ok()?;
    let (_, body) = response.split_once("\r\n\r\n")?;
    let json: serde_json::Value = serde_json::from_str(body).ok()?;
    json.get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn sha256_file(path: &Path) -> anyhow::Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}

fn normalize_version(raw: &str) -> String {
    let trimmed = raw.trim();
    let last = trimmed.split_whitespace().last().unwrap_or(trimmed);
    last.trim_start_matches('v').to_string()
}

fn print_human(envelope: &UpdateInventoryEnvelope) {
    println!("Focusa update {} (read-only)", envelope.command);
    println!("channel: {}", envelope.channel);
    println!(
        "latest: {} ({})",
        envelope.latest.version, envelope.latest.source
    );
    println!(
        "policy: enabled={} mode={} path={} exists={}",
        envelope.policy.enabled, envelope.policy.mode, envelope.policy.path, envelope.policy.exists
    );
    println!("parts:");
    for part in &envelope.parts {
        println!(
            "  - {} path={} version={} stale={} sha256={}",
            part.part,
            part.resolved_path.as_deref().unwrap_or("missing"),
            part.version.as_deref().unwrap_or("unknown"),
            part.stale
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unknown".into()),
            part.sha256
                .as_deref()
                .map(|s| &s[..12.min(s.len())])
                .unwrap_or("unknown")
        );
        println!("    {}", part.stale_reason);
    }
    if envelope.stale_parts.is_empty() {
        println!("stale_parts: none");
    } else {
        println!("stale_parts: {}", envelope.stale_parts.join(", "));
    }
    for warning in &envelope.warnings {
        println!("warning: {warning}");
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_version;

    #[test]
    fn normalizes_common_version_outputs() {
        assert_eq!(normalize_version("focusa 0.9.74-dev"), "0.9.74-dev");
        assert_eq!(normalize_version("v0.9.80-dev"), "0.9.80-dev");
        assert_eq!(normalize_version("0.9.80-dev"), "0.9.80-dev");
    }
}
