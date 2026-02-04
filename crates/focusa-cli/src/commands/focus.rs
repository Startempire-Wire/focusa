//! Focus stack CLI commands.

use clap::Subcommand;

#[derive(Subcommand)]
pub enum FocusCmd {
    /// Push a new focus frame.
    Push {
        /// Frame title.
        title: String,
        /// Frame goal.
        #[arg(long)]
        goal: String,
    },
    /// Pop (complete) the active frame.
    Pop,
    /// Complete the active frame with reason.
    Complete {
        /// Completion reason.
        #[arg(long, default_value = "goal_achieved")]
        reason: String,
    },
    /// Set active frame by ID.
    Set {
        /// Frame ID.
        frame_id: String,
    },
}

pub async fn run(cmd: FocusCmd, json: bool) -> anyhow::Result<()> {
    match cmd {
        FocusCmd::Push { title, goal } => {
            if json {
                println!("{{\"action\":\"push\",\"title\":\"{}\",\"goal\":\"{}\"}}", title, goal);
            } else {
                println!("Push frame: {} (goal: {})", title, goal);
            }
            // TODO: POST /v1/focus/push
        }
        FocusCmd::Pop => {
            println!("Pop active frame");
            // TODO: POST /v1/focus/pop
        }
        FocusCmd::Complete { reason } => {
            println!("Complete frame: {}", reason);
            // TODO: POST /v1/focus/pop with reason
        }
        FocusCmd::Set { frame_id } => {
            println!("Set active frame: {}", frame_id);
            // TODO: POST /v1/focus/set-active
        }
    }
    Ok(())
}
