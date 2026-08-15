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
    /// Prune the event ledger: drop epoch-junk placeholders, or export and
    /// prune events older than the hot window (default 30 days).
    Prune {
        /// Remove placeholder events carrying epoch-0 timestamps.
        #[arg(long)]
        epoch_junk: bool,
        /// Retain this many days of hot events.
        #[arg(long, default_value = "30")]
        before_days: u32,
        /// Skip cold JSONL export for the pruned events.
        #[arg(long)]
        no_export: bool,
        /// Show what would be pruned without mutating.
        #[arg(long)]
        dry_run: bool,
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
    /// Show recent snapshot ids.
    Recent {
        #[arg(long, default_value_t = 5)]
        limit: u32,
    },
    /// Diff two snapshots.
    Diff {
        #[arg(long = "from")]
        from_snapshot_id: String,
        #[arg(long = "to")]
        to_snapshot_id: String,
    },
    /// Create a fresh snapshot and compare it to the latest prior snapshot.
    CompareLatest {
        #[arg(long)]
        snapshot_reason: Option<String>,
        #[arg(long)]
        baseline_snapshot_id: Option<String>,
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
        EventsCmd::Prune {
            epoch_junk,
            before_days,
            no_export,
            dry_run,
        } => {
            let response = api
                .post(
                    "/v1/events/prune",
                    &serde_json::json!({
                        "epoch_junk": epoch_junk,
                        "before_days": before_days,
                        "export": !no_export,
                        "dry_run": dry_run,
                    }),
                )
                .await?;
            println!("{}", serde_json::to_string_pretty(&response)?);
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
                println!(
                    "snapshot create: {}",
                    resp["snapshot_id"].as_str().unwrap_or("unknown")
                );
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
                println!(
                    "snapshot restore: {}",
                    resp["snapshot_id"].as_str().unwrap_or("unknown")
                );
            }
        }
        StateCmd::Snapshot(SnapshotCmd::Recent { limit }) => {
            let bounded = limit.clamp(1, 20);
            let resp = api
                .get(&format!("/v1/focus/snapshots/recent?limit={bounded}"))
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!(
                    "snapshot recent: count={}",
                    resp["snapshots"]
                        .as_array()
                        .map(|items| items.len())
                        .unwrap_or(0)
                );
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
        StateCmd::Snapshot(SnapshotCmd::CompareLatest {
            snapshot_reason,
            baseline_snapshot_id,
        }) => {
            let baseline = match baseline_snapshot_id {
                Some(id) if !id.trim().is_empty() => Some(id),
                _ => {
                    let recent = api.get("/v1/focus/snapshots/recent?limit=1").await?;
                    recent
                        .get("snapshots")
                        .and_then(|v| v.as_array())
                        .and_then(|items| items.first())
                        .and_then(|item| item.get("snapshot_id"))
                        .and_then(|v| v.as_str())
                        .map(str::to_string)
                }
            };
            let created = api
                .post(
                    "/v1/focus/snapshots",
                    &json!({"snapshot_reason": snapshot_reason}),
                )
                .await?;
            let created_id = created["snapshot_id"].as_str().unwrap_or("").to_string();
            let resp = if let Some(baseline_id) = baseline.filter(|_| !created_id.is_empty()) {
                api.post(
                    "/v1/focus/snapshots/diff",
                    &json!({"from_snapshot_id": baseline_id, "to_snapshot_id": created_id}),
                )
                .await?
            } else {
                json!({"created": created, "diff": null, "status": "created_no_baseline"})
            };
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!(
                    "snapshot compare-latest: created={} changed={}",
                    created_id,
                    resp["checksum_changed"].as_bool().unwrap_or(false)
                );
            }
        }
    }
    Ok(())
}
