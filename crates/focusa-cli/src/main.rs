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

    /// Verbose output.
    #[arg(long, global = true)]
    verbose: bool,

    /// Suppress non-essential output.
    #[arg(long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

    /// Event log inspection.
    #[command(subcommand)]
    Events(commands::debug::EventsCmd),

    /// Dump full state (debug).
    State {
        #[command(subcommand)]
        cmd: commands::debug::StateCmd,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let default_filter = if cli.verbose {
        "focusa=debug"
    } else {
        "focusa=warn"
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| default_filter.into()),
        )
        .init();

    match cli.command {
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
                let active = resp["active_frame_id"]
                    .as_str()
                    .unwrap_or("none");
                println!("Active: {}", active);
                if let Some(stack) = resp["stack"].as_object() {
                    if let Some(frames) = stack.get("frames").and_then(|f| f.as_array()) {
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
        }
        Commands::Focus(cmd) => commands::focus::run(cmd, cli.json).await?,
        Commands::Gate(cmd) => commands::gate::run(cmd, cli.json).await?,
        Commands::Memory(cmd) => commands::memory::run(cmd, cli.json).await?,
        Commands::Ecs(cmd) => commands::ecs::run(cmd, cli.json).await?,
        Commands::Events(cmd) => commands::debug::run_events(cmd, cli.json).await?,
        Commands::State { cmd } => commands::debug::run_state(cmd).await?,
    }

    Ok(())
}
