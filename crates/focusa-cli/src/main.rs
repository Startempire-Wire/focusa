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

    /// Show daemon status.
    Status {
        /// Agent-first status envelope with Workpoint, Work-loop, token, and cache details.
        #[arg(long)]
        agent: bool,
    },

    /// Run full agent-first doctor checks.
    Doctor,

    /// Resume governed continuous work and refresh state.
    Continue(commands::continue_work::ContinueArgs),

    /// Focus stack operations.
    #[command(subcommand)]
    Focus(commands::focus::FocusCmd),

    /// Show focus stack overview.
    Stack,

    /// Focus Gate (candidate management).
    #[command(subcommand)]
    Gate(commands::gate::GateCmd),

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

fn classify_cli_error(message: &str) -> (&'static str, &str) {
    if message.contains("[API_TIMEOUT]") {
        ("API_TIMEOUT", message)
    } else if message.contains("[API_CONNECT_ERROR]") {
        ("API_CONNECT_ERROR", message)
    } else if message.contains("[API_HTTP_ERROR]") {
        ("API_HTTP_ERROR", message)
    } else if message.contains("[API_DECODE_ERROR]") {
        ("API_DECODE_ERROR", message)
    } else if message.contains("[API_REQUEST_ERROR]") {
        ("API_REQUEST_ERROR", message)
    } else if message.contains("[CLI_INPUT_ERROR]") {
        ("CLI_INPUT_ERROR", message)
    } else {
        ("COMMAND_ERROR", message)
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

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

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
        Commands::Status { agent } => {
            let api = api_client::ApiClient::new();
            let resp = api.get("/v1/status").await?;
            if agent {
                let workpoint = api.get("/v1/workpoint/current").await.unwrap_or_else(
                    |err| serde_json::json!({"status":"blocked","error":err.to_string()}),
                );
                let work_loop = api.get("/v1/work-loop/status").await.unwrap_or_else(
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
                    "recovery": ["focusa start", "journalctl -u focusa-daemon -n 80 --no-pager"],
                    "evidence_refs": ["/v1/status", "/v1/workpoint/current", "/v1/work-loop/status"],
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
                    println!("Recovery: focusa doctor && systemctl status focusa-daemon");
                    println!("Evidence: /v1/status, /v1/workpoint/current, /v1/work-loop/status");
                    println!("Docs: docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md");
                }
            } else if cli.json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                let version = resp["version"].as_u64().unwrap_or(0);
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
                println!("  version:     {}", version);
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
        Commands::Doctor => commands::doctor::run(cli.json).await,
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
        Commands::Memory(cmd) => commands::memory::run(cmd, cli.json).await,
        Commands::Ecs(cmd) => commands::ecs::run(cmd, cli.json).await,
        Commands::Env(cmd) => commands::env::run(cmd, cli.json).await,
        Commands::Events(cmd) => commands::debug::run_events(cmd, cli.json).await,
        Commands::Turns(cmd) => commands::turns::run(cmd, cli.json).await,
        Commands::State { cmd } => commands::debug::run_state(cmd, cli.json).await,
        Commands::Clt(cmd) => commands::clt::run(cmd, cli.json).await,
        Commands::Lineage(cmd) => commands::lineage::run(cmd, cli.json).await,
        Commands::Autonomy(cmd) => commands::autonomy::run(cmd, cli.json).await,
        Commands::Constitution(cmd) => commands::constitution::run(cmd, cli.json).await,
        Commands::Telemetry(cmd) => commands::telemetry::run(cmd, cli.json).await,
        Commands::Rfm(cmd) => commands::rfm::run(cmd, cli.json).await,
        Commands::Release(cmd) => commands::release::run(cmd, cli.json).await,
        Commands::Proposals(cmd) => commands::proposals::run(cmd, cli.json).await,
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
        Commands::Workpoint(cmd) => commands::workpoint::run(cmd, cli.json).await,
        Commands::Tokens(cmd) => commands::tokens::run(cmd, cli.json).await,
        Commands::Wrap { command } => commands::wrap::run(command).await,
    };

    if let Err(err) = result {
        if cli.json {
            let error_message = err.to_string();
            let (code, reason) = classify_cli_error(&error_message);
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "error",
                    "code": code,
                    "reason": reason,
                }))?
            );
            return Ok(());
        }
        return Err(err);
    }

    Ok(())
}
