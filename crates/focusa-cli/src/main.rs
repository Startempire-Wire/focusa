//! Focusa CLI — primary control interface.
//!
//! Source: docs/G1-13-cli.md
//!
//! Binary: `focusa`
//! Thin facade — zero business logic beyond arg parsing + API calls.

use clap::{Parser, Subcommand};

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

    /// Suppress non-essential output.
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

    /// Focus stack overview.
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

    /// Debug and inspection.
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

    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("focusa=debug")
            .init();
    }

    match cli.command {
        Commands::Start => {
            println!("Starting Focusa daemon...");
            // TODO: Start daemon process
        }
        Commands::Stop => {
            println!("Stopping Focusa daemon...");
            // TODO: Stop daemon
        }
        Commands::Status => {
            println!("Focusa daemon status: not running");
            // TODO: GET /v1/status
        }
        Commands::Focus(cmd) => commands::focus::run(cmd, cli.json).await?,
        Commands::Stack => {
            println!("Focus stack: (empty)");
            // TODO: GET /v1/focus/stack
        }
        Commands::Gate(cmd) => commands::gate::run(cmd, cli.json).await?,
        Commands::Memory(cmd) => commands::memory::run(cmd, cli.json).await?,
        Commands::Ecs(cmd) => commands::ecs::run(cmd, cli.json).await?,
        Commands::Events(cmd) => commands::debug::run_events(cmd, cli.json).await?,
        Commands::State { cmd } => commands::debug::run_state(cmd, cli.json).await?,
    }

    Ok(())
}
