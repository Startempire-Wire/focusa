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
                let events = resp["events"].as_array();
                match events {
                    Some(e) if e.is_empty() => println!("No events"),
                    Some(e) => {
                        for event in e {
                            println!("  {}", event);
                        }
                    }
                    None => println!("No events"),
                }
            }
        }
    }
    Ok(())
}

pub async fn run_state(cmd: StateCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        StateCmd::Dump => {
            let resp = api.get("/v1/status").await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
        }
    }
    Ok(())
}
