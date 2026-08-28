//! Focusa CLI — primary control interface.
//!
//! Source: docs/G1-13-cli.md
//!
//! Binary: `focusa`
//! Thin facade — zero business logic beyond arg parsing + API calls.

use clap::{Parser, Subcommand, ValueEnum};

mod api_client;
mod commands;

/// Pairing umbrella (focusa-ui0y v0.9.39-dev). Aggregates the pairing
/// subcommands under a single `focusa pairing ...` namespace.
#[derive(Subcommand, Debug)]
enum PairingCmd {
    /// Start a Mac/phone pairing flow (canonical replacement for `focusa pair`).
    Start(commands::pair::PairArgs),
    /// Interactive pairing wizard (Tailscale detect, room create, terminal QR, poll).
    Wizard(commands::pairing_wizard::WizardArgs),
    /// Non-interactive: create a room and print JSON with room_id + pair_url.
    CreateRoom(commands::pairing_wizard::CreateRoomArgs),
    /// Discover and write the best phone-reachable Focusa transport.
    #[command(subcommand)]
    Transport(commands::pairing_transport::TransportCmd),
    /// Single-command pairing root-cause report.
    Doctor(commands::pairing_doctor::DoctorArgs),
    /// Revoke + re-pair cycle harness (v0.9.39-dev core test).
    CycleTest(commands::pairing_cycle_test::CycleTestArgs),
    /// One-shot operator dashboard view (G11).
    Status(commands::pairing_dashboard::StatusArgs),
    /// Audit log of past pairings (G12).
    History(commands::pairing_dashboard::HistoryArgs),
    /// One-shot email-link for phone-camera-broken scenarios (G13).
    EmailLink(commands::pairing_email_link::EmailLinkArgs),
}

async fn run_pairing(cmd: PairingCmd) -> anyhow::Result<()> {
    match cmd {
        PairingCmd::Start(args) => commands::pair::run(args, false).await,
        PairingCmd::Wizard(args) => {
            commands::pairing_wizard::run(commands::pairing_wizard::WizardCmd::Wizard(args)).await
        }
        PairingCmd::CreateRoom(args) => {
            commands::pairing_wizard::run(commands::pairing_wizard::WizardCmd::CreateRoom(args))
                .await
        }
        PairingCmd::Transport(sub) => commands::pairing_transport::run(sub).await,
        PairingCmd::Doctor(args) => commands::pairing_doctor::run(args).await,
        PairingCmd::CycleTest(args) => commands::pairing_cycle_test::run(args).await,
        PairingCmd::Status(args) => commands::pairing_dashboard::run_status(args).await,
        PairingCmd::History(args) => commands::pairing_dashboard::run_history(args).await,
        PairingCmd::EmailLink(args) => commands::pairing_email_link::run(args).await,
    }
}

#[derive(Parser)]
#[command(
    name = "focusa",
    version = env!("CARGO_PKG_VERSION"),
    about = "Focusa cognitive governance CLI",
    disable_help_subcommand = true
)]
#[command(propagate_version = true)]
struct Cli {
    /// Output in JSON format.
    #[arg(long, global = true)]
    json: bool,

    /// Config file path.
    #[arg(long, global = true)]
    config: Option<String>,

    /// Verbose output.
    #[arg(long, global = true)]
    verbose: bool,

    /// Quiet mode — suppress non-essential output.
    #[arg(long, global = true)]
    quiet: bool,

    /// Inspect, preview, confirm, apply, or recover a lifecycle transaction.
    #[arg(long, global = true, value_enum, value_name = "ACTION")]
    lifecycle_action: Option<commands::lifecycle_guidance::GuidedAction>,

    /// Confirm the mutation selected by --lifecycle-action.
    #[arg(long, global = true, requires = "lifecycle_action")]
    confirm: bool,

    /// Separately confirm user-data deletion for a lifecycle purge.
    #[arg(long, global = true, requires = "lifecycle_action")]
    confirm_purge_data: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
enum StatusMode {
    Agent,
    Operator,
}

#[derive(Subcommand)]
enum Commands {
    /// Curated Focusa help and migration maps.
    Help(commands::help::HelpArgs),

    /// Start the Focusa daemon.
    Start,

    /// Stop the Focusa daemon.
    Stop,

    /// Human-facing first-impressions card explaining what focusa is FOR,
    /// the core concepts, the next 3 commands, and how to recover. LLM
    /// agents should read GET /llms.txt on the daemon instead.
    About,

    /// Durable audit timeline for workpoints, focus frames, decision events, and related daemon events.
    Audit(commands::audit::AuditArgs),

    /// Install and enable the Focusa daemon service (Linux systemd user / macOS LaunchAgent).
    InstallService(commands::service::InstallServiceArgs),

    /// Canonical Focusa install orchestrator (Spec 112 §15A). Downloads `focusa`,
    /// `focusa-daemon`, `focusa-tui`, verifies SHA256SUMS, places symlinks, renders the
    /// service unit via `install-service`, validates license, automates PATH, and emits a
    /// first-install walkthrough card inline to this terminal. The shell installers
    /// (`install.focusa.dev/focusa`, `install.focusa.dev/focusa.ps1`) are thin
    /// bootstrappers that `exec focusa install --target=auto` after detecting platform.
    Install(commands::install::InstallArgs),

    /// Signed OTA status, plan, apply, rollback, policy, scheduler, notification, and history surfaces (Spec 128).
    #[command(subcommand)]
    Update(commands::update::UpdateCmd),

    /// Inspect, evaluate, replay, and diff bounded compaction packets (Spec 130).
    #[command(subcommand)]
    Compaction(commands::compaction::CompactionCmd),

    /// Resolve explicit project/worktree/session routing without global daemon inference.
    #[command(subcommand, name = "daemon-routing")]
    DaemonRouting(commands::daemon_routing::DaemonRoutingCmd),

    /// Daemon-native durable Silent Session control plane (Spec 133).
    #[command(subcommand)]
    Silent(commands::silent::SilentCmd),

    /// Durable background execution with completion notification (docs/165).
    /// The ONLY sanctioned terminal-blocking-query surface (AGENTS.md TBQ rule).
    Bg {
        #[command(subcommand)]
        cmd: commands::bg::BgCmd,
    },

    /// Upgrade an existing Focusa install via the atomic installer path.
    Upgrade(commands::upgrade::UpgradeArgs),

    /// Mirror of `focusa install`. Default mode removes the daemon, service unit,
    /// symlinks, install root, license file, and reverts PATH in shell rc files.
    /// Use --keep-license / --keep-data / --keep-path-modifications / --purge
    /// to scope the removal. Idempotent: re-running on an already-uninstalled
    /// host returns ok with skip notes.
    Uninstall(commands::uninstall::UninstallArgs),

    /// macOS code signing + notarization inspection helper (focusa-covz).
    Codesign(commands::codesign::CodesignArgs),

    /// Show daemon status.
    Status {
        /// Optional canonical mode alias: `focusa status agent|operator`.
        mode: Option<StatusMode>,
        /// Agent-first status envelope with Workpoint, Work-loop, token, and cache details.
        #[arg(long)]
        agent: bool,
        /// Operator-facing session card with project, trajectory, Workpoint, evidence, health, and next action.
        #[arg(long)]
        operator: bool,
    },

    /// Run first-run Operator Preview onboarding.
    Onboard(commands::onboard::OnboardArgs),

    /// Open a Mac Pairing Room and print a phone-scannable QR.
    Pair(commands::pair::PairArgs),

    /// Pairing umbrella (focusa-ui0y v0.9.39-dev): wizard, transport, doctor.
    #[command(subcommand)]
    Pairing(PairingCmd),

    /// Discover and write the best phone-reachable Focusa transport (multi-transport bundle, focusa-ifc3).
    #[command(subcommand)]
    PairingTransport(commands::pairing_transport::TransportCmd),

    /// Single-command pairing root-cause report (focusa-gkrj).
    PairingDoctor(commands::pairing_doctor::DoctorArgs),

    /// Interactive pairing wizard + non-interactive room creation (focusa-ui0y v0.9.39-dev).
    #[command(subcommand)]
    PairingWizard(commands::pairing_wizard::WizardCmd),

    /// Run full agent-first doctor checks.
    Doctor(commands::doctor::DoctorArgs),

    /// License activation and entitlement operations (Spec92 §5.2).
    License(commands::license::LicenseArgs),

    /// Run Spec105 local CI/spec/evidence preflight.
    Preflight,

    /// Recover from crashed daemon / lost Workpoint context and surface recovery_hint.
    Recover(commands::recover::RecoverArgs),

    /// Explain a failure and print recovery commands.
    Explain { failure: String },

    /// Spec105 DX/UX report, requirement, and digest surfaces.
    #[command(subcommand)]
    Dxux(commands::dxux::DxuxCmd),

    /// Utility, bootstrap, and post-compaction cards.
    #[command(subcommand)]
    Utility(commands::utility::UtilityCmd),

    /// Recoverable cleanup of generated residue.
    Cleanup(commands::cleanup::CleanupArgs),

    /// Resume governed continuous work and refresh state.
    Continue(commands::continue_work::ContinueArgs),

    /// Governed Work Loop status, frontier, writer lease, and control operations.
    #[command(subcommand, name = "work-loop")]
    WorkLoop(commands::work_loop::WorkLoopCmd),

    /// Launch the focusa-tui dashboard or run a headless self-test snapshot.
    Tui(commands::tui::TuiArgs),

    /// Minimal low-risk project bootstrap (writes .focusa-project.json).
    Init(commands::init::InitArgs),

    /// Mission Deck walkthroughs (Spec 117 §12). List and start first-mission.
    Walkthrough(commands::walkthrough::WalkthroughArgs),

    /// Spec 111 agent context bootstrap and delivery.
    Preload(commands::preload::PreloadArgs),

    /// Launch Focusa Mission Deck (alias for `focusa-tui`).
    Deck(commands::deck::DeckArgs),

    /// Canonical workflow templates for LLM/operator execution.
    #[command(subcommand)]
    Workflow(commands::workflow::WorkflowCmd),

    /// Focus stack and Focus State operations.
    #[command(subcommand)]
    Focus(commands::focus::FocusCmd),

    /// Show focus stack overview.
    Stack,

    /// Focus Gate (candidate management).
    #[command(subcommand)]
    Gate(commands::gate::GateCmd),

    /// Provider-neutral work item closure authority (Spec 116).
    #[command(subcommand)]
    WorkItem(commands::work_item::WorkItemCmd),

    /// Action authority / mutation preflight operations.
    #[command(subcommand)]
    Action(commands::action::ActionCmd),

    /// Binary provenance and compatibility operations.
    #[command(subcommand)]
    Binary(commands::binary::BinaryCmd),

    /// Classify a completion claim against acceptance criteria (Spec107).
    #[command(subcommand)]
    Claim(commands::claim::ClaimCmd),

    /// Runtime inventory and daemon hygiene operations.
    #[command(subcommand)]
    Runtime(commands::runtime::RuntimeCmd),

    /// Memory operations.
    #[command(subcommand)]
    Memory(commands::memory::MemoryCmd),

    /// ECS (reference store) operations.
    #[command(subcommand)]
    Ecs(commands::ecs::EcsCmd),

    /// Export env vars for proxy routing.
    #[command(subcommand)]
    Env(commands::env::EnvCmd),

    /// Event log inspection.
    #[command(subcommand)]
    Events(commands::debug::EventsCmd),

    /// Turn-level observability.
    #[command(subcommand)]
    Turns(commands::turns::TurnsCmd),

    /// Dump full state (debug).
    State {
        #[command(subcommand)]
        cmd: commands::debug::StateCmd,
    },

    /// Context Lineage Tree.
    #[command(subcommand)]
    Clt(commands::clt::CltCmd),

    /// Lineage API parity domain.
    #[command(subcommand)]
    Lineage(commands::lineage::LineageCmd),

    /// Autonomy calibration.
    #[command(subcommand)]
    Autonomy(commands::autonomy::AutonomyCmd),

    /// Non-Pi agent awareness utility cards.
    #[command(subcommand)]
    Awareness(commands::awareness::AwarenessCmd),

    /// Agent Constitution.
    #[command(subcommand)]
    Constitution(commands::constitution::ConstitutionCmd),

    /// Project Agent Runtime Constitution compiler and delivery.
    #[command(subcommand)]
    AgentRuntime(commands::agent_runtime::AgentRuntimeCmd),

    /// Cognitive telemetry.
    #[command(subcommand)]
    Telemetry(commands::telemetry::TelemetryCmd),

    /// Reliability Focus Mode.
    #[command(subcommand)]
    Rfm(commands::rfm::RfmCmd),

    /// Release proof orchestration.
    #[command(subcommand)]
    Release(commands::release::ReleaseCmd),

    /// Proposal Resolution Engine.
    #[command(subcommand)]
    Proposals(commands::proposals::ProposalCmd),

    /// Prediction loop commands.
    #[command(subcommand)]
    Predict(commands::predict::PredictCmd),

    /// Reflection loop overlay.
    #[command(subcommand)]
    Reflect(commands::reflection::ReflectionCmd),

    /// Metacognition command domain.
    #[command(subcommand)]
    Metacognition(commands::metacognition::MetacognitionCmd),

    /// Ontology projections and vocab surfaces.
    #[command(subcommand)]
    Ontology(commands::ontology::OntologyCmd),

    /// Semantic-integrity operation registry and bounded execution surface.
    #[command(subcommand, name = "semantic-integrity")]
    SemanticIntegrity(commands::semantic_integrity::SemanticIntegrityCmd),

    /// Agent skills.
    #[command(subcommand)]
    Skills(commands::skills::SkillsCmd),

    /// Thread operations (docs/38).
    #[command(subcommand)]
    Thread(commands::threads::ThreadCmd),

    /// Export training datasets (docs/20-21).
    #[command(subcommand)]
    Export(commands::export::ExportCmd),

    /// Data contribution (docs/22).
    #[command(subcommand)]
    Contribute(commands::contribute::ContributeCmd),

    /// Cache management (docs/18-19).
    #[command(subcommand)]
    Cache(commands::cache::CacheCmd),

    /// Ontology working-set surface: scoped members, membership classes, freshness (Spec 49).
    #[command(name = "working-set")]
    #[command(subcommand)]
    WorkingSet(commands::working_set::WorkingSetCmd),

    /// Guided evaluator workflow: project selection → Workpoint → proof → Mission Deck handoff.
    #[command(name = "first-mission")]
    FirstMission(commands::first_mission::FirstMissionArgs),

    /// Setup namespace: wizard/init/doctor onboarding paths.
    #[command(subcommand)]
    Setup(commands::setup::SetupCmd),

    /// Project identity discovery and verification (Spec96).
    #[command(subcommand)]
    Project(commands::project::ProjectCmd),

    /// ResourceMode / LowMem control (Spec96).
    #[command(subcommand)]
    Resource(commands::resource::ResourceCmd),

    /// Project-scoped temporal authority, commitments, observations, and forecasts (Spec137).
    #[command(subcommand)]
    Temporal(commands::temporal::TemporalCmd),

    /// Trusted clock facts and awareness (Spec137).
    #[command(subcommand)]
    Time(commands::temporal_clients::TimeCmd),
    /// Canonical external deadline authority (Spec137).
    #[command(subcommand)]
    Deadline(commands::temporal_clients::DeadlineCmd),
    /// Grounded estimate and calibration authority (Spec137).
    #[command(subcommand)]
    Estimate(commands::temporal_clients::EstimateCmd),
    /// Evidence-backed material progress (Spec137).
    #[command(subcommand)]
    Progress(commands::temporal_clients::ProgressCmd),
    /// No-progress incident inspection (Spec137).
    #[command(subcommand, name = "no-progress")]
    NoProgress(commands::temporal_clients::NoProgressCmd),
    /// Lost-time incident inspection (Spec137).
    #[command(subcommand, name = "lost-time")]
    LostTime(commands::temporal_clients::LostTimeCmd),
    /// Opportunity posture inspection (Spec137).
    #[command(subcommand)]
    Opportunity(commands::temporal_clients::OpportunityCmd),
    /// Distributed cancellation inspection (Spec137).
    #[command(subcommand)]
    Cancellation(commands::temporal_clients::CancellationCmd),

    /// Per-project Trajectory Projection (Spec96).
    #[command(subcommand)]
    Trajectory(commands::trajectory::TrajectoryCmd),

    /// HLT Ledger: read/set/verify High-Level Trajectory history (Spec98/99).
    #[command(subcommand)]
    Hlt(commands::hlt::HltCmd),

    /// Bounded surgical traversal across Focusa surfaces (Spec96).
    #[command(subcommand)]
    Traverse(commands::traverse::TraverseCmd),

    /// Spec 100 Context Cognition packet view (advisory, read-only).
    #[command(subcommand)]
    ContextCognition(commands::context_cognition::ContextCognitionCmd),

    /// Spec 101 Bloatgaurd budget domains (read-only).
    #[command(subcommand)]
    Bloatgaurd(commands::bloatgaurd::BloatgaurdCmd),

    /// Spec 103/106 Call Stack design and drift verification.
    #[command(subcommand)]
    CallStack(commands::call_stack::CallStackCmd),

    /// Mac menubar OAuth-like device pairing for VPS connection (Spec focusa-ui0y).
    #[command(subcommand)]
    Device(commands::device_pairing::DeviceCmd),

    /// Spec88 Workpoint continuity operations.
    #[command(subcommand)]
    Workpoint(commands::workpoint::WorkpointCmd),

    /// API token management (docs/25).
    #[command(subcommand)]
    Tokens(commands::tokens::TokensCmd),

    /// Launch Pi only after bounded native-session preflight (Spec 130).
    #[command(subcommand)]
    Pi(commands::pi_launch::PiCmd),

    /// Wrap a harness CLI (Mode A proxy).
    ///
    /// Usage: focusa wrap -- <command> [args...]
    ///
    /// Starts the harness as subprocess, redirects API calls through Focusa.
    Wrap {
        /// Command and arguments to wrap.
        #[arg(trailing_var_arg = true, required = true)]
        command: Vec<String>,
    },
}

fn classify_cli_error(message: &str) -> (&'static str, &'static str, &'static str, &'static str) {
    if message.contains("[API_TIMEOUT]") {
        (
            "API_TIMEOUT",
            "API request timed out",
            "daemon overloaded or unreachable",
            "focusa doctor && focusa start",
        )
    } else if message.contains("[API_CONNECT_ERROR]") {
        (
            "API_CONNECT_ERROR",
            "Could not connect to Focusa API",
            "daemon down or port unavailable",
            "focusa start || focusa-daemon",
        )
    } else if message.contains("[CLI_SCOPE_REJECT]") {
        (
            "CLI_SCOPE_REJECT",
            "Scope is unsafe for durable project binding",
            "CLI validated an explicit project_root that maps to runtime/broad path",
            "Retry with a confirmed project folder path",
        )
    } else if message.contains("[API_HTTP_ERROR]") {
        (
            "API_HTTP_ERROR",
            "Focusa API returned an error status",
            "request rejected or server-side route failed",
            "focusa doctor && retry with --json",
        )
    } else if message.contains("[API_DECODE_ERROR]") {
        (
            "API_DECODE_ERROR",
            "Could not decode API response",
            "unexpected response shape or proxy error",
            "curl -sS http://127.0.0.1:8787/v1/health | jq .",
        )
    } else if message.contains("[API_REQUEST_ERROR]") {
        (
            "API_REQUEST_ERROR",
            "Focusa API request failed",
            "network/client failure",
            "focusa doctor",
        )
    } else if message.contains("[CLI_INPUT_ERROR]") {
        (
            "CLI_INPUT_ERROR",
            "CLI input rejected",
            "missing required safe flag or invalid arguments",
            "focusa --help",
        )
    } else {
        (
            "COMMAND_ERROR",
            "Command failed",
            "command-specific failure",
            "focusa doctor",
        )
    }
}

fn main() -> anyhow::Result<()> {
    // Clap's deeply nested command tree and Tokio initialization can exceed
    // the Windows process-main stack (0xC00000FD) before the command runs.
    // Parse and execute on an explicitly sized worker stack on every host so
    // Windows preflight has the same runtime behavior as Unix.
    std::thread::Builder::new()
        .name("focusa-main".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(async_main)
        .map_err(|error| anyhow::anyhow!("start Focusa main worker: {error}"))?
        .join()
        .map_err(|_| anyhow::anyhow!("Focusa main worker panicked"))?
}

#[tokio::main]
async fn async_main() -> anyhow::Result<()> {
    let raw_args: Vec<String> = std::env::args().collect();
    // Machine-readable errors must name the INVOKED command, not the recovery
    // suggestion (#367). Capture the subcommand line before clap moves it — but
    // argv is not a secret store: values of user-content flags (operator message
    // text, interactive keys, license keys, credentials) must never be echoed
    // into JSON error output, so their values are redacted in place.
    const REDACTED_VALUE_FLAGS: &[&str] = &[
        "--text",
        "--key",
        "--license-key",
        "--password",
        "--token",
        "--secret",
        "--message",
        "--body",
        "--input",
        "--note",
    ];
    let mut invoked_args: Vec<String> = Vec::new();
    let mut redact_next = false;
    for arg in raw_args.iter().skip(1) {
        if redact_next {
            invoked_args.push("[REDACTED]".to_string());
            redact_next = false;
            continue;
        }
        let flag = arg.split('=').next().unwrap_or(arg);
        if REDACTED_VALUE_FLAGS.contains(&flag) {
            if arg.contains('=') {
                let (name, _) = arg.split_once('=').unwrap_or((arg.as_str(), ""));
                invoked_args.push(format!("{name}=[REDACTED]"));
            } else {
                invoked_args.push(arg.clone());
                redact_next = true;
            }
            continue;
        }
        invoked_args.push(arg.clone());
    }
    let invoked: String = invoked_args.join(" ");
    // Handle -v (lowercase) as version before clap parsing.
    // Clap 4 auto-assigns -V for version but not -v.
    if raw_args.iter().any(|arg| arg == "-v") {
        println!("focusa {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    // Root help is intentionally concise for newcomers. Subcommand help still
    // routes through Clap and preserves complete command-specific options.
    if raw_args.len() == 2 && matches!(raw_args[1].as_str(), "--help" | "-h") {
        eprintln!("{}", commands::intro::render_help_banner());
        commands::help::print_root_help();
        return Ok(());
    }
    // Render the wordmark before subcommand help; stderr never pollutes JSON.
    if raw_args.iter().any(|arg| arg == "--help" || arg == "-h") {
        eprintln!("{}", commands::intro::render_help_banner());
    }
    let cli = Cli::parse_from(raw_args);

    // Initialize tracing with basic formatting
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                if cli.verbose {
                    "focusa=debug"
                } else {
                    "focusa=warn"
                }
                .into()
            }),
        )
        .with_writer(std::io::stderr)
        .finish();

    if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("[TRACING_INIT_WARNING] failed to set tracing subscriber: {err}");
    }

    let guided_flow = match &cli.command {
        Commands::Install(_) => Some(commands::lifecycle_guidance::Flow::Install),
        Commands::Update(_) => Some(commands::lifecycle_guidance::Flow::Update),
        Commands::Uninstall(_) => Some(commands::lifecycle_guidance::Flow::Uninstall),
        _ => None,
    };
    if cli.lifecycle_action.is_some() && guided_flow.is_none() {
        anyhow::bail!("--lifecycle-action is supported by install, update, and uninstall");
    }
    if let Some(flow) = guided_flow {
        let guided = commands::lifecycle_guidance::GuidedLifecycleArgs {
            action: cli.lifecycle_action,
            confirm: cli.confirm,
            confirm_purge_data: cli.confirm_purge_data,
        };
        if commands::lifecycle_guidance::prepare(&guided, flow, cli.json)? {
            return Ok(());
        }
    }

    let result: anyhow::Result<()> = match cli.command {
        Commands::Start => {
            let started = commands::daemon::start().await?;
            if !cli.json {
                if started {
                    println!("Focusa daemon started");
                } else {
                    println!("Focusa daemon already running (no-op)");
                }
            }
            Ok(())
        }
        Commands::Stop => {
            let outcome = commands::daemon::stop().await?;
            let status = match outcome {
                commands::daemon::StopOutcome::Stopped => "stopped",
                commands::daemon::StopOutcome::AlreadyStopped => "already_stopped",
            };
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "status": status,
                        "ok": true,
                        "command": "focusa stop",
                        "next_step_hint": if status == "stopped" { "Run focusa start when ready" } else { "Daemon is already stopped; run focusa start when ready" }
                    }))?
                );
            } else if outcome == commands::daemon::StopOutcome::Stopped {
                println!("Focusa daemon stopped");
            } else {
                println!("Focusa daemon already stopped (no-op)");
            }
            Ok(())
        }
        Commands::About => {
            commands::about::run(cli.json)?;
            Ok(())
        }
        Commands::Help(args) => commands::help::run(args, cli.json),
        Commands::Audit(args) => commands::audit::run(args, cli.json).await,
        Commands::InstallService(args) => commands::service::run(args, false).await,
        Commands::Install(args) => commands::install::run(args).await,
        Commands::Update(cmd) => commands::update::run(cmd, cli.json).await,
        Commands::Compaction(cmd) => commands::compaction::run(cmd, cli.json).await,
        Commands::DaemonRouting(cmd) => commands::daemon_routing::run(cmd, cli.json).await,
        Commands::Silent(cmd) => commands::silent::run(cmd, cli.json).await,
        Commands::Bg { cmd } => commands::bg::run(cmd, cli.json).await,
        Commands::Upgrade(args) => commands::upgrade::run(cli.json, args).await,
        Commands::Uninstall(args) => commands::uninstall::run(args).await,
        Commands::Codesign(args) => commands::codesign::run(args).await,
        Commands::Status {
            mode,
            agent,
            operator,
        } => {
            let agent = agent || mode == Some(StatusMode::Agent);
            let operator = operator || mode == Some(StatusMode::Operator);
            let api = api_client::ApiClient::new();
            let resp = api.get("/v1/status").await?;
            if operator {
                let health = api.get("/v1/health").await.unwrap_or_else(
                    |err| serde_json::json!({"status":"blocked","ok":false,"error":err.to_string()}),
                );
                let project = api.get("/v1/project/identity").await.unwrap_or_else(
                    |err| serde_json::json!({"status":"blocked","error":err.to_string()}),
                );
                let trajectory = api
                    .get("/v1/trajectory/view?mode=summary")
                    .await
                    .unwrap_or_else(
                        |err| serde_json::json!({"status":"blocked","error":err.to_string()}),
                    );
                let workpoint = api.post("/v1/workpoint/resume", &serde_json::json!({"mode":"operator_summary"})).await.unwrap_or_else(
                    |err| serde_json::json!({"status":"blocked","canonical":false,"error":err.to_string()}),
                );
                let evidence_count = workpoint
                    .get("evidence_refs")
                    .and_then(|v| v.as_array())
                    .map(|items| items.len())
                    .unwrap_or(0);
                let project_root = project
                    .pointer("/project_identity/project_root")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unbound");
                let continuity = workpoint
                    .get("continuity_id")
                    .and_then(|v| v.as_str())
                    .or_else(|| {
                        workpoint
                            .pointer("/resume_packet/continuity_id")
                            .and_then(|v| v.as_str())
                    })
                    .unwrap_or("unknown");
                let trajectory_summary = trajectory
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .or_else(|| trajectory.get("current_state").and_then(|v| v.as_str()))
                    .unwrap_or("trajectory unavailable or not yet defined");
                let active_gap = trajectory
                    .get("gap")
                    .and_then(|v| v.as_str())
                    .or_else(|| trajectory.get("active_gap").and_then(|v| v.as_str()))
                    .unwrap_or("unknown");
                let workpoint_id = workpoint
                    .get("workpoint_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("none");
                let next_action = workpoint
                    .get("next_step_hint")
                    .and_then(|v| v.as_str())
                    .or_else(|| workpoint.get("rendered_summary").and_then(|v| v.as_str()))
                    .unwrap_or("run focusa workpoint resume --mode compact_prompt");
                let envelope = serde_json::json!({
                    "status": "completed",
                    "summary": "Focusa operator session card",
                    "project": project_root,
                    "continuity": continuity,
                    "trajectory": trajectory_summary,
                    "trajectory_ladder": "HLT (High-Level Trajectory) -> MLG (Mid-Level Goal) -> STG (Short-Term Goal) -> Waypoints -> Workpoint; defer to operator while actively offering HLT-aligned route guidance",
                    "active_gap": active_gap,
                    "active_workpoint": workpoint_id,
                    "next_action": next_action,
                    "evidence_count": evidence_count,
                    "drift_status": "run focusa workpoint drift-check with the latest action to verify",
                    "health": if health.get("ok").and_then(|v| v.as_bool()) == Some(true) { "healthy" } else { "blocked" },
                    "details": {"status": resp, "health": health, "project": project, "trajectory": trajectory, "workpoint": workpoint}
                });
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&envelope)?);
                } else {
                    println!("FOCUSA SESSION CARD");
                    println!(
                        "Project: {}",
                        envelope["project"].as_str().unwrap_or("unbound")
                    );
                    println!(
                        "Continuity: {}",
                        envelope["continuity"].as_str().unwrap_or("unknown")
                    );
                    println!(
                        "Trajectory: {}",
                        envelope["trajectory"].as_str().unwrap_or("unavailable")
                    );
                    println!(
                        "Trajectory Ladder: {}",
                        envelope["trajectory_ladder"]
                            .as_str()
                            .unwrap_or("HLT -> MLG -> STG -> Waypoints -> Workpoint")
                    );
                    println!(
                        "Active Gap: {}",
                        envelope["active_gap"].as_str().unwrap_or("unknown")
                    );
                    println!(
                        "Active Workpoint: {}",
                        envelope["active_workpoint"].as_str().unwrap_or("none")
                    );
                    println!(
                        "Next Action: {}",
                        envelope["next_action"]
                            .as_str()
                            .unwrap_or("resume workpoint")
                    );
                    println!(
                        "Evidence: {} refs linked",
                        envelope["evidence_count"].as_u64().unwrap_or(0)
                    );
                    println!(
                        "Drift Status: {}",
                        envelope["drift_status"].as_str().unwrap_or("unknown")
                    );
                    println!(
                        "Health: {}",
                        envelope["health"].as_str().unwrap_or("unknown")
                    );
                }
            } else if agent {
                let workpoint = api.get("/v1/workpoint/current").await.unwrap_or_else(
                    |err| serde_json::json!({"status":"blocked","error":err.to_string()}),
                );
                let cwd = std::env::current_dir()
                    .ok()
                    .map(|path| path.to_string_lossy().into_owned());
                let work_loop =
                    match commands::scope_resolver::resolve_active_workstream_scope(cwd.as_deref())
                    {
                        Ok(scope) => {
                            let continuity_id = scope.continuity_id.as_deref().unwrap_or_default();
                            api.get_scoped(
                                "/v1/work-loop/status?summary_only=true",
                                &scope.project_root,
                                continuity_id,
                            )
                            .await
                            .unwrap_or_else(|err| {
                                serde_json::json!({
                                    "status":"not_configured",
                                    "failure_class":"work_loop_scope_unavailable",
                                    "error":err.to_string(),
                                    "daemon_restart_required":false,
                                })
                            })
                        }
                        Err(err) => serde_json::json!({
                            "status":"not_configured",
                            "failure_class":"work_loop_scope_unavailable",
                            "error":err.to_string(),
                            "daemon_restart_required":false,
                        }),
                    };
                let token_budget = api
                    .get("/v1/telemetry/token-budget/status?limit=5")
                    .await
                    .unwrap_or_else(
                        |err| serde_json::json!({"status":"blocked","error":err.to_string()}),
                    );
                let cache = api
                    .get("/v1/telemetry/cache-metadata/status?limit=5")
                    .await
                    .unwrap_or_else(
                        |err| serde_json::json!({"status":"blocked","error":err.to_string()}),
                    );
                let envelope = serde_json::json!({
                    "status": "completed",
                    "summary": "Agent status envelope refreshed from live Focusa surfaces",
                    "next_action": "Use focusa continue to resume governed work or focusa doctor for full diagnostics",
                    "why": "Spec92 requires a direct agent-first status view for current runtime/workflow state.",
                    "commands": ["focusa status --agent", "focusa continue", "focusa doctor"],
                    "recovery": ["focusa start", "focusa-daemon", "journalctl -u focusa-daemon -n 80 --no-pager (Linux service installs)"],
                    "evidence_refs": ["/v1/status", "/v1/workpoint/current", "/v1/work-loop/status?summary_only=true"],
                    "docs": ["docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md"],
                    "warnings": [],
                    "details": {"status": resp, "workpoint": workpoint, "work_loop": work_loop, "token_budget": token_budget, "cache": cache},
                });
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&envelope)?);
                } else {
                    println!(
                        "Status: {}",
                        envelope["status"].as_str().unwrap_or("completed")
                    );
                    println!(
                        "Summary: {}",
                        envelope["summary"]
                            .as_str()
                            .unwrap_or("agent status refreshed")
                    );
                    println!(
                        "Next action: {}",
                        envelope["next_action"]
                            .as_str()
                            .unwrap_or("focusa continue")
                    );
                    println!(
                        "Why: {}",
                        envelope["why"].as_str().unwrap_or("Spec92 agent status")
                    );
                    println!("Command: focusa continue");
                    println!("Recovery: focusa doctor && focusa start");
                    println!(
                        "Evidence: /v1/status, /v1/workpoint/current, /v1/work-loop/status?summary_only=true"
                    );
                    println!("Docs: docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md");
                }
            } else if cli.json {
                // Agent JSON status: enrich with additional fields from /v1/health,
                // /v1/project/identity, /v1/focus/frame/current, and /v1/license/status.
                let health = api.get("/v1/health").await.unwrap_or_else(
                    |err| serde_json::json!({"status":"blocked","ok":false,"error":err.to_string()}),
                );
                let project = api.get("/v1/project/identity").await.unwrap_or_else(
                    |err| serde_json::json!({"status":"blocked","error":err.to_string()}),
                );
                let frame = api.get("/v1/focus/frame/current").await.unwrap_or_else(
                    |err| serde_json::json!({"status":"blocked","error":err.to_string()}),
                );
                let license = api.get("/v1/license/status").await.unwrap_or_else(
                    |err| serde_json::json!({"status":"blocked","error":err.to_string()}),
                );
                let events = api.get("/v1/events/recent?limit=5").await.unwrap_or_else(
                    |err| serde_json::json!({"status":"blocked","error":err.to_string()}),
                );

                let uptime_secs = health
                    .get("uptime_ms")
                    .and_then(|v| v.as_u64())
                    .map(|ms| ms / 1000)
                    .unwrap_or(0);
                let current_frame = frame
                    .get("frame_id")
                    .or_else(|| frame.get("active_frame_id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("none");
                let license_status = license
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let scope_status = project
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let last_5_events: Vec<String> = events
                    .get("events")
                    .and_then(|e| e.as_array())
                    .map(|arr| {
                        arr.iter()
                            .take(5)
                            .filter_map(|e| {
                                e.get("kind")
                                    .and_then(|k| k.as_str())
                                    .map(|s| s.to_string())
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let envelope = serde_json::json!({
                    "status": resp.get("status").cloned().unwrap_or(serde_json::json!("unknown")),
                    "summary": "Focusa agent status (enriched)",
                    "daemon": {
                        "version": resp.get("app_version").and_then(|v| v.as_str()).unwrap_or(""),
                        "reducer": resp.get("version").and_then(|v| v.as_u64()).unwrap_or(0),
                        "uptime_secs": uptime_secs,
                        "pid": resp.get("runtime_process").and_then(|r| r.get("current_pid")).and_then(|p| p.as_u64()).unwrap_or(0),
                        "daemon_count": resp.get("runtime_process").and_then(|r| r.get("daemon_count")).and_then(|p| p.as_u64()).unwrap_or(0),
                        "duplicate_daemon_count": resp.get("runtime_process").and_then(|r| r.get("duplicate_daemon_count")).and_then(|p| p.as_u64()).unwrap_or(0),
                        "session_id": if resp["session"].is_null() { "none" } else { resp["session"]["session_id"].as_str().unwrap_or("unknown") },
                        "stack_depth": resp.get("stack_depth").and_then(|v| v.as_u64()).unwrap_or(0),
                        "ok": health.get("ok").and_then(|v| v.as_bool()).unwrap_or(false),
                    },
                    "current_frame": current_frame,
                    "license_status": license_status,
                    "scope_status": scope_status,
                    "last_5_events": last_5_events,
                    "details": {"status": resp, "health": health, "project": project, "frame": frame, "license": license, "events": events}
                });
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            } else {
                let app_version = resp["app_version"].as_str().unwrap_or("");
                let reducer_version = resp["version"].as_u64().unwrap_or(0);
                let depth = resp["stack_depth"].as_u64().unwrap_or(0);
                let session = if resp["session"].is_null() {
                    "none".to_string()
                } else {
                    resp["session"]["session_id"]
                        .as_str()
                        .unwrap_or("unknown")
                        .to_string()
                };
                let daemon_count = resp["runtime_process"]["daemon_count"]
                    .as_u64()
                    .unwrap_or(0);
                let duplicate_count = resp["runtime_process"]["duplicate_daemon_count"]
                    .as_u64()
                    .unwrap_or(0);
                let current_pid = resp["runtime_process"]["current_pid"].as_u64().unwrap_or(0);

                // Enrichment calls: current_frame, license_status, scope_status, uptime, last_5_events.
                let health = api.get("/v1/health").await.ok();
                let project = api.get("/v1/project/identity").await.ok();
                let frame = api.get("/v1/focus/frame/current").await.ok();
                let license = api.get("/v1/license/status").await.ok();
                let events = api.get("/v1/events/recent?limit=5").await.ok();
                let uptime_secs = health
                    .as_ref()
                    .and_then(|h| h.get("uptime_ms"))
                    .and_then(|v| v.as_u64())
                    .map(|ms| ms / 1000)
                    .unwrap_or(0);
                let current_frame = frame
                    .as_ref()
                    .and_then(|f| f.get("frame_id").or_else(|| f.get("active_frame_id")))
                    .and_then(|v| v.as_str())
                    .unwrap_or("none")
                    .to_string();
                let license_status = license
                    .as_ref()
                    .and_then(|l| l.get("status"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let scope_status = project
                    .as_ref()
                    .and_then(|p| p.get("status"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let last_5_events: Vec<String> = events
                    .as_ref()
                    .and_then(|e| e.get("events"))
                    .and_then(|e| e.as_array())
                    .map(|arr| {
                        arr.iter()
                            .take(5)
                            .filter_map(|e| {
                                e.get("kind")
                                    .and_then(|k| k.as_str())
                                    .map(|s| s.to_string())
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                println!("Focusa daemon: running");
                println!("  session:     {}", session);
                println!("  stack depth: {}", depth);
                if !app_version.is_empty() {
                    println!("  app version: {}", app_version);
                    println!("  reducer:     {}", reducer_version);
                } else {
                    println!("  version:     {}", reducer_version);
                }
                println!("  pid:         {}", current_pid);
                println!("  daemons:     {}", daemon_count);
                println!("  uptime_secs: {}", uptime_secs);
                println!("  current_frame: {}", current_frame);
                println!("  license_status: {}", license_status);
                println!("  scope_status: {}", scope_status);
                if !last_5_events.is_empty() {
                    println!("  last_5_events: {}", last_5_events.join(", "));
                }
                if duplicate_count > 0 {
                    println!(
                        "  warning:     duplicate daemons detected ({})",
                        duplicate_count
                    );
                }
            }
            Ok(())
        }
        Commands::Onboard(args) => {
            commands::help::warn_alias("focusa onboard", "focusa setup wizard");
            commands::onboard::run(args, cli.json).await
        }
        Commands::Pair(args) => {
            commands::help::warn_alias("focusa pair", "focusa pairing start");
            commands::pair::run(args, cli.json).await
        }
        Commands::PairingDoctor(args) => {
            commands::help::warn_alias("focusa pairing-doctor", "focusa pairing doctor");
            commands::pairing_doctor::run(args).await
        }
        Commands::PairingWizard(cmd) => {
            commands::help::warn_alias("focusa pairing-wizard", "focusa pairing wizard");
            commands::pairing_wizard::run(cmd).await
        }
        Commands::Pairing(cmd) => run_pairing(cmd).await,
        Commands::PairingTransport(cmd) => {
            commands::help::warn_alias("focusa pairing-transport", "focusa pairing transport");
            commands::pairing_transport::run(cmd).await
        }
        Commands::Doctor(args) => commands::doctor::run(cli.json, args).await,
        Commands::License(args) => commands::license::run(cli.json, args).await,
        Commands::Preflight => {
            commands::help::warn_alias(
                "focusa preflight",
                "focusa setup doctor / focusa quality preflight",
            );
            commands::dxux::preflight().await
        }
        Commands::Recover(args) => commands::recover::run(cli.json, args).await,
        Commands::Explain { failure } => commands::dxux::explain(failure).await,
        Commands::Dxux(cmd) => {
            let mut client = crate::api_client::ApiClient::new();
            commands::dxux::handle(&mut client, cmd).await
        }
        Commands::Utility(cmd) => {
            let mut client = crate::api_client::ApiClient::new();
            commands::utility::handle(&mut client, cmd, cli.json).await
        }
        Commands::Cleanup(args) => commands::cleanup::run(args, cli.json).await,
        Commands::Continue(args) => commands::continue_work::run(args, cli.json).await,
        Commands::WorkLoop(cmd) => commands::work_loop::run(cmd, cli.json).await,
        Commands::Tui(args) => commands::tui::run(args, cli.json).await,
        Commands::Init(args) => commands::init::run(args, cli.json).await,
        Commands::Walkthrough(args) => {
            commands::help::warn_alias("focusa walkthrough", "focusa setup walkthrough");
            commands::walkthrough::run(args, cli.json).await
        }
        Commands::Preload(args) => commands::preload::run(args, cli.json).await,
        Commands::Deck(args) => commands::deck::run(args, cli.json).await,
        Commands::Workflow(cmd) => commands::workflow::run(cmd, cli.json).await,
        Commands::Stack => {
            commands::help::warn_alias("focusa stack", "focusa focus stack");
            let api = api_client::ApiClient::new();
            let resp = api.get("/v1/focus/stack").await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                let active = resp["active_frame_id"].as_str().unwrap_or("none");
                println!("Active: {}", active);
                if let Some(stack) = resp["stack"].as_object()
                    && let Some(frames) = stack.get("frames").and_then(|f| f.as_array())
                {
                    if frames.is_empty() {
                        println!("  (empty stack)");
                    }
                    for frame in frames {
                        let status = frame["status"].as_str().unwrap_or("?");
                        let title = frame["title"].as_str().unwrap_or("?");
                        let id = frame["id"].as_str().unwrap_or("?");
                        let marker = if Some(id) == resp["active_frame_id"].as_str() {
                            "►"
                        } else {
                            " "
                        };
                        let short_id = if id.len() >= 8 { &id[..8] } else { id };
                        println!("  {} [{}] {} ({})", marker, status, title, short_id);
                    }
                }
            }
            Ok(())
        }
        Commands::Focus(cmd) => commands::focus::run(cmd, cli.json).await,
        Commands::Gate(cmd) => commands::gate::run(cmd, cli.json).await,
        Commands::WorkItem(cmd) => commands::work_item::run(cmd).await,
        Commands::Action(cmd) => commands::action::run(cmd, cli.json).await,
        Commands::Runtime(cmd) => commands::runtime::run(cmd, cli.json).await,
        Commands::Binary(cmd) => commands::binary::run(cmd, cli.json).await,
        Commands::Claim(cmd) => commands::claim::run(cmd, cli.json).await,
        Commands::Memory(cmd) => commands::memory::run(cmd, cli.json).await,
        Commands::Ecs(cmd) => commands::ecs::run(cmd, cli.json).await,
        Commands::Env(cmd) => commands::env::run(cmd, cli.json).await,
        Commands::Events(cmd) => commands::debug::run_events(cmd, cli.json).await,
        Commands::Turns(cmd) => commands::turns::run(cmd, cli.json).await,
        Commands::State { cmd } => commands::debug::run_state(cmd, cli.json).await,
        Commands::Clt(cmd) => commands::clt::run(cmd, cli.json).await,
        Commands::Lineage(cmd) => commands::lineage::run(cmd, cli.json).await,
        Commands::Autonomy(cmd) => commands::autonomy::run(cmd, cli.json).await,
        Commands::Awareness(cmd) => commands::awareness::run(cmd, cli.json).await,
        Commands::Constitution(cmd) => commands::constitution::run(cmd, cli.json).await,
        Commands::AgentRuntime(cmd) => commands::agent_runtime::run(cmd, cli.json).await,
        Commands::Telemetry(cmd) => commands::telemetry::run(cmd, cli.json).await,
        Commands::Rfm(cmd) => commands::rfm::run(cmd, cli.json).await,
        Commands::Release(cmd) => commands::release::run(cmd, cli.json).await,
        Commands::Proposals(cmd) => commands::proposals::run(cmd, cli.json).await,
        Commands::Predict(cmd) => commands::predict::run(cmd, cli.json).await,
        Commands::Reflect(cmd) => commands::reflection::run(cmd, cli.json).await,
        Commands::Metacognition(cmd) => commands::metacognition::run(cmd, cli.json).await,
        Commands::Ontology(cmd) => commands::ontology::run(cmd, cli.json).await,
        Commands::SemanticIntegrity(cmd) => commands::semantic_integrity::run(cmd, cli.json).await,
        Commands::Skills(cmd) => commands::skills::run(cmd, cli.json).await,
        Commands::Thread(cmd) => {
            commands::threads::run(cmd, cli.json, &api_client::ApiClient::new()).await
        }
        Commands::Export(cmd) => commands::export::run(cmd, cli.json).await,
        Commands::Contribute(cmd) => commands::contribute::run(cmd, cli.json).await,
        Commands::Cache(cmd) => commands::cache::run(cmd, cli.json).await,
        Commands::WorkingSet(cmd) => commands::working_set::run(cmd, cli.json).await,
        Commands::FirstMission(args) => commands::first_mission::run(args, cli.json).await,
        Commands::Setup(cmd) => commands::setup::run(cmd, cli.json).await,
        Commands::Project(cmd) => commands::project::run(cmd, cli.json).await,
        Commands::Resource(cmd) => commands::resource::run(cmd, cli.json).await,
        Commands::Temporal(cmd) => commands::temporal::run(cmd, cli.json).await,
        Commands::Time(cmd) => commands::temporal_clients::run_time(cmd, cli.json).await,
        Commands::Deadline(cmd) => commands::temporal_clients::run_deadline(cmd, cli.json).await,
        Commands::Estimate(cmd) => commands::temporal_clients::run_estimate(cmd, cli.json).await,
        Commands::Progress(cmd) => commands::temporal_clients::run_progress(cmd, cli.json).await,
        Commands::NoProgress(cmd) => {
            commands::temporal_clients::run_no_progress(cmd, cli.json).await
        }
        Commands::LostTime(cmd) => commands::temporal_clients::run_lost_time(cmd, cli.json).await,
        Commands::Opportunity(cmd) => {
            commands::temporal_clients::run_opportunity(cmd, cli.json).await
        }
        Commands::Cancellation(cmd) => {
            commands::temporal_clients::run_cancellation(cmd, cli.json).await
        }
        Commands::Trajectory(cmd) => commands::trajectory::run(cmd, cli.json).await,
        Commands::Hlt(cmd) => commands::hlt::run(cmd, cli.json).await,
        Commands::Traverse(cmd) => commands::traverse::run(cmd, cli.json).await,
        Commands::ContextCognition(cmd) => {
            let mut client = crate::api_client::ApiClient::new();
            commands::context_cognition::handle(&mut client, cmd).await
        }
        Commands::Bloatgaurd(cmd) => {
            let mut client = crate::api_client::ApiClient::new();
            commands::bloatgaurd::handle(&mut client, cmd).await
        }
        Commands::CallStack(cmd) => commands::call_stack::run(cmd, cli.json).await,
        Commands::Device(cmd) => {
            let mut client = crate::api_client::ApiClient::new();
            commands::device_pairing::handle(&mut client, cmd).await
        }
        Commands::Workpoint(cmd) => commands::workpoint::run(cmd, cli.json).await,
        Commands::Tokens(cmd) => commands::tokens::run(cmd, cli.json).await,
        Commands::Pi(cmd) => commands::pi_launch::run(cmd, cli.json),
        Commands::Wrap { command } => commands::wrap::run(command, cli.verbose).await,
    };

    if let Err(err) = result {
        if cli.json {
            let error_message = err.to_string();
            let (code, what_failed, likely_why, safe_recovery) = classify_cli_error(&error_message);
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "blocked",
                    "code": code,
                    "what_failed": what_failed,
                    "likely_why": likely_why,
                    "safe_recovery": safe_recovery,
                    "command": invoked,
                    "fallback": "focusa doctor",
                    "docs": ["docs/current/ERROR_EMPTY_STATES.md", "docs/current/TROUBLESHOOTING_CURRENT.md"],
                    "evidence_refs": [],
                    "severity": "blocked",
                    "details": { "raw_error": error_message },
                }))?
            );
            // Machine-readable errors must retain a failing process status. A JSON
            // envelope is not a successful install/preflight result.
            std::process::exit(1);
        }
        return Err(err);
    }

    Ok(())
}
