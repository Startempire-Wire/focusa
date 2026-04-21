//! Lineage CLI commands (API parity surface).

use crate::api_client::ApiClient;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum LineageCmd {
    /// Show current lineage head.
    Head {
        /// Optional session id for scoped head lookup.
        #[arg(long)]
        session_id: Option<String>,
    },
    /// Show full lineage tree.
    Tree {
        /// Optional session id for scoped tree lookup.
        #[arg(long)]
        session_id: Option<String>,
    },
    /// Show a single lineage node.
    Node {
        /// CLT node id.
        clt_node_id: String,
    },
    /// Show lineage path from node to root.
    Path {
        /// CLT node id.
        clt_node_id: String,
    },
    /// Show direct children for a node.
    Children {
        /// CLT node id.
        clt_node_id: String,
    },
    /// Show summary nodes.
    Summaries {
        /// Optional session id for scoped summaries lookup.
        #[arg(long)]
        session_id: Option<String>,
    },
}

fn with_session_query(path: &str, session_id: Option<&str>) -> String {
    match session_id {
        Some(session) if !session.trim().is_empty() => format!("{path}?session_id={session}"),
        _ => path.to_string(),
    }
}

pub async fn run(cmd: LineageCmd, json: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        LineageCmd::Head { session_id } => {
            let path = with_session_query("/v1/lineage/head", session_id.as_deref());
            let resp = api.get(&path).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!(
                    "Lineage head: {}",
                    resp["head"].as_str().unwrap_or("unknown")
                );
            }
        }
        LineageCmd::Tree { session_id } => {
            let path = with_session_query("/v1/lineage/tree", session_id.as_deref());
            let resp = api.get(&path).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                let total = resp["total"].as_u64().unwrap_or(0);
                let head = resp["head"].as_str().unwrap_or("unknown");
                let root = resp["root"].as_str().unwrap_or("unknown");
                println!("Lineage tree: nodes={total} head={head} root={root}");
            }
        }
        LineageCmd::Node { clt_node_id } => {
            let resp = api.get(&format!("/v1/lineage/node/{clt_node_id}")).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else if resp.get("node").is_some() {
                let node = &resp["node"];
                println!(
                    "Node {} [{}] parent={}",
                    node["node_id"].as_str().unwrap_or("?"),
                    node["node_type"].as_str().unwrap_or("?"),
                    node["parent_id"].as_str().unwrap_or("root")
                );
            } else {
                println!("Node lookup returned no result");
            }
        }
        LineageCmd::Path { clt_node_id } => {
            let resp = api.get(&format!("/v1/lineage/path/{clt_node_id}")).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("Lineage path depth: {}", resp["depth"].as_u64().unwrap_or(0));
            }
        }
        LineageCmd::Children { clt_node_id } => {
            let resp = api
                .get(&format!("/v1/lineage/children/{clt_node_id}"))
                .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("Lineage children total: {}", resp["total"].as_u64().unwrap_or(0));
            }
        }
        LineageCmd::Summaries { session_id } => {
            let path = with_session_query("/v1/lineage/summaries", session_id.as_deref());
            let resp = api.get(&path).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!(
                    "Lineage summary nodes: {}",
                    resp["total"].as_u64().unwrap_or(0)
                );
            }
        }
    }

    Ok(())
}
