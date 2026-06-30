//! `focusa claim` — completion claim gate CLI.
//!
//! Enforces Spec107 evidence-quality discipline before beads can be closed.

use clap::{Parser, Subcommand};
use focusa_core::claim_gate::{ClaimGateInput, ClaimGateOutput, GateDecision};
use std::io::{self, Read};

#[derive(Subcommand)]
pub enum ClaimCmd {
    /// Classify a completion claim against acceptance criteria.
    Classify(ClaimClassifyArgs),
}

#[derive(Parser, Debug)]
pub struct ClaimClassifyArgs {
    /// Work item / bead ID.
    #[arg(long)]
    pub work_item_id: Option<String>,

    /// Claim text / close reason.
    #[arg(short = 'c', long)]
    pub claim: Option<String>,

    /// Read claim text from stdin.
    #[arg(short, long)]
    pub stdin: bool,

    /// Operator has explicitly deferred a blocked claim.
    #[arg(long)]
    pub deferred: bool,
}

impl ClaimClassifyArgs {
    /// Run the claim gate.
    pub async fn run(self) -> anyhow::Result<()> {
        // Read claim text
        let claim_text = if self.stdin {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            buf.trim().to_string()
        } else if let Some(ref c) = self.claim {
            c.clone()
        } else {
            return Err(anyhow::anyhow!("--claim or --stdin required"));
        };

        let work_item_id = self.work_item_id.unwrap_or_else(|| "unknown".to_string());

        let input = ClaimGateInput {
            work_item_id,
            claim_text,
            operator_deferred: self.deferred,
            ..Default::default()
        };

        let output = ClaimGateOutput::build(&input);

        println!("{}", output.summary());

        if output.decision == GateDecision::Block {
            std::process::exit(1);
        }

        Ok(())
    }
}

/// Entry point for the claim command.
pub async fn run(cmd: ClaimCmd, _json_mode: bool) -> anyhow::Result<()> {
    match cmd {
        ClaimCmd::Classify(args) => args.run().await?,
    }
    Ok(())
}
