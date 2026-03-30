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
    Status,

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

    /// Proposal Resolution Engine.
    #[command(subcommand)]
    Proposals(commands::proposals::ProposalCmd),

    /// Reflection loop overlay.
    #[command(subcommand)]
    Reflect(commands::reflection::ReflectionCmd),

    /// Agent skills.
    #[command(subcommand)]
    Skills(commands::skills::SkillsCmd),

    /// Thread operations (docs/38).
    #[command(subcommand)]
    Thread(commands::threads::ThreadCmd),

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

    match cli.command {
        Commands::Start => {
            commands::daemon::start().await?;
            if !cli.json {
                println!("Focusa daemon started");
            }
        }
        Commands::Stop => {
            commands::daemon::stop().await?;
            if !cli.json {
                println!("Focusa daemon stopped");
            }
        }
        Commands::Status => {
            let api = api_client::ApiClient::new();
            let resp = api.get("/v1/status").await?;
            if cli.json {
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
                println!("Focusa daemon: running");
                println!("  session:     {}", session);
                println!("  stack depth: {}", depth);
                println!("  version:     {}", version);
            }
        }
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
        }
        Commands::Focus(cmd) => commands::focus::run(cmd, cli.json).await?,
        Commands::Gate(cmd) => commands::gate::run(cmd, cli.json).await?,
        Commands::Memory(cmd) => commands::memory::run(cmd, cli.json).await?,
        Commands::Ecs(cmd) => commands::ecs::run(cmd, cli.json).await?,
        Commands::Env(cmd) => commands::env::run(cmd, cli.json).await?,
        Commands::Events(cmd) => commands::debug::run_events(cmd, cli.json).await?,
        Commands::Turns(cmd) => commands::turns::run(cmd, cli.json).await?,
        Commands::State { cmd } => commands::debug::run_state(cmd).await?,
        Commands::Clt(cmd) => commands::clt::run(cmd, cli.json).await?,
        Commands::Autonomy(cmd) => commands::autonomy::run(cmd, cli.json).await?,
        Commands::Constitution(cmd) => commands::constitution::run(cmd, cli.json).await?,
        Commands::Telemetry(cmd) => commands::telemetry::run(cmd, cli.json).await?,
        Commands::Rfm(cmd) => commands::rfm::run(cmd, cli.json).await?,
        Commands::Proposals(cmd) => commands::proposals::run(cmd, cli.json).await?,
        Commands::Reflect(cmd) => commands::reflection::run(cmd, cli.json).await?,
        Commands::Skills(cmd) => commands::skills::run(cmd, cli.json).await?,
        Commands::Thread(cmd) => {
            commands::threads::run(cmd, cli.json, &api_client::ApiClient::new()).await?
        }
        Commands::Wrap { command } => commands::wrap::run(command).await?,
    }

    Ok(())
}
