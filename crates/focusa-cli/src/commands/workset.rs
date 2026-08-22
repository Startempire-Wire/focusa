//! `focusa workset` — Spec 149 flow ledger CLI (#271 slice 2).

use clap::{Args, Subcommand};
use serde_json::{Value, json};

#[derive(Args, Debug)]
pub struct WorksetArgs {
    #[command(subcommand)]
    pub cmd: WorksetCmd,
}

#[derive(Subcommand, Debug)]
pub enum WorksetCmd {
    /// Define a workset (membership boundary + completion contract).
    Define(DefineArgs),
    /// Append a ledger event (requirement admission/disposition/membership).
    Event(EventArgs),
    /// Read the deterministic replay projection.
    Projection(ProjectionArgs),
}

#[derive(Args, Debug)]
pub struct DefineArgs {
    #[arg(long)]
    pub workset_id: String,
    #[arg(long)]
    pub project_root: String,
    #[arg(long)]
    pub continuity_id: String,
    #[arg(long)]
    pub required_requirements: Vec<String>,
    #[arg(long, default_value_t = 1)]
    pub revision: u64,
}

#[derive(Args, Debug)]
pub struct EventArgs {
    #[arg(long)]
    pub workset_id: String,
    #[arg(long)]
    pub event: String,
    #[arg(long)]
    pub requirement_id: String,
    #[arg(long)]
    pub disposition: String,
    #[arg(long)]
    pub provider_ref: Option<String>,
    #[arg(long)]
    pub evidence_ref: Option<String>,
}

#[derive(Args, Debug)]
pub struct ProjectionArgs {
    #[arg(long)]
    pub workset_id: String,
}

pub async fn run(cmd: WorksetCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = crate::api_client::ApiClient::new();
    match cmd {
        WorksetCmd::Define(args) => {
            let definition = json!({
                "schema": "focusa.workset_ledger.v1",
                "workset_id": args.workset_id,
                "revision": args.revision,
                "scope": {
                    "project_root": args.project_root,
                    "continuity_id": args.continuity_id,
                },
                "completion_contract": {
                    "required_requirement_ids": args.required_requirements,
                    "release_gate_ref": null,
                },
            });
            let result: Value = api.post("/v1/worksets", &definition).await?;
            print_result(result, json_mode);
            Ok(())
        }
        WorksetCmd::Event(args) => {
            let event = match args.event.as_str() {
                "admit" => json!({
                    "event_type": "requirement_admitted",
                    "requirement_id": args.requirement_id,
                    "provider_ref": args.provider_ref.unwrap_or_default(),
                    "evidence_ref": args.evidence_ref,
                }),
                "dispose" => json!({
                    "event_type": "requirement_disposed",
                    "requirement_id": args.requirement_id,
                    "disposition": args.disposition,
                    "evidence_ref": args.evidence_ref,
                }),
                other => anyhow::bail!("unknown event {other}; supported: admit, dispose"),
            };
            let result: Value = api
                .post(&format!("/v1/worksets/{}/events", args.workset_id), &event)
                .await?;
            print_result(result, json_mode);
            Ok(())
        }
        WorksetCmd::Projection(args) => {
            let result: Value = api
                .get(&format!("/v1/worksets/{}/projection", args.workset_id))
                .await?;
            print_result(result, json_mode);
            Ok(())
        }
    }
}

fn print_result(result: Value, json_mode: bool) {
    if json_mode {
        println!(
            "{}",
            serde_json::to_string_pretty(&result).unwrap_or_default()
        );
    } else {
        println!("{}", serde_json::to_string(&result).unwrap_or_default());
    }
}
