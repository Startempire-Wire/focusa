//! Debug and inspection CLI commands.

use clap::Subcommand;

#[derive(Subcommand)]
pub enum EventsCmd {
    /// Tail recent events.
    Tail {
        #[arg(long, default_value = "20")]
        limit: u32,
    },
    /// Show a specific event.
    Show { event_id: String },
}

#[derive(Subcommand)]
pub enum StateCmd {
    /// Dump full cognitive state.
    Dump,
}

pub async fn run_events(cmd: EventsCmd, _json: bool) -> anyhow::Result<()> {
    match cmd {
        EventsCmd::Tail { limit } => println!("Last {} events: (none)", limit),
        EventsCmd::Show { event_id } => println!("Event: {}", event_id),
    }
    Ok(())
}

pub async fn run_state(cmd: StateCmd, _json: bool) -> anyhow::Result<()> {
    match cmd {
        StateCmd::Dump => {
            let state = focusa_core::types::FocusaState::new();
            println!("{}", serde_json::to_string_pretty(&state)?);
        }
    }
    Ok(())
}
