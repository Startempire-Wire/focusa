//! Ontology projection CLI.

use crate::api_client::ApiClient;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum OntologyCmd {
    /// Fetch ontology primitives (includes action_types/link_types vocab).
    Primitives,
    /// Fetch ontology world snapshot.
    World,
    /// Fetch ontology tool/action contracts.
    Contracts,
}

pub async fn run(cmd: OntologyCmd, json: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let (path, label) = match cmd {
        OntologyCmd::Primitives => ("/v1/ontology/primitives", "primitives"),
        OntologyCmd::World => ("/v1/ontology/world", "world"),
        OntologyCmd::Contracts => ("/v1/ontology/contracts", "contracts"),
    };

    let resp = api.get(path).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }

    match cmd {
        OntologyCmd::Primitives => {
            let action_count = resp
                .get("action_types")
                .and_then(|v| v.as_array())
                .map(|v| v.len())
                .unwrap_or(0);
            let link_count = resp
                .get("link_types")
                .and_then(|v| v.as_array())
                .map(|v| v.len())
                .unwrap_or(0);
            println!("ontology {}: action_types={} link_types={}", label, action_count, link_count);
        }
        _ => {
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
    }

    Ok(())
}
