//! Spec 128 signed over-the-air update inventory, planning, apply, rollback, policy,
//! scheduler, notification, and history surfaces.
//!
//! Status/check/plan remain read-only. Apply requires explicit consent plus verified
//! release provenance, signatures, checksums, compatibility, staging, atomic promotion,
//! smoke checks, and rollback journaling.

use anyhow::Context;
use clap::{Args, Subcommand};
use focusa_core::license::load_license_status;
use focusa_core::update::{
    ReleaseChannel, TrustedReleaseKey, UPDATE_POLICY_SCHEMA_V1, UpdateMode, UpdatePolicy,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

#[path = "update_trust.rs"]
mod update_trust;

#[derive(Subcommand, Debug)]
pub enum UpdateCmd {
    /// Read-only installed-surface inventory and stale-part summary.
    Status(UpdateStatusArgs),
    /// Read-only update check. Same inventory as status plus channel/latest context.
    Check(UpdateStatusArgs),
    /// Read-only update plan. Shows what would change, prompts, compatibility gates, and restart impact.
    Plan(UpdateStatusArgs),
    /// Guarded update apply. Mutates only with explicit consent and complete signed-release trust.
    Apply(UpdateApplyArgs),
    /// Read-only update history/observability view.
    History(UpdateHistoryArgs),
    /// Guarded rollback. Defaults to dry-run and restores only SHA-verified backups with consent.
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
    /// Enable/disable the explicit trusted developer-host override.
    #[arg(long)]
    pub dev_mode: Option<bool>,
    /// Enable/disable every release-managed surface in one move.
    #[arg(long)]
    pub all_surfaces: Option<bool>,
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

    /// Explicit operator consent required with --dry-run=false for verified rollback.
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
    #[arg(long, value_name = "VERSION")]
    pub unskip_version: Option<String>,
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

    /// Explicit operator consent. Required with --allow-apply and --dry-run=false.
    #[arg(long)]
    pub yes: bool,

    /// Explicitly request guarded mutation after every trust/safety gate passes.
    #[arg(long)]
    pub allow_apply: bool,

    /// Mark this invocation as background automatic apply; policy authority is mandatory.
    #[arg(long)]
    pub automatic: bool,
}

#[derive(Args, Debug, Clone)]
pub struct UpdateStatusArgs {
    /// Release channel to compare against. Defaults to the update policy
    /// channel when omitted.
    #[arg(long)]
    pub channel: Option<String>,

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
    blocked_reason: Vec<String>,
    restore_order: Vec<&'static str>,
    proof_required: Vec<&'static str>,
    data_safety: DataSafetyPlan,
    recovery_hint: String,
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
    effective_state: UpdateAdminState,
    force_check_preview: bool,
    trusted_dev_force_latest_allowed: bool,
    blocked_reason: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UpdateAdminState {
    schema: String,
    pinned_version: Option<String>,
    skipped_versions: Vec<String>,
    paused: bool,
    force_check_requested_at: Option<String>,
    trusted_dev_force_latest: bool,
}

impl Default for UpdateAdminState {
    fn default() -> Self {
        Self {
            schema: "focusa.update_admin_state.v1".into(),
            pinned_version: None,
            skipped_versions: Vec::new(),
            paused: false,
            force_check_requested_at: None,
            trusted_dev_force_latest: false,
        }
    }
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
    installed: serde_json::Value,
    latest: String,
    applied: bool,
    surfaces: Vec<String>,
    rollback: serde_json::Value,
    next_action: String,
    blockers: Vec<String>,
    error: Option<String>,
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
    manifest_resolved: bool,
    manifest_signature_verified: bool,
    provenance_verified: bool,
    deploy_proof_verified: bool,
    trusted_key_id: Option<String>,
    trusted_key_fingerprint: Option<String>,
    key_revoked: bool,
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

#[derive(Debug, Deserialize)]
struct TrustedReleaseKeySet {
    schema: String,
    keys: Vec<TrustedReleaseKey>,
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
            let automatic = args.automatic || std::env::var_os("INVOCATION_ID").is_some();
            let envelope = build_inventory("apply", args.status).await?;
            let plan = build_update_plan(envelope);
            let mut apply = build_apply_envelope(plan, dry_run, yes, allow_apply);
            if automatic && !apply.plan.policy.auto_apply_allowed {
                apply.consent.effective = false;
                apply.plan.apply_allowed = false;
                apply
                    .blocked_reason
                    .push("automatic_apply_not_authorized_by_policy".into());
                apply
                    .blocked_reason
                    .extend(apply.plan.policy.auto_apply_blocked_until.clone());
                apply.recovery_hint = "Enable an entitled automatic policy or run a separately authorized manual apply.".into();
            }
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
                        apply.blocked_reason.push(format!("apply_failed:{error:#}"));
                        apply.recovery_hint = "Promotion failed; any previously promoted parts were restored from the update backup journal.".into();
                    }
                }
            }
            refresh_apply_summary(&mut apply);
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&apply)?);
            } else {
                print_apply_human(&apply);
            }
            if apply.status == "failed_rolled_back" {
                anyhow::bail!("update apply failed and rollback was applied");
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
            let execute = !args.dry_run && args.yes;
            let part = args.part;
            let mut rollback = build_rollback_envelope(args);
            if execute {
                match execute_verified_rollback(part).await {
                    Ok(restored) => {
                        rollback.status = "completed";
                        rollback.read_only = false;
                        rollback.mutations_performed = !restored.is_empty();
                        rollback.rollback_executed = true;
                        rollback.blocked_reason.clear();
                        rollback.recovery_hint =
                            "Rollback completed from SHA-verified backup manifest.".to_string();
                    }
                    Err(error) => {
                        rollback.status = "failed";
                        rollback.blocked_reason = vec!["rollback_failed".to_string()];
                        eprintln!("focusa update rollback failed: {error}");
                    }
                }
            }
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&rollback)?);
            } else {
                print_rollback_human(&rollback);
            }
        }
        UpdateCmd::Admin(args) => {
            let admin = build_admin_envelope(args)?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&admin)?);
            } else {
                print_admin_human(&admin);
            }
        }
        UpdateCmd::Scheduler(args) => {
            if args.install {
                configure_scheduler(&args.channel, true)?;
            } else if args.uninstall {
                configure_scheduler(&args.channel, false)?;
            }
            let mutation_requested = args.install || args.uninstall;
            let scheduler = build_scheduler_envelope(args.channel, mutation_requested);
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
    let channel = args.channel.clone().unwrap_or_else(effective_channel);
    let latest = resolve_latest(&channel, args.latest_version.as_deref()).await;
    let daemon_health = probe_daemon_health(&args.daemon_health_url).await;
    let mut parts = vec![
        inspect_cli(&latest.version).await?,
        inspect_tui(&latest.version).await?,
    ];
    if crate::commands::install::release_requires_distribution_manifest(&latest.version) {
        parts.push(inspect_session_runner(&latest.version).await?);
        parts.push(inspect_distribution_manifest(&latest.version));
        parts.push(inspect_agent_context(&latest.version));
    }
    parts.extend([
        inspect_pi_extension(&latest.version),
        inspect_menubar(&latest.version),
        inspect_installer(&latest.version),
        inspect_daemon(&latest.version, daemon_health).await?,
    ]);
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
        if part.exists && part.version.is_none() {
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
            "Use this stale-part report as input to focusa update plan --json.".to_string(),
            "Use focusa update apply with explicit consent after the signed plan passes every safety gate.".to_string(),
        ]
    };
    Ok(UpdateInventoryEnvelope {
        schema: "focusa.update_inventory.v1",
        status: "completed",
        command: command_name,
        read_only: true,
        mutations_performed: false,
        channel,
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
    if read_update_admin_state()
        .map(|state| state.paused)
        .unwrap_or(false)
    {
        blockers.push("updates_paused_by_admin".to_string());
    }
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
                "This plan is read-only; run focusa update apply with explicit consent after every safety gate passes.",
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

fn build_scheduler_envelope(channel: String, mutations_performed: bool) -> UpdateSchedulerEnvelope {
    let policy = update_policy_summary();
    UpdateSchedulerEnvelope {
        schema: "focusa.update_scheduler.v1",
        status: if scheduler_installed() {
            "installed"
        } else {
            "planned_read_only"
        },
        read_only: !scheduler_installed(),
        mutations_performed,
        scheduler_installed: scheduler_installed(),
        background_worker_started: scheduler_installed(),
        channel,
        startup_check: SchedulerStartupCheck {
            enabled: true,
            delay_seconds: 45,
            reason: "avoid slowing interactive daemon startup",
        },
        interval: SchedulerInterval {
            base_seconds: 120,
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
            allowed: scheduler_installed(),
            reason: if scheduler_installed() {
                "installed platform scheduler invokes verified atomic CLI promotion with rollback and daemon health proof"
            } else {
                "install with focusa update scheduler --install to enable verified two-minute refresh"
            },
            requires: vec![
                "trusted_release_manifest",
                "update_lock_acquired",
                "rollback_snapshot_ready",
                "explicit_scheduler_apply_consent",
                "daemon_restart_policy_approved",
            ],
        },
        notifications: notification_routes(),
        next_actions: vec![
            "monitor update history and signed release trust status",
            "adjust maintenance-window policy when operator scheduling requires it",
            "use focusa update rollback --dry-run=false --yes if post-update health regresses",
        ],
        policy,
    }
}

const UPDATE_LAUNCHD_LABEL: &str = "com.startempire.focusa-update";

fn scheduler_installed() -> bool {
    if cfg!(target_os = "macos") {
        let Some(home) = std::env::var_os("HOME") else {
            return false;
        };
        let plist = PathBuf::from(home)
            .join("Library/LaunchAgents")
            .join(format!("{UPDATE_LAUNCHD_LABEL}.plist"));
        return plist.is_file()
            && launchd_user_target()
                .and_then(|target| {
                    std::process::Command::new("launchctl")
                        .args(["print", &target])
                        .status()
                        .ok()
                })
                .map(|status| status.success())
                .unwrap_or(false);
    }
    cfg!(target_os = "linux")
        && Path::new("/etc/systemd/system/focusa-update.timer").exists()
        && std::process::Command::new("systemctl")
            .args(["is-enabled", "--quiet", "focusa-update.timer"])
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
}

fn launchd_user_target() -> Option<String> {
    let output = std::process::Command::new("id").arg("-u").output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(format!(
        "gui/{}/{}",
        String::from_utf8_lossy(&output.stdout).trim(),
        UPDATE_LAUNCHD_LABEL
    ))
}

fn configure_scheduler(channel: &str, install: bool) -> anyhow::Result<()> {
    if cfg!(target_os = "macos") {
        return configure_launchd_scheduler(channel, install);
    }
    configure_systemd_scheduler(channel, install)
}

fn configure_launchd_scheduler(channel: &str, install: bool) -> anyhow::Result<()> {
    let home = std::env::var_os("HOME").context("HOME not set")?;
    let agents = PathBuf::from(home).join("Library/LaunchAgents");
    let plist = agents.join(format!("{UPDATE_LAUNCHD_LABEL}.plist"));
    let target = launchd_user_target().context("cannot resolve launchd user domain")?;
    if install {
        std::fs::create_dir_all(&agents)?;
        let focusa = std::env::current_exe()?.display().to_string();
        let body = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>Label</key><string>{UPDATE_LAUNCHD_LABEL}</string>
<key>ProgramArguments</key><array><string>{focusa}</string><string>update</string><string>apply</string><string>--channel</string><string>{channel}</string><string>--yes</string><string>--allow-apply</string><string>--automatic</string><string>--dry-run</string><string>false</string><string>--json</string></array>
<key>RunAtLoad</key><true/><key>StartInterval</key><integer>120</integer>
<key>ThrottleInterval</key><integer>300</integer><key>ProcessType</key><string>Background</string>
</dict></plist>
"#
        );
        std::fs::write(&plist, body)?;
        let _ = std::process::Command::new("launchctl")
            .args(["bootout", &target])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        let domain = target
            .rsplit_once('/')
            .map(|(domain, _)| domain)
            .context("invalid launchd user target")?;
        let status = std::process::Command::new("launchctl")
            .args(["bootstrap", domain, &plist.display().to_string()])
            .status()?;
        if !status.success() {
            anyhow::bail!(
                "launchctl bootstrap failed: {}",
                status.code().unwrap_or(-1)
            );
        }
    } else {
        let _ = std::process::Command::new("launchctl")
            .args(["bootout", &target])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        let _ = std::fs::remove_file(plist);
    }
    Ok(())
}

fn configure_systemd_scheduler(channel: &str, install: bool) -> anyhow::Result<()> {
    if !cfg!(target_os = "linux") || !is_root() {
        anyhow::bail!("systemd scheduler install requires Linux root");
    }
    let service = Path::new("/etc/systemd/system/focusa-update.service");
    let timer = Path::new("/etc/systemd/system/focusa-update.timer");
    if install {
        let runtime_path = std::env::var("PATH")
            .ok()
            .filter(|path| !path.contains(['\n', '\r', '"']))
            .unwrap_or_else(|| {
                "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".into()
            });
        std::fs::write(
            service,
            format!(
                r#"[Unit]
Description=Focusa verified OTA update check/apply
After=network-online.target
Wants=network-online.target

[Service]
Type=oneshot
Environment="PATH={runtime_path}"
Environment="FOCUSA_FOCUSA_PATH=/usr/local/bin/focusa"
Environment="FOCUSA_FOCUSA_TUI_PATH=/usr/local/bin/focusa-tui"
Environment="FOCUSA_FOCUSA_DAEMON_PATH=/usr/local/bin/focusa-daemon"
ExecStart=/usr/local/bin/focusa update apply --channel {channel} --yes --allow-apply --automatic --dry-run false --json
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
RandomizedDelaySec=24s
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
    let admin = read_update_admin_state().unwrap_or_default();
    // 320: channel mismatch + inversion do not use stale_parts alone; policy channel is the source of truth.
    let policy_channel = inventory.policy.channel.clone();
    let effective_channel = if policy_channel.is_empty() {
        inventory.channel.clone()
    } else {
        policy_channel.clone()
    };
    let channel_mismatch = !policy_channel.is_empty() && policy_channel != inventory.channel;
    let inversion = inventory.parts.iter().any(|part| {
        part.version
            .as_deref()
            .map(|installed| installed > inventory.latest.version.as_str())
            .unwrap_or(false)
    });
    let stale_parts = if admin.paused {
        Vec::new()
    } else {
        inventory.stale_parts
    };
    let has_warning = !stale_parts.is_empty() || channel_mismatch || inversion;
    let severity = if has_warning { "warning" } else { "none" };
    let mut body = if stale_parts.is_empty() {
        "Focusa surfaces are current or unknown; no update warning is required.".to_string()
    } else {
        format!(
            "Focusa update available for: {}. Run focusa update plan --json before applying.",
            stale_parts.join(", ")
        )
    };
    if channel_mismatch {
        body = format!(
            "{} Policy channel '{}' mismatches inventory channel '{}'; nightly is blocked until channels align.",
            body, policy_channel, inventory.channel
        );
    }
    if inversion {
        body = format!(
            "{} Installed newer than Latest {} on channel '{}' (version inversion: no downgrade offered).",
            body,
            inventory.latest.version.as_str(),
            effective_channel
        );
    }
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
                body: body.clone(),
                action: "focusa update status --json",
            },
            NotificationMessage {
                surface: "tui",
                title: "Focusa update status",
                body: body.clone(),
                action: "open Focusa TUI footer update indicator",
            },
            NotificationMessage {
                surface: "menubar",
                title: "Focusa update status",
                body,
                action: "open Focusa menubar update badge",
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
        tui: "active_footer_update_indicator",
        menubar: "active_update_badge",
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

#[derive(Deserialize)]
struct RollbackManifestEntry {
    part: String,
    target: PathBuf,
    backup: PathBuf,
    sha256: String,
}

#[derive(Deserialize)]
struct RollbackManifest {
    entries: Vec<RollbackManifestEntry>,
    strategy: Option<String>,
    release_tag: Option<String>,
    system_install: Option<bool>,
    github_repo: Option<String>,
}

async fn execute_verified_rollback(part: RollbackPart) -> anyhow::Result<Vec<String>> {
    let backups = update_state_root().join("backups");
    let manifest = std::fs::read_dir(&backups)?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path().join("rollback-manifest.json");
            path.exists().then_some(path)
        })
        .max_by_key(|path| path.metadata().and_then(|m| m.modified()).ok())
        .context("no rollback manifest available")?;
    let manifest: RollbackManifest = serde_json::from_slice(&std::fs::read(&manifest)?)?;
    if manifest.strategy.as_deref() == Some("exact_release_reinstall") {
        anyhow::ensure!(
            part == RollbackPart::All,
            "manifest-bound updates roll back as one full release; use --part all"
        );
        let tag = manifest
            .release_tag
            .as_deref()
            .context("manifest-bound rollback release tag is missing")?;
        let args = exact_release_install_args(
            tag,
            manifest
                .github_repo
                .as_deref()
                .unwrap_or("Startempire-Wire/focusa"),
            manifest.system_install.unwrap_or(false),
        );
        crate::commands::install::run(args)
            .await
            .context("exact prior-release reinstall failed")?;
        return Ok(vec!["full_release".into()]);
    }
    let wanted = |name: &str| match part {
        RollbackPart::All => true,
        RollbackPart::Cli => name == "cli",
        RollbackPart::Tui => name == "tui",
        RollbackPart::Daemon => name == "daemon",
    };
    for entry in manifest.entries.iter().filter(|entry| wanted(&entry.part)) {
        if sha256_file(&entry.backup)? != entry.sha256 {
            anyhow::bail!("backup checksum mismatch for {}", entry.part);
        }
    }
    let restoring_daemon = manifest
        .entries
        .iter()
        .any(|entry| wanted(&entry.part) && entry.part == "daemon");
    #[cfg(target_os = "linux")]
    let _system_deploy_lock = if let Some(daemon_path) = manifest
        .entries
        .iter()
        .find(|entry| wanted(&entry.part) && entry.part == "daemon")
        .map(|entry| entry.target.as_path())
        .filter(|path| crate::commands::system_service::is_canonical_system_daemon(path))
    {
        let system_bin = daemon_path
            .parent()
            .context("canonical daemon rollback target has no parent")?;
        let lock = crate::commands::system_service::acquire_system_deploy_lock(system_bin)?;
        crate::commands::system_service::preflight_system_install()?;
        Some(lock)
    } else {
        None
    };
    if restoring_daemon {
        stop_daemon_before_promotion()?;
    }
    let mut restored = Vec::new();
    let mut restored_daemon_path = None;
    for entry in manifest
        .entries
        .into_iter()
        .filter(|entry| wanted(&entry.part))
    {
        let failed = entry.target.with_extension("focusa-pre-rollback");
        if entry.target.exists() {
            move_file_cross_device_safe(&entry.target, &failed)?;
        }
        if let Err(error) = move_file_cross_device_safe(&entry.backup, &entry.target) {
            if failed.exists() {
                let _ = move_file_cross_device_safe(&failed, &entry.target);
            }
            return Err(error);
        }
        let _ = std::fs::remove_file(&failed);
        if entry.part == "daemon" {
            restored_daemon_path = Some(entry.target.clone());
        }
        restored.push(entry.part);
    }
    if restored.is_empty() {
        anyhow::bail!("no matching verified backup entries");
    }
    if let Some(daemon_path) = restored_daemon_path.as_deref() {
        restart_daemon_service(daemon_path)?;
    }
    let state = update_state_root();
    let journal = state.join("update-journal.json");
    std::fs::write(
        &journal,
        serde_json::to_vec_pretty(&json!({
            "schema":"focusa.update_journal.v1",
            "state":"rollback_completed",
            "restored":restored,
        }))?,
    )?;
    let history = state.join("update-history.jsonl");
    use std::io::Write as _;
    let mut history_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(history)?;
    writeln!(
        history_file,
        "{}",
        serde_json::to_string(&json!({"event":"rollback_completed","restored":restored}))?
    )?;
    Ok(restored)
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
            "dry_run_default_no_mutation".to_string(),
            "snapshot_integrity_verification_required".to_string(),
            "admin_confirmation_required".to_string(),
        ],
        restore_order: match args.part {
            RollbackPart::Daemon => vec!["daemon", "restart_daemon_after_health_contract_check"],
            RollbackPart::All => vec![
                "full_release",
                "daemon",
                "session_runner",
                "tui",
                "cli",
                "distribution_manifest",
                "agent_context",
                "health_contract_check",
                "callgraph_contract_check",
            ],
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
        recovery_hint: "No rollback was executed in dry-run mode. Inspect update history/journal, then rerun with --dry-run=false --yes.".to_string(),
    }
}

fn update_admin_state_path() -> anyhow::Result<PathBuf> {
    if let Some(path) = std::env::var_os("FOCUSA_UPDATE_ADMIN_STATE") {
        return Ok(PathBuf::from(path));
    }
    let home = std::env::var_os("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".config/focusa/update-admin.json"))
}

fn read_update_admin_state() -> anyhow::Result<UpdateAdminState> {
    let path = update_admin_state_path()?;
    if !path.is_file() {
        return Ok(UpdateAdminState::default());
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read update admin state {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("parse update admin state {}", path.display()))
}

fn write_update_admin_state(state: &UpdateAdminState) -> anyhow::Result<PathBuf> {
    let path = update_admin_state_path()?;
    let parent = path.parent().context("update admin state has no parent")?;
    std::fs::create_dir_all(parent)?;
    let staged = path.with_extension("json.tmp");
    std::fs::write(&staged, serde_json::to_vec_pretty(state)?)?;
    std::fs::rename(&staged, &path)?;
    Ok(path)
}

fn build_admin_envelope(args: UpdateAdminArgs) -> anyhow::Result<UpdateAdminEnvelope> {
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
    if let Some(version) = &args.unskip_version {
        requested.push(format!("unskip_version:{version}"));
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
    let trusted_force_allowed = args.trusted_dev_force_latest && dev_mode;
    let mut state = read_update_admin_state()?;
    let mutation_requested = !requested.is_empty();
    let can_mutate = mutation_requested
        && !args.dry_run
        && args.yes
        && (!args.trusted_dev_force_latest || trusted_force_allowed);
    if can_mutate {
        if let Some(version) = &args.pin_version {
            state.pinned_version = Some(normalize_version(version));
        }
        if args.unpin {
            state.pinned_version = None;
        }
        if let Some(version) = &args.skip_version {
            let version = normalize_version(version);
            if !state.skipped_versions.contains(&version) {
                state.skipped_versions.push(version);
                state.skipped_versions.sort();
            }
        }
        if let Some(version) = &args.unskip_version {
            let version = normalize_version(version);
            state.skipped_versions.retain(|entry| entry != &version);
        }
        if args.pause {
            state.paused = true;
        }
        if args.resume {
            state.paused = false;
        }
        if args.force_check {
            state.force_check_requested_at = Some(chrono::Utc::now().to_rfc3339());
        }
        if args.trusted_dev_force_latest {
            state.trusted_dev_force_latest = true;
            state.pinned_version = None;
        }
        write_update_admin_state(&state)?;
    }
    let blocked_reason = if !mutation_requested {
        Vec::new()
    } else if args.trusted_dev_force_latest && !trusted_force_allowed {
        vec!["trusted_dev_force_latest_requires_dev_mode"]
    } else if args.dry_run || !args.yes {
        vec!["mutation_requires_dry_run_false_and_yes"]
    } else {
        Vec::new()
    };
    Ok(UpdateAdminEnvelope {
        schema: "focusa.update_admin_control.v1",
        status: if can_mutate {
            "applied"
        } else {
            "preview_read_only"
        },
        read_only: !can_mutate,
        mutations_performed: can_mutate,
        dry_run: args.dry_run,
        consent_yes: args.yes,
        requested_controls: requested,
        policy_patch_preview: json!({
            "pin_version": args.pin_version,
            "unpin": args.unpin,
            "skip_version": args.skip_version,
            "unskip_version": args.unskip_version,
            "pause": args.pause,
            "resume": args.resume,
            "trusted_dev_force_latest": args.trusted_dev_force_latest,
        }),
        effective_state: state,
        force_check_preview: args.force_check,
        trusted_dev_force_latest_allowed: trusted_force_allowed,
        blocked_reason,
    })
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

type PromotedPart = (String, PathBuf, PathBuf, String);

fn move_file_cross_device_safe(source: &Path, destination: &Path) -> anyhow::Result<()> {
    const WINDOWS_LOCK_RETRIES: usize = 120;
    for attempt in 0..WINDOWS_LOCK_RETRIES {
        match std::fs::rename(source, destination) {
            Ok(()) => return Ok(()),
            Err(error) if error.raw_os_error() == Some(18) => {
                std::fs::copy(source, destination)?;
                std::fs::File::open(destination)?.sync_all()?;
                std::fs::remove_file(source)?;
                return Ok(());
            }
            Err(error)
                if cfg!(target_os = "windows")
                    && error.raw_os_error() == Some(5)
                    && attempt + 1 < WINDOWS_LOCK_RETRIES =>
            {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            Err(error) => return Err(error.into()),
        }
    }
    unreachable!("bounded move retry loop always returns")
}

fn terminate_portable_daemon_from_lock() {
    let data_dir = std::env::var_os("FOCUSA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("XDG_DATA_HOME").map(|path| PathBuf::from(path).join("focusa"))
        })
        .or_else(|| {
            std::env::var_os("HOME").map(|path| PathBuf::from(path).join(".local/share/focusa"))
        });
    let Some(lock) = data_dir.map(|path| path.join("focusa-daemon.lock")) else {
        return;
    };
    let pid = std::fs::read_to_string(lock).ok().and_then(|raw| {
        raw.lines()
            .find_map(|line| line.strip_prefix("pid="))
            .and_then(|value| value.trim().parse::<u32>().ok())
    });
    if let Some(pid) = pid {
        let _ = std::process::Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status();
        std::thread::sleep(Duration::from_millis(500));
    }
}

fn stop_daemon_before_promotion() -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "focusa-daemon.exe"])
            .output();
        std::thread::sleep(std::time::Duration::from_millis(750));
    }
    Ok(())
}

fn spawn_daemon_detached_with_retry(daemon_path: &Path) -> anyhow::Result<()> {
    const WINDOWS_SPAWN_RETRIES: usize = 120;
    for attempt in 0..WINDOWS_SPAWN_RETRIES {
        match std::process::Command::new(daemon_path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(_) => return Ok(()),
            Err(error)
                if cfg!(target_os = "windows")
                    && error.raw_os_error() == Some(5)
                    && attempt + 1 < WINDOWS_SPAWN_RETRIES =>
            {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("start promoted daemon {}", daemon_path.display()));
            }
        }
    }
    unreachable!("bounded daemon spawn retry loop always returns")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DaemonRestoreAction {
    None,
    Start,
    Stop,
}

fn daemon_restore_action(touched: bool, was_running: bool) -> DaemonRestoreAction {
    match (touched, was_running) {
        (false, _) => DaemonRestoreAction::None,
        (true, true) => DaemonRestoreAction::Start,
        (true, false) => DaemonRestoreAction::Stop,
    }
}

fn stop_daemon_service() -> anyhow::Result<()> {
    if cfg!(target_os = "macos") {
        let uid = std::process::Command::new("id").arg("-u").output()?;
        let target = format!(
            "gui/{}/com.startempire.focusa-daemon",
            String::from_utf8_lossy(&uid.stdout).trim()
        );
        let _ = std::process::Command::new("launchctl")
            .args(["kill", "SIGTERM", &target])
            .status();
    } else if cfg!(target_os = "windows") {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "focusa-daemon.exe"])
            .status();
    } else {
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "stop", "focusa-daemon.service"])
            .status();
    }
    terminate_portable_daemon_from_lock();
    Ok(())
}

fn restart_daemon_service(daemon_path: &Path) -> anyhow::Result<()> {
    if cfg!(target_os = "macos") {
        let uid = std::process::Command::new("id").arg("-u").output()?;
        let target = format!(
            "gui/{}/com.startempire.focusa-daemon",
            String::from_utf8_lossy(&uid.stdout).trim()
        );
        if std::process::Command::new("launchctl")
            .args(["kickstart", "-k", &target])
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
        {
            return Ok(());
        }
    } else if cfg!(target_os = "windows") {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "focusa-daemon.exe"])
            .output();
        spawn_daemon_detached_with_retry(daemon_path)?;
        return Ok(());
    } else {
        #[cfg(target_os = "linux")]
        if crate::commands::system_service::is_canonical_system_daemon(daemon_path) {
            return crate::commands::system_service::restart_existing_system_service();
        }
        if std::process::Command::new("systemctl")
            .args(["--user", "restart", "focusa-daemon.service"])
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
        {
            return Ok(());
        }
    }
    terminate_portable_daemon_from_lock();
    spawn_daemon_detached_with_retry(daemon_path)
}

fn rollback_promoted_parts(promoted: &[PromotedPart]) -> anyhow::Result<Vec<String>> {
    if promoted.iter().any(|(part, _, _, _)| part == "daemon") {
        stop_daemon_before_promotion().context("stop promoted daemon before rollback")?;
    }
    let mut restored = Vec::new();
    for (part, target, backup, _) in promoted.iter().rev() {
        if !backup.exists() {
            if target.exists() {
                std::fs::remove_file(target)?;
            }
            restored.push(part.clone());
            continue;
        }
        let failed = target.with_extension("focusa-failed");
        if target.exists() {
            move_file_cross_device_safe(target, &failed)?;
        }
        if let Err(error) = move_file_cross_device_safe(backup, target) {
            if failed.exists() {
                let _ = move_file_cross_device_safe(&failed, target);
            }
            return Err(error);
        }
        if failed.exists() {
            std::fs::remove_file(&failed)?;
        }
        restored.push(part.clone());
    }
    Ok(restored)
}

fn exact_release_install_args(
    tag: &str,
    github_repo: &str,
    system_install: bool,
) -> crate::commands::install::InstallArgs {
    let channel = if tag.contains("-nightly.") {
        crate::commands::install::Channel::Nightly
    } else if tag.contains('-') {
        crate::commands::install::Channel::Preview
    } else {
        crate::commands::install::Channel::Stable
    };
    crate::commands::install::InstallArgs {
        target: crate::commands::install::InstallTarget::Auto,
        channel,
        dry_run: false,
        preflight: false,
        no_animation: true,
        quiet: true,
        install_dependencies: false,
        assume_yes: false,
        license_key: None,
        eval: false,
        accept_license: true,
        no_service: false,
        reuse_existing_license: true,
        suppress_completion_output: true,
        release_tag_override: Some(tag.to_string()),
        system_install,
        persist_path: false,
        no_persist_path: true,
        on_shell: crate::commands::install::ShellFamily::Auto,
        json: false,
        github_repo: Some(github_repo.to_string()),
    }
}

async fn execute_manifest_bound_apply(
    plan: &UpdatePlanEnvelope,
    state: &Path,
    backup_root: &Path,
) -> anyhow::Result<Vec<String>> {
    let mutable_parts = plan
        .parts
        .iter()
        .filter(|part| {
            matches!(
                part.action,
                "would_update" | "would_install" | "would_update_package" | "would_install_package"
            )
        })
        .map(|part| part.part.to_string())
        .collect::<Vec<_>>();
    if mutable_parts.is_empty() {
        return Ok(Vec::new());
    }
    let system_install = plan.parts.iter().any(|part| {
        part.target_path
            .as_deref()
            .is_some_and(|path| path.starts_with("/usr/local/"))
    });
    let args =
        exact_release_install_args(&plan.latest.tag, &plan.latest.github_repo, system_install);
    let journal = state.join("update-journal.json");
    match crate::commands::install::run(args).await {
        Ok(()) => {
            if let Some(previous_version) = plan
                .parts
                .iter()
                .find(|part| part.part == "cli")
                .and_then(|part| part.current_version.as_deref())
                .or_else(|| {
                    plan.parts
                        .iter()
                        .find(|part| part.part == "daemon")
                        .and_then(|part| part.current_version.as_deref())
                })
                .filter(|version| normalize_version(version) != plan.latest.version)
            {
                std::fs::write(
                    backup_root.join("rollback-manifest.json"),
                    serde_json::to_vec_pretty(&json!({
                        "schema": "focusa.update_rollback_manifest.v1",
                        "strategy": "exact_release_reinstall",
                        "release_tag": release_tag_for_version(previous_version),
                        "system_install": system_install,
                        "github_repo": plan.latest.github_repo,
                        "entries": [],
                    }))?,
                )?;
            }
            std::fs::write(
                &journal,
                serde_json::to_vec_pretty(&json!({
                    "schema": "focusa.update_journal.v1",
                    "state": "completed",
                    "tag": plan.latest.tag,
                    "lifecycle_owner": "focusa_install",
                    "promoted": mutable_parts,
                }))?,
            )?;
            Ok(mutable_parts)
        }
        Err(error) => {
            std::fs::write(
                &journal,
                serde_json::to_vec_pretty(&json!({
                    "schema": "focusa.update_journal.v1",
                    "state": "failed_rolled_back",
                    "tag": plan.latest.tag,
                    "lifecycle_owner": "focusa_install",
                    "error": error.to_string(),
                }))?,
            )?;
            Err(error.context("canonical manifest-bound install transaction failed"))
        }
    }
}

async fn execute_verified_apply_locked(
    plan: &UpdatePlanEnvelope,
    state: &Path,
) -> anyhow::Result<Vec<String>> {
    let manifest_bound =
        crate::commands::install::release_requires_distribution_manifest(&plan.latest.version);
    #[cfg(target_os = "linux")]
    let _system_deploy_lock = if manifest_bound {
        // The one canonical install lifecycle acquires this same lock. Never
        // acquire it twice through the OTA compatibility adapter.
        None
    } else if let Some(daemon_path) = plan
        .parts
        .iter()
        .find(|part| part.part == "daemon")
        .and_then(|part| part.target_path.as_deref())
        .map(Path::new)
        .filter(|path| crate::commands::system_service::is_canonical_system_daemon(path))
    {
        let system_bin = daemon_path
            .parent()
            .context("canonical daemon target has no parent")?;
        let lock = crate::commands::system_service::acquire_system_deploy_lock(system_bin)?;
        crate::commands::system_service::preflight_system_install()?;
        Some(lock)
    } else {
        None
    };
    let stamp = format!("{}-{}", std::process::id(), chrono_like_timestamp());
    let stage = state.join("staging").join(&stamp);
    let backup_root = state.join("backups").join(&stamp);
    std::fs::create_dir_all(&stage)?;
    std::fs::create_dir_all(&backup_root)?;
    let journal = state.join("update-journal.json");
    let progress = state.join("update-progress.txt");
    std::fs::write(&progress, "staging")?;
    std::fs::write(
        &journal,
        serde_json::to_vec_pretty(&json!({
            "schema":"focusa.update_journal.v1", "state":"staging", "tag":plan.latest.tag, "started_at":stamp
        }))?,
    )?;
    if manifest_bound {
        return execute_manifest_bound_apply(plan, state, &backup_root).await;
    }
    let daemon_health_url = std::env::var("FOCUSA_DAEMON_HEALTH_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8787/v1/health".into());
    let daemon_was_running = probe_daemon_health(&daemon_health_url).await.is_some();
    // part, target path, backup path, SHA-256 of the pre-update target.
    let mut promoted: Vec<PromotedPart> = Vec::new();
    let mut package_promoted: Vec<String> = Vec::new();
    let operation = async {
        for part in plan
            .parts
            .iter()
            .filter(|part| matches!(part.action, "would_update" | "would_install"))
        {
            std::fs::write(&progress, format!("binary:{}:download", part.part))?;
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
            #[cfg(unix)]
            let mode = target.metadata().ok().map(|m| m.permissions());
            #[cfg(unix)]
            if mode.is_none() {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&staged, std::fs::Permissions::from_mode(0o755))?;
            }
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&staged)?
                .sync_all()?;
            let temp = parent.join(format!(
                ".{}.focusa-update-{}",
                target
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("focusa"),
                std::process::id()
            ));
            move_file_cross_device_safe(&staged, &temp)?;
            #[cfg(unix)]
            if let Some(permissions) = mode {
                std::fs::set_permissions(&temp, permissions)?;
            }
            // Windows refuses to replace a running executable. Stop only after
            // the staged daemon passed checksum verification, minimizing downtime.
            if part.part == "daemon" {
                stop_daemon_before_promotion()?;
            }
            std::fs::write(&progress, format!("binary:{}:promote", part.part))?;
            let backup = backup_root.join(target.file_name().context("target filename missing")?);
            let backup_sha256 = if target.exists() {
                let digest = sha256_file(&target)?;
                move_file_cross_device_safe(&target, &backup)
                    .with_context(|| format!("backup installed {}", part.part))?;
                digest
            } else {
                String::new()
            };
            if let Err(error) = move_file_cross_device_safe(&temp, &target)
                .with_context(|| format!("promote staged {}", part.part))
            {
                if backup.exists() {
                    let _ = move_file_cross_device_safe(&backup, &target);
                }
                return Err(error);
            }
            // Record immediately after promotion so *every* subsequent probe
            // failure enters the outer rollback path.
            promoted.push((part.part.to_string(), target.clone(), backup, backup_sha256));
            if std::env::var("FOCUSA_UPDATE_FAULT_AFTER_PROMOTE")
                .map(|fault_part| fault_part == part.part)
                .unwrap_or(false)
            {
                anyhow::bail!("injected fault after promoting {}", part.part);
            }
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
        if let Some((_, daemon_path, _, _)) =
            promoted.iter().find(|(part, _, _, _)| part == "daemon")
        {
            std::fs::write(&progress, "daemon:restart_and_health")?;
            restart_daemon_service(daemon_path)?;
            let mut observed_version = None;
            for _ in 0..20 {
                tokio::time::sleep(Duration::from_secs(1)).await;
                if let Some(version) = probe_daemon_health(&daemon_health_url).await {
                    if normalize_version(&version) == plan.latest.version {
                        observed_version = Some(version);
                        break;
                    }
                }
            }
            observed_version.with_context(|| {
                format!(
                    "daemon health did not reach OTA version {} within 20 seconds",
                    plan.latest.version
                )
            })?;
        }
        for part in plan.parts.iter().filter(|part| {
            matches!(
                part.action,
                "would_update_package" | "would_install_package"
            )
        }) {
            std::fs::write(&progress, format!("package:{}:download", part.part))?;
            let url = part
                .download_url
                .as_deref()
                .context("Pi extension asset URL missing")?;
            let expected = part
                .expected_sha256
                .as_deref()
                .context("Pi extension checksum missing")?;
            let archive = stage.join(format!("{}-{}.tar.gz", part.part, plan.latest.tag));
            let bytes = reqwest::get(url).await?.error_for_status()?.bytes().await?;
            if format!("{:x}", Sha256::digest(&bytes)) != expected {
                anyhow::bail!("Pi extension staged checksum mismatch");
            }
            std::fs::write(&archive, &bytes)?;
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&archive)
                .context("open staged Pi extension archive for durable flush")?
                .sync_all()
                .context("durably flush staged Pi extension archive")?;
            let package_json = PathBuf::from(
                part.target_path
                    .as_deref()
                    .context("Pi extension package path missing")?,
            );
            let extension_root = package_json
                .parent()
                .and_then(Path::parent)
                .context("Pi extension destination root missing")?;
            let installed = crate::commands::install::InstalledAsset {
                name: "focusa-pi-extension".into(),
                version: plan.latest.version.clone(),
                triple: "all".into(),
                sha256: expected.to_string(),
                install_path: archive.display().to_string(),
            };
            std::fs::write(&progress, format!("package:{}:activate", part.part))?;
            crate::commands::install::integrate_pi_extension(
                &installed,
                &stage,
                Some(extension_root),
                None,
            )
            .with_context(|| {
                format!(
                    "activate verified Pi extension package in {}",
                    extension_root.display()
                )
            })?;
            std::fs::write(
                state.join("pi-extension-silent-restart-required.json"),
                serde_json::to_vec_pretty(&json!({
                    "schema": "focusa.pi_extension_restart_required.v1",
                    "version": plan.latest.version,
                    "installed_at": chrono_like_timestamp(),
                    "action": "Focusa Pi extension activates through a non-conversational safe-idle runtime reload when supported, otherwise on the next natural Pi process start"
                }))?,
            )?;
            package_promoted.push(part.part.to_string());
        }
        Ok::<(), anyhow::Error>(())
    }
    .await;
    if let Err(error) = operation {
        let failed_phase = std::fs::read_to_string(&progress)
            .unwrap_or_else(|_| "unknown_transaction_phase".into());
        let error = error.context(format!("update transaction phase {failed_phase}"));
        let rollback_result = rollback_promoted_parts(&promoted);
        let daemon_was_touched = promoted.iter().any(|(part, _, _, _)| part == "daemon")
            || (cfg!(target_os = "windows") && plan.parts.iter().any(|part| part.part == "daemon"));
        let daemon_restore_result: anyhow::Result<()> =
            if daemon_restore_action(daemon_was_touched, daemon_was_running)
                == DaemonRestoreAction::Start
            {
                async {
                    let daemon_path = plan
                        .parts
                        .iter()
                        .find(|part| part.part == "daemon")
                        .and_then(|part| part.target_path.as_deref())
                        .map(Path::new)
                        .context("restore pre-update daemon: target path unavailable")?;
                    restart_daemon_service(daemon_path)?;
                    let mut healthy = false;
                    for _ in 0..20 {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        if probe_daemon_health(&daemon_health_url).await.is_some() {
                            healthy = true;
                            break;
                        }
                    }
                    if healthy {
                        Ok(())
                    } else {
                        anyhow::bail!("restore pre-update daemon: health check did not recover")
                    }
                }
                .await
            } else if daemon_restore_action(daemon_was_touched, daemon_was_running)
                == DaemonRestoreAction::Stop
            {
                async {
                    stop_daemon_service()?;
                    for _ in 0..20 {
                        if probe_daemon_health(&daemon_health_url).await.is_none() {
                            return Ok(());
                        }
                        tokio::time::sleep(Duration::from_millis(250)).await;
                    }
                    anyhow::bail!("restore pre-update daemon: daemon remained running")
                }
                .await
            } else {
                Ok(())
            };
        std::fs::write(
            &journal,
            serde_json::to_vec_pretty(&json!({
                "schema":"focusa.update_journal.v1",
                "state":"rolled_back",
                "error":error.to_string(),
                "daemon_was_running":daemon_was_running,
                "daemon_restore":"pre_update_state",
                "daemon_restore_ok":daemon_restore_result.is_ok()
            }))?,
        )?;
        if let Err(rollback_error) = rollback_result {
            return Err(anyhow::anyhow!(
                "update failed: {error}; rollback also failed: {rollback_error}"
            ));
        }
        if let Err(restore_error) = daemon_restore_result {
            return Err(anyhow::anyhow!(
                "update failed: {error}; files rolled back but daemon state restoration failed: {restore_error}"
            ));
        }
        return Err(error);
    }
    let mut names = promoted
        .iter()
        .map(|(part, _, _, _)| part.clone())
        .collect::<Vec<_>>();
    names.extend(package_promoted);
    let rollback_manifest = backup_root.join("rollback-manifest.json");
    let manifest_entries = promoted
        .iter()
        .filter(|(_, _, backup, digest)| backup.exists() && !digest.is_empty())
        .map(|(part, target, backup, digest)| {
            json!({"part":part,"target":target,"backup":backup,"sha256":digest})
        })
        .collect::<Vec<_>>();
    std::fs::write(
        &rollback_manifest,
        serde_json::to_vec_pretty(&json!({
            "schema":"focusa.update_rollback_manifest.v1",
            "tag":plan.latest.tag,
            "entries":manifest_entries,
        }))?,
    )?;
    if names.is_empty()
        && plan.parts.iter().any(|part| {
            matches!(
                part.action,
                "would_update" | "would_update_package" | "would_install_package"
            )
        })
    {
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

fn refresh_apply_summary(apply: &mut UpdateApplyEnvelope) {
    apply.applied = apply.apply_executed;
    apply.blockers = apply.blocked_reason.clone();
    apply.error = if apply.status == "blocked_read_only" && !apply.blockers.is_empty() {
        Some(format!("update_blocked:{}", apply.blockers.join(",")))
    } else {
        apply
            .blocked_reason
            .iter()
            .find(|reason| reason.starts_with("apply_failed:"))
            .cloned()
    };
    apply.rollback = serde_json::json!({
        "performed": apply.status == "failed_rolled_back",
        "available": true,
        "journal": "focusa.update.apply.journal.v1",
        "preserves": apply.data_safety.preserve,
    });
    apply.next_action = match apply.status {
        "completed" => "Run focusa update status --json and verify every installed surface.".to_string(),
        "already_current" => "No action required; all update-managed surfaces are current.".to_string(),
        "failed_rolled_back" => "Inspect error and rollback journal; repair the release or environment before retrying.".to_string(),
        _ if apply.blockers.iter().any(|reason| {
            ["release", "manifest", "checksum", "signature", "deploy_proof"]
                .iter()
                .any(|needle| reason.contains(needle))
        }) =>
            "Release producer must publish a signed deploy-success proof and pass OTA trust verification; do not bypass trust.".to_string(),
        _ => apply.recovery_hint.clone(),
    };
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
    let installed = serde_json::Value::Object(
        plan.parts
            .iter()
            .map(|part| {
                (
                    part.part.to_string(),
                    part.current_version
                        .clone()
                        .map_or(serde_json::Value::Null, serde_json::Value::String),
                )
            })
            .collect(),
    );
    let latest = plan.latest.version.clone();
    let surfaces = plan
        .parts
        .iter()
        .map(|part| part.part.to_string())
        .collect();
    let blockers = blocked_reason.clone();
    let next_action = if blockers
        .iter()
        .any(|reason| reason.contains("release_trust"))
    {
        "Release producer must publish a signed deploy-success proof and pass OTA trust verification; do not bypass trust.".to_string()
    } else {
        "Satisfy blockers, then rerun with --yes --allow-apply --dry-run false.".to_string()
    };
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
        execution_order: vec![
            "verify_exact_release_and_every_manifest_asset",
            "focusa_install_full_transaction",
            "cli_tui_session_runner_daemon_manifest_agent_context_pi",
            "daemon_restart_health_and_callgraph_acceptance",
            "rollback_entire_release_on_any_failure",
            "pi_extension_runtime_auto_reload",
            "menubar_signed_updater_auto_install_and_relaunch",
        ],
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
            "tui_version_matches_target_for_manifest_bound_release",
            "session_runner_version_matches_target_for_manifest_bound_release",
            "daemon_health_version_matches_target_when_daemon_changed",
            "daemon_api_contract_matches_target_when_daemon_changed",
            "distribution_manifest_matches_signed_release",
            "agent_context_matches_distribution_manifest",
            "installed_callgraph_acceptance_passes",
            "installer_version_matches_target_or_not_installed",
            "pi_extension_activation_receipt_matches_target_for_manifest_bound_release",
            "menubar_signed_updater_install_and_relaunch_or_not_installed",
            "no_data_env_license_overwrite",
            "rollback_journal_written",
        ],
        recovery_hint: "No update was applied. Use focusa update plan --json to inspect and resolve the reported trust or safety blockers before retrying.".into(),
        blocked_reason,
        installed,
        latest,
        applied: false,
        surfaces,
        rollback: serde_json::json!({"performed":false,"available":true}),
        next_action,
        blockers,
        error: None,
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
                "distribution_manifest_full_tree_contract",
                "agent_context_and_pi_archive_contracts",
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
                "promoting_manifest_bound_full_release",
                "systemd_health_and_callgraph_acceptance",
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
            "signed_authority_leases",
            "focusa.sqlite",
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

fn path_is_git_managed(path: &str) -> bool {
    let candidate = Path::new(path);
    let cwd = if candidate.is_dir() {
        candidate
    } else {
        candidate.parent().unwrap_or(candidate)
    };
    let Some(root) = std::process::Command::new("git")
        .args([
            "-C",
            cwd.to_string_lossy().as_ref(),
            "rev-parse",
            "--show-toplevel",
        ])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|root| PathBuf::from(root.trim()))
    else {
        return false;
    };
    let candidate = candidate
        .canonicalize()
        .unwrap_or_else(|_| candidate.to_path_buf());
    let Ok(relative) = candidate.strip_prefix(&root) else {
        return false;
    };
    let Ok(output) = std::process::Command::new("git")
        .args(["-C", root.to_string_lossy().as_ref(), "ls-files", "--"])
        .arg(relative)
        .output()
    else {
        return false;
    };
    output.status.success() && !output.stdout.is_empty()
}

fn part_plan(part: &InstalledPart, latest: &LatestVersion, order: &mut u8) -> PartPlan {
    let release_asset_available = latest.assets.iter().any(|asset| asset.part == part.part);
    let externally_managed = part.part == "pi_extension"
        && path_is_git_managed(part.resolved_path.as_deref().unwrap_or(&part.expected_path));
    let action = if part.part == "installer" && !release_asset_available {
        "release_asset_unavailable"
    } else if part.part == "menubar" {
        if !part.exists {
            "not_installed"
        } else {
            match part.stale {
                Some(true) => "delegated_auto_update",
                Some(false) => "no_op",
                None => "probe_required",
            }
        }
    } else if externally_managed {
        if !part.exists {
            "not_installed"
        } else {
            match part.stale {
                Some(true) => "notify_update",
                Some(false) => "no_op",
                None => "probe_required",
            }
        }
    } else if part.part == "pi_extension" {
        if !part.exists {
            "would_install_package"
        } else {
            match part.stale {
                Some(true) => "would_update_package",
                Some(false) => "no_op",
                None => "probe_required",
            }
        }
    } else if !part.exists {
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
    println!("apply_allowed: {}", plan.apply_allowed);
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
    let explicit_version = override_value
        .filter(|value| !value.trim().is_empty())
        .map(normalize_version)
        .or_else(|| {
            ["FOCUSA_LATEST_VERSION", "FOCUSA_UPDATE_LATEST_TAG"]
                .into_iter()
                .find_map(|key| {
                    std::env::var(key)
                        .ok()
                        .filter(|value| !value.trim().is_empty())
                        .map(|value| normalize_version(&value))
                })
        });
    let admin = read_update_admin_state().unwrap_or_default();
    let (pinned, skipped) = if let Some(explicit) = explicit_version.clone() {
        (Some(explicit), Vec::new())
    } else if admin.trusted_dev_force_latest {
        (None, Vec::new())
    } else {
        (admin.pinned_version, admin.skipped_versions)
    };
    match resolve_latest_github(channel, pinned.as_deref(), &skipped).await {
        Ok(latest) => latest,
        Err(error) => {
            let fallback_version =
                explicit_version.unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
            let mut latest =
                unresolved_latest(fallback_version, "github_release_resolution_failed");
            latest
                .trust
                .blockers
                .push(format!("github_release_resolver_failed:{error}"));
            latest
        }
    }
}

fn unresolved_latest(version: String, source: &str) -> LatestVersion {
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
        eligibility_status: "unresolved_fail_closed",
        trust: ReleaseTrustSummary {
            release_resolved: false,
            complete_asset_set: false,
            sha256sums_present: false,
            checksums_resolved: false,
            signature_verified: false,
            manifest_resolved: false,
            manifest_signature_verified: false,
            provenance_verified: false,
            deploy_proof_verified: false,
            trusted_key_id: None,
            trusted_key_fingerprint: None,
            key_revoked: false,
            ci_proof_required: true,
            signature_required: true,
            blockers: vec!["live_release_not_resolved".into()],
        },
        assets: Vec::new(),
    }
}

async fn resolve_latest_github(
    channel: &str,
    pinned_version: Option<&str>,
    skipped_versions: &[String],
) -> anyhow::Result<LatestVersion> {
    let repo = github_repo();
    let triple = target_triple();
    let client = reqwest::Client::new();
    let token = std::env::var("GITHUB_TOKEN")
        .ok()
        .or_else(|| std::env::var("GH_TOKEN").ok())
        .filter(|value| !value.trim().is_empty());
    let request = |url: String| {
        let request = client
            .get(url)
            .header("User-Agent", "focusa-update-resolver");
        if let Some(token) = token.as_deref() {
            request.bearer_auth(token)
        } else {
            request
        }
    };
    let releases = if let Some(pinned) = pinned_version {
        let tag = release_tag_for_version(pinned);
        let url = format!("https://api.github.com/repos/{repo}/releases/tags/{tag}");
        vec![
            request(url)
                .send()
                .await?
                .error_for_status()?
                .json::<GithubRelease>()
                .await?,
        ]
    } else {
        let url = format!("https://api.github.com/repos/{repo}/releases?per_page=20");
        request(url)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<GithubRelease>>()
            .await?
    };
    for release in releases {
        let normalized_tag = normalize_version(&release.tag_name);
        if release.draft
            || !(release_tag_matches_channel(&release.tag_name, channel)
                || (channel != "stable"
                    && release_tag_matches_channel(&release.tag_name, "stable")))
            || pinned_version.is_some_and(|pinned| normalize_version(pinned) != normalized_tag)
            || skipped_versions.contains(&normalized_tag)
        {
            continue;
        }
        if let Some(latest) = build_latest_from_release(repo.clone(), triple.clone(), release) {
            return Ok(latest);
        }
    }
    anyhow::bail!("no complete release found for channel={channel} target={triple}")
}

fn release_tag_for_version(version: &str) -> String {
    let normalized = normalize_version(version);
    format!("v{normalized}")
}

fn release_binary_asset_name(prefix: &str, tag: &str, triple: &str) -> String {
    let suffix = if triple.ends_with("-pc-windows-msvc") {
        ".exe"
    } else {
        ""
    };
    format!("{prefix}-{tag}-{triple}{suffix}")
}

fn build_latest_from_release(
    repo: String,
    triple: String,
    release: GithubRelease,
) -> Option<LatestVersion> {
    let tag = release.tag_name.clone();
    let mut assets = Vec::new();
    let requires_manifest = crate::commands::install::release_requires_distribution_manifest(&tag);
    let mut rust_surfaces = vec![
        ("cli", "focusa"),
        ("daemon", "focusa-daemon"),
        ("tui", "focusa-tui"),
    ];
    if requires_manifest {
        rust_surfaces.push(("session_runner", "focusa-session-runner"));
    }
    for (part, prefix) in rust_surfaces {
        let name = release_binary_asset_name(prefix, &tag, &triple);
        let gh_asset = release.assets.iter().find(|asset| asset.name == name)?;
        assets.push(ReleaseAssetRef {
            part,
            name,
            download_url: gh_asset.browser_download_url.clone(),
            sha256: None,
        });
    }
    let pi_extension_name = format!("focusa-pi-extension-{tag}.tar.gz");
    let pi_extension = release
        .assets
        .iter()
        .find(|asset| asset.name == pi_extension_name)?;
    assets.push(ReleaseAssetRef {
        part: "pi_extension",
        name: pi_extension_name,
        download_url: pi_extension.browser_download_url.clone(),
        sha256: None,
    });
    if requires_manifest {
        for (part, name) in [
            (
                "distribution_manifest",
                "distribution-manifest.json".to_string(),
            ),
            (
                "agent_context",
                format!("focusa-agent-context-{tag}.tar.gz"),
            ),
        ] {
            let asset = release.assets.iter().find(|asset| asset.name == name)?;
            assets.push(ReleaseAssetRef {
                part,
                name,
                download_url: asset.browser_download_url.clone(),
                sha256: None,
            });
        }
    }
    let installer_name = format!("focusa-installer-{tag}.sh");
    if let Some(installer) = release
        .assets
        .iter()
        .find(|asset| asset.name == installer_name)
    {
        assets.push(ReleaseAssetRef {
            part: "installer",
            name: installer_name,
            download_url: installer.browser_download_url.clone(),
            sha256: None,
        });
    }
    let sha256sums_present = release
        .assets
        .iter()
        .any(|asset| asset.name == "SHA256SUMS.txt");
    let mut blockers = Vec::new();
    let trust_result = update_trust::verify_release_metadata(&release, &mut assets);
    let checksums_resolved =
        trust_result.is_ok() && assets.iter().all(|asset| asset.sha256.is_some());
    let signature_verified = trust_result.is_ok();
    let key_revoked = trust_result
        .as_ref()
        .err()
        .is_some_and(|error| error.to_string().contains("revoked"));
    if let Err(error) = &trust_result {
        blockers.push(format!("release_trust_verification_failed:{error}"));
        blockers.push("release_signature_not_verified".into());
    }
    let (
        manifest_signature_verified,
        provenance_verified,
        deploy_proof_verified,
        trusted_key_id,
        trusted_key_fingerprint,
    ) = match trust_result {
        Ok(verified) => (
            verified.manifest_signature_verified,
            verified.provenance_verified,
            verified.deploy_proof_verified,
            Some(verified.trusted_key_id),
            Some(verified.trusted_key_fingerprint),
        ),
        Err(_) => (false, false, false, None, None),
    };
    if !manifest_signature_verified {
        blockers.push("release_manifest_signature_not_verified".into());
    }
    if !provenance_verified {
        blockers.push("release_provenance_not_verified".into());
    }
    if !deploy_proof_verified {
        blockers.push("release_deploy_proof_not_verified".into());
    }
    Some(LatestVersion {
        version: normalize_version(&tag),
        tag,
        source: "github_releases".into(),
        github_repo: repo,
        target_triple: triple,
        release_manifest_required: true,
        eligibility_status: if checksums_resolved
            && signature_verified
            && manifest_signature_verified
            && provenance_verified
            && deploy_proof_verified
        {
            "eligible_signed_manifest"
        } else {
            "blocked_untrusted_release"
        },
        trust: ReleaseTrustSummary {
            release_resolved: true,
            complete_asset_set: true,
            sha256sums_present,
            checksums_resolved,
            signature_verified,
            manifest_resolved: manifest_signature_verified,
            manifest_signature_verified,
            provenance_verified,
            deploy_proof_verified,
            trusted_key_id,
            trusted_key_fingerprint,
            key_revoked,
            ci_proof_required: true,
            signature_required: true,
            blockers,
        },
        assets,
    })
}

fn release_tag_matches_channel(tag: &str, channel: &str) -> bool {
    match channel {
        "stable" => tag.strip_prefix('v').is_some_and(|version| {
            let parts = version.split('.').collect::<Vec<_>>();
            parts.len() == 3
                && parts
                    .iter()
                    .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
        }),
        "dev" => tag.starts_with('v') && tag.ends_with("-dev"),
        "preview" => tag.starts_with('v') && tag.contains("-rc."),
        "nightly" => tag.starts_with('v') && tag.contains("-nightly."),
        _ => false,
    }
}

#[cfg(test)]
mod release_channel_tests {
    use super::release_tag_matches_channel;

    #[test]
    fn stable_channel_accepts_only_unsuffixed_semver_tags() {
        assert!(release_tag_matches_channel("v0.9.139", "stable"));
        assert!(!release_tag_matches_channel("v0.9.139-dev", "stable"));
        assert!(!release_tag_matches_channel("v0.9.139-rc.1", "stable"));
        assert!(!release_tag_matches_channel("0.9.139", "stable"));
        assert!(!release_tag_matches_channel("v0.9", "stable"));
    }

    #[test]
    fn prerelease_channels_remain_disjoint() {
        assert!(release_tag_matches_channel("v0.9.139-dev", "dev"));
        assert!(release_tag_matches_channel("v0.9.139-rc.1", "preview"));
        assert!(release_tag_matches_channel(
            "v0.9.139-nightly.42",
            "nightly"
        ));
        assert!(!release_tag_matches_channel("v0.9.139", "dev"));
        assert!(!release_tag_matches_channel("v0.9.139-dev", "preview"));
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn fetch_sha256sums_blocking(url: &str) -> anyhow::Result<String> {
    let output = std::process::Command::new("curl")
        .args(["-fsSL", "--max-time", "20", url])
        .output()?;
    if !output.status.success() {
        anyhow::bail!("curl exited {}", output.status.code().unwrap_or(-1));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[allow(dead_code)]
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
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu".into(),
        ("macos", "x86_64") => "x86_64-apple-darwin".into(),
        ("macos", "aarch64") => "aarch64-apple-darwin".into(),
        ("windows", "x86_64") => "x86_64-pc-windows-msvc".into(),
        ("windows", "aarch64") => "aarch64-pc-windows-msvc".into(),
        _ => format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
    }
}

fn update_policy_summary() -> UpdatePolicySummary {
    let path = update_policy_path();
    let exists = path.exists();
    let mut policy = read_update_policy().unwrap_or_else(|_| default_policy_from_license());
    refresh_update_policy_authority(&mut policy);
    UpdatePolicySummary {
        path: path.display().to_string(),
        exists,
        enabled: policy.enabled,
        channel: policy.channel.label().to_string(),
        mode: policy.mode.label().to_string(),
        auto_apply_allowed: policy.auto_apply_allowed,
        auto_apply_blocked_until: policy.auto_apply_blocked_until,
        note: if exists {
            "policy file loaded; apply still requires release trust, lock, rollback, and health gates"
                .into()
        } else {
            "license-derived default policy; no policy file exists yet".into()
        },
    }
}

fn license_summary() -> LicenseSummary {
    match load_license_status() {
        Ok(status) => {
            let policy_dev_override = read_update_policy()
                .map(|policy| policy.dev_mode_override)
                .unwrap_or(false);
            let dev_mode = policy_dev_override
                || status.tier == "dev_mode"
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
                note: "policy defaults are derived from license; automatic apply still requires all signed release and safety gates",
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

/// Default update channel: the policy file's channel when present, else the
/// license-derived default. Replaces the historical hardcoded "dev" default
/// that made status/check disagree with the configured policy channel.
fn effective_channel() -> String {
    match read_update_policy() {
        Ok(policy) => policy.channel.label().to_string(),
        Err(_) => default_policy_from_license().channel.label().to_string(),
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

fn refresh_update_policy_authority(policy: &mut UpdatePolicy) {
    let dev_override = policy.dev_mode_override
        || std::env::var("FOCUSA_DEV_MODE")
            .map(|value| matches!(value.as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);
    match load_license_status() {
        Ok(status) => {
            policy.license_level = if dev_override {
                "dev_mode".into()
            } else {
                status.tier
            };
            policy.refresh_auto_apply_authority(&status.features, dev_override);
        }
        Err(_) => {
            policy.license_level = if dev_override {
                "dev_mode".into()
            } else {
                "evaluation".into()
            };
            policy.refresh_auto_apply_authority(&[], dev_override);
        }
    }
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
            let mut policy = read_update_policy().unwrap_or_else(|_| default_policy_from_license());
            refresh_update_policy_authority(&mut policy);
            let out = serde_json::json!({
                "schema": "focusa.update_policy_status.v1",
                "status": "completed",
                "path": path,
                "exists": exists,
                "policy": policy,
                "mutations_performed": false,
                "auto_apply_allowed": policy.auto_apply_allowed,
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
                println!("auto_apply_allowed: {}", policy.auto_apply_allowed);
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
            if let Some(dev_mode) = args.dev_mode {
                policy.dev_mode_override = dev_mode;
                if dev_mode {
                    policy.channel = ReleaseChannel::Dev;
                    policy.mode = UpdateMode::Automatic;
                    policy.parts = focusa_core::update::UpdatePolicyParts::all_surfaces(true);
                    policy.maintenance_window = "always".into();
                }
            }
            if let Some(enabled) = args.all_surfaces {
                policy.parts = focusa_core::update::UpdatePolicyParts::all_surfaces(enabled);
            }
            refresh_update_policy_authority(&mut policy);
            let path = write_update_policy(&policy)?;
            let out = serde_json::json!({
                "schema": "focusa.update_policy_write.v1",
                "status": "completed",
                "path": path,
                "policy": policy,
                "mutations_performed": true,
                "mutation_scope": "update_policy_file_only",
                "auto_apply_allowed": policy.auto_apply_allowed,
                "next_action": "focusa update status --json"
            });
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!(
                    "updated policy: {}",
                    out["path"].as_str().unwrap_or("unknown")
                );
                println!("auto_apply_allowed: {}", policy.auto_apply_allowed);
            }
        }
    }
    Ok(())
}

fn configured_package_json(env_var: &str, repo_relative: &str) -> PathBuf {
    if let Some(path) = std::env::var_os(env_var) {
        return PathBuf::from(path);
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(repo_relative)
}

fn pi_extension_package_from_settings(settings_path: &Path) -> Option<PathBuf> {
    let settings = std::fs::read(settings_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())?;
    settings
        .get("extensions")?
        .as_array()?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .map(PathBuf::from)
        .map(|path| {
            if path.file_name().and_then(|name| name.to_str()) == Some("package.json") {
                path
            } else {
                path.join("package.json")
            }
        })
        .find(|package| {
            package.is_file()
                && std::fs::read(package)
                    .ok()
                    .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
                    .and_then(|value| value.get("name")?.as_str().map(str::to_string))
                    .map(|name| name.starts_with("focusa-"))
                    .unwrap_or(false)
        })
}

fn pi_extension_package_from_agent_dir(agent_dir: &Path) -> Option<PathBuf> {
    let settings = agent_dir.join("settings.json");
    if let Some(package) = pi_extension_package_from_settings(&settings) {
        return Some(package);
    }
    ["focusa", "focusa-runtime", "focusa-pi-bridge"]
        .iter()
        .map(|name| agent_dir.join("extensions").join(name).join("package.json"))
        .find(|package| {
            package.is_file()
                && std::fs::read(package)
                    .ok()
                    .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
                    .and_then(|value| value.get("name")?.as_str().map(str::to_string))
                    .map(|name| name.starts_with("focusa-"))
                    .unwrap_or(false)
        })
}

fn configured_pi_extension_package_json() -> PathBuf {
    if let Some(path) = std::env::var_os("FOCUSA_PI_EXTENSION_PACKAGE_JSON") {
        return PathBuf::from(path);
    }
    let agent_dir = std::env::var_os("PI_CODING_AGENT_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".pi/agent")));
    if let Some(package) = agent_dir
        .as_deref()
        .and_then(pi_extension_package_from_agent_dir)
    {
        return package;
    }
    agent_dir
        .unwrap_or_else(|| PathBuf::from(".pi/agent"))
        .join("extensions/focusa/package.json")
}

fn inspect_package_part(
    part: &'static str,
    package_json: PathBuf,
    latest: &str,
    notes: Vec<String>,
) -> InstalledPart {
    let expected_path = package_json.display().to_string();
    if !package_json.is_file() {
        return InstalledPart {
            part,
            expected_path,
            resolved_path: None,
            exists: false,
            version: None,
            version_source: "package_json",
            version_probe_safe: true,
            sha256: None,
            stale: None,
            stale_reason: format!("{part} is not installed or discoverable on this host"),
            notes,
        };
    }

    let parsed = std::fs::read(&package_json)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
    let version = parsed
        .as_ref()
        .and_then(|value| value.get("version"))
        .and_then(serde_json::Value::as_str)
        .map(normalize_version);
    let sha256 = sha256_file(&package_json).ok();
    let stale = version
        .as_deref()
        .map(|installed| version_is_stale(installed, latest));
    let stale_reason = match (&version, stale) {
        (Some(installed), Some(true)) => {
            format!("installed {part} version {installed} is behind latest {latest}")
        }
        (Some(installed), Some(false)) => format!(
            "installed {part} version {installed} {} latest {latest}",
            version_relation(installed, latest)
        ),
        _ => format!("{part} package.json does not expose a valid version"),
    };

    InstalledPart {
        part,
        expected_path,
        resolved_path: Some(package_json.display().to_string()),
        exists: true,
        version,
        version_source: "package_json",
        version_probe_safe: true,
        sha256,
        stale,
        stale_reason,
        notes,
    }
}

fn inspect_pi_extension(latest: &str) -> InstalledPart {
    inspect_package_part(
        "pi_extension",
        configured_pi_extension_package_json(),
        latest,
        vec![
            "Pi extension updates remain package-channel managed and are never binary-promoted by focusa update apply".to_string(),
        ],
    )
}

fn inspect_installer(latest: &str) -> InstalledPart {
    let expected = std::env::var_os("FOCUSA_INSTALLER_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/local/lib/focusa/install-focusa.sh"));
    let exists = expected.is_file();
    let version = exists
        .then(|| {
            std::process::Command::new(&expected)
                .arg("--version")
                .output()
                .ok()
        })
        .flatten()
        .filter(|output| output.status.success())
        .map(|output| normalize_version(&String::from_utf8_lossy(&output.stdout)))
        .filter(|version| !version.is_empty());
    let stale = version
        .as_deref()
        .map(|current| version_is_stale(current, latest));
    InstalledPart {
        part: "installer",
        expected_path: expected.display().to_string(),
        resolved_path: exists.then(|| expected.display().to_string()),
        exists,
        version,
        version_source: "safe_--version",
        version_probe_safe: true,
        sha256: exists.then(|| sha256_file(&expected).ok()).flatten(),
        stale,
        stale_reason: "public installer follows separately signed installer release proof".into(),
        notes: vec![
            "verified installer release assets are atomically promoted by focusa update apply"
                .into(),
        ],
    }
}

fn inspect_menubar(latest: &str) -> InstalledPart {
    inspect_package_part(
        "menubar",
        configured_package_json("FOCUSA_MENUBAR_PACKAGE_JSON", "apps/menubar/package.json"),
        latest,
        vec![
            "menubar delegates to its signed updater, which installs and relaunches automatically under the shared Focusa OTA policy"
                .to_string(),
        ],
    )
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
        .map(|version| version_is_stale(version, latest))
        .or_else(|| sha256.as_ref().map(|_| true));
    let stale_reason = match (&version, stale, &path) {
        (_, _, None) => "tui binary not found".into(),
        (Some(version), Some(true), _) => {
            format!("installed tui version {version} is behind latest {latest}")
        }
        (Some(version), Some(false), _) => format!(
            "installed tui version {version} {} latest {latest}",
            version_relation(version, latest)
        ),
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

async fn inspect_session_runner(latest: &str) -> anyhow::Result<InstalledPart> {
    let path = resolve_path(
        "focusa-session-runner",
        "/usr/local/bin/focusa-session-runner",
    );
    inspect_executable_part(
        "session_runner",
        "/usr/local/bin/focusa-session-runner",
        path,
        latest,
        true,
    )
    .await
}

fn inspect_manifest_part(
    part: &'static str,
    expected: PathBuf,
    latest: &str,
    notes: Vec<String>,
) -> InstalledPart {
    let exists = expected.is_file();
    let version = std::fs::read(&expected)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|value| {
            value
                .get("release_version")
                .and_then(serde_json::Value::as_str)
                .map(normalize_version)
        });
    let stale = version
        .as_deref()
        .map(|installed| version_is_stale(installed, latest));
    InstalledPart {
        part,
        expected_path: expected.display().to_string(),
        resolved_path: exists.then(|| expected.display().to_string()),
        exists,
        version,
        version_source: "distribution_manifest_release_version",
        version_probe_safe: true,
        sha256: exists.then(|| sha256_file(&expected).ok()).flatten(),
        stale,
        stale_reason: if exists {
            format!("{part} must match the complete signed distribution")
        } else {
            format!("{part} is not installed")
        },
        notes,
    }
}

fn inspect_distribution_manifest(latest: &str) -> InstalledPart {
    let expected = std::env::var_os("FOCUSA_DISTRIBUTION_MANIFEST")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if resolve_path("focusa", "/usr/local/bin/focusa").as_deref()
                == Some("/usr/local/bin/focusa")
            {
                PathBuf::from("/usr/local/lib/focusa/distribution-manifest.json")
            } else {
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".focusa/distribution-manifest.json")
            }
        });
    inspect_manifest_part(
        "distribution_manifest",
        expected,
        latest,
        vec!["promoted only by the canonical manifest-bound install transaction".into()],
    )
}

fn inspect_agent_context(latest: &str) -> InstalledPart {
    let expected = std::env::var_os("FOCUSA_AGENT_CONTEXT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".focusa/agent-context")
        })
        .join("distribution-manifest.json");
    inspect_manifest_part(
        "agent_context",
        expected,
        latest,
        vec!["skills, current docs, and generated clients move with the signed release".into()],
    )
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
    let stale = version.as_ref().map(|v| version_is_stale(v, latest));
    let stale_reason = match (&version, stale) {
        (Some(v), Some(true)) => {
            format!("running daemon health version {v} is behind latest {latest}")
        }
        (Some(v), Some(false)) => format!(
            "running daemon health version {v} {} latest {latest}",
            version_relation(v, latest)
        ),
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
    let stale = version.as_ref().map(|v| version_is_stale(v, latest));
    let stale_reason = match (&version, stale, &path) {
        (_, _, None) => format!("{part} binary not found"),
        (Some(v), Some(true), _) => {
            format!("installed {part} version {v} is behind latest {latest}")
        }
        (Some(v), Some(false), _) => format!(
            "installed {part} version {v} {} latest {latest}",
            version_relation(v, latest)
        ),
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
    // A root-owned system scheduler manages the shared /usr/local surfaces.
    // Never let a private root install shadow globally executable binaries.
    if is_root() && Path::new(canonical).exists() {
        return Some(canonical.into());
    }
    if let Some(home) = std::env::var_os("HOME") {
        let installed_name = if cfg!(target_os = "windows") {
            format!("{command}.exe")
        } else {
            command.to_string()
        };
        let user_install = PathBuf::from(home).join(".focusa/bin").join(installed_name);
        if user_install.exists() {
            return Some(user_install.to_string_lossy().to_string());
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

fn version_parts(raw: &str) -> Option<(u64, u64, u64)> {
    let cleaned = normalize_version(raw);
    let base = cleaned.split(['-', '+']).next().unwrap_or(&cleaned);
    let mut parts = base.split('.');
    let major = parts.next()?.trim().parse().ok()?;
    let minor = parts.next()?.trim().parse().ok()?;
    let patch = parts.next()?.trim().parse().ok()?;
    Some((major, minor, patch))
}

/// Numeric ordering; when either side cannot be parsed, fall back to the
/// historical string-inequality semantics (any difference is stale).
fn version_is_stale(installed: &str, latest: &str) -> bool {
    match (version_parts(installed), version_parts(latest)) {
        (Some(installed), Some(latest)) => installed < latest,
        _ => normalize_version(installed) != normalize_version(latest),
    }
}

/// "is ahead of" when installed is numerically newer, else "matches".
fn version_relation(installed: &str, latest: &str) -> &'static str {
    match (version_parts(installed), version_parts(latest)) {
        (Some(installed), Some(latest)) if installed > latest => "is ahead of",
        _ => "matches",
    }
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
    use super::{
        DaemonRestoreAction, PromotedPart, daemon_restore_action, exact_release_install_args,
        inspect_package_part, normalize_version, path_is_git_managed,
        pi_extension_package_from_agent_dir, pi_extension_package_from_settings,
        release_binary_asset_name, release_tag_for_version, rollback_promoted_parts,
    };
    #[cfg(target_os = "macos")]
    use super::{restart_daemon_service, stop_daemon_service};

    #[test]
    fn normalizes_common_version_outputs() {
        assert_eq!(normalize_version("focusa 0.9.74-dev"), "0.9.74-dev");
        assert_eq!(normalize_version("v0.9.80-dev"), "0.9.80-dev");
        assert_eq!(normalize_version("0.9.80-dev"), "0.9.80-dev");
    }

    #[test]
    fn exact_release_version_normalizes_to_tag_endpoint_identity() {
        assert_eq!(release_tag_for_version("0.9.117-dev"), "v0.9.117-dev");
        assert_eq!(release_tag_for_version("v0.9.117-dev"), "v0.9.117-dev");
    }

    #[test]
    fn manifest_bound_update_and_rollback_reuse_exact_install_lifecycle() {
        let stable = exact_release_install_args("v0.9.188", "Startempire-Wire/focusa", true);
        assert_eq!(stable.release_tag_override.as_deref(), Some("v0.9.188"));
        assert_eq!(stable.channel, crate::commands::install::Channel::Stable);
        assert!(stable.system_install && stable.reuse_existing_license);
        assert!(stable.suppress_completion_output);

        let legacy = exact_release_install_args("v0.9.177", "Startempire-Wire/focusa", true);
        assert_eq!(legacy.release_tag_override.as_deref(), Some("v0.9.177"));
    }

    #[test]
    fn release_binary_asset_names_cover_native_windows_targets_once() {
        assert_eq!(
            release_binary_asset_name("focusa", "v0.9.117-dev", "x86_64-pc-windows-msvc"),
            "focusa-v0.9.117-dev-x86_64-pc-windows-msvc.exe"
        );
        assert_eq!(
            release_binary_asset_name("focusa-daemon", "v0.9.117-dev", "aarch64-pc-windows-msvc"),
            "focusa-daemon-v0.9.117-dev-aarch64-pc-windows-msvc.exe"
        );
        assert_eq!(
            release_binary_asset_name("focusa-tui", "v0.9.117-dev", "aarch64-apple-darwin"),
            "focusa-tui-v0.9.117-dev-aarch64-apple-darwin"
        );
        assert_eq!(
            release_binary_asset_name(
                "focusa-session-runner",
                "v0.9.188",
                "aarch64-pc-windows-msvc"
            ),
            "focusa-session-runner-v0.9.188-aarch64-pc-windows-msvc.exe"
        );
    }

    #[test]
    fn pi_package_promotion_refuses_only_tracked_git_content() {
        let root = std::env::temp_dir().join(format!(
            "focusa-pi-update-source-{}-{}",
            std::process::id(),
            super::chrono_like_timestamp()
        ));
        std::fs::create_dir_all(root.join("apps/pi-extension")).expect("create source fixture");
        let status = std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .status()
            .expect("initialize git fixture");
        assert!(status.success());
        let package = root.join("apps/pi-extension/package.json");
        std::fs::write(&package, "{}").expect("write package fixture");
        assert!(
            !path_is_git_managed(package.to_str().unwrap()),
            "untracked install inside a parent repository remains updater-managed"
        );
        let status = std::process::Command::new("git")
            .args(["add", "apps/pi-extension/package.json"])
            .current_dir(&root)
            .status()
            .expect("track source fixture");
        assert!(status.success());
        assert!(path_is_git_managed(package.to_str().unwrap()));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn package_inventory_reports_exact_version_hash_and_staleness() {
        let root = std::env::temp_dir().join(format!(
            "focusa-package-inventory-{}-{}",
            std::process::id(),
            super::chrono_like_timestamp()
        ));
        std::fs::create_dir_all(&root).expect("create package fixture");
        let package = root.join("package.json");
        std::fs::write(
            &package,
            br#"{"name":"focusa-test","version":"0.9.100-dev"}"#,
        )
        .expect("write package fixture");

        let current = inspect_package_part("pi_extension", package.clone(), "0.9.100-dev", vec![]);
        assert!(current.exists);
        assert_eq!(current.version.as_deref(), Some("0.9.100-dev"));
        assert_eq!(current.stale, Some(false));
        assert!(current.sha256.is_some());

        let stale = inspect_package_part("pi_extension", package, "0.9.101-dev", vec![]);
        assert_eq!(stale.stale, Some(true));
        std::fs::remove_dir_all(root).expect("remove package fixture");
    }

    #[test]
    fn pi_settings_resolve_active_focusa_extension_package() {
        let root = std::env::temp_dir().join(format!(
            "focusa-pi-settings-{}-{}",
            std::process::id(),
            super::chrono_like_timestamp()
        ));
        let extension = root.join("extension");
        std::fs::create_dir_all(&extension).expect("create extension fixture");
        std::fs::write(
            extension.join("package.json"),
            br#"{"name":"focusa-pi-bridge","version":"0.9.102-dev"}"#,
        )
        .expect("write extension package");
        let settings = root.join("settings.json");
        std::fs::write(
            &settings,
            format!(r#"{{"extensions":["{}"]}}"#, extension.display()),
        )
        .expect("write Pi settings");

        assert_eq!(
            pi_extension_package_from_settings(&settings),
            Some(extension.join("package.json"))
        );
        std::fs::remove_dir_all(root).expect("remove Pi settings fixture");
    }

    #[test]
    fn pi_agent_dir_falls_back_to_auto_loaded_focusa_runtime_package() {
        let root = std::env::temp_dir().join(format!(
            "focusa-pi-agent-dir-{}-{}",
            std::process::id(),
            super::chrono_like_timestamp()
        ));
        let runtime = root.join("extensions/focusa-runtime");
        std::fs::create_dir_all(&runtime).expect("create runtime fixture");
        std::fs::write(root.join("settings.json"), br#"{"extensions":[]}"#)
            .expect("write settings fixture");
        std::fs::write(
            root.join("extensions/focusa-runtime/package.json"),
            br#"{"name":"focusa-pi-bridge","version":"0.9.143"}"#,
        )
        .expect("write runtime package");

        assert_eq!(
            pi_extension_package_from_agent_dir(&root),
            Some(runtime.join("package.json"))
        );
        std::fs::remove_dir_all(root).expect("remove Pi agent fixture");
    }

    #[test]
    fn absent_external_package_is_unknown_not_stale() {
        let package = std::env::temp_dir().join(format!(
            "focusa-missing-package-{}-{}.json",
            std::process::id(),
            super::chrono_like_timestamp()
        ));
        let part = inspect_package_part("menubar", package, "0.9.100-dev", vec![]);
        assert!(!part.exists);
        assert_eq!(part.stale, None);
        assert!(part.sha256.is_none());
    }

    #[test]
    fn atomic_rollback_restores_previous_binary_and_removes_failed_candidate() {
        let root = std::env::temp_dir().join(format!(
            "focusa-update-rollback-{}-{}",
            std::process::id(),
            super::chrono_like_timestamp()
        ));
        std::fs::create_dir_all(&root).expect("create rollback fixture");
        let target = root.join("focusa");
        let backup = root.join("focusa.backup");
        std::fs::write(&target, b"new-broken").expect("write promoted target");
        std::fs::write(&backup, b"old-known-good").expect("write backup");
        let promoted: Vec<PromotedPart> = vec![(
            "cli".into(),
            target.clone(),
            backup.clone(),
            "old-digest".into(),
        )];
        let restored = rollback_promoted_parts(&promoted).expect("rollback succeeds");
        assert_eq!(restored, vec!["cli"]);
        assert_eq!(
            std::fs::read(&target).expect("read restored"),
            b"old-known-good"
        );
        assert!(!backup.exists());
        assert!(!target.with_extension("focusa-failed").exists());
        std::fs::remove_dir_all(root).expect("remove rollback fixture");
    }

    #[test]
    fn failed_update_restores_exact_pre_transaction_daemon_state() {
        assert_eq!(
            daemon_restore_action(true, true),
            DaemonRestoreAction::Start
        );
        assert_eq!(
            daemon_restore_action(true, false),
            DaemonRestoreAction::Stop
        );
        assert_eq!(
            daemon_restore_action(false, true),
            DaemonRestoreAction::None
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn launchd_stop_restart_round_trip_restores_running_state() {
        let root = std::env::temp_dir().join(format!(
            "focusa-launchd-rollback-{}-{}",
            std::process::id(),
            super::chrono_like_timestamp()
        ));
        std::fs::create_dir_all(&root).expect("create launchd fixture");
        let plist = root.join("com.startempire.focusa-daemon.plist");
        std::fs::write(
            &plist,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>Label</key><string>com.startempire.focusa-daemon</string>
<key>ProgramArguments</key><array><string>/bin/sleep</string><string>300</string></array>
<key>RunAtLoad</key><true/><key>KeepAlive</key><false/>
</dict></plist>
"#,
        )
        .expect("write launchd fixture");
        let uid = String::from_utf8_lossy(
            &std::process::Command::new("id")
                .arg("-u")
                .output()
                .expect("read uid")
                .stdout,
        )
        .trim()
        .to_string();
        let domain = format!("gui/{uid}");
        let target = format!("{domain}/com.startempire.focusa-daemon");
        let _ = std::process::Command::new("launchctl")
            .args(["bootout", &target])
            .status();

        let running = || {
            std::process::Command::new("launchctl")
                .args(["print", &target])
                .output()
                .is_ok_and(|output| {
                    output.status.success()
                        && String::from_utf8_lossy(&output.stdout).contains("state = running")
                })
        };
        let wait_for = |expected: bool| {
            for _ in 0..50 {
                if running() == expected {
                    return true;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            false
        };
        let result = (|| -> anyhow::Result<()> {
            anyhow::ensure!(
                std::process::Command::new("launchctl")
                    .args(["bootstrap", &domain])
                    .arg(&plist)
                    .status()?
                    .success(),
                "bootstrap launchd fixture"
            );
            anyhow::ensure!(wait_for(true), "fixture did not start");
            stop_daemon_service()?;
            anyhow::ensure!(wait_for(false), "fixture did not stop");
            restart_daemon_service(std::path::Path::new("/bin/false"))?;
            anyhow::ensure!(wait_for(true), "fixture did not restart");
            Ok(())
        })();
        let _ = std::process::Command::new("launchctl")
            .args(["bootout", &target])
            .status();
        let _ = std::fs::remove_dir_all(root);
        result.expect("real launchd state round trip");
    }

    #[test]
    fn rollback_new_install_without_backup_removes_promoted_target() {
        let root = std::env::temp_dir().join(format!(
            "focusa-update-no-backup-{}-{}",
            std::process::id(),
            super::chrono_like_timestamp()
        ));
        std::fs::create_dir_all(&root).expect("create rollback fixture");
        let target = root.join("focusa");
        let backup = root.join("missing.backup");
        std::fs::write(&target, b"current").expect("write target");
        let promoted: Vec<PromotedPart> =
            vec![("cli".into(), target.clone(), backup, String::new())];
        let restored = rollback_promoted_parts(&promoted).expect("rollback succeeds");
        assert_eq!(restored, vec!["cli"]);
        assert!(!target.exists());
        std::fs::remove_dir_all(root).expect("remove rollback fixture");
    }
}

#[cfg(test)]
mod version_staleness_tests {
    use super::*;

    #[test]
    fn behind_is_stale_ahead_and_current_are_not() {
        assert!(version_is_stale("0.9.151", "0.9.152"));
        assert!(
            !version_is_stale("0.9.153", "0.9.152"),
            "ahead must not be stale"
        );
        assert!(!version_is_stale("0.9.152", "0.9.152"));
    }

    #[test]
    fn channel_suffixes_do_not_fabricate_staleness() {
        assert!(!version_is_stale("0.9.152", "0.9.152-dev"));
        assert!(!version_is_stale("0.9.152-dev", "0.9.152"));
        assert!(
            !version_is_stale("v0.9.152", "0.9.152"),
            "v prefix is cosmetic"
        );
    }

    #[test]
    fn relation_words_distinguish_ahead_from_match() {
        assert_eq!(version_relation("0.9.153", "0.9.152"), "is ahead of");
        assert_eq!(version_relation("0.9.152", "0.9.152-dev"), "matches");
    }

    #[test]
    fn unparseable_versions_fall_back_to_string_compare() {
        // When either side cannot be parsed, any string difference is stale
        // (historical behavior preserved for unknown version shapes).
        assert!(version_is_stale("current", "0.9.152"));
        assert!(version_is_stale("0.9.152", "current"));
        assert!(!version_is_stale("current", "current"));
    }
}
