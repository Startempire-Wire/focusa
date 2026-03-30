//! Export CLI — docs/21-data-export-cli.md
//!
//! focusa export <dataset_type> --output <path> [options]

use crate::api_client::ApiClient;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum ExportCmd {
    /// Export SFT dataset.
    Sft {
        /// Output path.
        #[arg(long)]
        output: String,
        /// Minimum UXP score (default 0.7).
        #[arg(long, default_value = "0.7")]
        min_uxp: f64,
        /// Maximum UFI score (default 0.3).
        #[arg(long, default_value = "0.3")]
        max_ufi: f64,
        /// Dry run — show stats without writing.
        #[arg(long)]
        dry_run: bool,
    },
    /// Export preference dataset.
    Preference {
        #[arg(long)]
        output: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// Export contrastive dataset.
    Contrastive {
        #[arg(long)]
        output: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// Export long-horizon dataset.
    LongHorizon {
        #[arg(long)]
        output: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// Show export pipeline status.
    Status,
}

pub async fn run(cmd: ExportCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        ExportCmd::Status => {
            let resp = api.get("/v1/training/status").await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("Export Pipeline Status:");
                println!(
                    "  Contribution: {}",
                    if resp["contribution_enabled"].as_bool().unwrap_or(false) {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
                println!(
                    "  Queue: {} items ({} pending, {} approved)",
                    resp["queue_size"].as_u64().unwrap_or(0),
                    resp["pending"].as_u64().unwrap_or(0),
                    resp["approved"].as_u64().unwrap_or(0),
                );
                println!(
                    "  Total contributed: {}",
                    resp["total_contributed"].as_u64().unwrap_or(0)
                );
            }
        }
        ExportCmd::Sft {
            output,
            min_uxp,
            max_ufi,
            dry_run,
        } => {
            if dry_run {
                println!("Dry run: would export SFT dataset to {}", output);
                println!("  min_uxp: {}, max_ufi: {}", min_uxp, max_ufi);
                // TODO: query export history for eligible record count
                println!("  (record count estimation requires live session data)");
            } else {
                println!("Exporting SFT dataset to {} ...", output);
                println!("  min_uxp: {}, max_ufi: {}", min_uxp, max_ufi);
                // TODO: implement full export pipeline via API
                println!("  Export pipeline: build_example + detect_pii + to_jsonl");
                println!("  (full pipeline requires session replay — not yet implemented)");
            }
        }
        ExportCmd::Preference { output, dry_run }
        | ExportCmd::Contrastive { output, dry_run }
        | ExportCmd::LongHorizon { output, dry_run } => {
            if dry_run {
                println!("Dry run: would export dataset to {}", output);
            } else {
                println!("Exporting dataset to {} ...", output);
                println!("  (dataset family export requires session replay)");
            }
        }
    }
    Ok(())
}
