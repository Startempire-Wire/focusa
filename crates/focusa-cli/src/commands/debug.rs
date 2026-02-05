//! Debug and inspection CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;

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
pub enum StateCmd {
    /// Dump full cognitive state.
    Dump,
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
            let resp = api
                .get(&format!("/v1/events/{}", event_id))
                .await?;
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

pub async fn run_state(cmd: StateCmd) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        StateCmd::Dump => {
            let resp = api.get("/v1/state/dump").await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
    }
    Ok(())
}
