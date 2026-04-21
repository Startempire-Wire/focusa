//! Debug and inspection CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand)]
pub enum EventsCmd {
    /// Tail recent events.
    Tail {
        #[arg(long, default_value = "20")]
        limit: u32,
    },
    /// Show a specific event.
    Show {
        /// Event ID.
        event_id: String,
    },
}

#[derive(Subcommand)]
pub enum SnapshotCmd {
    /// Create snapshot bound to CLT node.
    Create {
        #[arg(long)]
        clt_node_id: Option<String>,
        #[arg(long)]
        snapshot_reason: Option<String>,
    },
    /// Restore snapshot by id.
    Restore {
        #[arg(long)]
        snapshot_id: String,
        #[arg(long, default_value = "exact")]
        restore_mode: String,
    },
    /// Diff two snapshots.
    Diff {
        #[arg(long = "from")]
        from_snapshot_id: String,
        #[arg(long = "to")]
        to_snapshot_id: String,
    },
}

#[derive(Subcommand)]
pub enum StateCmd {
    /// Dump full cognitive state.
    Dump,
    /// Snapshot operations.
    #[command(subcommand)]
    Snapshot(SnapshotCmd),
}

pub async fn run_events(cmd: EventsCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        EventsCmd::Tail { limit } => {
            let resp = api
                .get(&format!("/v1/events/recent?limit={}", limit))
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                let total = resp["total"].as_u64().unwrap_or(0);
                let returned = resp["returned"].as_u64().unwrap_or(0);
                let events = resp["events"].as_array();
                match events {
                    Some(e) if e.is_empty() => println!("No events"),
                    Some(e) => {
                        println!("Events ({} of {} total):", returned, total);
                        for event in e {
                            let ts = event["timestamp"].as_str().unwrap_or("?");
                            let etype = event["type"].as_str().unwrap_or("?");
                            let id = event["id"].as_str().unwrap_or("?");
                            let short_id = if id.len() >= 8 { &id[..8] } else { id };
                            println!("  {} [{}] {}", ts, short_id, etype);
                        }
                    }
                    None => println!("No events"),
                }
            }
        }
        EventsCmd::Show { event_id } => {
            let resp = api.get(&format!("/v1/events/{}", event_id)).await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else if let Some(event) = resp.get("event") {
                println!("{}", serde_json::to_string_pretty(event)?);
            } else {
                eprintln!("Event not found: {}", event_id);
            }
        }
    }
    Ok(())
}

pub async fn run_state(cmd: StateCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        StateCmd::Dump => {
            let resp = api.get("/v1/state/dump").await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
        StateCmd::Snapshot(SnapshotCmd::Create {
            clt_node_id,
            snapshot_reason,
        }) => {
            let body = json!({
                "clt_node_id": clt_node_id,
                "snapshot_reason": snapshot_reason,
            });
            let resp = api.post("/v1/focus/snapshots", &body).await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("snapshot create: {}", resp["snapshot_id"].as_str().unwrap_or("unknown"));
            }
        }
        StateCmd::Snapshot(SnapshotCmd::Restore {
            snapshot_id,
            restore_mode,
        }) => {
            let body = json!({
                "snapshot_id": snapshot_id,
                "restore_mode": restore_mode,
            });
            let resp = api.post("/v1/focus/snapshots/restore", &body).await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("snapshot restore: {}", resp["snapshot_id"].as_str().unwrap_or("unknown"));
            }
        }
        StateCmd::Snapshot(SnapshotCmd::Diff {
            from_snapshot_id,
            to_snapshot_id,
        }) => {
            let body = json!({
                "from_snapshot_id": from_snapshot_id,
                "to_snapshot_id": to_snapshot_id,
            });
            let resp = api.post("/v1/focus/snapshots/diff", &body).await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!(
                    "snapshot diff: changed={}",
                    resp["checksum_changed"].as_bool().unwrap_or(false)
                );
            }
        }
    }
    Ok(())
}
