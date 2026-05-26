//! Lineage CLI commands (API parity surface).

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::{Value, json};

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
    /// Extract decisions/constraints/risks from a bounded lineage summary window.
    Extract {
        /// Max signals per category.
        #[arg(long = "max-candidates", default_value_t = 12)]
        max_candidates: usize,
        /// Optional session id for scoped tree lookup.
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

fn print_json(value: &Value) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn bounded_unique_signals(nodes: &[Value], keys: &[&str], cap: usize) -> Vec<String> {
    let mut out = Vec::new();
    for node in nodes {
        let Some(payload) = node.get("payload").and_then(Value::as_object) else {
            continue;
        };
        for key in keys {
            let Some(value) = payload.get(*key) else {
                continue;
            };
            if let Some(items) = value.as_array() {
                for item in items {
                    let text = item.as_str().unwrap_or("").trim().to_string();
                    if !text.is_empty() && !out.contains(&text) {
                        out.push(text);
                    }
                    if out.len() >= cap {
                        return out;
                    }
                }
            } else {
                let text = value.as_str().unwrap_or("").trim().to_string();
                if !text.is_empty() && !out.contains(&text) {
                    out.push(text);
                }
                if out.len() >= cap {
                    return out;
                }
            }
        }
    }
    out
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
                println!(
                    "Lineage path depth: {}",
                    resp["depth"].as_u64().unwrap_or(0)
                );
            }
        }
        LineageCmd::Children { clt_node_id } => {
            let resp = api
                .get(&format!("/v1/lineage/children/{clt_node_id}"))
                .await?;
            if json {
                print_json(&resp)?;
            } else {
                println!(
                    "Lineage children total: {}",
                    resp["total"].as_u64().unwrap_or(0)
                );
            }
        }
        LineageCmd::Summaries { session_id } => {
            let path = with_session_query("/v1/lineage/summaries", session_id.as_deref());
            let resp = api.get(&path).await?;
            if json {
                print_json(&resp)?;
            } else {
                println!(
                    "Lineage summary nodes: {}",
                    resp["total"].as_u64().unwrap_or(0)
                );
            }
        }
        LineageCmd::Extract {
            max_candidates,
            session_id,
        } => {
            let cap = max_candidates.clamp(1, 50);
            let mut path = format!("/v1/lineage/tree?selector=summaries&limit={cap}");
            if let Some(session) = session_id.filter(|session| !session.trim().is_empty()) {
                path.push_str(&format!("&session_id={session}"));
            }
            let resp = api.get(&path).await?;
            let nodes: Vec<Value> = resp
                .get("nodes")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let decisions =
                bounded_unique_signals(&nodes, &["decisions", "decision", "decision_text"], cap);
            let constraints = bounded_unique_signals(
                &nodes,
                &["constraints", "constraint", "constraint_text"],
                cap,
            );
            let risks =
                bounded_unique_signals(&nodes, &["risks", "risk", "blockers", "blocker"], cap);
            let summary_nodes = nodes
                .iter()
                .filter(|node| {
                    node.get("node_type")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .eq_ignore_ascii_case("summary")
                })
                .count();
            let reflection_trigger = summary_nodes >= cap / 3 || risks.len() >= 3;
            let out = json!({
                "lineage": {
                    "root": resp.get("root").cloned().unwrap_or(Value::Null),
                    "head": resp.get("head").cloned().unwrap_or(Value::Null),
                    "nodes": nodes.len(),
                    "summary_nodes": summary_nodes,
                },
                "signals": {"decisions": decisions, "constraints": constraints, "risks": risks},
                "reflection_trigger": reflection_trigger,
                "next_tools": ["focusa_metacog_capture", "focusa_metacog_reflect", "focusa_workpoint_checkpoint"],
            });
            if json {
                print_json(&out)?;
            } else {
                println!(
                    "Lineage extract: decisions={} constraints={} risks={} trigger={}",
                    out.pointer("/signals/decisions")
                        .and_then(Value::as_array)
                        .map(|v| v.len())
                        .unwrap_or(0),
                    out.pointer("/signals/constraints")
                        .and_then(Value::as_array)
                        .map(|v| v.len())
                        .unwrap_or(0),
                    out.pointer("/signals/risks")
                        .and_then(Value::as_array)
                        .map(|v| v.len())
                        .unwrap_or(0),
                    out.get("reflection_trigger")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                );
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
