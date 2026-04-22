//! Lineage CLI commands (API parity surface).

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::json;

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
    /// Compare two snapshots for learning deltas.
    Compare {
        #[arg(long = "from-snapshot-id")]
        from_snapshot_id: String,
        #[arg(long = "to-snapshot-id")]
        to_snapshot_id: String,
    },
}

fn with_session_query(path: &str, session_id: Option<&str>) -> String {
    match session_id {
        Some(session) if !session.trim().is_empty() => format!("{path}?session_id={session}"),
        _ => path.to_string(),
    }
}

fn print_json(value: &serde_json::Value) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

pub async fn run(cmd: LineageCmd, json: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        LineageCmd::Head { session_id } => {
            let path = with_session_query("/v1/lineage/head", session_id.as_deref());
            let resp = api.get(&path).await?;
            if json {
                print_json(&resp)?;
            } else {
                println!("Lineage head: {}", resp["head"].as_str().unwrap_or("unknown"));
            }
        }
        LineageCmd::Tree { session_id } => {
            let path = with_session_query("/v1/lineage/tree", session_id.as_deref());
            let resp = api.get(&path).await?;
            if json {
                print_json(&resp)?;
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
                print_json(&resp)?;
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
                print_json(&resp)?;
            } else {
                println!("Lineage path depth: {}", resp["depth"].as_u64().unwrap_or(0));
            }
        }
        LineageCmd::Children { clt_node_id } => {
            let resp = api.get(&format!("/v1/lineage/children/{clt_node_id}")).await?;
            if json {
                print_json(&resp)?;
            } else {
                println!("Lineage children total: {}", resp["total"].as_u64().unwrap_or(0));
            }
        }
        LineageCmd::Summaries { session_id } => {
            let path = with_session_query("/v1/lineage/summaries", session_id.as_deref());
            let resp = api.get(&path).await?;
            if json {
                print_json(&resp)?;
            } else {
                println!("Lineage summary nodes: {}", resp["total"].as_u64().unwrap_or(0));
            }
        }
        LineageCmd::Compare {
            from_snapshot_id,
            to_snapshot_id,
        } => {
            let resp = api
                .post(
                    "/v1/focus/snapshots/diff",
                    &json!({
                        "from_snapshot_id": from_snapshot_id,
                        "to_snapshot_id": to_snapshot_id,
                    }),
                )
                .await?;
            if json {
                print_json(&resp)?;
            } else {
                println!(
                    "Lineage compare: changed={} version_delta={} clt_node_changed={}",
                    resp["checksum_changed"].as_bool().unwrap_or(false),
                    resp["version_delta"].as_u64().unwrap_or(0),
                    resp["clt_node_changed"].as_bool().unwrap_or(false)
                );
            }
        }
    }

    Ok(())
}
