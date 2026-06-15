use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::{Value, json};

#[derive(Subcommand)]
pub enum ClaimCmd {
    /// Gate a completion claim before bd close/final report.
    Preclose {
        /// Work item id, e.g. focusa-ui0y.15.
        #[arg(long)]
        work_item_id: Option<String>,
        /// Completion claim text.
        #[arg(long)]
        claim: String,
        /// Acceptance criterion; repeat for multiple criteria.
        #[arg(long = "acceptance")]
        acceptance_criteria: Vec<String>,
        /// Evidence ref; repeat for multiple refs.
        #[arg(long = "evidence-ref")]
        evidence_refs: Vec<String>,
        /// Evidence summary; repeat for multiple summaries.
        #[arg(long = "evidence-summary")]
        evidence_summaries: Vec<String>,
    },
}

pub async fn handle(client: &mut ApiClient, cmd: ClaimCmd) -> anyhow::Result<()> {
    match cmd {
        ClaimCmd::Preclose {
            work_item_id,
            claim,
            acceptance_criteria,
            evidence_refs,
            evidence_summaries,
        } => {
            let body = json!({
                "work_item_id": work_item_id,
                "claim": claim,
                "acceptance_criteria": acceptance_criteria,
                "evidence_refs": evidence_refs,
                "evidence_summaries": evidence_summaries,
            });
            print_preclose(&client.post("/v1/claim/preclose", &body).await?)
        }
    }
    Ok(())
}

fn print_preclose(value: &Value) {
    let decision = value
        .get("decision")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let evidence_class = value
        .get("evidence_class")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    println!("claim preclose {decision} | evidence_class={evidence_class}");
    if let Some(reason) = value.get("reason").and_then(Value::as_str) {
        println!("reason: {reason}");
    }
    print_list(
        "missing_required_evidence",
        value.get("missing_required_evidence"),
    );
    print_list("overclaim_risks", value.get("overclaim_risks"));
    print_list("recovery_commands", value.get("recovery_commands"));
}

fn print_list(label: &str, value: Option<&Value>) {
    if let Some(items) = value.and_then(Value::as_array) {
        if !items.is_empty() {
            println!("{label}:");
            for item in items.iter().filter_map(Value::as_str) {
                println!("  - {item}");
            }
        }
    }
}
