//! Spec 128 read-only update inventory/status/check/plan/apply guard.
//!
//! This command intentionally performs no mutation: no downloads, no binary
//! replacement, no daemon restart. It only inventories local Focusa surfaces
//! and reports stale parts against an operator-supplied or environment-supplied
//! latest version placeholder until the release manifest resolver is wired.

use anyhow::Context;
use clap::{Args, Subcommand};
use focusa_core::license::load_license_status;
use focusa_core::update::{ReleaseChannel, UPDATE_POLICY_SCHEMA_V1, UpdateMode, UpdatePolicy};
use serde::{Deserialize, Serialize};
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
    /// Guarded update apply surface. Defaults to dry-run/blocked; no mutation until all gates are wired.
    Apply(UpdateApplyArgs),
    /// Read-only update history/observability view.
    History(UpdateHistoryArgs),
    /// Read-only rollback plan. Does not restore binaries unless future gates are wired.
    Rollback(UpdateRollbackArgs),
    /// Read-only admin control preview: pin/skip/pause/resume/force-check/trusted-dev-force-latest.
    Admin(UpdateAdminArgs),
    /// Read-only scheduler/background updater status and plan.
    Scheduler(UpdateSchedulerArgs),
    /// Read-only stale/update notification payload for CLI/API/Pi/TUI/menubar.
    Notifications(UpdateStatusArgs),
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
pub struct UpdateSchedulerArgs {
    /// Show scheduler plan for this channel.
    #[arg(long, default_value = "dev")]
    pub channel: String,

    /// Install and enable the systemd verified-update timer (Linux/root only).
    #[arg(long, conflicts_with = "uninstall")]
    pub install: bool,

    /// Disable and remove the systemd verified-update timer (Linux/root only).
    #[arg(long, conflicts_with = "install")]
    pub uninstall: bool,
}

#[derive(Args, Debug)]
pub struct UpdateHistoryArgs {
    /// Maximum history event lines to show.
    #[arg(long, default_value_t = 20)]
    pub limit: usize,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RollbackPart {
    Cli,
    Tui,
    Daemon,
    All,
}

#[derive(Args, Debug)]
pub struct UpdateRollbackArgs {
    /// Part to rollback.
    #[arg(long, value_enum, default_value_t = RollbackPart::All)]
    pub part: RollbackPart,

    /// Dry-run rollback plan. Default posture; performs no mutation.
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub dry_run: bool,

    /// Explicit operator consent for future rollback. Still blocked in this scaffold.
    #[arg(long)]
    pub yes: bool,
}

#[derive(Args, Debug)]
pub struct UpdateAdminArgs {
    #[arg(long, value_name = "VERSION")]
    pub pin_version: Option<String>,
    #[arg(long)]
    pub unpin: bool,
    #[arg(long, value_name = "VERSION")]
    pub skip_version: Option<String>,
    #[arg(long)]
    pub pause: bool,
    #[arg(long)]
    pub resume: bool,
    #[arg(long)]
    pub force_check: bool,
    #[arg(long)]
    pub trusted_dev_force_latest: bool,
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub dry_run: bool,
    #[arg(long)]
    pub yes: bool,
}

#[derive(Args, Debug)]
pub struct UpdateApplyArgs {
    #[command(flatten)]
    pub status: UpdateStatusArgs,

    /// Dry-run apply. Default posture; performs no mutation.
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub dry_run: bool,

    /// Explicit operator consent for future apply. Still blocked until implementation gates pass.
    #[arg(long)]
    pub yes: bool,

    /// Explicitly request mutation when future apply is implemented. Still blocked in this slice.
    #[arg(long)]
    pub allow_apply: bool,
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
    target_path: Option<String>,
    expected_sha256: Option<String>,
    download_url: Option<String>,
    action: &'static str,
    reason: String,
    restart_required: bool,
    order: u8,
}

#[derive(Debug, Serialize)]
struct UpdateSchedulerEnvelope {
    schema: &'static str,
    status: &'static str,
    read_only: bool,
    mutations_performed: bool,
    scheduler_installed: bool,
    background_worker_started: bool,
    channel: String,
    policy: UpdatePolicySummary,
    startup_check: SchedulerStartupCheck,
    interval: SchedulerInterval,
    offline: SchedulerOfflineRules,
    maintenance: SchedulerMaintenanceWindow,
    automatic_apply: SchedulerAutomaticApply,
    notifications: NotificationRoutes,
    next_actions: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct SchedulerStartupCheck {
    enabled: bool,
    delay_seconds: u64,
    reason: &'static str,
}

#[derive(Debug, Serialize)]
struct SchedulerInterval {
    base_seconds: u64,
    jitter_percent: u8,
    backoff: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct SchedulerOfflineRules {
    skip_when_offline: bool,
    retry_backoff: Vec<&'static str>,
    max_silent_failures_before_notice: u8,
}

#[derive(Debug, Serialize)]
struct SchedulerMaintenanceWindow {
    respected: bool,
    default_window: &'static str,
    user_override_path: String,
}

#[derive(Debug, Serialize)]
struct SchedulerAutomaticApply {
    allowed: bool,
    reason: &'static str,
    requires: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct NotificationRoutes {
    cli: bool,
    api: bool,
    pi_doctor: bool,
    tui: &'static str,
    menubar: &'static str,
}

#[derive(Debug, Serialize)]
struct UpdateNotificationsEnvelope {
    schema: &'static str,
    status: &'static str,
    read_only: bool,
    mutations_performed: bool,
    stale_parts: Vec<String>,
    severity: &'static str,
    surfaces: NotificationRoutes,
    messages: Vec<NotificationMessage>,
    suppress_if: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct NotificationMessage {
    surface: &'static str,
    title: &'static str,
    body: String,
    action: &'static str,
}

#[derive(Debug, Serialize)]
struct UpdateHistoryEnvelope {
    schema: &'static str,
    status: &'static str,
    read_only: bool,
    mutations_performed: bool,
    history_path: String,
    journal_path: String,
    retention: RetentionPolicy,
    observability: UpdateObservability,
    events: Vec<String>,
    next_tools: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct RetentionPolicy {
    keep_last_successful_bundles: u8,
    keep_days: u16,
    prune_requires_admin_confirmation: bool,
}

#[derive(Debug, Serialize)]
struct UpdateObservability {
    counters: Vec<&'static str>,
    events: Vec<&'static str>,
    log_paths: Vec<String>,
}

#[derive(Debug, Serialize)]
struct UpdateRollbackEnvelope {
    schema: &'static str,
    status: &'static str,
    read_only: bool,
    mutations_performed: bool,
    rollback_executed: bool,
    part: RollbackPart,
    dry_run: bool,
    consent_yes: bool,
    blocked_reason: Vec<&'static str>,
    restore_order: Vec<&'static str>,
    proof_required: Vec<&'static str>,
    data_safety: DataSafetyPlan,
    recovery_hint: &'static str,
}

#[derive(Debug, Serialize)]
struct UpdateAdminEnvelope {
    schema: &'static str,
    status: &'static str,
    read_only: bool,
    mutations_performed: bool,
    dry_run: bool,
    consent_yes: bool,
    requested_controls: Vec<String>,
    policy_patch_preview: serde_json::Value,
    force_check_preview: bool,
    trusted_dev_force_latest_allowed: bool,
    blocked_reason: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct UpdateApplyEnvelope {
    schema: &'static str,
    status: &'static str,
    read_only: bool,
    mutations_performed: bool,
    apply_requested: bool,
    apply_executed: bool,
    dry_run: bool,
    consent: ApplyConsent,
    plan: UpdatePlanEnvelope,
    execution_order: Vec<&'static str>,
    daemon_restart: DaemonRestartPlan,
    data_safety: DataSafetyPlan,
    proof_required: Vec<&'static str>,
    blocked_reason: Vec<String>,
    recovery_hint: String,
}

#[derive(Debug, Serialize)]
struct ApplyConsent {
    yes: bool,
    allow_apply: bool,
    effective: bool,
    note: &'static str,
}

#[derive(Debug, Serialize)]
struct DaemonRestartPlan {
    allowed: bool,
    required: bool,
    when: &'static str,
    health_proof: &'static str,
}

#[derive(Debug, Serialize)]
struct DataSafetyPlan {
    overwrite_data: bool,
    overwrite_env: bool,
    overwrite_license: bool,
    preserve: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct LatestVersion {
    version: String,
    tag: String,
    source: String,
    github_repo: String,
    target_triple: String,
    release_manifest_required: bool,
    eligibility_status: &'static str,
    trust: ReleaseTrustSummary,
    assets: Vec<ReleaseAssetRef>,
}

#[derive(Debug, Serialize, Clone)]
struct ReleaseTrustSummary {
    release_resolved: bool,
    complete_asset_set: bool,
    sha256sums_present: bool,
    checksums_resolved: bool,
    signature_verified: bool,
    ci_proof_required: bool,
    signature_required: bool,
    blockers: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
struct ReleaseAssetRef {
    part: &'static str,
    name: String,
    download_url: String,
    sha256: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
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
        UpdateCmd::Apply(args) => {
            let dry_run = args.dry_run;
            let yes = args.yes;
            let allow_apply = args.allow_apply;
            let envelope = build_inventory("apply", args.status).await?;
            let plan = build_update_plan(envelope);
            let mut apply = build_apply_envelope(plan, dry_run, yes, allow_apply);
            if apply.consent.effective && apply.plan.apply_allowed {
                match execute_verified_apply(&apply.plan).await {
                    Ok(promoted) => {
                        let changed = !promoted.is_empty();
                        apply.status = if changed {
                            "completed"
                        } else {
                            "already_current"
                        };
                        apply.read_only = !changed;
                        apply.mutations_performed = changed;
                        apply.apply_executed = changed;
                        apply.blocked_reason.clear();
                        apply.recovery_hint = if changed {
                            format!(
                                "Promoted: {}. Use focusa update status --json to verify all surfaces.",
                                promoted.join(", ")
                            )
                        } else {
                            "No stale verified assets; all installed update-managed parts are current.".into()
                        };
                    }
                    Err(error) => {
                        apply.status = "failed_rolled_back";
                        apply.blocked_reason.push(format!("apply_failed:{error}"));
                        apply.recovery_hint = "Promotion failed; any previously promoted parts were restored from the update backup journal.".into();
                    }
                }
            }
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&apply)?);
            } else {
                print_apply_human(&apply);
            }
        }
        UpdateCmd::History(args) => {
            let history = build_history_envelope(args.limit);
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&history)?);
            } else {
                print_history_human(&history);
            }
        }
        UpdateCmd::Rollback(args) => {
            let rollback = build_rollback_envelope(args);
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&rollback)?);
            } else {
                print_rollback_human(&rollback);
            }
        }
        UpdateCmd::Admin(args) => {
            let admin = build_admin_envelope(args);
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&admin)?);
            } else {
                print_admin_human(&admin);
            }
        }
        UpdateCmd::Scheduler(args) => {
            if args.install {
                configure_systemd_scheduler(&args.channel, true)?;
            } else if args.uninstall {
                configure_systemd_scheduler(&args.channel, false)?;
            }
            let scheduler = build_scheduler_envelope(args.channel);
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&scheduler)?);
            } else {
                print_scheduler_human(&scheduler);
            }
        }
        UpdateCmd::Notifications(args) => {
            let inventory = build_inventory("notifications", args).await?;
            let notifications = build_notifications_envelope(inventory);
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&notifications)?);
            } else {
                print_notifications_human(&notifications);
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
    let latest = resolve_latest(&args.channel, args.latest_version.as_deref()).await;
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
    let mut blockers = inventory.latest.trust.blockers.clone();
    if !inventory.latest.trust.release_resolved {
        blockers.push("latest_release_manifest_resolver_not_wired".to_string());
    }
    if !inventory.latest.trust.checksums_resolved {
        blockers.push("release_asset_checksums_not_resolved".to_string());
    }
    blockers.sort();
    blockers.dedup();
    let mut order = 1u8;
    let mut parts = Vec::new();
    for part in inventory.parts.iter().filter(|p| p.part != "daemon") {
        parts.push(part_plan(part, &inventory.latest, &mut order));
    }
    if let Some(daemon) = inventory.parts.iter().find(|p| p.part == "daemon") {
        parts.push(part_plan(daemon, &inventory.latest, &mut order));
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
        apply_allowed: blockers.is_empty(),
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
            "Run focusa update apply --yes --allow-apply --dry-run false after reviewing this plan.".into(),
            "Daemon promotion remains last and requires a separate health/contract restart proof.".into(),
            "Use focusa update history --json to inspect completed promotions and recovery records.".into(),
        ],
    }
}

fn build_scheduler_envelope(channel: String) -> UpdateSchedulerEnvelope {
    let policy = update_policy_summary();
    UpdateSchedulerEnvelope {
        schema: "focusa.update_scheduler.v1",
        status: if systemd_scheduler_installed() {
            "installed"
        } else {
            "planned_read_only"
        },
        read_only: !systemd_scheduler_installed(),
        mutations_performed: false,
        scheduler_installed: systemd_scheduler_installed(),
        background_worker_started: systemd_scheduler_installed(),
        channel,
        startup_check: SchedulerStartupCheck {
            enabled: true,
            delay_seconds: 45,
            reason: "avoid slowing interactive daemon startup",
        },
        interval: SchedulerInterval {
            base_seconds: 21_600,
            jitter_percent: 20,
            backoff: vec!["5m", "15m", "1h", "6h"],
        },
        offline: SchedulerOfflineRules {
            skip_when_offline: true,
            retry_backoff: vec!["network_error", "dns_error", "release_host_timeout"],
            max_silent_failures_before_notice: 3,
        },
        maintenance: SchedulerMaintenanceWindow {
            respected: true,
            default_window: "02:00-05:00 local time",
            user_override_path: update_state_root()
                .join("maintenance-window.json")
                .display()
                .to_string(),
        },
        automatic_apply: SchedulerAutomaticApply {
            allowed: systemd_scheduler_installed(),
            reason: if systemd_scheduler_installed() {
                "systemd timer invokes explicit verified CLI promotion; daemon restart remains separately gated"
            } else {
                "install with focusa update scheduler --install to enable verified two-minute refresh"
            },
            requires: vec![
                "trusted_release_manifest",
                "update_lock_acquired",
                "rollback_snapshot_ready",
                "explicit_systemd_apply_consent",
                "daemon_restart_policy_approved",
            ],
        },
        notifications: notification_routes(),
        next_actions: vec![
            "wire daemon startup check after runtime tests",
            "wire interval worker after scheduler proof",
            "keep apply disabled until Spec128 gates pass",
        ],
        policy,
    }
}

fn systemd_scheduler_installed() -> bool {
    cfg!(target_os = "linux")
        && Path::new("/etc/systemd/system/focusa-update.timer").exists()
        && std::process::Command::new("systemctl")
            .args(["is-enabled", "--quiet", "focusa-update.timer"])
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
}

fn configure_systemd_scheduler(channel: &str, install: bool) -> anyhow::Result<()> {
    if !cfg!(target_os = "linux") || !is_root() {
        anyhow::bail!("systemd scheduler install requires Linux root");
    }
    let service = Path::new("/etc/systemd/system/focusa-update.service");
    let timer = Path::new("/etc/systemd/system/focusa-update.timer");
    if install {
        std::fs::write(
            service,
            format!(
                r#"[Unit]
Description=Focusa verified OTA update check/apply
After=network-online.target
Wants=network-online.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/focusa update apply --channel {channel} --yes --allow-apply --dry-run false --json
"#
            ),
        )?;
        std::fs::write(
            timer,
            r#"[Unit]
Description=Focusa verified OTA update timer

[Timer]
OnBootSec=2min
OnUnitActiveSec=2min
RandomizedDelaySec=30s
Persistent=true
Unit=focusa-update.service

[Install]
WantedBy=timers.target
"#,
        )?;
        run_systemctl(&["daemon-reload"])?;
        run_systemctl(&["enable", "--now", "focusa-update.timer"])?;
    } else {
        let _ = run_systemctl(&["disable", "--now", "focusa-update.timer"]);
        let _ = std::fs::remove_file(timer);
        let _ = std::fs::remove_file(service);
        run_systemctl(&["daemon-reload"])?;
    }
    Ok(())
}

fn run_systemctl(args: &[&str]) -> anyhow::Result<()> {
    let status = std::process::Command::new("systemctl")
        .args(args)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!(
            "systemctl {} exited {}",
            args.join(" "),
            status.code().unwrap_or(-1)
        )
    }
}

fn is_root() -> bool {
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim() == "0")
        .unwrap_or(false)
}

fn build_notifications_envelope(inventory: UpdateInventoryEnvelope) -> UpdateNotificationsEnvelope {
    let stale_parts = inventory.stale_parts;
    let severity = if stale_parts.is_empty() {
        "none"
    } else {
        "warning"
    };
    let body = if stale_parts.is_empty() {
        "Focusa surfaces are current or unknown; no update warning is required.".to_string()
    } else {
        format!(
            "Focusa update available for: {}. Run focusa update plan --json before applying.",
            stale_parts.join(", ")
        )
    };
    UpdateNotificationsEnvelope {
        schema: "focusa.update_notifications.v1",
        status: "completed",
        read_only: true,
        mutations_performed: false,
        stale_parts,
        severity,
        surfaces: notification_routes(),
        messages: vec![
            NotificationMessage {
                surface: "cli",
                title: "Focusa update status",
                body: body.clone(),
                action: "focusa update plan",
            },
            NotificationMessage {
                surface: "api",
                title: "Focusa update status",
                body: body.clone(),
                action: "POST /v1/update/plan",
            },
            NotificationMessage {
                surface: "pi_doctor",
                title: "Focusa update status",
                body,
                action: "focusa update status --json",
            },
        ],
        suppress_if: vec![
            "version_pinned",
            "version_skipped",
            "updates_paused",
            "offline_without_prior_success",
        ],
    }
}

fn notification_routes() -> NotificationRoutes {
    NotificationRoutes {
        cli: true,
        api: true,
        pi_doctor: true,
        tui: "planned_when_tui_update_banner_available",
        menubar: "planned_when_menubar_update_badge_available",
    }
}

fn print_scheduler_human(scheduler: &UpdateSchedulerEnvelope) {
    println!("Focusa update scheduler: {}", scheduler.status);
    println!("installed: {}", scheduler.scheduler_installed);
    println!("worker_started: {}", scheduler.background_worker_started);
    println!(
        "interval: {}s ±{}%",
        scheduler.interval.base_seconds, scheduler.interval.jitter_percent
    );
    println!("auto_apply_allowed: {}", scheduler.automatic_apply.allowed);
}

fn print_notifications_human(notifications: &UpdateNotificationsEnvelope) {
    println!("Focusa update notifications: {}", notifications.severity);
    for message in &notifications.messages {
        println!("{}: {} — {}", message.surface, message.title, message.body);
    }
}

fn build_history_envelope(limit: usize) -> UpdateHistoryEnvelope {
    let base = update_state_root();
    let history_path = base.join("update-history.jsonl");
    let journal_path = base.join("update-journal.json");
    let events = std::fs::read_to_string(&history_path)
        .ok()
        .map(|raw| raw.lines().rev().take(limit).map(str::to_string).collect())
        .unwrap_or_default();
    UpdateHistoryEnvelope {
        schema: "focusa.update_history.v1",
        status: "completed",
        read_only: true,
        mutations_performed: false,
        history_path: history_path.display().to_string(),
        journal_path: journal_path.display().to_string(),
        retention: RetentionPolicy {
            keep_last_successful_bundles: 3,
            keep_days: 30,
            prune_requires_admin_confirmation: true,
        },
        observability: UpdateObservability {
            counters: vec![
                "update_check_total",
                "update_plan_total",
                "update_apply_blocked_total",
                "update_apply_success_total",
                "update_rollback_total",
            ],
            events: vec![
                "check_started",
                "plan_created",
                "apply_blocked",
                "stage_verified",
                "promote_started",
                "daemon_restart_prompted",
                "rollback_started",
                "rollback_completed",
            ],
            log_paths: vec![
                base.join("update.log").display().to_string(),
                history_path.display().to_string(),
                journal_path.display().to_string(),
            ],
        },
        events,
        next_tools: vec![
            "focusa update plan --json",
            "focusa update rollback --dry-run --json",
        ],
    }
}

fn build_rollback_envelope(args: UpdateRollbackArgs) -> UpdateRollbackEnvelope {
    UpdateRollbackEnvelope {
        schema: "focusa.update_rollback.v1",
        status: "blocked_read_only",
        read_only: true,
        mutations_performed: false,
        rollback_executed: false,
        part: args.part,
        dry_run: args.dry_run,
        consent_yes: args.yes,
        blocked_reason: vec![
            "rollback_executor_not_enabled_in_spec128_08_scaffold",
            "snapshot_integrity_verification_required",
            "admin_confirmation_required",
        ],
        restore_order: match args.part {
            RollbackPart::Daemon => vec!["daemon", "restart_daemon_after_health_contract_check"],
            RollbackPart::All => vec!["daemon", "tui", "cli", "health_contract_check"],
            RollbackPart::Cli => vec!["cli"],
            RollbackPart::Tui => vec!["tui"],
        },
        proof_required: vec![
            "snapshot_sha256_verified",
            "same_filesystem_atomic_rename_available",
            "post_rollback_version_matches_snapshot",
            "no_data_env_license_overwrite",
            "history_event_written",
        ],
        data_safety: DataSafetyPlan {
            overwrite_data: false,
            overwrite_env: false,
            overwrite_license: false,
            preserve: build_safety_plan().preserves,
        },
        recovery_hint: "No rollback was executed. Inspect update history/journal and rerun with future rollback gates when implemented.",
    }
}

fn build_admin_envelope(args: UpdateAdminArgs) -> UpdateAdminEnvelope {
    let mut requested = Vec::new();
    if let Some(version) = &args.pin_version {
        requested.push(format!("pin_version:{version}"));
    }
    if args.unpin {
        requested.push("unpin".into());
    }
    if let Some(version) = &args.skip_version {
        requested.push(format!("skip_version:{version}"));
    }
    if args.pause {
        requested.push("pause".into());
    }
    if args.resume {
        requested.push("resume".into());
    }
    if args.force_check {
        requested.push("force_check".into());
    }
    if args.trusted_dev_force_latest {
        requested.push("trusted_dev_force_latest".into());
    }
    let dev_mode = std::env::var("FOCUSA_DEV_MODE")
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);
    UpdateAdminEnvelope {
        schema: "focusa.update_admin_control.v1",
        status: "preview_read_only",
        read_only: true,
        mutations_performed: false,
        dry_run: args.dry_run,
        consent_yes: args.yes,
        requested_controls: requested,
        policy_patch_preview: json!({
            "pin_version": args.pin_version,
            "unpin": args.unpin,
            "skip_version": args.skip_version,
            "pause": args.pause,
            "resume": args.resume,
            "trusted_dev_force_latest": args.trusted_dev_force_latest,
        }),
        force_check_preview: args.force_check,
        trusted_dev_force_latest_allowed: args.trusted_dev_force_latest && dev_mode,
        blocked_reason: vec![
            "admin_control_write_executor_not_enabled_in_spec128_08_scaffold",
            "dry_run_preview_only",
        ],
    }
}

fn print_history_human(history: &UpdateHistoryEnvelope) {
    println!("Focusa update history: {}", history.status);
    println!("history: {}", history.history_path);
    println!("journal: {}", history.journal_path);
    println!("events: {}", history.events.len());
}

fn print_rollback_human(rollback: &UpdateRollbackEnvelope) {
    println!("Focusa update rollback: {}", rollback.status);
    println!(
        "part: {:?} dry_run: {} executed: {}",
        rollback.part, rollback.dry_run, rollback.rollback_executed
    );
    println!("restore_order: {}", rollback.restore_order.join(" -> "));
    println!("blocked_reason: {}", rollback.blocked_reason.join(", "));
}

fn print_admin_human(admin: &UpdateAdminEnvelope) {
    println!("Focusa update admin: {}", admin.status);
    println!("requested: {}", admin.requested_controls.join(", "));
    println!("mutations_performed: {}", admin.mutations_performed);
}

async fn execute_verified_apply(plan: &UpdatePlanEnvelope) -> anyhow::Result<Vec<String>> {
    let state = update_state_root();
    std::fs::create_dir_all(&state)?;
    let lock_path = state.join("update.lock");
    let lock = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
        .with_context(|| format!("another update owns {}", lock_path.display()))?;
    let result = execute_verified_apply_locked(plan, &state).await;
    drop(lock);
    let _ = std::fs::remove_file(&lock_path);
    result
}

async fn execute_verified_apply_locked(
    plan: &UpdatePlanEnvelope,
    state: &Path,
) -> anyhow::Result<Vec<String>> {
    let stamp = format!("{}-{}", std::process::id(), chrono_like_timestamp());
    let stage = state.join("staging").join(&stamp);
    let backup_root = state.join("backups").join(&stamp);
    std::fs::create_dir_all(&stage)?;
    std::fs::create_dir_all(&backup_root)?;
    let journal = state.join("update-journal.json");
    std::fs::write(
        &journal,
        serde_json::to_vec_pretty(&json!({
            "schema":"focusa.update_journal.v1", "state":"staging", "tag":plan.latest.tag, "started_at":stamp
        }))?,
    )?;
    let mut promoted: Vec<(String, PathBuf, PathBuf)> = Vec::new();
    let operation = async {
        for part in plan
            .parts
            .iter()
            .filter(|part| matches!(part.action, "would_update" | "would_install"))
        {
            let url = part
                .download_url
                .as_deref()
                .context("release asset URL missing")?;
            let expected = part
                .expected_sha256
                .as_deref()
                .context("release asset checksum missing")?;
            let target = PathBuf::from(part.target_path.as_deref().context("target path missing")?);
            let parent = target.parent().context("target has no parent")?;
            std::fs::create_dir_all(parent)?;
            let staged = stage.join(format!("{}-{}", part.part, plan.latest.tag));
            let response = reqwest::get(url).await?.error_for_status()?;
            let bytes = response.bytes().await?;
            std::fs::write(&staged, &bytes)?;
            let actual = sha256_file(&staged)?;
            if actual != expected {
                anyhow::bail!(
                    "{} checksum mismatch: expected {expected}, got {actual}",
                    part.part
                );
            }
            let mode = target.metadata().ok().map(|m| m.permissions());
            #[cfg(unix)]
            if mode.is_none() {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&staged, std::fs::Permissions::from_mode(0o755))?;
            }
            std::fs::File::open(&staged)?.sync_all()?;
            let temp = parent.join(format!(
                ".{}.focusa-update-{}",
                target
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("focusa"),
                std::process::id()
            ));
            std::fs::rename(&staged, &temp)?;
            if let Some(permissions) = mode {
                std::fs::set_permissions(&temp, permissions)?;
            }
            let backup = backup_root.join(target.file_name().context("target filename missing")?);
            if target.exists() {
                std::fs::rename(&target, &backup)?;
            }
            if let Err(error) = std::fs::rename(&temp, &target) {
                if backup.exists() {
                    let _ = std::fs::rename(&backup, &target);
                }
                return Err(error.into());
            }
            // Record immediately after promotion so *every* subsequent probe
            // failure enters the outer rollback path.
            promoted.push((part.part.to_string(), target.clone(), backup));
            if part.part != "daemon" {
                let target_path = target.to_string_lossy();
                let got = if part.part == "tui" {
                    match probe_version_command(&target_path)
                        .await
                        .map(|value| normalize_version(&value))
                    {
                        Some(version) => Some(version),
                        None => probe_tui_version(&target_path).await,
                    }
                } else {
                    probe_version_command(&target_path)
                        .await
                        .map(|v| normalize_version(&v))
                }
                .context("post-promotion version probe failed")?;
                if got != plan.latest.version {
                    anyhow::bail!(
                        "{} smoke version mismatch: expected {}, got {}",
                        part.part,
                        plan.latest.version,
                        got
                    );
                }
            }
        }
        Ok::<(), anyhow::Error>(())
    }
    .await;
    if let Err(error) = operation {
        for (_, target, backup) in promoted.iter().rev() {
            if backup.exists() {
                let failed = target.with_extension("focusa-failed");
                let _ = std::fs::rename(target, &failed);
                let _ = std::fs::rename(backup, target);
                let _ = std::fs::remove_file(failed);
            }
        }
        std::fs::write(
            &journal,
            serde_json::to_vec_pretty(
                &json!({"schema":"focusa.update_journal.v1","state":"rolled_back","error":error.to_string()}),
            )?,
        )?;
        return Err(error);
    }
    let names = promoted
        .iter()
        .map(|(part, _, _)| part.clone())
        .collect::<Vec<_>>();
    if names.is_empty() && plan.parts.iter().any(|part| part.action == "would_update") {
        anyhow::bail!("no stale release parts were promoted");
    }
    std::fs::write(
        &journal,
        serde_json::to_vec_pretty(
            &json!({"schema":"focusa.update_journal.v1","state":"completed","tag":plan.latest.tag,"promoted":names}),
        )?,
    )?;
    let history = state.join("update-history.jsonl");
    let event =
        json!({"event":"apply_completed","tag":plan.latest.tag,"promoted":names,"journal":journal});
    use std::io::Write as _;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(history)?;
    writeln!(file, "{}", serde_json::to_string(&event)?)?;
    Ok(names)
}

fn chrono_like_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn build_apply_envelope(
    plan: UpdatePlanEnvelope,
    dry_run: bool,
    yes: bool,
    allow_apply: bool,
) -> UpdateApplyEnvelope {
    let mut blocked_reason = plan.apply_blocked_until.clone();
    if !plan.apply_allowed {
        blocked_reason.push("apply_requirements_not_satisfied".to_string());
    }
    if dry_run {
        blocked_reason.push("dry_run_requested".to_string());
    }
    if !(yes && allow_apply) {
        blocked_reason.push("explicit_yes_and_allow_apply_required".to_string());
    }
    let daemon_required = plan
        .parts
        .iter()
        .any(|part| part.part == "daemon" && part.action == "would_update");
    UpdateApplyEnvelope {
        schema: "focusa.update_apply.v1",
        status: if yes && allow_apply && !dry_run && plan.apply_allowed {
            "ready_to_apply"
        } else {
            "blocked_read_only"
        },
        read_only: true,
        mutations_performed: false,
        apply_requested: yes || allow_apply || !dry_run,
        apply_executed: false,
        dry_run,
        consent: ApplyConsent {
            yes,
            allow_apply,
            effective: yes && allow_apply && !dry_run,
            note: "consent allows verified promotion only after release trust and policy gates pass",
        },
        execution_order: vec!["cli", "tui", "daemon_last", "restart_daemon_only_if_changed_and_allowed"],
        daemon_restart: DaemonRestartPlan {
            allowed: false,
            required: daemon_required,
            when: "after daemon binary promotion, policy approval, and health/version/contract proof",
            health_proof: "GET /v1/health version and API contract must match target release",
        },
        data_safety: DataSafetyPlan {
            overwrite_data: false,
            overwrite_env: false,
            overwrite_license: false,
            preserve: plan.safety.preserves.clone(),
        },
        proof_required: vec![
            "release_manifest_signature_verified",
            "asset_sha256_verified",
            "cli_version_matches_target",
            "tui_version_matches_target_or_not_installed",
            "daemon_health_version_matches_target_when_daemon_changed",
            "daemon_api_contract_matches_target_when_daemon_changed",
            "no_data_env_license_overwrite",
            "rollback_journal_written",
        ],
        recovery_hint: "No update was applied. Use focusa update plan --json to inspect blockers; apply remains disabled until Spec128 apply gates are implemented.".into(),
        blocked_reason,
        plan,
    }
}

fn print_apply_human(apply: &UpdateApplyEnvelope) {
    println!("Focusa update apply: {}", apply.status);
    println!("read_only: {}", apply.read_only);
    println!("mutations_performed: {}", apply.mutations_performed);
    println!("apply_executed: {}", apply.apply_executed);
    println!("execution_order: {}", apply.execution_order.join(" -> "));
    println!("blocked_reason: {}", apply.blocked_reason.join(", "));
    println!("recovery: {}", apply.recovery_hint);
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

fn part_plan(part: &InstalledPart, latest: &LatestVersion, order: &mut u8) -> PartPlan {
    let action = if !part.exists {
        "would_install"
    } else {
        match part.stale {
            Some(true) => "would_update",
            Some(false) => "no_op",
            None => "probe_required",
        }
    };
    let restart_required = part.part == "daemon" && part.stale == Some(true);
    let plan = PartPlan {
        part: part.part,
        current_version: part.version.clone(),
        target_version: latest.version.clone(),
        target_path: part
            .resolved_path
            .clone()
            .or_else(|| Some(part.expected_path.clone())),
        expected_sha256: latest
            .assets
            .iter()
            .find(|asset| asset.part == part.part)
            .and_then(|asset| asset.sha256.clone()),
        download_url: latest
            .assets
            .iter()
            .find(|asset| asset.part == part.part)
            .map(|asset| asset.download_url.clone()),
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

async fn resolve_latest(channel: &str, override_value: Option<&str>) -> LatestVersion {
    if let Some(v) = override_value.filter(|s| !s.trim().is_empty()) {
        return placeholder_latest(normalize_version(v), "--latest-version");
    }
    for env_key in ["FOCUSA_LATEST_VERSION", "FOCUSA_UPDATE_LATEST_TAG"] {
        if let Ok(v) = std::env::var(env_key) {
            if !v.trim().is_empty() {
                return placeholder_latest(normalize_version(&v), env_key);
            }
        }
    }
    match resolve_latest_github(channel).await {
        Ok(latest) => latest,
        Err(error) => {
            let mut latest = placeholder_latest(
                env!("CARGO_PKG_VERSION").into(),
                "current_cli_package_version",
            );
            latest
                .trust
                .blockers
                .push(format!("github_release_resolver_failed:{error}"));
            latest
        }
    }
}

fn placeholder_latest(version: String, source: &str) -> LatestVersion {
    let tag = if version.starts_with('v') {
        version.clone()
    } else {
        format!("v{version}")
    };
    LatestVersion {
        version,
        tag,
        source: source.into(),
        github_repo: github_repo(),
        target_triple: target_triple(),
        release_manifest_required: true,
        eligibility_status: "placeholder_until_manifest_resolver",
        trust: ReleaseTrustSummary {
            release_resolved: false,
            complete_asset_set: false,
            sha256sums_present: false,
            checksums_resolved: false,
            signature_verified: false,
            ci_proof_required: true,
            signature_required: true,
            blockers: vec!["live_release_not_resolved".into()],
        },
        assets: Vec::new(),
    }
}

async fn resolve_latest_github(channel: &str) -> anyhow::Result<LatestVersion> {
    let repo = github_repo();
    let triple = target_triple();
    let url = format!("https://api.github.com/repos/{repo}/releases?per_page=20");
    let releases = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", "focusa-update-resolver")
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<GithubRelease>>()
        .await?;
    for release in releases {
        if release.draft || !release_tag_matches_channel(&release.tag_name, channel) {
            continue;
        }
        if let Some(latest) = build_latest_from_release(repo.clone(), triple.clone(), release) {
            return Ok(latest);
        }
    }
    anyhow::bail!("no complete release found for channel={channel} target={triple}")
}

fn build_latest_from_release(
    repo: String,
    triple: String,
    release: GithubRelease,
) -> Option<LatestVersion> {
    let tag = release.tag_name;
    let mut assets = Vec::new();
    for (part, prefix) in [
        ("cli", "focusa"),
        ("daemon", "focusa-daemon"),
        ("tui", "focusa-tui"),
    ] {
        let name = format!("{prefix}-{tag}-{triple}");
        let gh_asset = release.assets.iter().find(|asset| asset.name == name)?;
        assets.push(ReleaseAssetRef {
            part,
            name,
            download_url: gh_asset.browser_download_url.clone(),
            sha256: None,
        });
    }
    let checksum_asset = release
        .assets
        .iter()
        .find(|asset| asset.name == "SHA256SUMS" || asset.name == "SHA256SUMS.txt");
    let mut blockers = Vec::new();
    let mut checksums_resolved = false;
    let mut sha256sums_present = false;
    let mut signature_verified = false;
    if let Some(checksum_asset) = checksum_asset {
        sha256sums_present = true;
        match fetch_sha256sums_blocking(&checksum_asset.browser_download_url) {
            Ok(sums) => {
                for asset in &mut assets {
                    asset.sha256 = lookup_sha256(&sums, &asset.name);
                }
                checksums_resolved = assets.iter().all(|asset| asset.sha256.is_some());
                if !checksums_resolved {
                    blockers.push("release_sha256sums_missing_required_asset".into());
                }
            }
            Err(error) => blockers.push(format!("release_sha256sums_fetch_failed:{error}")),
        }
    } else {
        blockers.push("release_sha256sums_asset_missing".into());
    }
    if let (Some(checksum_asset), Some(sig_asset), Some(pem_asset)) = (
        checksum_asset,
        release
            .assets
            .iter()
            .find(|asset| asset.name == "SHA256SUMS.txt.sig"),
        release
            .assets
            .iter()
            .find(|asset| asset.name == "SHA256SUMS.txt.pem"),
    ) {
        match verify_sha256sums_signature(
            &checksum_asset.browser_download_url,
            &sig_asset.browser_download_url,
            &pem_asset.browser_download_url,
        ) {
            Ok(()) => signature_verified = true,
            Err(error) => blockers.push(format!("release_sha256sums_signature_invalid:{error}")),
        }
    } else {
        blockers.push("release_sha256sums_signature_assets_missing".into());
    }
    if !signature_verified {
        blockers.push("release_signature_not_verified".into());
    }
    Some(LatestVersion {
        version: normalize_version(&tag),
        tag,
        source: "github_releases".into(),
        github_repo: repo,
        target_triple: triple,
        release_manifest_required: true,
        eligibility_status: if checksums_resolved {
            "eligible_with_sha256sums"
        } else {
            "blocked_missing_checksums"
        },
        trust: ReleaseTrustSummary {
            release_resolved: true,
            complete_asset_set: true,
            sha256sums_present,
            checksums_resolved,
            signature_verified,
            ci_proof_required: true,
            signature_required: true,
            blockers,
        },
        assets,
    })
}

fn release_tag_matches_channel(tag: &str, channel: &str) -> bool {
    match channel {
        "dev" | "stable" => tag.starts_with('v') && tag.ends_with("-dev"),
        "preview" => tag.contains("-rc."),
        "nightly" => tag.contains("-nightly."),
        _ => false,
    }
}

fn verify_sha256sums_signature(
    checksums_url: &str,
    signature_url: &str,
    certificate_url: &str,
) -> anyhow::Result<()> {
    let dir = std::env::temp_dir().join(format!("focusa-update-verify-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;
    let checksums = dir.join("SHA256SUMS.txt");
    let signature = dir.join("SHA256SUMS.txt.sig");
    let cert_b64 = dir.join("SHA256SUMS.txt.pem.b64");
    let cert = dir.join("cert.pem");
    let public_key = dir.join("public.pem");
    let download = |url: &str, path: &std::path::Path| -> anyhow::Result<()> {
        let status = std::process::Command::new("curl")
            .args(["-fsSL", "--max-time", "20", url, "-o"])
            .arg(path)
            .status()?;
        if status.success() {
            Ok(())
        } else {
            anyhow::bail!("curl exited {}", status.code().unwrap_or(-1))
        }
    };
    let result = (|| -> anyhow::Result<()> {
        download(checksums_url, &checksums)?;
        download(signature_url, &signature)?;
        download(certificate_url, &cert_b64)?;
        let decoded = std::process::Command::new("base64")
            .args(["-d"])
            .arg(&cert_b64)
            .output()?;
        if !decoded.status.success() {
            anyhow::bail!("certificate base64 decode failed")
        }
        std::fs::write(&cert, decoded.stdout)?;
        let decoded = std::process::Command::new("base64")
            .args(["-d"])
            .arg(&signature)
            .output()?;
        if !decoded.status.success() {
            anyhow::bail!("signature base64 decode failed")
        }
        std::fs::write(&signature, decoded.stdout)?;
        let issuer = std::process::Command::new("openssl")
            .args(["x509", "-in"])
            .arg(&cert)
            .args(["-noout", "-issuer"])
            .output()?;
        if !issuer.status.success()
            || !String::from_utf8_lossy(&issuer.stdout).contains("sigstore.dev")
        {
            anyhow::bail!("certificate issuer is not sigstore.dev")
        }
        let status = std::process::Command::new("openssl")
            .args(["x509", "-in"])
            .arg(&cert)
            .args(["-pubkey", "-noout"])
            .stdout(std::fs::File::create(&public_key)?)
            .status()?;
        if !status.success() {
            anyhow::bail!("certificate public key extraction failed")
        }
        let status = std::process::Command::new("openssl")
            .args(["dgst", "-sha256", "-verify"])
            .arg(&public_key)
            .args(["-signature"])
            .arg(&signature)
            .arg(&checksums)
            .stdout(std::process::Stdio::null())
            .status()?;
        if !status.success() {
            anyhow::bail!("openssl signature verification failed")
        }
        Ok(())
    })();
    let _ = std::fs::remove_dir_all(&dir);
    result
}

fn fetch_sha256sums_blocking(url: &str) -> anyhow::Result<String> {
    let output = std::process::Command::new("curl")
        .args(["-fsSL", "--max-time", "20", url])
        .output()?;
    if !output.status.success() {
        anyhow::bail!("curl exited {}", output.status.code().unwrap_or(-1));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn lookup_sha256(sums: &str, asset_name: &str) -> Option<String> {
    sums.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let digest = parts.next()?;
        let name = parts.next()?.trim_start_matches('*');
        if name == asset_name && digest.len() == 64 {
            Some(digest.to_string())
        } else {
            None
        }
    })
}

fn github_repo() -> String {
    std::env::var("FOCUSA_GITHUB_REPO").unwrap_or_else(|_| "Startempire-Wire/focusa".into())
}

fn target_triple() -> String {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        // Musl assets avoid stale glibc floors on long-lived AlmaLinux/RHEL hosts.
        ("linux", "x86_64") => "x86_64-unknown-linux-musl".into(),
        ("linux", "aarch64") => "aarch64-unknown-linux-musl".into(),
        ("macos", "x86_64") => "x86_64-apple-darwin".into(),
        ("macos", "aarch64") => "aarch64-apple-darwin".into(),
        ("windows", "x86_64") => "x86_64-pc-windows-msvc.exe".into(),
        _ => format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
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
    let sha256 = path.as_deref().and_then(|p| sha256_file(Path::new(p)).ok());
    let version = match path.as_deref() {
        Some(path) => probe_tui_version(path).await,
        None => None,
    };
    // An installed TUI that cannot produce a safe version is refreshable:
    // trusted atomic promotion verifies the staged replacement and preserves
    // the old binary for rollback, avoiding a permanent probe_required state.
    let stale = version
        .as_ref()
        .map(|version| version != latest)
        .or_else(|| sha256.as_ref().map(|_| true));
    let stale_reason = match (&version, stale, &path) {
        (_, _, None) => "tui binary not found".into(),
        (Some(version), Some(true), _) => {
            format!("installed tui version {version} differs from latest {latest}")
        }
        (Some(version), Some(false), _) => {
            format!("installed tui version {version} matches latest {latest}")
        }
        _ => "tui headless version probe unavailable".into(),
    };
    Ok(InstalledPart {
        part: "tui",
        expected_path: "/usr/local/bin/focusa-tui".into(),
        resolved_path: path,
        exists: sha256.is_some(),
        version,
        version_source: "tui_headless_self_test",
        version_probe_safe: true,
        sha256,
        stale,
        stale_reason,
        notes: vec!["tui version is read from --headless-self-test JSON".into()],
    })
}

async fn inspect_daemon(latest: &str, health: Option<String>) -> anyhow::Result<InstalledPart> {
    let path = resolve_path("focusa-daemon", "/usr/local/bin/focusa-daemon");
    let sha256 = path.as_deref().and_then(|p| sha256_file(Path::new(p)).ok());
    let exists = path.is_some();
    // Prefer the running daemon's health version, but fall back to the binary
    // only after its --version path is guaranteed side-effect-free.
    let version = match health.as_deref() {
        Some(version) => Some(normalize_version(version)),
        None => match path.as_deref() {
            Some(path) => probe_version_command(path)
                .await
                .map(|value| normalize_version(&value)),
            None => None,
        },
    };
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
        version_source: "daemon_health_endpoint_or_binary_--version",
        version_probe_safe: true,
        sha256,
        stale,
        stale_reason,
        notes: vec![
            "daemon --version is a side-effect-free fallback when health is unavailable".into(),
        ],
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
    let override_key = format!(
        "FOCUSA_{}_PATH",
        command.to_ascii_uppercase().replace('-', "_")
    );
    if let Some(path) = std::env::var_os(override_key) {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path.to_string_lossy().to_string());
        }
    }
    if Path::new(canonical).exists() {
        return Some(canonical.into());
    }
    which::which(command)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

async fn probe_tui_version(path: &str) -> Option<String> {
    let output = timeout(
        Duration::from_secs(5),
        Command::new(path).arg("--headless-self-test").output(),
    )
    .await
    .ok()?
    .ok()?;
    if !output.status.success() {
        return None;
    }
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    json.get("about_version")
        .and_then(|value| value.as_str())
        .map(normalize_version)
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
