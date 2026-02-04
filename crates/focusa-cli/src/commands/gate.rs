//! Focus Gate CLI commands.

use clap::Subcommand;

#[derive(Subcommand)]
pub enum GateCmd {
    /// List candidates.
    List,
    /// Suppress a candidate.
    Suppress {
        candidate_id: String,
        #[arg(long, default_value = "10m")]
        r#for: String,
    },
    /// Resolve a candidate.
    Resolve { candidate_id: String },
    /// Promote candidate → push focus frame.
    Promote { candidate_id: String },
    /// Pin a candidate.
    Pin { candidate_id: String },
    /// Unpin a candidate.
    Unpin { candidate_id: String },
}

pub async fn run(cmd: GateCmd, _json: bool) -> anyhow::Result<()> {
    match cmd {
        GateCmd::List => println!("Gate candidates: (none)"),
        GateCmd::Suppress { candidate_id, r#for } => {
            println!("Suppress {} for {}", candidate_id, r#for)
        }
        GateCmd::Resolve { candidate_id } => println!("Resolve {}", candidate_id),
        GateCmd::Promote { candidate_id } => println!("Promote {}", candidate_id),
        GateCmd::Pin { candidate_id } => println!("Pin {}", candidate_id),
        GateCmd::Unpin { candidate_id } => println!("Unpin {}", candidate_id),
    }
    Ok(())
}
