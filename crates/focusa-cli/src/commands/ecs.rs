//! ECS (reference store) CLI commands.

use clap::Subcommand;

#[derive(Subcommand)]
pub enum EcsCmd {
    /// List stored handles.
    List,
    /// Print artifact content.
    Cat { handle_id: String },
    /// Show handle metadata.
    Meta { handle_id: String },
    /// Rehydrate artifact with token limit.
    Rehydrate {
        handle_id: String,
        #[arg(long, default_value = "300")]
        max_tokens: u32,
    },
}

pub async fn run(cmd: EcsCmd, _json: bool) -> anyhow::Result<()> {
    match cmd {
        EcsCmd::List => println!("ECS handles: (none)"),
        EcsCmd::Cat { handle_id } => println!("Cat: {}", handle_id),
        EcsCmd::Meta { handle_id } => println!("Meta: {}", handle_id),
        EcsCmd::Rehydrate {
            handle_id,
            max_tokens,
        } => println!("Rehydrate {} (max {} tokens)", handle_id, max_tokens),
    }
    Ok(())
}
