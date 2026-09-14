//! CLI usage: `focusa utility card`, `focusa utility bootstrap`, `focusa utility post-compaction`.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::Value;

#[derive(Subcommand)]
pub enum UtilityCmd {
    /// Print full Focusa utility card.
    Card,
    /// Print startup bootstrap card.
    Bootstrap,
    /// Print post-compaction recovery card.
    PostCompaction,
}

pub async fn handle(
    client: &mut ApiClient,
    cmd: UtilityCmd,
    json_mode: bool,
) -> anyhow::Result<()> {
    if json_mode {
        let resp: serde_json::Value = match cmd {
            UtilityCmd::Card => client.get("/v1/utility/card").await?,
            UtilityCmd::Bootstrap => client.get("/v1/utility/bootstrap").await?,
            UtilityCmd::PostCompaction => client.get("/v1/utility/post-compaction").await?,
        };
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }
    match cmd {
        UtilityCmd::Card => print_card(&client.get("/v1/utility/card").await?),
        UtilityCmd::Bootstrap => {
            print_named("bootstrap", &client.get("/v1/utility/bootstrap").await?)
        }
        UtilityCmd::PostCompaction => print_named(
            "post-compaction",
            &client.get("/v1/utility/post-compaction").await?,
        ),
    }
    Ok(())
}

fn print_card(value: &Value) {
    let schema = value
        .get("schema")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    println!("utility card {status} | {schema}");
    for scalar in ["authority_boundary", "preferred_layer", "purpose"] {
        if let Some(text) = value.get(scalar).and_then(Value::as_str) {
            println!("{scalar}: {text}");
        }
    }
    for section in [
        "usefulness_bar",
        "scope_gate",
        "bootstrap_card",
        "post_compaction_card",
        "exact_next_actions",
        "do_not_drift",
        "evidence_policy",
        "brevity_rules",
        "recovery_order",
    ] {
        println!("{section}:");
        print_list(value.get(section));
    }
}

fn print_named(label: &str, value: &Value) {
    let status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    println!("utility {label} {status}");
    {
        let scalar = "authority_boundary";
        if let Some(text) = value.get(scalar).and_then(Value::as_str) {
            println!("{scalar}: {text}");
        }
    }
    for section in [
        "usefulness_bar",
        "scope_gate",
        "bootstrap_card",
        "post_compaction_card",
        "exact_next_actions",
        "do_not_drift",
        "evidence_policy",
        "recovery_order",
    ] {
        if value.get(section).is_some() {
            println!("{section}:");
            print_list(value.get(section));
        }
    }
}

fn print_list(value: Option<&Value>) {
    if let Some(items) = value.and_then(Value::as_array) {
        for item in items.iter().filter_map(Value::as_str) {
            println!("  - {item}");
        }
    }
}
