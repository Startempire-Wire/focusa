//! Memory CLI commands.

use clap::Subcommand;

#[derive(Subcommand)]
pub enum MemoryCmd {
    /// List semantic memory.
    List,
    /// Set a semantic key=value.
    Set { key_value: String },
    /// Show procedural rules.
    Rules,
}

pub async fn run(cmd: MemoryCmd, _json: bool) -> anyhow::Result<()> {
    match cmd {
        MemoryCmd::List => println!("Semantic memory: (empty)"),
        MemoryCmd::Set { key_value } => println!("Set: {}", key_value),
        MemoryCmd::Rules => println!("Procedural rules: (none)"),
    }
    Ok(())
}
