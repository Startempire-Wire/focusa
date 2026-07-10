//! `focusa audit` — durable event timeline for Workpoints, Focus frames, and decisions.
//!
//! Thin, backward-compatible wrapper over the existing `/v1/events/recent`
//! route. It does not create a second audit store; SQLite events remain the
//! durable source of truth.

use crate::api_client::ApiClient;
use clap::Args;
use serde_json::{Value, json};

#[derive(Args, Debug)]
pub struct AuditArgs {
    /// Lower time bound, forwarded to /v1/events/recent as-is. Examples: 2026-07-03T00:00:00Z, 24h.
    #[arg(long)]
    pub since: Option<String>,

    /// Filter timeline to events mentioning a Beads issue id.
    #[arg(long = "beads-issue")]
    pub beads_issue: Option<String>,

    /// Filter timeline to events mentioning a Workpoint id.
    #[arg(long)]
    pub workpoint_id: Option<String>,

    /// Event type filter forwarded to /v1/events/recent.
    #[arg(long = "event-type")]
    pub event_type: Option<String>,

    /// Max events to fetch before client-side bead/workpoint filtering.
    #[arg(long, default_value_t = 50)]
    pub limit: usize,
}

pub async fn run(args: AuditArgs, json_mode: bool) -> anyhow::Result<()> {
    let limit = args.limit.clamp(1, 1000);
    let mut query = format!("/v1/events/recent?limit={limit}");
    if let Some(since) = args.since.as_deref().filter(|s| !s.trim().is_empty()) {
        query.push_str("&since=");
        query.push_str(&urlencoding::encode(since.trim()));
    }
    if let Some(event_type) = args.event_type.as_deref().filter(|s| !s.trim().is_empty()) {
        query.push_str("&event_type=");
        query.push_str(&urlencoding::encode(event_type.trim()));
    }

    let api = ApiClient::new();
    let raw = api.get(&query).await?;
    let mut events = raw
        .get("events")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    if let Some(bead) = args.beads_issue.as_deref().filter(|s| !s.trim().is_empty()) {
        events.retain(|event| event_mentions(event, bead));
    }
    if let Some(workpoint_id) = args
        .workpoint_id
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        events.retain(|event| event_mentions(event, workpoint_id));
    }

    let envelope = json!({
        "status": "completed",
        "authority": "daemon_global_advisory",
        "canonical": false,
        "next_step_hint": "This surface needs Spec104 scoped API work before it can be treated as project-canonical.",
        "source": "/v1/events/recent",
        "query": {
            "since": args.since,
            "beads_issue": args.beads_issue,
            "workpoint_id": args.workpoint_id,
            "event_type": args.event_type,
            "limit": limit,
        },
        "returned": events.len(),
        "events": events,
        "next_cursor": raw.get("next_cursor").cloned().unwrap_or(Value::Null),
        "truncated": raw.get("truncated").cloned().unwrap_or(Value::Bool(false)),
        "bounds": raw.get("bounds").cloned().unwrap_or(Value::Null),
    });

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        print_human(&envelope);
    }
    Ok(())
}

fn event_mentions(event: &Value, needle: &str) -> bool {
    serde_json::to_string(event)
        .map(|s| s.contains(needle))
        .unwrap_or(false)
}

fn print_human(envelope: &Value) {
    println!(
        "Status: {}",
        envelope["status"].as_str().unwrap_or("unknown")
    );
    println!(
        "Source: {}",
        envelope["source"].as_str().unwrap_or("/v1/events/recent")
    );
    println!(
        "Authority: {} canonical={}",
        envelope["authority"]
            .as_str()
            .unwrap_or("daemon_global_advisory"),
        envelope["canonical"].as_bool().unwrap_or(false)
    );
    println!("Returned: {}", envelope["returned"].as_u64().unwrap_or(0));
    if let Some(events) = envelope["events"].as_array() {
        for event in events.iter().take(20) {
            let ts = event
                .get("timestamp")
                .and_then(Value::as_str)
                .unwrap_or("unknown-time");
            let ty = event
                .get("type")
                .or_else(|| event.get("event_type"))
                .and_then(Value::as_str)
                .unwrap_or("unknown-event");
            let id = event
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("unknown-id");
            println!("- {ts} {ty} {id}");
        }
    }
    println!("Next: focusa audit --limit 100 --json");
}
