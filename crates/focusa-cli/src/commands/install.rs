//! Focusa install — single Rust orchestrator (Spec 112 §15A).
//!
//! Replaces the shell-heavy `scripts/install-focusa.sh` with a Rust subcommand
//! that owns all install behavior:
//!   * signed authority-lease resolution and verified-email device authorization
//!   * four-binary asset download (`focusa`, daemon, TUI, session runner)
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
use focusa_core::license::load_license_status;
use focusa_core::update::{UPDATE_POLICY_SCHEMA_V1, UpdatePolicy};
use focusa_license::authority::{
    EntitlementSnapshot, EntitlementState, LeaseVerificationContext, SignedEnvelope,
};
use focusa_license::authority_client::{
    DeviceAuthorizationSession, DeviceAuthorizationStatus, DeviceCodePollResponse,
    DeviceCodeStartRequest, PollAction,
};
use focusa_license::authority_credentials::{
    CredentialHandle, KeyringCredentialStore, load_or_create_node_identity,
    rotate_refresh_credential,
};
use focusa_license::authority_http::{
    AuthorityEndpointSet, AuthorityHttpClient, AuthorityHttpPolicy, DeviceCodePollRequest,
};
use focusa_license::authority_store::{
    AUTHORITY_STATE_FILE, PersistedAuthorityState, embedded_production_trust_roots,
    resolve_authority_state,
};
use focusa_license::license_migration::{
    LegacyLicenseSourceClass, LicenseMigrationJournalEntry, LicenseMigrationStatus,
    append_license_migration_entry, inventory_legacy_license_files, migration_id_for_source_digest,
};
use focusa_terminal_ui::install::completion::InstallCompletionSummary;
use focusa_terminal_ui::install::event::NullEventSink;
use focusa_terminal_ui::install::presenter::{PlainPresenter, Presenter, presenter_for_mode};
use focusa_terminal_ui::{
    AnimatedPresenterState, AnimatedRenderLoop, CancellationToken, InstallEvent, InstallEventSink,
    InstallPhase, InstallRendererMode, detect_capabilities, install_signal_handlers,
    validate_environment,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::JoinHandle;
use std::time::Duration;

const CANONICAL_RELEASE_BINARIES: [&str; 4] = [
    "focusa",
    "focusa-daemon",
    "focusa-tui",
    "focusa-session-runner",
];

struct UiChannel {
    sender: mpsc::Sender<InstallEvent>,
    failed: AtomicBool,
    warned: AtomicBool,
    fallback: Mutex<PlainPresenter>,
}

impl UiChannel {
    fn fail(&self, message: String) {
        if !self.failed.swap(true, Ordering::AcqRel) && !self.warned.swap(true, Ordering::AcqRel) {
            let warning = InstallEvent::PhaseWarning {
                phase: InstallPhase::InitializeEnvironment,
                message: "animated installer UI unavailable; continuing in plain mode".into(),
                recovery_hint: Some(message),
            };
            if let Ok(mut fallback) = self.fallback.lock() {
                fallback.handle_event(&warning);
            }
        }
    }
}

struct InstallerUi {
    channel: Option<Arc<UiChannel>>,
    presenter: Option<Mutex<Box<dyn Presenter>>>,
    renderer: Option<JoinHandle<()>>,
}

impl InstallerUi {
    fn new(
        mode: InstallRendererMode,
        quiet: bool,
        seed: u64,
        cancellation: &CancellationToken,
    ) -> Self {
        if mode.is_animated() {
            let (sender, receiver) = mpsc::channel();
            let channel = Arc::new(UiChannel {
                sender,
                failed: AtomicBool::new(false),
                warned: AtomicBool::new(false),
                fallback: Mutex::new(PlainPresenter::new(quiet)),
            });
            let render_channel = Arc::clone(&channel);
            let token = cancellation.clone();
            let handle = std::thread::spawn(move || {
                let _selected =
                    presenter_for_mode(mode, Arc::new(AnimatedPresenterState::new()), quiet);
                let result = AnimatedRenderLoop::new(mode, seed).run(receiver, token);
                if let Err(error) = result {
                    render_channel.fail(error.to_string());
                }
            });
            Self {
                channel: Some(channel),
                presenter: None,
                renderer: Some(handle),
            }
        } else if mode == InstallRendererMode::Plain {
            Self {
                channel: None,
                presenter: Some(Mutex::new(presenter_for_mode(
                    mode,
                    Arc::new(AnimatedPresenterState::new()),
                    quiet,
                ))),
                renderer: None,
            }
        } else {
            Self {
                channel: None,
                presenter: None,
                renderer: None,
            }
        }
    }

    fn finish(&mut self) {
        self.channel.take();
        if let Some(handle) = self.renderer.take() {
            let _ = handle.join();
        }
    }
}

impl InstallEventSink for InstallerUi {
    fn emit(&self, event: InstallEvent) {
        if let Some(channel) = &self.channel {
            if channel.failed.load(Ordering::Acquire) {
                if let Ok(mut fallback) = channel.fallback.lock() {
                    fallback.handle_event(&event);
                }
            } else if channel.sender.send(event.clone()).is_err() {
                channel.fail("renderer channel closed".into());
                if let Ok(mut fallback) = channel.fallback.lock() {
                    fallback.handle_event(&event);
                }
            }
        } else if let Some(presenter) = &self.presenter {
            if let Ok(mut presenter) = presenter.lock() {
                presenter.handle_event(&event);
            }
        }
    }
}

impl Drop for InstallerUi {
    fn drop(&mut self) {
        self.finish();
    }
}

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

    /// Disable terminal install animation and use plain output.
    #[arg(long)]
    pub no_animation: bool,

    /// Suppress decorative output.
    #[arg(long)]
    pub quiet: bool,

    /// Install missing bootstrap dependencies during preflight after explicit consent.
    #[arg(long)]
    pub install_dependencies: bool,

    /// Approve dependency installation without an interactive prompt.
    #[arg(long, requires = "install_dependencies")]
    pub assume_yes: bool,

    /// Deprecated raw-key input; installation requires an authority-issued signed lease.
    #[arg(long, value_name = "KEY")]
    pub license_key: Option<String>,

    /// Request verified-email limited activation (Spec 172 verified_no_license):
    /// the authority issues a signed limited-access assertion and no local
    /// Evaluation grant is ever created.
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

    /// Internal upgrade path: reuse an existing active local license record.
    #[arg(skip)]
    pub reuse_existing_license: bool,

    /// Internal delegated-install path: let the caller own the completion envelope.
    #[arg(skip)]
    pub suppress_completion_output: bool,

    /// Internal delegated-install path: bind every downloaded surface to one exact tag.
    #[arg(skip)]
    pub release_tag_override: Option<String>,

    /// Internal upgrade path: preserve an existing authoritative system surface.
    #[arg(skip)]
    pub system_install: bool,

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
    pub distro: String,
    pub os_version: String,
    pub kernel: String,
    pub arch: String,
    pub libc: String,
    pub shell: String,
    pub terminal: String,
    pub package_manager: Option<String>,
    pub service_manager: Option<String>,
    pub privileged: bool,
    pub path_target: String,
    pub path_target_writable: bool,
    pub path_targets: Vec<PathTargetSummary>,
    pub existing_focusa: Option<String>,
    pub existing_surfaces: Vec<ExistingSurface>,
    pub cpu: String,
    pub memory: String,
    pub disk: String,
    pub network: NetworkInventory,
    pub tls: TlsInventory,
    pub proxy: ProxyInventory,
    pub daemon_health: DaemonHealthInventory,
    pub license_override: LicenseOverrideInventory,
    pub update_policy: UpdatePolicyInventory,
    pub compatibility: CompatibilityInventory,
}

#[derive(Debug, Serialize)]
pub struct PathTargetSummary {
    pub path: String,
    pub exists: bool,
    pub writable: bool,
    pub on_path: bool,
    pub focusa_present: bool,
}

#[derive(Debug, Serialize)]
pub struct ExistingSurface {
    pub kind: String,
    pub path: String,
    pub present: bool,
    pub writable: bool,
}

#[derive(Debug, Serialize)]
pub struct NetworkInventory {
    pub default_route: bool,
    pub resolv_conf_present: bool,
    pub nameserver_count: usize,
    pub dns_probe_hint: String,
}

#[derive(Debug, Serialize)]
pub struct TlsInventory {
    pub cert_stores_found: Vec<String>,
    pub cert_store_count: usize,
    pub has_any_store: bool,
}

#[derive(Debug, Serialize)]
pub struct ProxyInventory {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub all_proxy: Option<String>,
    pub no_proxy: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DaemonHealthInventory {
    pub running: bool,
    pub pid: Option<u32>,
    pub lock_file_present: bool,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct LicenseOverrideInventory {
    pub requested_eval: bool,
    pub requested_license_key: bool,
    pub accept_license_requested: bool,
    pub dev_mode_requested: bool,
    pub local_tier: String,
    pub override_active: bool,
    pub effective_mode: String,
}

#[derive(Debug, Serialize)]
pub struct UpdatePolicyInventory {
    pub source: String,
    pub path: String,
    pub exists: bool,
    pub channel: String,
    pub mode: String,
    pub enabled: bool,
    pub auto_apply_allowed: bool,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct CompatibilityInventory {
    pub status: String,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PreflightDependency {
    pub name: String,
    pub present: bool,
    pub install_hint: Option<String>,
    pub install_plan: DependencyInstallPlan,
}

#[derive(Debug, Serialize)]
pub struct DependencyInstallPlan {
    pub manager: String,
    pub package: String,
    pub repository: String,
    pub install_mode: String,
    pub install_command: String,
    pub dry_run_command: String,
    pub privilege_required: bool,
    pub recovery_hint: String,
}

#[derive(Debug, Serialize)]
pub struct DependencyInstallOffer {
    pub can_offer: bool,
    pub auto_install_performed: bool,
    pub requires_explicit_consent: bool,
    pub install_requested: bool,
    pub assume_yes_requested: bool,
    pub consent_status: String,
    pub execution: Option<DependencyInstallExecution>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DependencyInstallExecution {
    pub status: String,
    pub commands: Vec<String>,
    pub installed: Vec<String>,
    pub already_present: Vec<String>,
    pub failures: Vec<String>,
    pub retryable_failures: Vec<String>,
    pub rollback_status: String,
    pub recovery_evidence: Vec<String>,
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
    let mut system = detect_preflight_system(args);
    let dependencies = detect_dependencies(system.package_manager.as_deref());
    let missing_dependencies = dependencies
        .iter()
        .filter(|dep| !dep.present)
        .map(|dep| dep.name.clone())
        .collect::<Vec<_>>();
    let compatibility = classify_compatibility(&system, &missing_dependencies);
    system.compatibility = compatibility;
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
            install_requested: args.install_dependencies,
            assume_yes_requested: args.assume_yes,
            consent_status: if args.install_dependencies {
                "pending".into()
            } else {
                "not_requested".into()
            },
            execution: None,
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

fn detect_preflight_system(args: &InstallArgs) -> PreflightSystem {
    let package_manager = if cfg!(windows) {
        // Windows preflight is deliberately side-effect free: package-manager
        // command probing can recurse through PATHEXT shims on hosted agents.
        None
    } else {
        first_command(&[
            "dnf", "yum", "apt-get", "brew", "pacman", "zypper", "choco", "winget",
        ])
    };
    let service_manager = if cfg!(windows) {
        Some("windows-service".into())
    } else if have_cmd("systemctl") {
        Some("systemd".into())
    } else if have_cmd("launchctl") {
        Some("launchd".into())
    } else {
        None
    };
    let existing_focusa = if cfg!(windows) {
        None
    } else {
        find_command("focusa")
    };
    let path_targets = detect_path_targets(existing_focusa.as_deref());
    let path_target = path_targets
        .first()
        .map(|entry| entry.path.clone())
        .unwrap_or_else(|| "/usr/local/bin".into());
    let path_target_writable = path_targets
        .iter()
        .find(|entry| entry.path == path_target)
        .map(|entry| entry.writable)
        .unwrap_or(false);
    let (distro, os_version) = detect_distro_version();
    let existing_surfaces = detect_existing_surfaces(
        existing_focusa.as_deref(),
        package_manager.as_deref(),
        service_manager.as_deref(),
    );
    PreflightSystem {
        os: std::env::consts::OS.to_string(),
        distro,
        os_version,
        kernel: detect_kernel_version(),
        arch: std::env::consts::ARCH.to_string(),
        libc: detect_libc(),
        shell: std::env::var("SHELL").unwrap_or_else(|_| "unknown".into()),
        terminal: std::env::var("TERM").unwrap_or_else(|_| "unknown".into()),
        package_manager,
        service_manager: service_manager.clone(),
        privileged: is_root(),
        path_target: path_target.clone(),
        path_target_writable,
        path_targets,
        existing_focusa: if cfg!(windows) { None } else { existing_focusa },
        existing_surfaces,
        cpu: detect_cpu_summary(),
        memory: detect_memory_summary(),
        disk: detect_disk_summary("/"),
        network: detect_network_summary(),
        tls: detect_tls_inventory(),
        proxy: detect_proxy_inventory(),
        daemon_health: detect_daemon_health(),
        license_override: detect_license_override(args),
        update_policy: detect_update_policy(),
        compatibility: CompatibilityInventory {
            status: "unknown".into(),
            blockers: Vec::new(),
            warnings: Vec::new(),
        },
    }
}

fn detect_distro_version() -> (String, String) {
    const LSB_OS_RELEASE: &str = "/etc/os-release";
    const ALTERNATE_OS_RELEASE: &str = "/usr/lib/os-release";
    let mut distro = "unknown".to_string();
    let mut version = "unknown".to_string();

    for path in [LSB_OS_RELEASE, ALTERNATE_OS_RELEASE] {
        let content = match std::fs::read_to_string(path) {
            Ok(value) => value,
            Err(_) => continue,
        };
        for line in content.lines() {
            if let Some(value) = line.strip_prefix("ID=") {
                distro = value.trim().trim_matches('"').to_string();
            } else if let Some(value) = line.strip_prefix("VERSION_ID=") {
                version = value.trim().trim_matches('"').to_string();
            } else if let Some(value) = line.strip_prefix("VERSION=")
                && version == "unknown"
            {
                version = value.trim().trim_matches('"').to_string();
            } else if let Some(value) = line.strip_prefix("PRETTY_NAME=")
                && distro == "unknown"
            {
                distro = value.trim().trim_matches('"').to_string();
            }
        }
        if distro != "unknown" || version != "unknown" {
            break;
        }
    }

    if distro == "unknown" {
        distro = "unknown".to_string();
    }
    if version == "unknown" {
        version = "unknown".to_string();
    }
    (distro, version)
}

fn detect_kernel_version() -> String {
    if cfg!(windows) {
        return "unknown".into();
    }
    let output = std::process::Command::new("uname").arg("-r").output().ok();
    output
        .and_then(|out| {
            if !out.status.success() {
                return None;
            }
            String::from_utf8(out.stdout)
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn detect_libc() -> String {
    if cfg!(windows) {
        return "n/a".into();
    }
    std::process::Command::new("getconf")
        .arg("GNU_LIBC_VERSION")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn detect_path_targets(existing_focusa: Option<&str>) -> Vec<PathTargetSummary> {
    let path_candidates = if cfg!(windows) {
        vec![
            std::env::var("PROGRAMFILES")
                .unwrap_or_else(|_| "C:\\Program Files\\Focusa\\bin".into()),
            std::env::var("LOCALAPPDATA").unwrap_or_else(|_| {
                "C:\\Users\\Focusa\\AppData\\Local\\Programs\\Focusa\\bin".into()
            }),
        ]
    } else {
        let mut candidates = vec!["/usr/local/bin".to_string()];
        if let Some(home) = std::env::var_os("HOME") {
            candidates.push(
                std::path::PathBuf::from(home)
                    .join(".local/bin")
                    .display()
                    .to_string(),
            );
        }
        candidates
    };

    path_candidates
        .into_iter()
        .map(|path| {
            let on_path = std::env::var_os("PATH")
                .map(|value| {
                    std::env::split_paths(&value).any(|entry| entry.display().to_string() == path)
                })
                .unwrap_or(false);
            let exists = std::path::Path::new(&path).exists();
            let writable = std::fs::OpenOptions::new().write(true).open(&path).is_ok();
            let focusa_present = existing_focusa
                .map(|focusa| focusa.starts_with(&path))
                .unwrap_or(false);
            PathTargetSummary {
                path,
                exists,
                writable,
                on_path,
                focusa_present,
            }
        })
        .collect()
}

fn detect_existing_surfaces(
    existing_focusa: Option<&str>,
    package_manager: Option<&str>,
    service_manager: Option<&str>,
) -> Vec<ExistingSurface> {
    let mut surfaces = Vec::new();

    if let Some(path) = existing_focusa {
        let writable = std::fs::metadata(path)
            .map(|meta| !meta.permissions().readonly())
            .unwrap_or(false);
        surfaces.push(ExistingSurface {
            kind: "cli_binary".into(),
            path: path.to_string(),
            present: true,
            writable,
        });
    }

    if let Some(manager) = package_manager {
        if let Some(cmd_path) = first_command(&[manager]) {
            surfaces.push(ExistingSurface {
                kind: "package_manager".into(),
                path: cmd_path.clone(),
                present: true,
                writable: false,
            });
        }
    }

    if let Some(manager) = service_manager {
        let user_home = std::env::var_os("HOME").map(std::path::PathBuf::from);
        if manager == "systemd" {
            if let Some(home) = user_home {
                let unit = home.join(".config/systemd/user/focusa-daemon.service");
                let path = unit.display().to_string();
                let present = unit.exists();
                surfaces.push(ExistingSurface {
                    kind: "service_unit_user".into(),
                    path,
                    present,
                    writable: false,
                });
            }
        } else if manager == "launchd" {
            if let Some(home) = user_home {
                let plist = home.join("Library/LaunchAgents/com.startempire.focusa-daemon.plist");
                let path = plist.display().to_string();
                let present = plist.exists();
                surfaces.push(ExistingSurface {
                    kind: "launchd_plist".into(),
                    path,
                    present,
                    writable: false,
                });
            }
        }
    }

    for lock_path in [
        "/tmp/focusa-daemon.lock",
        "/tmp/focusa/focusa-daemon.lock",
        "runtime/focusa-daemon.lock",
    ] {
        let path = std::path::PathBuf::from(lock_path);
        if path.exists() {
            surfaces.push(ExistingSurface {
                kind: "daemon_lock_file".into(),
                path: path.display().to_string(),
                present: true,
                writable: false,
            });
        }
    }

    surfaces
}

fn detect_cpu_summary() -> String {
    if cfg!(windows) {
        return "windows-uninstrumented".to_string();
    }

    if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
        let mut model = None;
        let mut cores = None;
        for line in content.lines() {
            if model.is_none() && line.starts_with("model name") {
                model = line
                    .split_once(':')
                    .map(|(_, value)| value.trim().to_string());
            }
            if cores.is_none() && line.starts_with("cpu cores") {
                cores = line
                    .split_once(':')
                    .and_then(|(_, value)| value.trim().parse::<u64>().ok());
            }
        }
        if let Some(model) = model {
            return match cores {
                Some(core_count) => format!("{model} ({core_count} cores)"),
                None => model,
            };
        }
    }
    if let Some(model) = std::process::Command::new("sysctl")
        .arg("-n")
        .arg("machdep.cpu.brand_string")
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                String::from_utf8(out.stdout)
                    .ok()
                    .map(|value| value.trim().to_string())
            } else {
                None
            }
        })
    {
        if !model.is_empty() {
            return model;
        }
    }

    if let Some(nproc) = std::process::Command::new("nproc")
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                String::from_utf8(out.stdout).ok()
            } else {
                None
            }
        })
    {
        let trimmed = nproc.trim().to_string();
        if !trimmed.is_empty() {
            return format!("{} logical cores", trimmed);
        }
    }

    "unknown".to_string()
}

fn detect_memory_summary() -> String {
    if cfg!(windows) {
        return "windows-uninstrumented".to_string();
    }

    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(total_kb) = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|value| value.parse::<u64>().ok())
                {
                    return format!("{} MiB", total_kb / 1024);
                }
            }
        }
    }

    if let Some(total_bytes) = std::process::Command::new("sysctl")
        .arg("-n")
        .arg("hw.memsize")
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                String::from_utf8(out.stdout)
                    .ok()
                    .and_then(|value| value.trim().parse::<u64>().ok())
            } else {
                None
            }
        })
    {
        return format!("{} MiB", total_bytes / 1024 / 1024);
    }

    "unknown".to_string()
}

fn detect_disk_summary(path: &str) -> String {
    if cfg!(windows) {
        return "windows-uninstrumented".to_string();
    }
    let output = std::process::Command::new("df")
        .arg("-P")
        .arg("-k")
        .arg(path)
        .output()
        .ok();
    output
        .and_then(|out| {
            if !out.status.success() {
                return None;
            }
            String::from_utf8(out.stdout)
                .ok()
                .and_then(|value| value.lines().nth(1).and_then(parse_df_line_summary))
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn parse_df_line_summary(line: &str) -> Option<String> {
    let mut fields = line.split_whitespace();
    let _filesystem = fields.next()?;
    let total = fields.next().and_then(|value| value.parse::<u64>().ok())?;
    let _used = fields.next()?.parse::<u64>().ok()?;
    let available = fields.next()?.parse::<u64>().ok()?;
    Some(format!("{available} KiB free / {total} KiB total"))
}

fn detect_network_summary() -> NetworkInventory {
    if cfg!(windows) {
        return NetworkInventory {
            default_route: false,
            resolv_conf_present: false,
            nameserver_count: 0,
            dns_probe_hint: "windows-not-probed".into(),
        };
    }

    let default_route = std::fs::read_to_string("/proc/net/route")
        .map(|content| {
            content
                .lines()
                .skip(1)
                .any(|line| line.split_whitespace().nth(1) == Some("00000000"))
        })
        .unwrap_or(false);

    let resolv_lines = std::fs::read_to_string("/etc/resolv.conf").unwrap_or_default();
    let nameserver_count = resolv_lines
        .lines()
        .filter(|line| line.starts_with("nameserver"))
        .count();
    let dns_probe_hint = if nameserver_count > 0 {
        "resolv_present".to_string()
    } else if resolv_lines.is_empty() {
        "resolv_missing".to_string()
    } else {
        "resolv_has_no_nameserver".to_string()
    };

    NetworkInventory {
        default_route,
        resolv_conf_present: std::path::Path::new("/etc/resolv.conf").exists(),
        nameserver_count,
        dns_probe_hint,
    }
}

fn detect_tls_inventory() -> TlsInventory {
    let paths = if cfg!(windows) {
        vec![
            "C:\\Windows\\System32\\config\\systemprofile\\AppData\\LocalLow\\Microsoft\\Cryptnet\\URL".to_string(),
            "C:\\Windows\\System32\\drivers\\etc\\ca\\certs".to_string(),
        ]
    } else {
        vec![
            "/etc/ssl/certs".to_string(),
            "/etc/ssl/cert.pem".to_string(),
            "/usr/local/share/ca-certificates".to_string(),
            "/usr/share/ca-certificates".to_string(),
            "/etc/pki/tls/certs".to_string(),
        ]
    };
    let mut cert_stores_found = Vec::new();
    for path in paths {
        if std::path::Path::new(&path).exists() {
            cert_stores_found.push(path);
        }
    }
    let cert_store_count = cert_stores_found.len();
    TlsInventory {
        has_any_store: cert_store_count > 0,
        cert_store_count,
        cert_stores_found,
    }
}

fn detect_proxy_inventory() -> ProxyInventory {
    fn redact_proxy(value: &str) -> String {
        let value = value.trim();
        if let Some((scheme, tail)) = value.split_once("://") {
            match tail.find('@') {
                Some(index) => format!("{scheme}://{}", &tail[index + 1..]),
                None => value.to_string(),
            }
        } else {
            value.to_string()
        }
    }

    ProxyInventory {
        http_proxy: std::env::var("HTTP_PROXY")
            .ok()
            .map(|value| redact_proxy(&value)),
        https_proxy: std::env::var("HTTPS_PROXY")
            .ok()
            .map(|value| redact_proxy(&value)),
        all_proxy: std::env::var("ALL_PROXY")
            .ok()
            .map(|value| redact_proxy(&value)),
        no_proxy: std::env::var("NO_PROXY")
            .ok()
            .map(|value| value.trim().to_string()),
    }
}

fn detect_daemon_health() -> DaemonHealthInventory {
    let lock_file_present = [
        "/tmp/focusa-daemon.lock",
        "/tmp/focusa/focusa-daemon.lock",
        "runtime/focusa-daemon.lock",
    ]
    .iter()
    .any(|path| std::path::Path::new(path).exists());
    if cfg!(windows) {
        return DaemonHealthInventory {
            running: false,
            pid: None,
            lock_file_present,
            status: "windows-untracked".into(),
        };
    }
    let pid = std::process::Command::new("pgrep")
        .arg("focusa-daemon")
        .output()
        .ok()
        .and_then(|out| {
            if !out.status.success() {
                return None;
            }
            String::from_utf8(out.stdout).ok().and_then(|output| {
                output
                    .lines()
                    .next()
                    .and_then(|line| line.trim().parse::<u32>().ok())
            })
        });
    let status = if pid.is_some() {
        "running".to_string()
    } else {
        "stopped_or_unreachable".to_string()
    };
    DaemonHealthInventory {
        running: pid.is_some(),
        pid,
        lock_file_present,
        status,
    }
}

fn detect_license_override(args: &InstallArgs) -> LicenseOverrideInventory {
    let dev_mode_requested = std::env::var("FOCUSA_DEV_MODE")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
        .unwrap_or(false);
    let local_tier = load_license_status()
        .map(|status| status.tier)
        .unwrap_or_else(|_| "unknown".into());
    let override_active = args.license_key.is_some() || dev_mode_requested;
    let effective_mode = if args.eval {
        "authority_limited_access_request".into()
    } else if args.accept_license || args.license_key.is_some() {
        "unsupported_legacy_input".into()
    } else if dev_mode_requested {
        "dev_mode".into()
    } else {
        "default".into()
    };

    LicenseOverrideInventory {
        requested_eval: args.eval,
        requested_license_key: args.license_key.is_some(),
        accept_license_requested: args.accept_license,
        dev_mode_requested,
        local_tier,
        override_active,
        effective_mode,
    }
}

fn detect_update_policy() -> UpdatePolicyInventory {
    let path = update_policy_path();
    let exists = path.exists();
    let default = derive_default_update_policy();
    let source;
    let note;
    let channel;
    let mode;
    let enabled;
    let auto_apply_allowed;

    if let Ok(raw) = std::fs::read_to_string(&path) {
        if let Ok(policy) = serde_json::from_str::<UpdatePolicy>(&raw) {
            if policy.schema == UPDATE_POLICY_SCHEMA_V1 {
                source = "explicit_file".to_string();
                note = "policy loaded from configured file".to_string();
                channel = policy.channel.label().into();
                mode = policy.mode.label().into();
                enabled = policy.enabled;
                auto_apply_allowed = policy.auto_apply_allowed;
            } else {
                source = "fallback".to_string();
                note = "policy schema mismatch; using default".to_string();
                channel = default.channel.label().into();
                mode = default.mode.label().into();
                enabled = default.enabled;
                auto_apply_allowed = default.auto_apply_allowed;
            }
        } else {
            source = "fallback".to_string();
            note = "policy failed to parse; using default".to_string();
            channel = default.channel.label().into();
            mode = default.mode.label().into();
            enabled = default.enabled;
            auto_apply_allowed = default.auto_apply_allowed;
        }
    } else {
        source = "fallback".to_string();
        note = "policy file missing; using default".to_string();
        channel = default.channel.label().into();
        mode = default.mode.label().into();
        enabled = default.enabled;
        auto_apply_allowed = default.auto_apply_allowed;
    }

    UpdatePolicyInventory {
        source,
        path: path.display().to_string(),
        exists,
        channel,
        mode,
        enabled,
        auto_apply_allowed,
        note,
    }
}

fn update_policy_path() -> std::path::PathBuf {
    std::env::var_os("FOCUSA_UPDATE_POLICY")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/usr/local/lib/focusa/update-policy.json"))
}

fn derive_default_update_policy() -> UpdatePolicy {
    let dev_mode_requested = std::env::var("FOCUSA_DEV_MODE")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
        .unwrap_or(false);
    match load_license_status() {
        Ok(status) => {
            UpdatePolicy::default_for_license(status.tier, &status.features, dev_mode_requested)
        }
        Err(_) => UpdatePolicy::default_for_license("evaluation", &[], dev_mode_requested),
    }
}

fn classify_compatibility(
    system: &PreflightSystem,
    missing_dependencies: &[String],
) -> CompatibilityInventory {
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();

    if !missing_dependencies.is_empty() {
        blockers.push("missing bootstrap dependency".into());
    }
    if system.kernel == "unknown" {
        warnings.push("kernel unknown".into());
    }
    if !system.path_target_writable {
        warnings.push("primary path target not writable".into());
    }
    if !system.path_targets.iter().any(|entry| entry.focusa_present) {
        warnings.push("focusa is not currently discoverable on common PATH targets".into());
    }

    let status = if blockers.is_empty() {
        if warnings.is_empty() {
            "compatible".to_string()
        } else {
            "compatible_with_warnings".to_string()
        }
    } else {
        "blocked".to_string()
    };

    CompatibilityInventory {
        status,
        blockers,
        warnings,
    }
}

const SUPPORTED_PI_NPM_PACKAGE: &str = "@earendil-works/pi-coding-agent@0.81.1";
const UIAI_ENGINE_RELEASE_TAG: &str = "engine-vw20-multipool-20260705-2119";
const UIAI_ENGINE_LINUX_AMD64_SHA256: &str =
    "963883a19eec91c81ee88bc70c23e8db77f0cc12c673be872f6ee3bda3bba5b5";

fn uiai_engine_url() -> String {
    std::env::var("UIAI_ENGINE_URL").unwrap_or_else(|_| "http://127.0.0.1:7456".into())
}

fn uiai_engine_healthy() -> bool {
    let health_url = format!("{}/health", uiai_engine_url().trim_end_matches('/'));
    let auth_header = std::env::var("UIAI_BEARER_TOKEN")
        .ok()
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
        .map(|token| format!("Authorization: Bearer {token}"));
    let mut command = std::process::Command::new("curl");
    command.args(["--max-time", "2", "--fail", "--silent"]);
    command.stdout(std::process::Stdio::null());
    command.stderr(std::process::Stdio::null());
    if let Some(header) = auth_header.as_deref() {
        command.args(["--header", header]);
    }
    command
        .arg(health_url)
        .status()
        .is_ok_and(|status| status.success())
}

fn uiai_engine_install_command() -> String {
    if !(cfg!(target_os = "linux") && cfg!(target_arch = "x86_64")) {
        return "printf '%s\\n' 'UIAI local install is unsupported on this platform; set UIAI_ENGINE_URL to a healthy private/remote endpoint and rerun' >&2; exit 2".into();
    }
    let asset_url = format!(
        "https://github.com/WPUIAI/uiai-engine/releases/download/{UIAI_ENGINE_RELEASE_TAG}/uiai-engine-linux-amd64"
    );
    format!(
        "set -eu; tmp=$(mktemp -d); trap 'rm -rf \"$tmp\"' EXIT; \
         curl -fsSLo \"$tmp/uiai-engine\" '{asset_url}'; \
         printf '%s  %s\\n' '{UIAI_ENGINE_LINUX_AMD64_SHA256}' uiai-engine > \"$tmp/SHA256SUMS\"; \
         (cd \"$tmp\" && sha256sum -c SHA256SUMS); \
         install -d \"$HOME/.focusa/bin\" \"$HOME/.config/systemd/user\" \"$HOME/.local/state/focusa\"; \
         install -m 0755 \"$tmp/uiai-engine\" \"$HOME/.focusa/bin/uiai-engine\"; \
         printf '%s\\n' '[Unit]' 'Description=Focusa-managed UIAI Engine' 'After=network-online.target' '' '[Service]' 'Type=simple' 'ExecStart=%h/.focusa/bin/uiai-engine' 'Restart=on-failure' 'RestartSec=3' 'StandardOutput=append:%h/.local/state/focusa/uiai-engine.log' 'StandardError=append:%h/.local/state/focusa/uiai-engine.log' '' '[Install]' 'WantedBy=default.target' > \"$HOME/.config/systemd/user/focusa-uiai-engine.service\"; \
         systemctl --user daemon-reload; systemctl --user enable --now \"$HOME/.config/systemd/user/focusa-uiai-engine.service\"; \
         i=0; until curl --max-time 1 --fail --silent '{}/health' >/dev/null; do i=$((i+1)); [ \"$i\" -lt 20 ] || exit 3; sleep 0.25; done",
        uiai_engine_url().trim_end_matches('/')
    )
}

fn command_semver(name: &str) -> Option<(u64, u64, u64)> {
    let command = find_command(name)?;
    #[cfg(windows)]
    let output = {
        let extension = std::path::Path::new(&command)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if matches!(extension.as_str(), "cmd" | "bat") {
            // PowerShell's call operator preserves an absolute script path as
            // one token even when it contains spaces (for example npm.cmd).
            let command_line = format!("& '{}' --version", command.replace('\'', "''"));
            std::process::Command::new("powershell.exe")
                .args(["-NoProfile", "-NonInteractive", "-Command"])
                .arg(command_line)
                .output()
                .ok()?
        } else {
            std::process::Command::new(&command)
                .arg("--version")
                .output()
                .ok()?
        }
    };
    #[cfg(not(windows))]
    let output = std::process::Command::new(command)
        .arg("--version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = format!(
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let version = text
        .split(|character: char| !(character.is_ascii_digit() || character == '.'))
        .find(|value| {
            value
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_digit())
        })?;
    let mut parts = version
        .split('.')
        .filter_map(|part| part.parse::<u64>().ok());
    Some((
        parts.next()?,
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    ))
}

fn dependency_present(name: &str) -> bool {
    match name {
        "node" => command_semver("node").is_some_and(|version| version >= (20, 0, 0)),
        "npm" => command_semver("npm").is_some(),
        "pi" => command_semver("pi").is_some_and(|version| version >= (0, 81, 1)),
        "uiai-engine" => uiai_engine_healthy(),
        "python3" if cfg!(windows) => have_cmd("python3") || have_cmd("python"),
        "sha256sum" => have_cmd("sha256sum") || have_cmd("shasum"),
        _ => have_cmd(name),
    }
}

fn detect_dependencies(package_manager: Option<&str>) -> Vec<PreflightDependency> {
    [
        "curl",
        "python3",
        "sha256sum",
        "tar",
        "node",
        "npm",
        "pi",
        "uiai-engine",
    ]
    .into_iter()
    .map(|name| {
        let install_plan = dependency_install_plan(package_manager, name);
        PreflightDependency {
            name: name.into(),
            present: dependency_present(name),
            install_hint: Some(install_plan.install_command.clone()),
            install_plan,
        }
    })
    .collect()
}

fn dependency_package(manager: &str, name: &str) -> String {
    match (manager, name) {
        ("brew", "python3") => "python".into(),
        ("brew", "node" | "npm") => "node".into(),
        ("choco", "node" | "npm") => "nodejs-lts".into(),
        ("winget", "node" | "npm") => "OpenJS.NodeJS.LTS".into(),
        (_, "node") => "nodejs".into(),
        ("brew", "sha256sum") => "coreutils".into(),
        ("brew", "tar") => "gnu-tar".into(),
        ("choco", "python3") => "python312".into(),
        ("choco", "sha256sum") => "gnuwin32-coreutils.install".into(),
        ("choco", "tar") => "gnuwin32-tar".into(),
        ("winget", "curl") => "cURL.cURL".into(),
        ("winget", "python3") => "Python.Python.3.12".into(),
        ("winget", "sha256sum") => "GnuWin32.CoreUtils".into(),
        ("winget", "tar") => "GnuWin32.Tar".into(),
        (_, "sha256sum") => "coreutils".into(),
        (_, other) => other.into(),
    }
}

fn dependency_install_plan(package_manager: Option<&str>, name: &str) -> DependencyInstallPlan {
    if name == "uiai-engine" {
        return DependencyInstallPlan {
            manager: if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
                "focusa-pinned-release"
            } else {
                "remote-endpoint"
            }
            .into(),
            package: format!("WPUIAI/uiai-engine@{UIAI_ENGINE_RELEASE_TAG}"),
            repository: "GitHub release with embedded pinned SHA256".into(),
            install_mode: if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
                "user_service"
            } else {
                "verified_remote_endpoint"
            }
            .into(),
            install_command: uiai_engine_install_command(),
            dry_run_command: format!(
                "curl --max-time 2 --fail --silent {}/health",
                uiai_engine_url().trim_end_matches('/')
            ),
            privilege_required: false,
            recovery_hint: format!(
                "Linux/amd64 may install the pinned checksummed engine. Otherwise set UIAI_ENGINE_URL to a healthy private endpoint and verify {}/health before rerunning Focusa install.",
                uiai_engine_url().trim_end_matches('/')
            ),
        };
    }
    if name == "pi" {
        return DependencyInstallPlan {
            manager: "npm".into(),
            package: SUPPORTED_PI_NPM_PACKAGE.into(),
            repository: "npm registry".into(),
            install_mode: "user_global".into(),
            install_command: format!("npm install --global {SUPPORTED_PI_NPM_PACKAGE}"),
            dry_run_command: format!("npm view {SUPPORTED_PI_NPM_PACKAGE} version"),
            privilege_required: false,
            recovery_hint: "If npm reports EACCES, configure a user-owned npm prefix, ensure its bin directory is on PATH, then rerun focusa install --install-dependencies."
                .into(),
        };
    }
    let manager = package_manager.unwrap_or("manual");
    let package = dependency_package(manager, name);
    let (repository, install_command, dry_run_command, privilege_required) = match manager {
        "dnf" => (
            "configured DNF repositories",
            format!("sudo dnf install -y {package}"),
            format!("sudo dnf --assumeno install {package}"),
            true,
        ),
        "yum" => (
            "configured YUM repositories",
            format!("sudo yum install -y {package}"),
            format!("sudo yum --assumeno install {package}"),
            true,
        ),
        "apt-get" => (
            "configured APT repositories",
            format!("sudo apt-get update && sudo apt-get install -y {package}"),
            format!("apt-get -s install {package}"),
            true,
        ),
        "brew" => (
            "Homebrew core",
            format!("brew install {package}"),
            format!("brew info {package}"),
            false,
        ),
        "pacman" => (
            "configured pacman repositories",
            format!("sudo pacman -S --needed {package}"),
            format!("pacman -Si {package}"),
            true,
        ),
        "zypper" => (
            "configured Zypper repositories",
            format!("sudo zypper install -y {package}"),
            format!("sudo zypper --dry-run install {package}"),
            true,
        ),
        "choco" => (
            "Chocolatey community repository",
            format!("choco install {package} -y --no-progress"),
            format!("choco search --exact {package}"),
            false,
        ),
        "winget" => (
            "winget community source",
            format!(
                "winget install --id {package} --exact --accept-package-agreements --accept-source-agreements"
            ),
            format!("winget show --id {package} --exact"),
            false,
        ),
        _ => (
            "platform package repository",
            format!("install dependency manually: {package}"),
            format!("command -v {name}"),
            false,
        ),
    };
    let install_mode = match manager {
        "brew" => "user_local",
        "manual" => "manual",
        _ => "system",
    };
    DependencyInstallPlan {
        manager: manager.into(),
        package,
        repository: repository.into(),
        install_mode: install_mode.into(),
        install_command,
        dry_run_command,
        privilege_required,
        recovery_hint: format!(
            "If installation fails, inspect {manager} output, repair repository access, rerun the dry-run command, then rerun focusa install --preflight --json."
        ),
    }
}

fn dependency_install_consent(args: &InstallArgs) -> Result<&'static str> {
    if !args.install_dependencies {
        return Ok("not_requested");
    }
    if args.assume_yes {
        return Ok("approved");
    }
    if args.json || !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Ok("consent_required");
    }
    print!("Install the missing bootstrap and agent-workflow dependencies now? [y/N] ");
    std::io::stdout().flush()?;
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    Ok(
        if matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
            "approved"
        } else {
            "declined"
        },
    )
}

fn dependency_lock_contention(error: &str) -> bool {
    let error = error.to_ascii_lowercase();
    error.contains("lock")
        || error.contains("another process")
        || error.contains("currently in use")
}

fn execute_dependency_plans_with<F>(
    dependencies: &[&PreflightDependency],
    mut runner: F,
) -> DependencyInstallExecution
where
    F: FnMut(&str) -> std::result::Result<(), String>,
{
    let mut commands = Vec::new();
    let mut installed = Vec::new();
    let mut already_present = Vec::new();
    let mut failures = Vec::new();
    let mut retryable_failures = Vec::new();
    let mut recovery_evidence = Vec::new();
    for dependency in dependencies {
        if dependency.present {
            already_present.push(dependency.name.clone());
            continue;
        }
        let command = dependency.install_plan.install_command.clone();
        commands.push(command.clone());
        match runner(&command) {
            Ok(()) => installed.push(dependency.name.clone()),
            Err(error) if dependency_lock_contention(&error) => {
                retryable_failures.push(format!("{}: {error}", dependency.name));
                match runner(&command) {
                    Ok(()) => installed.push(dependency.name.clone()),
                    Err(retry_error) => {
                        failures.push(format!("{}: {retry_error}", dependency.name));
                        recovery_evidence.push(dependency.install_plan.recovery_hint.clone());
                    }
                }
            }
            Err(error) => {
                failures.push(format!("{}: {error}", dependency.name));
                recovery_evidence.push(dependency.install_plan.recovery_hint.clone());
            }
        }
    }
    let status = if failures.is_empty() {
        if installed.is_empty() {
            "no_op"
        } else {
            "completed"
        }
    } else if installed.is_empty() {
        "failed"
    } else {
        "partial_failure"
    };
    let rollback_status = if failures.is_empty() {
        "not_needed"
    } else if installed.is_empty() {
        "no_changes_to_rollback"
    } else {
        "partial_success_preserved_non_destructively"
    };
    DependencyInstallExecution {
        status: status.into(),
        commands,
        installed,
        already_present,
        failures,
        retryable_failures,
        rollback_status: rollback_status.into(),
        recovery_evidence,
    }
}

fn execute_dependency_plans(dependencies: &[&PreflightDependency]) -> DependencyInstallExecution {
    execute_dependency_plans_with(dependencies, |command| {
        let output = if cfg!(windows) {
            std::process::Command::new("cmd")
                .args(["/C", command])
                .output()
        } else {
            std::process::Command::new("sh")
                .args(["-c", command])
                .output()
        }
        .map_err(|error| error.to_string())?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    })
}

fn terminal_ux_preflight(no_animation: bool) -> TerminalUxPreflight {
    let capabilities = detect_capabilities(no_animation, false, false);
    let interactive_tty = capabilities.stderr_is_terminal
        && !capabilities.ci
        && !capabilities.term.is_empty()
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
    find_command(name).is_some()
}

fn find_command(name: &str) -> Option<String> {
    #[cfg(windows)]
    {
        // Resolve directly from PATH so `.exe` and command shims are reliable
        // in non-interactive PowerShell/CI processes without executing them.
        let path = std::env::var_os("PATH")?;
        let has_extension = std::path::Path::new(name).extension().is_some();
        let candidates = if has_extension {
            vec![name.to_string()]
        } else {
            vec![
                // Native executables and Windows command shims must win over
                // extensionless POSIX shims that npm also places on PATH.
                format!("{name}.exe"),
                format!("{name}.cmd"),
                format!("{name}.bat"),
                format!("{name}.com"),
                name.to_string(),
            ]
        };
        for directory in std::env::split_paths(&path) {
            for candidate in &candidates {
                let resolved = directory.join(candidate);
                if resolved.is_file() {
                    return Some(resolved.display().to_string());
                }
            }
        }
        return None;
    }
    #[cfg(not(windows))]
    {
        which::which(name)
            .ok()
            .map(|path| path.display().to_string())
    }
}

fn is_root() -> bool {
    if cfg!(windows) {
        return false;
    }
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
        "distro/version: {} {}",
        report.system.distro, report.system.os_version
    );
    println!(
        "kernel/libc: {} / {}",
        report.system.kernel, report.system.libc
    );
    println!(
        "cpu: {} | memory: {} | disk: {}",
        report.system.cpu, report.system.memory, report.system.disk
    );
    println!(
        "network: default_route={} nameservers={} tls_stores={} proxy_http={}",
        report.system.network.default_route,
        report.system.network.nameserver_count,
        report.system.tls.cert_store_count,
        report.system.proxy.http_proxy.as_deref().unwrap_or("none")
    );
    println!(
        "compatibility: {} blockers={} warnings={}",
        report.system.compatibility.status,
        report.system.compatibility.blockers.len(),
        report.system.compatibility.warnings.len()
    );
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
    println!(
        "read_only: {} mutations_performed: {}",
        report.read_only, report.mutations_performed
    );
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

fn cancellation_result<T>(
    install_root: &std::path::Path,
    stash_path: &std::path::Path,
    stashed: bool,
    sink: &dyn InstallEventSink,
) -> Result<T> {
    sink.emit(InstallEvent::PhaseFailed {
        phase: InstallPhase::Finalize,
        message: "Installation cancelled by operator".into(),
        recovery_hint: Some("staged downloads were removed before rollback".into()),
    });
    sink.emit(InstallEvent::RollbackStarted {
        reason: "installation cancelled by operator".into(),
    });
    cleanup_staged_downloads(install_root);
    let rollback = phase_atomic_recover(install_root, stash_path, stashed);
    match rollback {
        Ok(()) => {
            sink.emit(InstallEvent::RollbackSucceeded);
            Err(anyhow!(
                "installation cancelled by operator; {}",
                if stashed {
                    "prior installation restored"
                } else {
                    "clean-state cleanup completed; no prior installation existed"
                }
            ))
        }
        Err(error) => {
            sink.emit(InstallEvent::RollbackFailed {
                message: error.to_string(),
                recovery_hint: format!("restore the prior install from {}", stash_path.display()),
            });
            Err(anyhow!(
                "installation cancelled by operator; rollback failed: {error}"
            ))
        }
    }
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
        let mut report = build_preflight_report(&args, target, &install_root);
        if args.install_dependencies && report.missing_dependencies.is_empty() {
            report.dependency_install_offer.consent_status = "not_needed".into();
            report.dependency_install_offer.message =
                "all required dependencies are already present".into();
        } else if args.install_dependencies {
            let consent = dependency_install_consent(&args)?;
            report.dependency_install_offer.consent_status = consent.into();
            match consent {
                "approved" => {
                    let dependencies = report.dependencies.iter().collect::<Vec<_>>();
                    let execution = execute_dependency_plans(&dependencies);
                    report.read_only = false;
                    report.mutations_performed = !execution.commands.is_empty();
                    report.dependency_install_offer.auto_install_performed = true;
                    report.dependency_install_offer.message = if execution.failures.is_empty() {
                        "dependency installation commands completed; rerun preflight to verify PATH visibility".into()
                    } else {
                        "one or more dependency installation commands failed; follow recovery hints"
                            .into()
                    };
                    report.status = if execution.failures.is_empty() {
                        "dependency_install_completed"
                    } else {
                        "dependency_install_failed"
                    };
                    report.dependency_install_offer.execution = Some(execution);
                }
                "declined" => {
                    report.status = "dependency_install_declined";
                    report.dependency_install_offer.message =
                        "operator declined dependency installation; no package command executed"
                            .into();
                }
                _ => {
                    report.status = "dependency_install_consent_required";
                    report.dependency_install_offer.message =
                        "noninteractive/JSON installation requires --install-dependencies --assume-yes".into();
                }
            }
        }
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

    // Install/verify workflow prerequisites before touching the existing Focusa
    // installation. These are customer-owned tools and are intentionally outside
    // the Focusa rollback stash; explicit consent is required.
    let dependency_report = build_preflight_report(&args, target, &install_root);
    if !dependency_report.missing_dependencies.is_empty() {
        if !args.install_dependencies {
            bail!(
                "required bootstrap/agent-workflow dependencies are missing: {}; rerun with --install-dependencies (and --assume-yes for unattended installs)",
                dependency_report.missing_dependencies.join(", ")
            );
        }
        match dependency_install_consent(&args)? {
            "approved" => {
                let dependencies = dependency_report.dependencies.iter().collect::<Vec<_>>();
                let execution = execute_dependency_plans(&dependencies);
                if !execution.failures.is_empty() {
                    bail!(
                        "dependency installation failed: {}; recovery: {}",
                        execution.failures.join("; "),
                        execution.recovery_evidence.join("; ")
                    );
                }
                let refreshed = build_preflight_report(&args, target, &install_root);
                if !refreshed.missing_dependencies.is_empty() {
                    bail!(
                        "dependency commands completed but required tools are still unavailable: {}; refresh PATH or follow each install recovery hint, then rerun",
                        refreshed.missing_dependencies.join(", ")
                    );
                }
            }
            "declined" => bail!("operator declined required workflow dependency installation"),
            _ => bail!(
                "dependency installation consent required; rerun with --install-dependencies --assume-yes for unattended installs"
            ),
        }
    }

    // Real install wrapped in atomicity (focusa-112-atomicity, Spec 112 §6):
    //   1. Stash any existing install to .focusa.stash
    //   2. Execute each phase
    //   3. Run smoke test (focusa --version on the new binary)
    //   4. On smoke-test failure: rollback to stash
    //   5. On success: remove stash
    let stash_path = install_root.with_extension("stash");
    let cancellation = CancellationToken::new();
    let _signals = install_signal_handlers(&cancellation)
        .map_err(|error| anyhow!("install cancellation handlers: {error}"))?;
    let capabilities = detect_capabilities(args.no_animation, args.json, args.quiet);
    let mut ui = InstallerUi::new(
        capabilities.mode,
        args.quiet,
        capabilities.animation_seed,
        &cancellation,
    );
    if cancellation.is_cancelled() {
        return cancellation_result(&install_root, &stash_path, false, &ui);
    }
    let sink = &ui;
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
    if cancellation.is_cancelled() {
        cleanup_staged_downloads(&install_root);
        return cancellation_result(&install_root, &stash_path, stashed, &ui);
    }
    let mut result = match execute_real_install(
        &args,
        target,
        channel,
        &install_root,
        &cancellation,
        sink,
    )
    .await
    {
        Ok(result) => result,
        Err(e) if cancellation.is_cancelled() => {
            cleanup_staged_downloads(&install_root);
            return cancellation_result(&install_root, &stash_path, stashed, &ui);
        }
        Err(e) => {
            sink.emit(InstallEvent::PhaseFailed {
                phase: InstallPhase::Finalize,
                message: "Installer phase failed".into(),
                recovery_hint: Some(e.to_string()),
            });
            cleanup_staged_downloads(&install_root);
            sink.emit(InstallEvent::RollbackStarted {
                reason: "recovering from installer phase failure".into(),
            });
            let rollback = phase_atomic_recover(&install_root, &stash_path, stashed);
            let recovery = match rollback {
                Ok(()) => {
                    sink.emit(InstallEvent::RollbackSucceeded);
                    if stashed {
                        "installer phase failed; prior installation restored; correct the reported release or network error, then rerun `focusa install`"
                    } else {
                        "fresh install failed; staged files removed; correct the reported release or network error, then rerun `focusa install`"
                    }
                    .to_string()
                }
                Err(rollback_error) => {
                    sink.emit(InstallEvent::RollbackFailed {
                        message: rollback_error.to_string(),
                        recovery_hint: format!(
                            "restore the prior install from {}",
                            stash_path.display()
                        ),
                    });
                    format!(
                        "automatic rollback failed; restore the prior install from {} before retrying",
                        stash_path.display()
                    )
                }
            };
            return Err(e).context(recovery);
        }
    };
    let bin_dir = install_root.join("bin");
    let expected_tag = result
        .assets
        .first()
        .map(|asset| asset.version.as_str())
        .ok_or_else(|| anyhow!("installed release identity is missing"))?;
    if let Err(e) = phase_smoke_test(target, &bin_dir, expected_tag).await {
        sink.emit(InstallEvent::PhaseFailed {
            phase: InstallPhase::RunHealthChecks,
            message: "Installed focusa --version smoke test failed".into(),
            recovery_hint: Some(e.to_string()),
        });
        cleanup_staged_downloads(&install_root);
        sink.emit(InstallEvent::RollbackStarted {
            reason: "recovering from smoke-test failure".into(),
        });
        let rollback = phase_atomic_recover(&install_root, &stash_path, stashed);
        let recovery = match rollback {
            Ok(()) => {
                sink.emit(InstallEvent::RollbackSucceeded);
                if stashed {
                    "installed binary smoke test failed; prior installation restored; verify the release artifact, then rerun `focusa install`"
                } else {
                    "installed binary smoke test failed; staged files removed; verify the release artifact, then rerun `focusa install`"
                }
                .to_string()
            }
            Err(rollback_error) => {
                sink.emit(InstallEvent::RollbackFailed {
                    message: rollback_error.to_string(),
                    recovery_hint: format!(
                        "restore the prior install from {}",
                        stash_path.display()
                    ),
                });
                format!(
                    "smoke test and automatic rollback failed; restore the prior install from {} before retrying",
                    stash_path.display()
                )
            }
        };
        return Err(e).context(recovery);
    }
    if cancellation.is_cancelled() {
        cleanup_staged_downloads(&install_root);
        return cancellation_result(&install_root, &stash_path, stashed, &ui);
    }
    // Persist the verified release only after the smoke gate; this marker is the
    // anti-rollback authority for future downloads and is itself atomic.
    if let Some(version) = result.assets.first().map(|asset| asset.version.as_str()) {
        write_verified_version_marker(&install_root, version)?;
    }
    if stashed {
        if let Err(error) = phase_restore_customer_data(&stash_path, &install_root) {
            phase_atomic_rollback(&install_root, &stash_path).ok();
            return Err(error).context(
                "failed to restore customer data from the prior install; prior installation restored",
            );
        }
    }
    if args.system_install {
        let expected_tag = result
            .assets
            .first()
            .map(|asset| asset.version.as_str())
            .ok_or_else(|| anyhow!("verified Focusa CLI asset identity is missing"))?;
        match promote_system_links(
            &bin_dir,
            std::path::Path::new("/usr/local/bin"),
            expected_tag,
            !args.no_service && matches!(target, InstallTarget::Linux | InstallTarget::Auto),
        ) {
            Ok(service_restarted) => {
                result.service_status = if service_restarted {
                    "authoritative system service restarted".into()
                } else if args.no_service {
                    "system promotion completed; service restart skipped".into()
                } else {
                    "system promotion completed; no active system service".into()
                };
            }
            Err(error) => {
                if let Err(rollback_error) =
                    phase_atomic_recover(&install_root, &stash_path, stashed)
                {
                    return Err(error).context(format!(
                    "authoritative system promotion failed and local rollback failed: {rollback_error}; restore {} before retrying",
                    stash_path.display()
                ));
                }
                return Err(error).context(
                    "authoritative system promotion failed; prior system links and installation restored",
                );
            }
        }
    }
    if stashed {
        if let Err(error) = phase_atomic_cleanup(&stash_path) {
            return Err(error).context(
                "new installation and customer data are intact, but prior stash cleanup failed; remove the reported stash after verification",
            );
        }
    }

    // launchd can retain the old executable inode while the prior install is
    // still present as `.focusa.stash`. Restart once more after commit/cleanup
    // so the running daemon necessarily resolves the promoted symlink target.
    if target == InstallTarget::Darwin && !args.no_service && cfg!(target_os = "macos") {
        match crate::commands::service::restart_launchd_after_commit() {
            Ok(()) => result.service_status = "registered and restarted after commit".into(),
            Err(error) => {
                result.service_status = format!(
                    "registered; post-commit restart warning: {error}; run `focusa restart`"
                );
            }
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
    let authoritative_bin_dir = if args.system_install {
        std::path::Path::new("/usr/local/bin")
    } else {
        bin_dir.as_path()
    };
    let summary = InstallCompletionSummary {
        version: version.clone(),
        target: format!("{:?}", target),
        channel: format!("{:?}", channel),
        install_root: install_root.display().to_string(),
        cli_path: authoritative_bin_dir
            .join(installed_binary_name(target, "focusa"))
            .display()
            .to_string(),
        daemon_path: authoritative_bin_dir
            .join(installed_binary_name(target, "focusa-daemon"))
            .display()
            .to_string(),
        daemon_health: "smoke-test pending separate daemon health check".into(),
        tui_path: authoritative_bin_dir
            .join(installed_binary_name(target, "focusa-tui"))
            .display()
            .to_string(),
        runner_path: authoritative_bin_dir
            .join(installed_binary_name(target, "focusa-session-runner"))
            .display()
            .to_string(),
        service_status: result.service_status.clone(),
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
    ui.finish();

    // The renderer has restored the transient UI before this single durable
    // human summary or single JSON document is written. Delegating commands
    // suppress this envelope so they can emit exactly one caller-owned result.
    if !args.suppress_completion_output {
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
    }

    Ok(())
}

// ----- Phase 1: License re-validation (focusa-112-license-revalidate) -----
async fn phase_license(args: &InstallArgs, channel: Channel) -> Result<String> {
    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .ok_or_else(|| anyhow!("HOME not set; cannot resolve authority entitlement"))?;
    let config_dir = home.join(".config/focusa");
    let required_feature = install_channel_feature(channel);

    if let Some(snapshot) = resolve_installer_entitlement(&config_dir, required_feature)? {
        return Ok(format!(
            "authority_{}_sequence_{}",
            entitlement_state_label(snapshot.state),
            snapshot.sequence.unwrap_or_default()
        ));
    }
    if args.reuse_existing_license {
        bail!(
            "E_AUTHORITY_EXISTING_UNUSABLE: existing signed authority lease is missing, expired, revoked, or lacks {required_feature}; reactivate before upgrade"
        );
    }
    let legacy_path = config_dir.join("license.json");
    let legacy_status = if legacy_path.is_file() {
        load_license_status().ok()
    } else {
        None
    };
    let pending_migration =
        begin_legacy_license_migration(&config_dir, &legacy_path, legacy_status.as_ref())?;
    if legacy_status
        .as_ref()
        .is_some_and(|legacy| legacy.commercial_use && legacy.status == "active")
    {
        bail!(
            "E_AUTHORITY_PAID_MIGRATION_REQUIRED: an active paid legacy entitlement was found; preserve it and complete authority migration without repurchase before installing any runnable assets"
        );
    }
    if args
        .license_key
        .as_deref()
        .is_some_and(|key| !key.trim().is_empty())
    {
        bail!(
            "E_AUTHORITY_RAW_KEY_FORBIDDEN: raw license keys cannot authorize installation; use authority device authorization so a signed, node-bound lease is issued"
        );
    }

    // Spec 152E §21 surface consolidation: an interactive terminal renders
    // the universal email → verify → offer → checkout/poll → key/lease flow
    // through the shared activation client. Noninteractive installs keep the
    // device-code authorization path below (verified-email, signed lease).
    if crate::commands::activation_flow::interactive_available() {
        authorize_installer_activation_flow(&config_dir, args, channel).await?;
    } else {
        acquire_installer_entitlement(&config_dir, required_feature, args.json).await?;
    }
    let snapshot =
        resolve_installer_entitlement(&config_dir, required_feature)?.ok_or_else(|| {
            anyhow!(
                "E_AUTHORITY_LEASE_UNUSABLE: authority authorization completed without a usable signed product/channel lease"
            )
        })?;
    if let Some(migration) = pending_migration {
        complete_legacy_license_migration(&migration, &snapshot)?;
    }
    Ok(format!(
        "authority_{}_sequence_{}",
        entitlement_state_label(snapshot.state),
        snapshot.sequence.unwrap_or_default()
    ))
}

struct PendingLegacyMigration {
    migration_id: uuid::Uuid,
    source_class: LegacyLicenseSourceClass,
    source_digest: String,
    journal_path: std::path::PathBuf,
}

fn begin_legacy_license_migration(
    config_dir: &std::path::Path,
    legacy_path: &std::path::Path,
    legacy_status: Option<&focusa_core::license::LicenseStatus>,
) -> Result<Option<PendingLegacyMigration>> {
    if !legacy_path.is_file() {
        return Ok(None);
    }
    let source_class = if legacy_status.is_some_and(|status| status.commercial_use) {
        LegacyLicenseSourceClass::PaidKeyRecord
    } else {
        LegacyLicenseSourceClass::EvaluationRecord
    };
    let inventory = inventory_legacy_license_files(&[(source_class, legacy_path.to_path_buf())])
        .context("inventory legacy license for authority migration")?;
    let Some(item) = inventory.into_iter().next() else {
        return Ok(None);
    };
    let migration = PendingLegacyMigration {
        migration_id: migration_id_for_source_digest(&item.source_digest),
        source_class,
        source_digest: item.source_digest,
        journal_path: config_dir.join("license-migration.jsonl"),
    };
    for status in [
        LicenseMigrationStatus::Discovered,
        LicenseMigrationStatus::AwaitingAuthority,
    ] {
        append_license_migration_entry(
            &migration.journal_path,
            migration_entry(&migration, status, None),
        )
        .context("persist legacy license migration preflight")?;
    }
    Ok(Some(migration))
}

fn complete_legacy_license_migration(
    migration: &PendingLegacyMigration,
    snapshot: &EntitlementSnapshot,
) -> Result<()> {
    for status in [
        LicenseMigrationStatus::AuthorityIssued,
        LicenseMigrationStatus::Committed,
    ] {
        append_license_migration_entry(
            &migration.journal_path,
            migration_entry(migration, status, Some(snapshot)),
        )
        .context("commit authority-backed legacy license migration")?;
    }
    Ok(())
}

fn migration_entry(
    migration: &PendingLegacyMigration,
    status: LicenseMigrationStatus,
    snapshot: Option<&EntitlementSnapshot>,
) -> LicenseMigrationJournalEntry {
    LicenseMigrationJournalEntry {
        schema: String::new(),
        migration_id: migration.migration_id,
        sequence: 0,
        source_class: migration.source_class,
        source_digest: migration.source_digest.clone(),
        status,
        authority_lease_id: snapshot.and_then(|value| value.lease_id.clone()),
        authority_lease_sequence: snapshot.and_then(|value| value.sequence),
        authority_lease_digest: snapshot.and_then(|value| value.lease_digest.clone()),
        preserved_data_refs: vec![
            "node_identity".into(),
            "device_pairing".into(),
            "projects".into(),
            "workpoints".into(),
            "evidence".into(),
        ],
        evidence_refs: vec!["evidence:legacy-license-source-digest".into()],
        observed_at: chrono::Utc::now(),
        previous_entry_hash: String::new(),
        entry_hash: String::new(),
    }
}

fn resolve_installer_entitlement(
    config_dir: &std::path::Path,
    required_feature: &str,
) -> Result<Option<EntitlementSnapshot>> {
    let identity = load_or_create_node_identity(config_dir, "focusa")
        .context("resolve node identity for authority entitlement")?;
    let context = LeaseVerificationContext {
        expected_product: "focusa".into(),
        expected_node_id: identity.node_id,
        now: chrono::Utc::now(),
        minimum_sequence: None,
        expected_previous_digest: None,
    };
    let snapshot = resolve_authority_state(
        &config_dir.join(AUTHORITY_STATE_FILE),
        embedded_production_trust_roots(),
        &context,
    );
    if !matches!(
        snapshot.state,
        EntitlementState::Active | EntitlementState::OfflineGrace
    ) || snapshot.product != "focusa"
        || !snapshot
            .features
            .get(required_feature)
            .copied()
            .unwrap_or(false)
    {
        return Ok(None);
    }
    Ok(Some(snapshot))
}

async fn acquire_installer_entitlement(
    config_dir: &std::path::Path,
    required_feature: &str,
    json_output: bool,
) -> Result<()> {
    let identity = load_or_create_node_identity(config_dir, "focusa")
        .context("create node-bound authority identity")?;
    let request = DeviceCodeStartRequest {
        request_id: uuid::Uuid::now_v7(),
        product: "focusa".into(),
        node_id: identity.node_id.clone(),
        requested_features: vec![required_feature.into()],
    };
    let origin = std::env::var("FOCUSA_AUTHORITY_ORIGIN")
        .unwrap_or_else(|_| "https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/".into());
    let origin = reqwest::Url::parse(&origin).context("parse FOCUSA_AUTHORITY_ORIGIN")?;
    let endpoints = AuthorityEndpointSet {
        start: origin.join("device/start")?,
        poll: origin.join("device/poll")?,
        refresh: origin.join("lease/refresh")?,
        nodes: origin.join("nodes")?,
        deactivate_node: origin.join("nodes/deactivate")?,
    };
    let client = AuthorityHttpClient::new(AuthorityHttpPolicy {
        endpoints,
        timeout: Duration::from_secs(30),
        max_response_bytes: 1024 * 1024,
    })
    .context("initialize authority client")?;
    let challenge = client
        .start(&request)
        .await
        .context("start device authorization")?;
    if json_output {
        eprintln!(
            "authority_verification_uri={} authority_user_code={}",
            challenge.verification_uri, challenge.user_code
        );
    } else {
        eprintln!(
            "Verify your email and authorize this install at {} using code {}",
            challenge.verification_uri, challenge.user_code
        );
    }
    let device_code = challenge.device_code.clone();
    let mut session = DeviceAuthorizationSession::new(
        &request,
        challenge,
        chrono::Utc::now().timestamp_millis(),
        180,
    )
    .context("initialize device authorization session")?;
    loop {
        let now_ms = chrono::Utc::now().timestamp_millis();
        match session
            .poll_action(now_ms)
            .context("evaluate device authorization poll")?
        {
            PollAction::Wait { until_unix_ms } => {
                let delay = until_unix_ms.saturating_sub(now_ms) as u64;
                tokio::time::sleep(Duration::from_millis(delay.min(60_000))).await;
            }
            PollAction::Poll => {
                let response: DeviceCodePollResponse = client
                    .poll(&DeviceCodePollRequest {
                        request_id: request.request_id,
                        device_code: device_code.clone(),
                    })
                    .await
                    .context("poll device authorization")?;
                session
                    .observe_poll(response, chrono::Utc::now().timestamp_millis())
                    .context("apply device authorization response")?;
            }
            PollAction::Terminal => break,
        }
    }
    if session.status() != DeviceAuthorizationStatus::Authorized {
        bail!(
            "E_AUTHORITY_DEVICE_DENIED: authority device authorization ended without an issued lease"
        );
    }
    let material = session
        .material()
        .ok_or_else(|| anyhow!("authority omitted authorized lease material"))?;
    let key_set_raw = material.key_set_envelope.as_deref().ok_or_else(|| {
        anyhow!("E_AUTHORITY_KEYSET_MISSING: authority omitted signed key-set envelope")
    })?;
    let key_set: SignedEnvelope =
        serde_json::from_str(key_set_raw).context("decode authority key-set envelope")?;
    let lease: SignedEnvelope =
        serde_json::from_str(&material.signed_lease).context("decode authority lease envelope")?;
    let context = LeaseVerificationContext {
        expected_product: "focusa".into(),
        expected_node_id: identity.node_id.clone(),
        now: chrono::Utc::now(),
        minimum_sequence: None,
        expected_previous_digest: None,
    };
    let roots = embedded_production_trust_roots().context("load production authority roots")?;
    let (state, snapshot) =
        PersistedAuthorityState::from_verified_envelopes(key_set, lease, &roots, &context)
            .context("verify issued authority lease")?;
    if !matches!(
        snapshot.state,
        EntitlementState::Active | EntitlementState::OfflineGrace
    ) || !snapshot
        .features
        .get(required_feature)
        .copied()
        .unwrap_or(false)
    {
        bail!("E_AUTHORITY_LEASE_UNUSABLE: issued lease does not grant {required_feature}");
    }
    let handle = CredentialHandle::for_node("focusa", &identity.node_id)
        .context("derive protected refresh-credential handle")?;
    rotate_refresh_credential(
        &KeyringCredentialStore,
        &handle,
        &material.refresh_credential,
        chrono::Utc::now(),
    )
    .context("persist refresh credential in native protected storage")?;
    state
        .write_atomic(&config_dir.join(AUTHORITY_STATE_FILE))
        .context("persist verified authority state")?;
    Ok(())
}

/// Spec 152E §21 + Spec 172 §2.7: the Rust installer renders the universal
/// activation flow (email → verify → offer → checkout/poll → key/lease,
/// existing key, verified-email limited access via the Spec 172
/// limited-access overlay, resume, cancel, timeout, recovery) through the
/// shared activation client when an interactive terminal is available.
/// `--eval` maps to limited-access intent; the authority decides eligibility
/// and no client-side Evaluation or local grant exists. Terminal delivery
/// persists the verified signed lease through the canonical authority store
/// and the poll credential through the protected store. Card data is never
/// accepted and nothing is self-issued.
async fn authorize_installer_activation_flow(
    config_dir: &std::path::Path,
    args: &InstallArgs,
    channel: Channel,
) -> Result<()> {
    use crate::commands::activation_flow::{
        ActivationFlowSessionPersist, INSTALLER_FLOW, StdinFlowInput, interactive_available,
        resolve_flow_node_identity, run_activation_flow,
    };
    use focusa_license::{ActivationHttpClient, ActivationHttpPolicy};

    if !interactive_available() {
        bail!(
            "E_AUTHORITY_INTERACTIVE_REQUIRED: universal activation flow needs an interactive terminal; use device-code authorization for noninteractive installs"
        );
    }
    let identity =
        resolve_flow_node_identity(config_dir).map_err(|error| anyhow!(error.to_string()))?;
    let origin = std::env::var("FOCUSA_AUTHORITY_ORIGIN")
        .unwrap_or_else(|_| "https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/".to_string());
    let base_url = reqwest::Url::parse(&origin).context("parse FOCUSA_AUTHORITY_ORIGIN")?;
    let policy = ActivationHttpPolicy {
        base_url,
        timeout: Duration::from_secs(30),
        max_response_bytes: 1024 * 1024,
    };
    let client = ActivationHttpClient::new(policy)
        .map_err(|error| anyhow!("initialize activation authority transport: {error}"))?;
    let persist = ActivationFlowSessionPersist::new(config_dir);
    let mut input = StdinFlowInput;
    let outcome = run_activation_flow(
        client,
        INSTALLER_FLOW,
        &mut input,
        None,
        Some(identity.node_id.clone()),
        if args.eval {
            Some(focusa_license::ActivationJourney::LimitedAccess)
        } else {
            None
        },
        Some(600),
        args.json,
        Some(&persist),
    )?;
    if !outcome.terminal || outcome.presenter_state != "activated" {
        bail!(
            "E_AUTHORITY_ACTIVATION_UNSETTLED: interactive activation settled as {} without a usable lease; recovery, export, repair, and uninstall remain available",
            outcome.presenter_state
        );
    }
    let _ = channel; // channel grants are validated by phase_license after this.
    Ok(())
}

fn install_channel_feature(channel: Channel) -> &'static str {
    match channel {
        Channel::Stable => "focusa.install.channel.stable",
        Channel::Preview => "focusa.install.channel.preview",
        Channel::Nightly => "focusa.install.channel.nightly",
    }
}

fn entitlement_state_label(state: EntitlementState) -> &'static str {
    match state {
        EntitlementState::Unactivated => "unactivated",
        EntitlementState::Active => "active",
        EntitlementState::OfflineGrace => "offline_grace",
        EntitlementState::RecoveryOnly => "recovery_only",
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

fn release_tag(channel: Channel, override_tag: Option<&str>) -> Result<String> {
    let selected = override_tag
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(str::to_string)
        .or_else(|| {
            std::env::var("FOCUSA_RELEASE_TAG")
                .ok()
                .map(|tag| tag.trim().to_string())
                .filter(|tag| !tag.is_empty())
        })
        .unwrap_or_else(|| match channel {
            Channel::Stable => format!("v{}", env!("CARGO_PKG_VERSION")),
            Channel::Preview => format!("v{}-preview", env!("CARGO_PKG_VERSION")),
            Channel::Nightly => format!("v{}-nightly", env!("CARGO_PKG_VERSION")),
        });
    validate_release_tag(channel, &selected)?;
    Ok(selected)
}

pub(crate) fn validate_release_tag(channel: Channel, tag: &str) -> Result<()> {
    let body = tag
        .strip_prefix('v')
        .ok_or_else(|| anyhow!("release tag must start with v"))?;
    let valid_numeric =
        |value: &str| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit());
    let core_valid = |core: &str| {
        let parts = core.split('.').collect::<Vec<_>>();
        parts.len() == 3 && parts.into_iter().all(valid_numeric)
    };
    let valid = match channel {
        Channel::Stable => core_valid(body),
        Channel::Preview => body.split_once('-').is_some_and(|(core, suffix)| {
            core_valid(core)
                && (suffix == "preview"
                    || suffix == "dev"
                    || suffix.starts_with("dev.")
                    || suffix == "rc"
                    || suffix.starts_with("rc."))
        }),
        Channel::Nightly => body.split_once('-').is_some_and(|(core, suffix)| {
            core_valid(core) && (suffix == "nightly" || suffix.starts_with("nightly."))
        }),
    };
    if !valid {
        bail!("release tag {tag} is invalid for {:?} channel", channel);
    }
    Ok(())
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

fn bind_resolved_release_tag(
    channel: Channel,
    requested_tag: &str,
    release: &serde_json::Value,
) -> Result<String> {
    let remote_tag = release
        .get("tag_name")
        .and_then(|value| value.as_str())
        .ok_or_else(|| anyhow!("GitHub release response omitted tag_name"))?;
    if remote_tag != requested_tag {
        bail!("GitHub release identity mismatch for requested tag");
    }
    validate_release_tag(channel, remote_tag)?;
    Ok(remote_tag.to_string())
}

async fn resolve_release(
    channel: Channel,
    github_repo: &str,
    release_tag_override: Option<&str>,
) -> Result<ResolvedRelease> {
    let tag = release_tag(channel, release_tag_override)?;
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
        bind_resolved_release_tag(channel, &tag, &release)?
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
    release_tag_override: Option<&str>,
    install_root: &std::path::Path,
    sink: &dyn InstallEventSink,
    cancellation: &CancellationToken,
) -> Result<Vec<InstalledAsset>> {
    sink.emit(InstallEvent::PhaseStarted {
        phase: InstallPhase::DownloadAssets,
        message: "streaming assets to staged files".into(),
    });
    let repo = github_repo.unwrap_or("Startempire-Wire/focusa");
    let release = resolve_release(channel, repo, release_tag_override).await?;
    let tag_name = release.tag;
    let client = release.client;
    let triple = triple_for(target);
    let assets = CANONICAL_RELEASE_BINARIES;
    let mut out = Vec::new();
    let executable_suffix = release_executable_suffix(target);
    for asset_name in assets {
        let expected = format!("{asset_name}-{tag_name}-{triple}{executable_suffix}");
        let install_path = install_root
            .join("bin")
            .join(installed_binary_name(target, asset_name));
        std::fs::create_dir_all(install_path.parent().expect("bin parent"))?;
        reject_release_rollback(install_root, &tag_name)?;
        let staged = install_path.with_extension("download");
        let asset_url = release_asset_url(repo, &tag_name, &expected);
        let existing_mode = std::fs::metadata(&install_path)
            .ok()
            .map(|metadata| file_mode(&metadata));
        download_asset_with_retry(&client, &asset_url, &staged, &expected, sink, cancellation)
            .await?;
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
    release_tag_override: Option<&str>,
    install_root: &std::path::Path,
    sink: &dyn InstallEventSink,
    cancellation: &CancellationToken,
) -> Result<Option<InstalledAsset>> {
    if which::which("pi").is_err() {
        return Ok(None);
    }
    let repo = github_repo.unwrap_or("Startempire-Wire/focusa");
    let release = resolve_release(channel, repo, release_tag_override).await?;
    let name = format!("focusa-pi-extension-{}.tar.gz", release.tag);
    let share = install_root.join("share");
    std::fs::create_dir_all(&share)?;
    let install_path = share.join(&name);
    let staged = install_path.with_extension("download");
    let url = release_asset_url(repo, &release.tag, &name);
    download_asset_with_retry(&release.client, &url, &staged, &name, sink, cancellation).await?;
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
    release_tag_override: Option<&str>,
    install_root: &std::path::Path,
    sink: &dyn InstallEventSink,
    cancellation: &CancellationToken,
) -> Result<InstalledAsset> {
    let repo = github_repo.unwrap_or("Startempire-Wire/focusa");
    let tag = release_tag(channel, release_tag_override)?;
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
    download_asset_with_retry(&client, &url, &staged, &name, sink, cancellation).await?;
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

async fn download_asset_with_retry(
    client: &reqwest::Client,
    url: &str,
    staged: &std::path::Path,
    label: &str,
    sink: &dyn InstallEventSink,
    cancellation: &CancellationToken,
) -> Result<()> {
    let mut last_error = None;
    for attempt in 1_u64..=5 {
        ensure_not_cancelled(cancellation)?;
        let result = async {
            let response = client
                .get(url)
                .send()
                .await
                .with_context(|| format!("download {label} from {}", redact_url(url)))?
                .error_for_status()
                .with_context(|| format!("download {label} from {}", redact_url(url)))?;
            stream_asset_to_staged(response, staged, label, sink, cancellation).await
        }
        .await;
        match result {
            Ok(()) => return Ok(()),
            Err(error) if cancellation.is_cancelled() => return Err(error),
            Err(error) => {
                last_error = Some(error);
                if attempt < 5 {
                    eprintln!(
                        "warning: transient download failure for {label}; retrying ({attempt}/5)"
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(attempt * 2)).await;
                }
            }
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow!("download {label} failed without an error")))
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

fn tar_command() -> std::process::Command {
    if cfg!(windows) {
        return std::process::Command::new("tar");
    }
    let binary = ["/usr/bin/tar", "/bin/tar"]
        .into_iter()
        .find(|path| std::path::Path::new(path).is_file())
        .unwrap_or("tar");
    let inherited = std::env::var("PATH").unwrap_or_default();
    let mut command = std::process::Command::new(binary);
    command.env(
        "PATH",
        format!("/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:{inherited}"),
    );
    command
}

fn resolve_npm_binary(explicit: Option<&std::path::Path>) -> Result<std::path::PathBuf> {
    let candidate = explicit
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("FOCUSA_NPM_BIN").map(std::path::PathBuf::from))
        .or_else(|| find_command("npm").map(std::path::PathBuf::from))
        .or_else(|| {
            find_command("node").and_then(|node| {
                std::path::Path::new(&node)
                    .parent()
                    .map(|parent| parent.join(if cfg!(windows) { "npm.cmd" } else { "npm" }))
                    .filter(|path| path.is_file())
            })
        })
        .ok_or_else(|| {
            anyhow!("npm executable unavailable; set FOCUSA_NPM_BIN to the absolute npm path")
        })?;
    let candidate = if candidate.is_absolute() {
        candidate
    } else {
        std::env::current_dir()
            .context("resolve current directory for npm executable")?
            .join(candidate)
    };
    if !candidate.is_file() {
        bail!(
            "resolve npm executable {} (exists={})",
            candidate.display(),
            candidate.exists()
        );
    }
    // Preserve the launcher/symlink path: its parent commonly contains `node`.
    Ok(candidate)
}

fn rename_pi_extension_path(source: &std::path::Path, destination: &std::path::Path) -> Result<()> {
    const WINDOWS_LOCK_RETRIES: usize = 120;
    for attempt in 0..WINDOWS_LOCK_RETRIES {
        match std::fs::rename(source, destination) {
            Ok(()) => return Ok(()),
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
    unreachable!("bounded Pi extension rename loop always returns")
}

fn retired_focusa_pi_extension_name(name: &str) -> bool {
    [
        "focusa.legacy-",
        "focusa-runtime.legacy-",
        "focusa-pi-bridge.legacy-",
    ]
    .iter()
    .any(|prefix| name.starts_with(prefix))
}

fn focusa_pi_bridge_package(path: &std::path::Path) -> bool {
    std::fs::read(path.join("package.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|value| value.get("name")?.as_str().map(str::to_string))
        .is_some_and(|name| name == "focusa-pi-bridge")
}

/// Move retired Focusa extension packages outside Pi's auto-discovery root.
///
/// A backup name such as `focusa-runtime.legacy-0.9.143` does not disable the
/// package: Pi sees its `pi.extensions` manifest and registers every tool a
/// second time. Preserve verified Focusa packages for recovery, but never
/// leave them under `~/.pi/agent/extensions` where they can break startup.
fn quarantine_retired_focusa_pi_extensions(
    extensions_root: &std::path::Path,
) -> Result<Vec<std::path::PathBuf>> {
    if !extensions_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut retired = Vec::new();
    for entry in std::fs::read_dir(extensions_root)
        .with_context(|| format!("inspect Pi extension root {}", extensions_root.display()))?
    {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let candidate = entry.path();
        if !retired_focusa_pi_extension_name(name) || !focusa_pi_bridge_package(&candidate) {
            continue;
        }
        let retired_root = extensions_root
            .parent()
            .unwrap_or(extensions_root)
            .join("retired-extensions");
        std::fs::create_dir_all(&retired_root).with_context(|| {
            format!(
                "create retired Pi extension root {}",
                retired_root.display()
            )
        })?;
        let destination = retired_root.join(format!("{name}-{}", uuid::Uuid::now_v7()));
        rename_pi_extension_path(&candidate, &destination).with_context(|| {
            format!(
                "quarantine retired Focusa extension {} outside Pi auto-discovery",
                candidate.display()
            )
        })?;
        retired.push(destination);
    }
    Ok(retired)
}

pub(crate) fn integrate_pi_extension(
    asset: &InstalledAsset,
    install_root: &std::path::Path,
    destination_root: Option<&std::path::Path>,
    npm_binary: Option<&std::path::Path>,
) -> Result<String> {
    let archive = std::path::Path::new(&asset.install_path);
    let listing = tar_command()
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
    let extracted = tar_command()
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
    let staged_candidate = stage_root.join("pi-extension");
    let staged = match staged_candidate.canonicalize() {
        Ok(path) => path,
        Err(error) => {
            cleanup();
            bail!(
                "resolve extracted Pi extension directory {} (exists={}): {error}",
                staged_candidate.display(),
                staged_candidate.exists()
            );
        }
    };
    let npm_binary = match resolve_npm_binary(npm_binary) {
        Ok(path) => path,
        Err(error) => {
            cleanup();
            return Err(error);
        }
    };
    let npm_parent = npm_binary
        .parent()
        .unwrap_or_else(|| std::path::Path::new(""));
    let command_path = match std::env::join_paths(
        std::iter::once(npm_parent.to_path_buf()).chain(
            std::env::var_os("PATH")
                .as_deref()
                .map(std::env::split_paths)
                .into_iter()
                .flatten(),
        ),
    ) {
        Ok(path) => path,
        Err(error) => {
            cleanup();
            return Err(error).context("construct npm dependency PATH");
        }
    };
    let npm = match std::process::Command::new(&npm_binary)
        .args(["install", "--omit=dev", "--ignore-scripts"])
        .env("PATH", command_path)
        .current_dir(&staged)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            cleanup();
            bail!(
                "run npm dependency setup: executable={} executable_exists={} cwd={} cwd_exists={} cause={error}",
                npm_binary.display(),
                npm_binary.exists(),
                staged.display(),
                staged.is_dir()
            );
        }
    };
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
    let root = destination_root
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("FOCUSA_PI_EXT_DIR").map(std::path::PathBuf::from))
        .or_else(|| {
            std::env::var_os("HOME")
                .map(|home| std::path::PathBuf::from(home).join(".pi/agent/extensions"))
        })
        .ok_or_else(|| anyhow!("HOME is unavailable; cannot locate Pi extensions"))?;
    std::fs::create_dir_all(&root)
        .with_context(|| format!("create Pi extension root {}", root.display()))?;
    quarantine_retired_focusa_pi_extensions(&root)
        .context("quarantine retired Focusa Pi extension packages")?;
    let destination = root.join("focusa");
    let backup = root.join(format!(".focusa-backup-{}", uuid::Uuid::now_v7()));
    if destination.exists() {
        rename_pi_extension_path(&destination, &backup)
            .with_context(|| format!("backup active Pi extension {}", destination.display()))?;
    }
    if let Err(error) = rename_pi_extension_path(&staged, &destination) {
        if backup.exists() {
            let _ = rename_pi_extension_path(&backup, &destination);
        }
        cleanup();
        return Err(error).with_context(|| {
            format!(
                "activate Pi extension {} from {}",
                destination.display(),
                staged.display()
            )
        });
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
    let listing = tar_command()
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
    let extraction = tar_command()
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

fn remove_path_if_present(path: &std::path::Path) -> Result<()> {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return Ok(());
    };
    if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

fn copy_skill_tree(source: &std::path::Path, destination: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target = destination.join(entry.file_name());
        if file_type.is_dir() {
            copy_skill_tree(&entry.path(), &target)?;
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), target)?;
        } else {
            bail!("agent context skill contains unsupported link or special file");
        }
    }
    Ok(())
}

fn synchronize_agent_context_skills(
    context_root: &std::path::Path,
    home: &std::path::Path,
) -> Result<Vec<std::path::PathBuf>> {
    let canonical_root = context_root.join("skills");
    let mut skills = std::fs::read_dir(&canonical_root)?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter(|entry| entry.path().join("SKILL.md").is_file())
        .collect::<Vec<_>>();
    skills.sort_by_key(|entry| entry.file_name());
    if skills.is_empty() {
        bail!("agent context has no synchronizable skills");
    }
    if skills.iter().any(|entry| {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        !name.starts_with("focusa") && name != "predictive-power"
    }) {
        bail!("agent context skill synchronization is limited to Focusa-owned names");
    }

    // Pi discovers ~/.pi/agent/skills natively. Keep exactly one canonical
    // destination and reconcile only Focusa-owned names from the legacy root;
    // unrelated user skills remain untouched.
    let canonical_root = home.join(".pi/agent/skills");
    let legacy_root = home.join(".pi/skills");
    let transaction = uuid::Uuid::now_v7();
    let mut activated = Vec::<(std::path::PathBuf, Option<std::path::PathBuf>)>::new();
    let mut reconciled_legacy = Vec::<(std::path::PathBuf, std::path::PathBuf)>::new();
    let result = (|| -> Result<Vec<std::path::PathBuf>> {
        std::fs::create_dir_all(&canonical_root)?;
        for skill in &skills {
            let name = skill.file_name();
            let destination = canonical_root.join(&name);
            let stage = canonical_root.join(format!(
                ".focusa-skill-stage-{transaction}-{}",
                name.to_string_lossy()
            ));
            let backup = canonical_root.join(format!(
                ".focusa-skill-backup-{transaction}-{}",
                name.to_string_lossy()
            ));
            remove_path_if_present(&stage)?;
            copy_skill_tree(&skill.path(), &stage)?;
            let prior = if destination.exists() || destination.is_symlink() {
                std::fs::rename(&destination, &backup)?;
                Some(backup)
            } else {
                None
            };
            if let Err(error) = std::fs::rename(&stage, &destination) {
                if let Some(backup) = prior.as_ref() {
                    let _ = std::fs::rename(backup, &destination);
                }
                let _ = remove_path_if_present(&stage);
                return Err(error).context("activate synchronized Focusa skill");
            }
            activated.push((destination, prior));
        }

        if legacy_root.is_dir() {
            for skill in &skills {
                let name = skill.file_name();
                let legacy = legacy_root.join(&name);
                if !legacy.exists() && !legacy.is_symlink() {
                    continue;
                }
                let backup = legacy_root.join(format!(
                    ".focusa-skill-legacy-backup-{transaction}-{}",
                    name.to_string_lossy()
                ));
                std::fs::rename(&legacy, &backup)?;
                reconciled_legacy.push((legacy, backup));
            }
        }

        Ok(activated
            .iter()
            .map(|(destination, _)| destination.clone())
            .collect())
    })();

    let destinations = match result {
        Ok(destinations) => destinations,
        Err(error) => {
            for (legacy, backup) in reconciled_legacy.into_iter().rev() {
                let _ = std::fs::rename(backup, legacy);
            }
            for (destination, backup) in activated.into_iter().rev() {
                let _ = remove_path_if_present(&destination);
                if let Some(backup) = backup {
                    let _ = std::fs::rename(backup, destination);
                }
            }
            return Err(error);
        }
    };
    for (_, backup) in &activated {
        if let Some(backup) = backup {
            remove_path_if_present(backup)?;
        }
    }
    for (_, backup) in &reconciled_legacy {
        remove_path_if_present(backup)?;
    }
    Ok(destinations)
}

fn install_skill_doctor(
    context_root: &std::path::Path,
    install_root: &std::path::Path,
) -> Result<()> {
    let source = context_root.join("bin/focusa-skill-doctor");
    if !source.is_file() {
        return Ok(());
    }
    let destination = install_root.join("bin/focusa-skill-doctor");
    std::fs::create_dir_all(install_root.join("bin"))?;
    std::fs::copy(&source, &destination)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&destination, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
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
fn place_symlinks(
    target: InstallTarget,
    bin_dir: &std::path::Path,
    _install_root: &std::path::Path,
) -> Result<()> {
    if matches!(
        target,
        InstallTarget::WindowsX64 | InstallTarget::WindowsArm64
    ) {
        return Ok(());
    }
    std::fs::create_dir_all(bin_dir).with_context(|| format!("create {}", bin_dir.display()))?;
    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .ok_or_else(|| anyhow!("HOME not set"))?;
    let local_bin = home.join(".local/bin");
    for bin in CANONICAL_RELEASE_BINARIES {
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

#[cfg(unix)]
struct SystemLinkEntry {
    system_path: std::path::PathBuf,
    system_backup: std::path::PathBuf,
    had_system_original: bool,
    local_path: std::path::PathBuf,
    local_backup: std::path::PathBuf,
    local_swapped: bool,
}

#[cfg(unix)]
fn rollback_system_links(entries: &[SystemLinkEntry]) -> Result<()> {
    let mut failures = Vec::new();
    for entry in entries.iter().rev() {
        if entry.local_swapped {
            if let Err(error) = std::fs::remove_file(&entry.local_path)
                && error.kind() != std::io::ErrorKind::NotFound
            {
                failures.push(format!("remove {}: {error}", entry.local_path.display()));
            }
            if let Err(error) = std::fs::rename(&entry.local_backup, &entry.local_path) {
                failures.push(format!(
                    "restore {} from {}: {error}",
                    entry.local_path.display(),
                    entry.local_backup.display()
                ));
            }
        }
        if let Err(error) = std::fs::remove_file(&entry.system_path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            failures.push(format!("remove {}: {error}", entry.system_path.display()));
        }
        if entry.had_system_original
            && let Err(error) = std::fs::rename(&entry.system_backup, &entry.system_path)
        {
            failures.push(format!(
                "restore {} from {}: {error}",
                entry.system_path.display(),
                entry.system_backup.display()
            ));
        }
    }
    if !failures.is_empty() {
        bail!("system link rollback failed: {}", failures.join("; "));
    }
    Ok(())
}

#[cfg(unix)]
fn error_after_system_rollback(error: anyhow::Error, entries: &[SystemLinkEntry]) -> anyhow::Error {
    match rollback_system_links(entries) {
        Ok(()) => error,
        Err(rollback_error) => error.context(rollback_error.to_string()),
    }
}

#[cfg(unix)]
fn system_daemon_active() -> Result<bool> {
    let output = std::process::Command::new("systemctl")
        .args(["is-active", "--quiet", "focusa-daemon.service"])
        .output()
        .context("inspect authoritative system daemon state")?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(3 | 4) => Ok(false),
        _ => {
            let detail = String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(240)
                .collect::<String>();
            bail!(
                "inspect authoritative system daemon failed: exit={}{}",
                output.status.code().unwrap_or(-1),
                if detail.trim().is_empty() {
                    String::new()
                } else {
                    format!(" ({})", detail.trim())
                }
            )
        }
    }
}

#[cfg(unix)]
fn restart_system_daemon() -> Result<()> {
    let output = std::process::Command::new("systemctl")
        .args(["restart", "focusa-daemon.service"])
        .output()
        .context("restart authoritative system daemon")?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr)
            .chars()
            .take(240)
            .collect::<String>();
        bail!(
            "restart authoritative system daemon failed: exit={}{}",
            output.status.code().unwrap_or(-1),
            if detail.trim().is_empty() {
                String::new()
            } else {
                format!(" ({})", detail.trim())
            }
        );
    }
    if !system_daemon_active()? {
        bail!("authoritative system daemon is not active after restart");
    }
    Ok(())
}

#[cfg(unix)]
fn promote_system_links(
    bin_dir: &std::path::Path,
    system_bin: &std::path::Path,
    expected_tag: &str,
    restart_service: bool,
) -> Result<bool> {
    std::fs::create_dir_all(system_bin)
        .with_context(|| format!("create authoritative system path {}", system_bin.display()))?;
    let transaction = format!("{}", std::process::id());
    let mut entries: Vec<SystemLinkEntry> = Vec::new();
    for name in CANONICAL_RELEASE_BINARIES {
        let local_path = bin_dir.join(name);
        if !local_path.is_file() {
            return Err(error_after_system_rollback(
                anyhow!(
                    "verified system promotion target is missing: {}",
                    local_path.display()
                ),
                &entries,
            ));
        }
        let system_path = system_bin.join(name);
        let system_backup = system_bin.join(format!(".focusa-{name}.rollback-{transaction}"));
        let system_staged = system_bin.join(format!(".focusa-{name}.staged-{transaction}"));
        let local_backup = bin_dir.join(format!(".{name}.promoted-{transaction}"));
        let local_staged = bin_dir.join(format!(".{name}.system-link-{transaction}"));
        if system_backup.exists()
            || system_staged.exists()
            || local_backup.exists()
            || local_staged.exists()
        {
            return Err(error_after_system_rollback(
                anyhow!("stale system promotion transaction exists for {name}"),
                &entries,
            ));
        }
        let had_system_original = std::fs::symlink_metadata(&system_path).is_ok();
        if had_system_original && let Err(error) = std::fs::rename(&system_path, &system_backup) {
            return Err(error_after_system_rollback(
                anyhow!(error).context(format!("stash authoritative {}", system_path.display())),
                &entries,
            ));
        }
        entries.push(SystemLinkEntry {
            system_path: system_path.clone(),
            system_backup,
            had_system_original,
            local_path: local_path.clone(),
            local_backup: local_backup.clone(),
            local_swapped: false,
        });
        if let Err(error) = std::fs::copy(&local_path, &system_staged)
            .with_context(|| format!("stage authoritative {}", system_path.display()))
            .and_then(|_| {
                std::fs::rename(&system_staged, &system_path)
                    .with_context(|| format!("promote authoritative {}", system_path.display()))
            })
        {
            let _ = std::fs::remove_file(&system_staged);
            return Err(error_after_system_rollback(error, &entries));
        }
        if let Err(error) = std::fs::rename(&local_path, &local_backup)
            .with_context(|| format!("stash promoted local {}", local_path.display()))
        {
            return Err(error_after_system_rollback(error, &entries));
        }
        entries.last_mut().expect("promotion entry").local_swapped = true;
        if let Err(error) = create_symlink(&system_path, &local_staged).and_then(|()| {
            std::fs::rename(&local_staged, &local_path)
                .with_context(|| format!("link local install to {}", system_path.display()))
        }) {
            let _ = std::fs::remove_file(&local_staged);
            return Err(error_after_system_rollback(error, &entries));
        }
    }
    let expected_version = expected_tag.strip_prefix('v').unwrap_or(expected_tag);
    for name in CANONICAL_RELEASE_BINARIES {
        let smoke = std::process::Command::new(system_bin.join(name))
            .arg("--version")
            .output();
        let valid = smoke.as_ref().is_ok_and(|output| {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout)
                    .split_whitespace()
                    .any(|part| part == expected_version)
        });
        if !valid {
            return Err(error_after_system_rollback(
                anyhow!("authoritative system {name} --version did not report {expected_version}"),
                &entries,
            ));
        }
    }
    let service_restarted = if restart_service {
        match system_daemon_active() {
            Ok(true) => {
                if let Err(restart_error) = restart_system_daemon() {
                    let rollback_error = rollback_system_links(&entries).err();
                    let restore_error = restart_system_daemon().err();
                    return Err(restart_error).context(format!(
                        "new daemon restart failed; system rollback={}; prior daemon restart={}",
                        rollback_error
                            .map(|error| error.to_string())
                            .unwrap_or_else(|| "ok".into()),
                        restore_error
                            .map(|error| error.to_string())
                            .unwrap_or_else(|| "ok".into())
                    ));
                }
                true
            }
            Ok(false) => false,
            Err(error) => return Err(error_after_system_rollback(error, &entries)),
        }
    } else {
        false
    };
    for entry in &entries {
        if entry.had_system_original
            && let Err(error) = std::fs::remove_file(&entry.system_backup)
        {
            eprintln!(
                "warning: committed system promotion retained rollback {}: {error}",
                entry.system_backup.display()
            );
        }
        if let Err(error) = std::fs::remove_file(&entry.local_backup) {
            eprintln!(
                "warning: committed system promotion retained local staging {}: {error}",
                entry.local_backup.display()
            );
        }
    }
    Ok(service_restarted)
}

#[cfg(not(unix))]
fn promote_system_links(
    _bin_dir: &std::path::Path,
    _system_bin: &std::path::Path,
    _expected_tag: &str,
    _restart_service: bool,
) -> Result<bool> {
    bail!("authoritative system installation is unsupported on this platform")
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
const LEGACY_PATH_MARKER_BEGIN: &str = "# >>> focusa PATH >>>";
const LEGACY_PATH_MARKER_END: &str = "# <<< focusa PATH <<<";

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
    let legacy_block =
        format!("{LEGACY_PATH_MARKER_BEGIN}\n{path_line}\n{LEGACY_PATH_MARKER_END}\n");
    if content.contains(PATH_MARKER_BEGIN) && content.contains(PATH_MARKER_END) {
        // Remove the prior bootstrapper's equivalent block when both formats
        // exist, otherwise repeated repair installs duplicate the PATH entry.
        if content.contains(&legacy_block) {
            std::fs::write(rc, content.replace(&legacy_block, ""))
                .with_context(|| format!("migrate legacy PATH block in {}", rc.display()))?;
        }
        return Ok(());
    }
    if content.contains(&legacy_block) {
        std::fs::write(rc, content.replace(&legacy_block, &block))
            .with_context(|| format!("migrate legacy PATH block in {}", rc.display()))?;
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
    let binary = bin_dir.join(installed_binary_name(target, "focusa"));
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
    let pi_extensions_root = std::env::var_os("FOCUSA_PI_EXT_DIR")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(|home| std::path::PathBuf::from(home).join(".pi/agent/extensions"))
        });
    let focusa_pi_extension = pi_extensions_root.as_ref().map(|root| root.join("focusa"));
    let pi_detected = dependency_present("pi");
    let pi_integrated = focusa_pi_extension
        .as_ref()
        .is_some_and(|root| root.join("package.json").is_file());
    let uiai_url = uiai_engine_url();
    let uiai_healthy = uiai_engine_healthy();
    let _ = target;
    let _ = channel;
    let _ = asset_count;
    FirstInstallWalkthrough {
        version: env!("CARGO_PKG_VERSION").to_string(),
        environment_summary: summary,
        next_steps,
        agent_integrations: vec![
            {
                let context_root = install_root.join("agent-context");
                let integrated = context_root.join("AGENTS.md").is_file()
                    && context_root.join("skills").is_dir();
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
            },
            AgentIntegration {
                agent: "pi-coding-agent".to_string(),
                detected: pi_detected,
                integrated: pi_detected && pi_integrated,
                config_path: focusa_pi_extension
                    .as_ref()
                    .map(|path| path.display().to_string()),
                next_step: Some("pi --version && pi --no-session -p \"Reply with Focusa ready\"".into()),
                expected_outcome: Some(
                    "Pi starts with the checksum-verified bundled Focusa extension active".into(),
                ),
                recovery_hint: Some(
                    "Run focusa install --install-dependencies --assume-yes; then verify the managed Focusa extension package and reload Pi"
                        .into(),
                ),
            },
            AgentIntegration {
                agent: "uiai-engine".to_string(),
                detected: uiai_healthy,
                integrated: uiai_healthy,
                config_path: Some(uiai_url.clone()),
                next_step: Some(format!(
                    "curl --fail --silent {}/health",
                    uiai_url.trim_end_matches('/')
                )),
                expected_outcome: Some("UIAI Engine returns a healthy JSON envelope".into()),
                recovery_hint: Some(
                    "On Linux/amd64 rerun dependency installation for the pinned engine; on other platforms set UIAI_ENGINE_URL to a healthy private endpoint"
                        .into(),
                ),
            },
        ],
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
    println!("[ agent workflow readiness ]");
    for integration in &walkthrough.agent_integrations {
        println!(
            "  {}: detected={} integrated={}\n     next: {}\n     recovery: {}",
            integration.agent,
            integration.detected,
            integration.integrated,
            integration.next_step.as_deref().unwrap_or("—"),
            integration.recovery_hint.as_deref().unwrap_or("—"),
        );
    }
    println!("\nHint: for LLM agents, GET /llms.txt on the daemon serves the canonical primer.");
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

fn phase_atomic_recover(
    install_root: &std::path::Path,
    stash: &std::path::Path,
    stashed: bool,
) -> Result<()> {
    if stashed {
        phase_atomic_rollback(install_root, stash)
    } else {
        remove_existing_path(install_root)
            .with_context(|| format!("remove failed fresh install {}", install_root.display()))
    }
}

fn installer_managed_entry(name: &str) -> bool {
    matches!(
        name,
        "bin"
            | "libexec"
            | "share"
            | "agent-context"
            | ".focusa-version"
            | "install-manifest.json"
            | "install-metadata.json"
    ) || name.starts_with(".pi-extension-stage-")
        || name.starts_with(".agent-context-stage-")
        || name.starts_with(".agent-context-backup-")
}

fn remove_existing_path(path: &std::path::Path) -> Result<()> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error).with_context(|| format!("inspect {}", path.display())),
    };
    if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

fn copy_customer_entry(source: &std::path::Path, destination: &std::path::Path) -> Result<()> {
    let metadata = std::fs::symlink_metadata(source)
        .with_context(|| format!("inspect preserved data {}", source.display()))?;
    if metadata.file_type().is_symlink() {
        remove_existing_path(destination)?;
        let target = std::fs::read_link(source)?;
        #[cfg(unix)]
        std::os::unix::fs::symlink(target, destination)?;
        #[cfg(windows)]
        {
            if source.is_dir() {
                std::os::windows::fs::symlink_dir(target, destination)?;
            } else {
                std::os::windows::fs::symlink_file(target, destination)?;
            }
        }
        return Ok(());
    }
    if metadata.is_dir() {
        std::fs::create_dir_all(destination)?;
        for entry in std::fs::read_dir(source)? {
            let entry = entry?;
            copy_customer_entry(&entry.path(), &destination.join(entry.file_name()))?;
        }
        return Ok(());
    }
    remove_existing_path(destination)?;
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(source, destination).with_context(|| {
        format!(
            "preserve customer data {} -> {}",
            source.display(),
            destination.display()
        )
    })?;
    Ok(())
}

fn phase_restore_customer_data(
    stash: &std::path::Path,
    install_root: &std::path::Path,
) -> Result<()> {
    for entry in std::fs::read_dir(stash)
        .with_context(|| format!("read prior install stash {}", stash.display()))?
    {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if installer_managed_entry(&name) {
            continue;
        }
        copy_customer_entry(&entry.path(), &install_root.join(&name))?;
    }
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

/// Smoke test every canonical binary before install commit. Each producer must
/// expose `--version`, so a missing, stale, or non-runnable fourth binary fails
/// the same atomic rollback boundary as the CLI.
async fn phase_smoke_test(
    target: InstallTarget,
    bin_dir: &std::path::Path,
    expected_tag: &str,
) -> Result<()> {
    let expected_version = expected_tag.strip_prefix('v').unwrap_or(expected_tag);
    for name in CANONICAL_RELEASE_BINARIES {
        let binary = bin_dir.join(installed_binary_name(target, name));
        if !binary.exists() {
            return Err(anyhow!(
                "smoke test failed: {name} binary not present at {}",
                binary.display()
            ));
        }
        match std::process::Command::new(&binary)
            .arg("--version")
            .output()
        {
            Ok(output)
                if output.status.success()
                    && String::from_utf8_lossy(&output.stdout)
                        .split_whitespace()
                        .any(|part| part == expected_version) => {}
            Ok(output) => {
                let detail: String = String::from_utf8_lossy(&output.stderr)
                    .chars()
                    .take(240)
                    .collect();
                return Err(anyhow!(
                    "smoke test failed: {name} --version did not report {expected_version}; exit={}{}",
                    output.status.code().unwrap_or(-1),
                    if detail.trim().is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", detail.trim())
                    }
                ));
            }
            Err(error) => {
                return Err(anyhow!(
                    "smoke test failed: could not exec {name} --version: {error}"
                ));
            }
        }
    }
    Ok(())
}

fn bin_dir_for(install_root: &std::path::Path) -> std::path::PathBuf {
    install_root.join("bin")
}

// ----- Phase 3b: macOS codesign verify (focusa-112-codesign-verify) -----
fn verify_macos_codesign(
    target: InstallTarget,
    channel: Channel,
    asset: &InstalledAsset,
) -> Result<()> {
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
        if channel == Channel::Stable {
            bail!(
                "stable macOS install requires a valid code signature for {}: codesign exited {}",
                asset.name,
                status.code().unwrap_or(-1)
            );
        }
        eprintln!(
            "warning: {:?} macOS asset {} is unsigned/ad-hoc; accepted only for preview evaluation",
            channel, asset.name
        );
        return Ok(());
    }
    eprintln!("✓ macOS codesign verified for {}", asset.name);
    Ok(())
}

#[derive(Debug)]
struct RealInstallResult {
    license_status: String,
    assets: Vec<InstalledAsset>,
    walkthrough: FirstInstallWalkthrough,
    service_status: String,
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
    let phase = phase_license(args, channel).await?;
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
        args.release_tag_override.as_deref(),
        install_root,
        sink,
        cancellation,
    )
    .await?;
    sink.emit(InstallEvent::PhaseStarted {
        phase: InstallPhase::IntegratePi,
        message: "Checking optional Pi integration".into(),
    });
    let pi_extension = match phase_pi_extension_download(
        channel,
        args.github_repo.as_deref(),
        args.release_tag_override.as_deref(),
        install_root,
        sink,
        cancellation,
    )
    .await
    {
        Ok(asset) => asset,
        Err(error) => {
            sink.emit(InstallEvent::PhaseWarning {
                phase: InstallPhase::IntegratePi,
                message: "Pi extension download/integration unavailable".into(),
                recovery_hint: Some(redact_url(&error.to_string())),
            });
            None
        }
    };
    let agent_context = phase_agent_context_download(
        channel,
        args.github_repo.as_deref(),
        args.release_tag_override.as_deref(),
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
    if let Some(pi_asset) = pi_extension {
        match verify_checksum(&pi_asset).await {
            Ok(()) => match integrate_pi_extension(&pi_asset, install_root, None, None) {
                Ok(path) => {
                    sink.emit(InstallEvent::PhaseSucceeded {
                        phase: InstallPhase::IntegratePi,
                        detail: Some(format!("verified at {}", redact_url(&path))),
                    });
                }
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
            verify_macos_codesign(target, channel, asset)?;
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
    let agent_context_root = install_agent_context_archive(agent_context_asset, install_root)?;
    install_skill_doctor(&agent_context_root, install_root)?;
    // Prove all promoted binaries before any external symlink, service, or shell
    // profile mutation. A failed fresh install can then remove the install root
    // without leaving dangling links or a partially registered service.
    let expected_tag = assets
        .first()
        .map(|asset| asset.version.as_str())
        .ok_or_else(|| anyhow!("verified release identity is missing"))?;
    phase_smoke_test(target, &bin_dir, expected_tag)
        .await
        .context("pre-commit binary smoke test failed")?;
    place_symlinks(target, &bin_dir, install_root)?;
    sink.emit(InstallEvent::PhaseSucceeded {
        phase: InstallPhase::InstallBinaries,
        detail: Some("Staged binaries promoted".into()),
    });
    ensure_not_cancelled(cancellation)?;
    sink.emit(InstallEvent::PhaseStarted {
        phase: InstallPhase::RegisterService,
        message: "Registering service".into(),
    });
    let service_status = if !args.no_service && !args.system_install {
        match delegate_service_render(target, &bin_dir, args.dry_run).await? {
            ServiceRegistrationOutcome::Registered(detail) => {
                sink.emit(InstallEvent::PhaseSucceeded {
                    phase: InstallPhase::RegisterService,
                    detail: Some(detail),
                });
                "registered".to_string()
            }
            ServiceRegistrationOutcome::Warning(message) => {
                sink.emit(InstallEvent::PhaseWarning {
                    phase: InstallPhase::RegisterService,
                    message: message.clone(),
                    recovery_hint: Some(
                        "Run focusa-daemon manually or rerun with --no-service".into(),
                    ),
                });
                "warning".to_string()
            }
        }
    } else {
        sink.emit(InstallEvent::PhaseSkipped {
            phase: InstallPhase::RegisterService,
            reason: if args.system_install {
                "authoritative system service restart deferred until system promotion".into()
            } else {
                "--no-service".into()
            },
        });
        if args.system_install {
            "system restart pending".to_string()
        } else {
            "skipped".to_string()
        }
    };

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

    let home = install_root
        .parent()
        .ok_or_else(|| anyhow!("install root has no home parent"))?;
    synchronize_agent_context_skills(&agent_context_root, home)?;
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
        service_status,
    })
}

// ----- Phase 5: Service rendering delegation (focusa-112-service-delegate) -----
enum ServiceRegistrationOutcome {
    Registered(String),
    Warning(String),
}

async fn delegate_service_render(
    target: InstallTarget,
    bin_dir: &std::path::Path,
    dry_run: bool,
) -> Result<ServiceRegistrationOutcome> {
    // Delegate rendering and activation to the canonical service module with
    // explicit promoted paths; current_exe may still point into the prior
    // transactional stash during an upgrade.
    match target {
        InstallTarget::WindowsX64 | InstallTarget::WindowsArm64 => {
            return Ok(ServiceRegistrationOutcome::Warning(
                "Windows service registration is unavailable in this installer build".into(),
            ));
        }
        InstallTarget::Darwin if !cfg!(target_os = "macos") => {
            return Ok(ServiceRegistrationOutcome::Warning(
                "macOS service registration skipped on a non-macOS host".into(),
            ));
        }
        InstallTarget::Linux | InstallTarget::Auto if cfg!(target_os = "macos") => {
            return Ok(ServiceRegistrationOutcome::Warning(
                "Linux service registration skipped on macOS".into(),
            ));
        }
        _ => {}
    }
    crate::commands::service::run(
        crate::commands::service::InstallServiceArgs {
            no_enable: false,
            json: false,
            daemon_path: Some(bin_dir.join(installed_binary_name(target, "focusa-daemon"))),
            cli_path: Some(bin_dir.join(installed_binary_name(target, "focusa"))),
        },
        dry_run,
    )
    .await?;
    Ok(ServiceRegistrationOutcome::Registered(
        "Service registration completed and loaded".into(),
    ))
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
    let mut assets_planned = CANONICAL_RELEASE_BINARIES
        .into_iter()
        .map(|name| AssetPlan {
            name: name.to_string(),
            version: "<detected>".to_string(),
            triple: triple_for(target),
            install_path: root
                .join("bin")
                .join(installed_binary_name(target, name))
                .display()
                .to_string(),
        })
        .collect::<Vec<_>>();
    assets_planned.push(AssetPlan {
        name: "focusa-agent-context".to_string(),
        version: "<detected>".to_string(),
        triple: "all".to_string(),
        install_path: root
            .join("share")
            .join("focusa-agent-context-<version>.tar.gz")
            .display()
            .to_string(),
    });
    Ok(InstallPlan {
        target,
        channel: args.channel,
        install_root: root.display().to_string(),
        assets_planned,
        symlink_planned: format!(
            "{}/.local/bin/focusa",
            std::env::var("HOME").unwrap_or_default()
        ),
        service_manager_planned: match target {
            InstallTarget::Linux => "systemd --user".to_string(),
            InstallTarget::Darwin => "launchd user agent".to_string(),
            InstallTarget::WindowsX64 | InstallTarget::WindowsArm64 => {
                "Windows service warning".to_string()
            }
            InstallTarget::Auto => "auto".to_string(),
        },
        shell_rc_plan: vec![
            "~/.bashrc".to_string(),
            "~/.zshrc".to_string(),
            "~/.config/fish/config.fish".to_string(),
        ],
        license_mode: if args.license_key.is_some() {
            "unsupported_raw_key".to_string()
        } else if args.eval {
            "authority_limited_access".to_string()
        } else {
            "authority_existing_or_limited_access".to_string()
        },
        notes: vec![
            "--target auto-detected from uname / GetSystemInfo".to_string(),
            "runnable assets activate only after signed product/channel entitlement".to_string(),
            "PATH automation writes idemptoent export lines to rc files".to_string(),
        ],
        first_install_walkthrough_v1: Some(build_first_install_walkthrough(
            target,
            args.channel,
            &root.join("bin"),
            root,
            CANONICAL_RELEASE_BINARIES.len() + 1,
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

fn installed_binary_name(target: InstallTarget, base: &str) -> String {
    format!("{base}{}", release_executable_suffix(target))
}

fn release_executable_suffix(target: InstallTarget) -> &'static str {
    match target {
        InstallTarget::WindowsX64 | InstallTarget::WindowsArm64 => ".exe",
        _ => "",
    }
}

fn triple_for(target: InstallTarget) -> String {
    match target {
        // Static musl is the portable default for older production glibc hosts.
        InstallTarget::Linux => "x86_64-unknown-linux-musl".to_string(),
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
#[path = "install_e6_failure_matrix_tests.rs"]
mod install_e6_failure_matrix_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    fn write_executable(path: &std::path::Path, body: &str) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(path, body).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    fn release_tag_override_is_channel_exact() {
        assert_eq!(
            release_tag(Channel::Stable, Some("v0.9.187")).unwrap(),
            "v0.9.187"
        );
        assert!(release_tag(Channel::Stable, Some("v0.9.187-dev")).is_err());
        assert!(release_tag(Channel::Preview, Some("v0.9.187-devil")).is_err());
        assert!(release_tag(Channel::Nightly, Some("v0.9.187-nightly.1")).is_ok());
        let exact = serde_json::json!({"tag_name":"v0.9.187"});
        assert_eq!(
            bind_resolved_release_tag(Channel::Stable, "v0.9.187", &exact).unwrap(),
            "v0.9.187"
        );
        assert!(
            bind_resolved_release_tag(
                Channel::Stable,
                "v0.9.187",
                &serde_json::json!({"tag_name":"v0.9.188"})
            )
            .is_err()
        );
    }

    #[cfg(unix)]
    #[test]
    fn system_promotion_is_atomic_and_rollback_safe() {
        let fixture =
            std::env::temp_dir().join(format!("focusa-system-promotion-{}", uuid::Uuid::now_v7()));
        let bin = fixture.join("verified/bin");
        let system = fixture.join("usr-local-bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::create_dir_all(&system).unwrap();
        for name in CANONICAL_RELEASE_BINARIES {
            write_executable(&bin.join(name), "#!/bin/sh\nprintf 'focusa 0.9.187\\n'\n");
            std::fs::write(system.join(name), format!("old-{name}")).unwrap();
        }
        assert!(!promote_system_links(&bin, &system, "v0.9.187", false).unwrap());
        for name in CANONICAL_RELEASE_BINARIES {
            assert!(system.join(name).is_file());
            assert!(
                !std::fs::symlink_metadata(system.join(name))
                    .unwrap()
                    .file_type()
                    .is_symlink()
            );
            assert_eq!(
                std::fs::read_link(bin.join(name)).unwrap(),
                system.join(name)
            );
        }

        for name in CANONICAL_RELEASE_BINARIES {
            std::fs::remove_file(bin.join(name)).unwrap();
            write_executable(&bin.join(name), "#!/bin/sh\nprintf 'focusa 0.9.187\\n'\n");
            std::fs::remove_file(system.join(name)).unwrap();
            std::fs::write(system.join(name), format!("restored-{name}")).unwrap();
        }
        write_executable(
            &bin.join("focusa-daemon"),
            "#!/bin/sh\nprintf 'focusa-daemon 0.9.186\\n'\n",
        );
        assert!(promote_system_links(&bin, &system, "v0.9.187", false).is_err());
        for name in CANONICAL_RELEASE_BINARIES {
            assert_eq!(
                std::fs::read_to_string(system.join(name)).unwrap(),
                format!("restored-{name}")
            );
            assert!(bin.join(name).is_file());
            assert!(
                !std::fs::symlink_metadata(bin.join(name))
                    .unwrap()
                    .file_type()
                    .is_symlink()
            );
        }
        std::fs::remove_dir_all(fixture).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn npm_resolution_accepts_absolute_nonstandard_install() {
        use std::os::unix::fs::PermissionsExt;
        let fixture =
            std::env::temp_dir().join(format!("focusa-npm-resolution-{}", uuid::Uuid::now_v7()));
        let npm = fixture.join("servbay/node/bin/npm");
        let npm_cli = fixture.join("servbay/node/lib/npm-cli.js");
        std::fs::create_dir_all(npm.parent().unwrap()).unwrap();
        std::fs::create_dir_all(npm_cli.parent().unwrap()).unwrap();
        std::fs::write(&npm_cli, "#!/usr/bin/env node\n").unwrap();
        std::fs::set_permissions(&npm_cli, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::os::unix::fs::symlink(&npm_cli, &npm).unwrap();
        let resolved = resolve_npm_binary(Some(&npm)).unwrap();
        assert_eq!(
            resolved, npm,
            "launcher path must preserve sibling node lookup"
        );
        let _ = std::fs::remove_dir_all(fixture);
    }

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
        assert_eq!(
            triple_for(InstallTarget::Linux),
            "x86_64-unknown-linux-musl"
        );
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
    fn windows_release_assets_and_install_paths_use_exe_suffix() {
        assert_eq!(release_executable_suffix(InstallTarget::WindowsX64), ".exe");
        assert_eq!(
            release_executable_suffix(InstallTarget::WindowsArm64),
            ".exe"
        );
        assert_eq!(
            installed_binary_name(InstallTarget::WindowsX64, "focusa-daemon"),
            "focusa-daemon.exe"
        );
        assert_eq!(release_executable_suffix(InstallTarget::Linux), "");
        assert_eq!(release_executable_suffix(InstallTarget::Darwin), "");
    }

    #[test]
    fn dry_run_plan_lists_four_binaries_and_context() {
        let args = InstallArgs {
            target: InstallTarget::Linux,
            channel: Channel::Stable,
            dry_run: true,
            preflight: false,
            no_animation: false,
            quiet: false,
            install_dependencies: false,
            assume_yes: false,
            license_key: None,
            eval: false,
            accept_license: false,
            no_service: false,
            reuse_existing_license: false,
            suppress_completion_output: false,
            release_tag_override: None,
            system_install: false,
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
        assert_eq!(plan.assets_planned.len(), 5);
        for name in CANONICAL_RELEASE_BINARIES {
            assert!(plan.assets_planned.iter().any(|asset| asset.name == name));
        }
        assert!(
            plan.assets_planned
                .iter()
                .any(|asset| asset.name == "focusa-agent-context" && asset.triple == "all")
        );
        assert_eq!(plan.license_mode, "authority_existing_or_limited_access");
    }

    #[test]
    fn dry_run_plan_with_eval_flag_marks_limited_access_license() {
        let args = InstallArgs {
            target: InstallTarget::Darwin,
            channel: Channel::Stable,
            dry_run: true,
            preflight: false,
            no_animation: false,
            quiet: false,
            install_dependencies: false,
            assume_yes: false,
            license_key: None,
            eval: true,
            accept_license: false,
            no_service: false,
            reuse_existing_license: false,
            suppress_completion_output: false,
            release_tag_override: None,
            system_install: false,
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
        assert_eq!(plan.license_mode, "authority_limited_access");
        assert!(plan.service_manager_planned.contains("launchd"));
    }

    #[test]
    fn pi_extension_archive_install_is_checksum_stage_and_activation_safe() {
        let fixture = std::env::temp_dir().join(format!(
            "focusa-pi-extension-install-{}",
            uuid::Uuid::now_v7()
        ));
        let package = fixture.join("package/pi-extension");
        let fake_bin = fixture.join("bin");
        let extensions = fixture.join("extensions");
        std::fs::create_dir_all(&package).unwrap();
        std::fs::create_dir_all(&fake_bin).unwrap();
        let legacy = extensions.join("focusa-runtime.legacy-0.9.143");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(
            legacy.join("package.json"),
            r#"{"name":"focusa-pi-bridge","version":"0.9.143","pi":{"extensions":["./src/index.ts"]}}"#,
        )
        .unwrap();
        std::fs::write(
            package.join("package.json"),
            r#"{"name":"focusa-pi-bridge"}"#,
        )
        .unwrap();
        let npm = fake_bin.join("npm");
        std::fs::write(
            &npm,
            "#!/bin/sh\nmkdir -p node_modules\nprintf staged > node_modules/.focusa-smoke\n",
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&npm, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let archive = fixture.join("focusa-pi-extension.tar.gz");
        assert!(
            tar_command()
                .args(["-czf"])
                .arg(&archive)
                .args(["-C"])
                .arg(fixture.join("package"))
                .arg("pi-extension")
                .status()
                .unwrap()
                .success()
        );
        let asset = InstalledAsset {
            name: "focusa-pi-extension-vtest.tar.gz".to_string(),
            version: "vtest".to_string(),
            triple: "all".to_string(),
            sha256: String::new(),
            install_path: archive.display().to_string(),
        };
        let destination =
            integrate_pi_extension(&asset, &fixture, Some(&extensions), Some(&npm)).unwrap();
        let destination_expected = extensions.join("focusa").display().to_string();
        let destination_preserved = destination == destination_expected;
        let package_json_present = extensions.join("focusa/package.json").is_file();
        let smoke_marker_present = extensions
            .join("focusa/node_modules/.focusa-smoke")
            .is_file();
        assert_eq!(destination, destination_expected);
        assert!(extensions.join("focusa/package.json").is_file());
        assert!(
            extensions
                .join("focusa/node_modules/.focusa-smoke")
                .is_file()
        );
        assert!(!legacy.exists());
        let retired = std::fs::read_dir(fixture.join("retired-extensions"))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(retired.len(), 1);
        assert!(retired[0].path().join("package.json").is_file());
        println!(
            "E6_PI_PRESENT_SUCCESS destination_preserved={destination_preserved} package_json={package_json_present} smoke_marker={smoke_marker_present}"
        );
        let _ = std::fs::remove_dir_all(fixture);
    }

    #[test]
    fn retired_focusa_pi_extension_is_preserved_outside_auto_discovery() {
        let fixture = std::env::temp_dir().join(format!(
            "focusa-retired-pi-extension-{}",
            uuid::Uuid::now_v7()
        ));
        let extensions = fixture.join("extensions");
        let legacy = extensions.join("focusa-runtime.legacy-0.9.143");
        let unrelated = extensions.join("vendor.legacy-1");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::create_dir_all(&unrelated).unwrap();
        std::fs::write(
            legacy.join("package.json"),
            r#"{"name":"focusa-pi-bridge","version":"0.9.143","pi":{"extensions":["./src/index.ts"]}}"#,
        )
        .unwrap();
        std::fs::write(
            unrelated.join("package.json"),
            r#"{"name":"vendor-extension"}"#,
        )
        .unwrap();

        let retired = quarantine_retired_focusa_pi_extensions(&extensions).unwrap();

        assert_eq!(retired.len(), 1);
        assert!(!legacy.exists());
        assert!(unrelated.is_dir());
        assert!(retired[0].starts_with(fixture.join("retired-extensions")));
        assert!(retired[0].join("package.json").is_file());
        let package: serde_json::Value =
            serde_json::from_slice(&std::fs::read(retired[0].join("package.json")).unwrap())
                .unwrap();
        assert_eq!(package["version"], "0.9.143");
        let _ = std::fs::remove_dir_all(fixture);
    }

    #[test]
    fn agent_context_archive_installs_required_files_atomically() {
        let fixture = std::env::temp_dir().join(format!(
            "focusa-agent-context-install-{}",
            uuid::Uuid::now_v7()
        ));
        let package = fixture.join("package/focusa-agent-context");
        std::fs::create_dir_all(package.join("skills/focusa")).unwrap();
        std::fs::create_dir_all(package.join("bin")).unwrap();
        std::fs::write(package.join("AGENTS.md"), "# Focusa agents\n").unwrap();
        std::fs::write(
            package.join("skills/focusa/SKILL.md"),
            "---\nname: focusa\n---\n",
        )
        .unwrap();
        std::fs::write(
            package.join("bin/focusa-skill-doctor"),
            "#!/bin/sh\nexit 0\n",
        )
        .unwrap();
        let archive = fixture.join("focusa-agent-context-vtest.tar.gz");
        let status = tar_command()
            .args(["-czf"])
            .arg(&archive)
            .arg("-C")
            .arg(fixture.join("package"))
            .arg("focusa-agent-context")
            .status()
            .unwrap();
        assert!(status.success());
        let home = fixture.join("home");
        let install_root = home.join(".focusa");
        std::fs::create_dir_all(install_root.join("agent-context")).unwrap();
        std::fs::write(install_root.join("agent-context/old-marker"), "old").unwrap();
        std::fs::create_dir_all(home.join(".pi/skills/focusa")).unwrap();
        std::fs::write(home.join(".pi/skills/focusa/SKILL.md"), "stale").unwrap();
        std::fs::create_dir_all(home.join(".pi/skills/operator-custom")).unwrap();
        std::fs::write(
            home.join(".pi/skills/operator-custom/SKILL.md"),
            "---\nname: operator-custom\n---\n",
        )
        .unwrap();
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
        let synchronized = synchronize_agent_context_skills(&installed, &home).unwrap();
        assert_eq!(synchronized, vec![home.join(".pi/agent/skills/focusa")]);
        assert_eq!(
            std::fs::read_to_string(home.join(".pi/agent/skills/focusa/SKILL.md")).unwrap(),
            "---\nname: focusa\n---\n"
        );
        assert!(!home.join(".pi/skills/focusa").exists());
        assert_eq!(
            std::fs::read_to_string(home.join(".pi/skills/operator-custom/SKILL.md")).unwrap(),
            "---\nname: operator-custom\n---\n"
        );

        // Model Pi's combined built-in, settings/project, and extension-provided
        // discovery paths. Only the canonical user root may contain Focusa's
        // installed skill; unrelated skills on every other path survive.
        let project_skills = fixture.join("project/.pi/skills");
        let extension_skills = fixture.join("extension/skills");
        std::fs::create_dir_all(project_skills.join("operator-project")).unwrap();
        std::fs::create_dir_all(extension_skills.join("operator-extension")).unwrap();
        let discovery_roots = [
            home.join(".pi/agent/skills"),
            home.join(".pi/skills"),
            project_skills,
            extension_skills,
        ];
        let focusa_discoveries = discovery_roots
            .iter()
            .filter(|root| root.join("focusa/SKILL.md").is_file())
            .count();
        assert_eq!(focusa_discoveries, 1);

        install_skill_doctor(&installed, &install_root).unwrap();
        assert!(install_root.join("bin/focusa-skill-doctor").is_file());
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
        let status = tar_command()
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
    fn fresh_install_recovery_removes_every_partial_install_artifact() {
        let root = std::env::temp_dir().join(format!("focusa-fresh-{}", uuid::Uuid::now_v7()));
        let stash = root.with_extension("stash");
        std::fs::create_dir_all(root.join("bin")).unwrap();
        std::fs::write(root.join("bin/focusa"), b"partial").unwrap();

        phase_atomic_recover(&root, &stash, false).unwrap();

        assert!(!root.exists());
        assert!(!stash.exists());
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
    fn successful_rerun_restores_customer_data_without_restoring_old_binaries() {
        let fixture = std::env::temp_dir().join(format!(
            "focusa-rerun-preserve-data-{}",
            uuid::Uuid::now_v7()
        ));
        let install_root = fixture.join(".focusa");
        let stash = fixture.join(".focusa.stash");
        std::fs::create_dir_all(install_root.join("bin")).unwrap();
        std::fs::create_dir_all(install_root.join("state")).unwrap();
        std::fs::write(install_root.join("bin/focusa"), "old-binary").unwrap();
        std::fs::write(install_root.join("state/customer.json"), "preserve-me").unwrap();
        std::fs::write(install_root.join("focusa.sqlite"), "customer-db").unwrap();

        assert!(phase_atomic_stash(&install_root, &stash).unwrap());
        std::fs::create_dir_all(install_root.join("bin")).unwrap();
        std::fs::write(install_root.join("bin/focusa"), "new-binary").unwrap();
        std::fs::write(install_root.join(".focusa-version"), "v2").unwrap();

        phase_restore_customer_data(&stash, &install_root).unwrap();
        assert_eq!(
            std::fs::read_to_string(install_root.join("bin/focusa")).unwrap(),
            "new-binary"
        );
        assert_eq!(
            std::fs::read_to_string(install_root.join("state/customer.json")).unwrap(),
            "preserve-me"
        );
        assert_eq!(
            std::fs::read_to_string(install_root.join("focusa.sqlite")).unwrap(),
            "customer-db"
        );
        phase_atomic_cleanup(&stash).unwrap();
        assert!(!stash.exists());
        std::fs::remove_dir_all(fixture).unwrap();
    }

    #[test]
    fn cancellation_result_is_nonzero_and_reports_no_prior_install() {
        let root =
            std::env::temp_dir().join(format!("focusa-cancel-result-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&root).unwrap();
        let sink = NullEventSink;
        let result: Result<()> =
            cancellation_result(&root, &root.join("missing.stash"), false, &sink);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("no prior installation existed")
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
            install_dependencies: false,
            assume_yes: false,
            license_key: Some("focusa_live_xxxxx".to_string()),
            eval: false,
            accept_license: false,
            no_service: false,
            reuse_existing_license: false,
            suppress_completion_output: false,
            release_tag_override: None,
            system_install: false,
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
        assert_eq!(plan.license_mode, "unsupported_raw_key");
    }

    #[test]
    fn dependency_plan_matrix_is_explicit_and_recoverable() {
        let managers = [
            "dnf", "yum", "apt-get", "brew", "pacman", "zypper", "choco", "winget",
        ];
        let dependencies = ["curl", "python3", "sha256sum", "tar", "node", "npm"];
        for manager in managers {
            for dependency in dependencies {
                let plan = dependency_install_plan(Some(manager), dependency);
                assert_eq!(plan.manager, manager);
                assert!(!plan.package.is_empty());
                assert!(plan.install_command.contains(&plan.package));
                assert!(plan.dry_run_command.contains(&plan.package));
                assert!(!plan.repository.is_empty());
                assert!(plan.recovery_hint.contains("--preflight --json"));
            }
        }
        assert_eq!(
            dependency_install_plan(Some("winget"), "curl").package,
            "cURL.cURL"
        );
        assert_eq!(
            dependency_install_plan(Some("brew"), "tar").package,
            "gnu-tar"
        );
        assert!(dependency_install_plan(Some("apt-get"), "python3").privilege_required);
        assert!(!dependency_install_plan(Some("brew"), "python3").privilege_required);
        assert_eq!(dependency_install_plan(Some("brew"), "npm").package, "node");
        assert_eq!(
            dependency_install_plan(Some("winget"), "node").package,
            "OpenJS.NodeJS.LTS"
        );
    }

    #[test]
    fn agent_workflow_dependency_plans_are_pinned_ordered_and_verifiable() {
        let dependencies = detect_dependencies(Some("dnf"));
        assert_eq!(
            dependencies
                .iter()
                .map(|dependency| dependency.name.as_str())
                .collect::<Vec<_>>(),
            vec![
                "curl",
                "python3",
                "sha256sum",
                "tar",
                "node",
                "npm",
                "pi",
                "uiai-engine",
            ]
        );
        let pi = dependency_install_plan(Some("dnf"), "pi");
        assert_eq!(pi.manager, "npm");
        assert_eq!(pi.package, SUPPORTED_PI_NPM_PACKAGE);
        assert!(pi.install_command.contains("npm install --global"));
        let uiai = dependency_install_plan(Some("dnf"), "uiai-engine");
        assert!(uiai.package.contains(UIAI_ENGINE_RELEASE_TAG));
        assert!(
            uiai.install_command
                .contains(UIAI_ENGINE_LINUX_AMD64_SHA256)
        );
        assert!(!uiai.install_command.contains("/latest/"));
        assert!(uiai.install_command.contains(
            "systemctl --user enable --now \"$HOME/.config/systemd/user/focusa-uiai-engine.service\""
        ));
        assert!(uiai.dry_run_command.contains("/health"));
    }

    #[test]
    fn dependency_execution_reports_success_and_failure_without_hiding_commands() {
        let mut dependencies = detect_dependencies(Some("dnf"));
        dependencies[0].present = false;
        dependencies[1].present = false;
        let selected = dependencies.iter().take(2).collect::<Vec<_>>();
        let execution = execute_dependency_plans_with(&selected, |command| {
            if command.contains("python3") {
                Err("fixture rejection".into())
            } else {
                Ok(())
            }
        });
        assert_eq!(execution.status, "partial_failure");
        assert_eq!(execution.commands.len(), 2);
        assert_eq!(execution.installed, vec!["curl"]);
        assert_eq!(execution.failures.len(), 1);
        assert_eq!(
            execution.rollback_status,
            "partial_success_preserved_non_destructively"
        );
        assert!(execution.failures[0].contains("fixture rejection"));
    }

    #[test]
    fn dependency_execution_skips_present_and_retries_lock_contention_once() {
        let mut dependencies = detect_dependencies(Some("dnf"));
        dependencies[0].present = true;
        dependencies[1].present = false;
        let selected = dependencies.iter().take(2).collect::<Vec<_>>();
        let mut attempts = 0;
        let execution = execute_dependency_plans_with(&selected, |_| {
            attempts += 1;
            if attempts == 1 {
                Err("package manager lock held by another process".into())
            } else {
                Ok(())
            }
        });
        assert_eq!(attempts, 2);
        assert_eq!(execution.status, "completed");
        assert_eq!(execution.already_present, vec!["curl"]);
        assert_eq!(execution.installed, vec!["python3"]);
        assert_eq!(execution.retryable_failures.len(), 1);
        assert_eq!(execution.rollback_status, "not_needed");
    }

    #[cfg(windows)]
    #[test]
    fn windows_dependency_preflight_native_resolves_path_semver_and_health() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _lock = ENV_LOCK.lock().unwrap();
        let fixture = std::env::temp_dir().join(format!(
            "focusa-windows-dependency-preflight-{}",
            uuid::Uuid::now_v7()
        ));
        std::fs::create_dir_all(&fixture).unwrap();
        let rustc = find_command("rustc").expect("Windows Rust toolchain must expose rustc.exe");
        let pi_source = fixture.join("pi_fixture.rs");
        std::fs::write(&pi_source, "fn main() { println!(\"pi 0.81.1\"); }").unwrap();
        let pi_exe = fixture.join("pi.exe");
        assert!(
            std::process::Command::new(rustc)
                .arg(&pi_source)
                .arg("-o")
                .arg(&pi_exe)
                .status()
                .unwrap()
                .success(),
            "failed to compile native pi.exe fixture"
        );
        std::fs::write(fixture.join("pi.cmd"), "@echo off\r\necho pi 0.81.1\r\n").unwrap();
        std::fs::write(
            fixture.join("pi"),
            "#!/bin/sh\necho extensionless shim must not win\n",
        )
        .unwrap();
        let script_fixture = fixture.join("Program Files fixture");
        std::fs::create_dir_all(&script_fixture).unwrap();
        std::fs::write(
            script_fixture.join("npm.cmd"),
            "@echo off\r\necho 10.9.2\r\n",
        )
        .unwrap();
        std::fs::write(
            script_fixture.join("npm"),
            "#!/bin/sh\necho extensionless shim must not win\n",
        )
        .unwrap();
        let previous_path = std::env::var_os("PATH");
        let previous_uiai = std::env::var_os("UIAI_ENGINE_URL");
        let mut paths = vec![fixture.clone(), script_fixture];
        if let Some(path) = previous_path.as_ref() {
            paths.extend(std::env::split_paths(path));
        }
        // SAFETY: this Windows-only test is the sole filtered test in its CI
        // process and holds ENV_LOCK for the complete mutation/restoration.
        unsafe { std::env::set_var("PATH", std::env::join_paths(paths).unwrap()) };

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let mut request = [0_u8; 1024];
                        let _ = stream.read(&mut request);
                        stream
                            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK")
                            .unwrap();
                        return;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        if std::time::Instant::now() >= deadline {
                            panic!("UIAI health fixture received no request");
                        }
                        std::thread::sleep(std::time::Duration::from_millis(20));
                    }
                    Err(error) => panic!("UIAI health fixture accept failed: {error}"),
                }
            }
        });
        // SAFETY: guarded and restored before this test releases ENV_LOCK.
        unsafe { std::env::set_var("UIAI_ENGINE_URL", format!("http://{address}")) };

        let result = std::panic::catch_unwind(|| {
            // Consume the bounded health fixture before assertions that can
            // unwind, so its server thread always terminates deterministically.
            assert!(dependency_present("uiai-engine"));
            assert!(find_command("pi").is_some_and(|path| path.ends_with("pi.exe")));
            assert!(command_semver("pi").is_some_and(|version| version >= (0, 81, 1)));
            assert!(dependency_present("pi"));
            assert_eq!(command_semver("npm"), Some((10, 9, 2)));
            assert!(dependency_present("npm"));
        });
        server.join().unwrap();
        // SAFETY: restore the process environment while ENV_LOCK is held.
        unsafe {
            if let Some(path) = previous_path {
                std::env::set_var("PATH", path);
            } else {
                std::env::remove_var("PATH");
            }
            if let Some(url) = previous_uiai {
                std::env::set_var("UIAI_ENGINE_URL", url);
            } else {
                std::env::remove_var("UIAI_ENGINE_URL");
            }
        }
        let _ = std::fs::remove_dir_all(fixture);
        result.unwrap();
    }

    #[test]
    fn path_persistence_migrates_legacy_block_without_duplication() {
        let fixture =
            std::env::temp_dir().join(format!("focusa-path-migration-{}", uuid::Uuid::now_v7()));
        let rc = fixture.join(".zshrc");
        std::fs::create_dir_all(&fixture).unwrap();
        let line = "export PATH=\"$HOME/.local/bin:$PATH\"";
        std::fs::write(
            &rc,
            format!("{LEGACY_PATH_MARKER_BEGIN}\n{line}\n{LEGACY_PATH_MARKER_END}\n"),
        )
        .unwrap();

        persist_path_to_rc(&rc, line).unwrap();
        let content = std::fs::read_to_string(&rc).unwrap();
        assert_eq!(content.matches(line).count(), 1);
        assert!(content.contains(PATH_MARKER_BEGIN));
        assert!(!content.contains(LEGACY_PATH_MARKER_BEGIN));
        let _ = std::fs::remove_dir_all(fixture);
    }

    #[test]
    fn path_persistence_removes_legacy_duplicate_beside_current_block() {
        let fixture =
            std::env::temp_dir().join(format!("focusa-path-deduplicate-{}", uuid::Uuid::now_v7()));
        let rc = fixture.join(".zshrc");
        std::fs::create_dir_all(&fixture).unwrap();
        let line = "export PATH=\"$HOME/.local/bin:$PATH\"";
        std::fs::write(
            &rc,
            format!(
                "{LEGACY_PATH_MARKER_BEGIN}\n{line}\n{LEGACY_PATH_MARKER_END}\n{PATH_MARKER_BEGIN}\n{line}\n{PATH_MARKER_END}\n"
            ),
        )
        .unwrap();

        persist_path_to_rc(&rc, line).unwrap();
        let content = std::fs::read_to_string(&rc).unwrap();
        assert_eq!(content.matches(line).count(), 1);
        assert!(!content.contains(LEGACY_PATH_MARKER_BEGIN));
        let _ = std::fs::remove_dir_all(fixture);
    }
}
