//! Focusa CLI — primary control interface.
//!
//! Source: docs/G1-13-cli.md
//!
//! Binary: `focusa`
//! Thin facade — zero business logic beyond arg parsing + API calls.

use clap::{Parser, Subcommand};

mod api_client;
mod commands;

#[derive(Parser)]
#[command(name = "focusa", about = "Focusa cognitive governance CLI")]
#[command(version, propagate_version = true)]
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

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Focusa daemon.
    Start,

    /// Stop the Focusa daemon.
    Stop,

    /// Install and enable the Focusa daemon service (Linux systemd user / macOS LaunchAgent).
    InstallService(commands::service::InstallServiceArgs),

    /// macOS code signing + notarization inspection helper (focusa-covz).
    Codesign(commands::codesign::CodesignArgs),

    /// Show daemon status.
    Status {
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

    /// Discover and write the best phone-reachable Focusa transport (multi-transport bundle, focusa-ifc3).
    #[command(subcommand)]
    PairingTransport(commands::pairing_transport::TransportCmd),

    /// Single-command pairing root-cause report (focusa-gkrj).
    PairingDoctor(commands::pairing_doctor::DoctorArgs),

    /// Run full agent-first doctor checks.
    Doctor(commands::doctor::DoctorArgs),

    /// License activation and entitlement operations (Spec92 §5.2).
    License(commands::license::LicenseArgs),

    /// Run Spec105 local CI/spec/evidence preflight.
    Preflight,

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

    /// Focus stack and Focus State operations.
    #[command(subcommand)]
    Focus(commands::focus::FocusCmd),

    /// Show focus stack overview.
    Stack,

    /// Focus Gate (candidate management).
    #[command(subcommand)]
    Gate(commands::gate::GateCmd),

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

    /// Project identity discovery and verification (Spec96).
    #[command(subcommand)]
    Project(commands::project::ProjectCmd),

    /// ResourceMode / LowMem control (Spec96).
    #[command(subcommand)]
    Resource(commands::resource::ResourceCmd),

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

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
            commands::daemon::stop().await?;
            if !cli.json {
                println!("Focusa daemon stopped");
            }
            Ok(())
        }
        Commands::InstallService(args) => commands::service::run(args, false).await,
        Commands::Codesign(args) => commands::codesign::run(args).await,
        Commands::Status { agent, operator } => {
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
                let work_loop = api
                    .get("/v1/work-loop/status?summary_only=true")
                    .await
                    .unwrap_or_else(
                        |err| serde_json::json!({"status":"blocked","error":err.to_string()}),
                    );
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
                println!("{}", serde_json::to_string_pretty(&resp)?);
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
                if duplicate_count > 0 {
                    println!(
                        "  warning:     duplicate daemons detected ({})",
                        duplicate_count
                    );
                }
            }
            Ok(())
        }
        Commands::Onboard(args) => commands::onboard::run(args, cli.json).await,
        Commands::Pair(args) => commands::pair::run(args, cli.json).await,
        Commands::PairingDoctor(args) => commands::pairing_doctor::run(args).await,
        Commands::PairingTransport(cmd) => commands::pairing_transport::run(cmd).await,
        Commands::Doctor(args) => commands::doctor::run(cli.json, args).await,
        Commands::License(args) => commands::license::run(cli.json, args).await,
        Commands::Preflight => commands::dxux::preflight().await,
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
        Commands::Stack => {
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
        Commands::Telemetry(cmd) => commands::telemetry::run(cmd, cli.json).await,
        Commands::Rfm(cmd) => commands::rfm::run(cmd, cli.json).await,
        Commands::Release(cmd) => commands::release::run(cmd, cli.json).await,
        Commands::Proposals(cmd) => commands::proposals::run(cmd, cli.json).await,
        Commands::Predict(cmd) => commands::predict::run(cmd, cli.json).await,
        Commands::Reflect(cmd) => commands::reflection::run(cmd, cli.json).await,
        Commands::Metacognition(cmd) => commands::metacognition::run(cmd, cli.json).await,
        Commands::Ontology(cmd) => commands::ontology::run(cmd, cli.json).await,
        Commands::Skills(cmd) => commands::skills::run(cmd, cli.json).await,
        Commands::Thread(cmd) => {
            commands::threads::run(cmd, cli.json, &api_client::ApiClient::new()).await
        }
        Commands::Export(cmd) => commands::export::run(cmd, cli.json).await,
        Commands::Contribute(cmd) => commands::contribute::run(cmd, cli.json).await,
        Commands::Cache(cmd) => commands::cache::run(cmd, cli.json).await,
        Commands::Project(cmd) => commands::project::run(cmd, cli.json).await,
        Commands::Resource(cmd) => commands::resource::run(cmd, cli.json).await,
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
        Commands::Wrap { command } => commands::wrap::run(command).await,
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
                    "command": safe_recovery,
                    "fallback": "focusa doctor",
                    "docs": ["docs/current/ERROR_EMPTY_STATES.md", "docs/current/TROUBLESHOOTING_CURRENT.md"],
                    "evidence_refs": [],
                    "severity": "blocked",
                    "details": { "raw_error": error_message },
                }))?
            );
            return Ok(());
        }
        return Err(err);
    }

    Ok(())
}
