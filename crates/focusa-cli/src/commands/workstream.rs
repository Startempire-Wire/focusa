//! `focusa workstream` — workstream-rooted canonical runtime CLI (#125 slice 5).
//!
//! `migrate` projects existing project profiles into workstream roots with
//! a preview/apply flow. The daemon owns the migration (single writer to
//! the SQLite ledger); this command surfaces the typed transaction.

use clap::{Args, Subcommand};
use serde_json::{json, Value};

#[derive(Args, Debug)]
pub struct WorkstreamArgs {
    #[command(subcommand)]
    pub cmd: WorkstreamCmd,
}

#[derive(Subcommand, Debug)]
pub enum WorkstreamCmd {
    /// Migrate existing project profiles into workstream roots (Spec 164 §slice 5).
    Migrate(MigrateArgs),
}

#[derive(Args, Debug)]
pub struct MigrateArgs {
    /// Preview only — report candidates, create nothing.
    #[arg(long)]
    pub preview: bool,
    /// Apply — upsert workstream roots for every candidate.
    #[arg(long)]
    pub apply: bool,
}

pub async fn run(cmd: WorkstreamCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = crate::api_client::ApiClient::new();
    match cmd {
        WorkstreamCmd::Migrate(args) => {
            if !args.preview && !args.apply {
                anyhow::bail!("specify --preview or --apply (or both: preview is reported first)");
            }
            let result: Value = api
                .post(
                    "/v1/workstreams/migrate",
                    &json!({ "preview": args.preview, "apply": args.apply }),
                )
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!(
                    "{}",
                    result
                        .get("status")
                        .and_then(|value| value.as_str())
                        .unwrap_or("unknown")
                );
                if let Some(count) = result.get("count").and_then(|value| value.as_i64()) {
                    println!("candidates: {count}");
                }
                if let Some(created) = result.get("created").and_then(|value| value.as_array()) {
                    for workstream_id in created {
                        if let Some(id) = workstream_id.as_str() {
                            println!("created: {id}");
                        }
                    }
                }
                if let Some(already) = result
                    .get("already_exists")
                    .and_then(|value| value.as_array())
                {
                    for workstream_id in already {
                        if let Some(id) = workstream_id.as_str() {
                            println!("already exists: {id}");
                        }
                    }
                }
            }
            Ok(())
        }
    }
}
