//! Turn-level observability commands.

use crate::api_client::ApiClient;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Subcommand)]
pub enum TurnsCmd {
    /// List recent turns (start/completion summary).
    List {
        #[arg(long, default_value = "50")]
        limit: u32,
        #[arg(long, default_value_t = false)]
        include_open: bool,
        /// Filter by session id or "current".
        #[arg(long)]
        session: Option<String>,
    },
    /// Show a single turn's events (from recent window).
    Show {
        /// Turn ID (full or prefix).
        turn_id: String,
        #[arg(long, default_value = "200")]
        limit: u32,
        /// Print full event JSON payloads.
        #[arg(long, default_value_t = false)]
        full: bool,
        /// Filter by session id or "current".
        #[arg(long)]
        session: Option<String>,
    },
}

#[derive(Default, Clone)]
struct TurnInfo {
    turn_id: String,
    harness_name: Option<String>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    output_len: Option<usize>,
    error_count: usize,
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    session_id: Option<String>,
}

fn parse_ts(value: &Value) -> Option<DateTime<Utc>> {
    let ts = value.as_str()?;
    DateTime::parse_from_rfc3339(ts)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn short_id(id: &str) -> &str {
    if id.len() >= 8 { &id[..8] } else { id }
}

fn normalize_event_type(event_type: &str) -> &str {
    match event_type {
        "TurnStarted" => "turn_started",
        "TurnCompleted" => "turn_completed",
        other => other,
    }
}

fn collect_turns(events: &[Value]) -> Vec<TurnInfo> {
    let mut map: HashMap<String, TurnInfo> = HashMap::new();

    for event in events {
        let event_type = normalize_event_type(
            event.get("type").and_then(|v| v.as_str()).unwrap_or(""),
        );
        let turn_id = event.get("turn_id").and_then(|v| v.as_str()).unwrap_or("");
        if turn_id.is_empty() {
            continue;
        }

        let entry = map.entry(turn_id.to_string()).or_insert_with(|| TurnInfo {
            turn_id: turn_id.to_string(),
            ..Default::default()
        });

        if entry.session_id.is_none() {
            entry.session_id = event
                .get("session_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }

        match event_type {
            "turn_started" => {
                entry.started_at = parse_ts(&event["timestamp"]);
                entry.harness_name = event
                    .get("harness_name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            }
            "turn_completed" => {
                entry.completed_at = parse_ts(&event["timestamp"]);
                entry.harness_name = entry.harness_name.clone().or_else(|| {
                    event
                        .get("harness_name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                });
                entry.output_len = event
                    .get("assistant_output")
                    .and_then(|v| v.as_str())
                    .map(|s| s.len());
                entry.error_count = event
                    .get("errors")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                entry.prompt_tokens = event.get("prompt_tokens").and_then(|v| v.as_u64());
                entry.completion_tokens = event.get("completion_tokens").and_then(|v| v.as_u64());
            }
            _ => {}
        }
    }

    let mut turns: Vec<TurnInfo> = map.into_values().collect();
    turns.sort_by_key(|t| t.started_at.or(t.completed_at));
    turns
}

#[cfg(test)]
mod tests {
    use super::collect_turns;
    use serde_json::json;

    #[test]
    fn collect_turns_accepts_pascal_case_persisted_event_types() {
        let turn_id = "turn-1";
        let events = vec![
            json!({
                "type": "TurnStarted",
                "turn_id": turn_id,
                "session_id": "session-1",
                "timestamp": "2026-04-14T20:00:00Z",
                "harness_name": "pi"
            }),
            json!({
                "type": "TurnCompleted",
                "turn_id": turn_id,
                "session_id": "session-1",
                "timestamp": "2026-04-14T20:00:05Z",
                "harness_name": "pi",
                "assistant_output": "done",
                "errors": []
            }),
        ];

        let turns = collect_turns(&events);
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].turn_id, turn_id);
        assert!(turns[0].started_at.is_some());
        assert!(turns[0].completed_at.is_some());
        assert_eq!(turns[0].output_len, Some(4));
    }
}

async fn resolve_session_filter(
    api: &ApiClient,
    session: &Option<String>,
) -> anyhow::Result<Option<String>> {
    if let Some(session_filter) = session.as_deref() {
        if session_filter == "current" {
            let state = api.get("/v1/state/dump").await?;
            return Ok(state["session"]["session_id"]
                .as_str()
                .map(|s| s.to_string()));
        }
        return Ok(Some(session_filter.to_string()));
    }
    Ok(None)
}

pub async fn run(cmd: TurnsCmd, json: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        TurnsCmd::List {
            limit,
            include_open,
            session,
        } => {
            let resp = api
                .get(&format!("/v1/events/recent?limit={}", limit * 6))
                .await?;
            let events = resp["events"].as_array().cloned().unwrap_or_default();
            let mut turns = collect_turns(&events);

            if let Some(sid) = resolve_session_filter(&api, &session).await? {
                turns.retain(|t| t.session_id.as_deref() == Some(&sid));
            }

            if !include_open {
                turns.retain(|t| t.completed_at.is_some());
            }
            if turns.len() > limit as usize {
                turns = turns.split_off(turns.len() - limit as usize);
            }

            if json {
                let out: Vec<Value> = turns
                    .iter()
                    .map(|t| {
                        let duration = match (t.started_at, t.completed_at) {
                            (Some(s), Some(e)) => Some((e - s).num_seconds()),
                            _ => None,
                        };
                        serde_json::json!({
                            "turn_id": t.turn_id,
                            "harness": t.harness_name,
                            "started_at": t.started_at.map(|v| v.to_rfc3339()),
                            "completed_at": t.completed_at.map(|v| v.to_rfc3339()),
                            "duration_secs": duration,
                            "output_len": t.output_len,
                            "errors": t.error_count,
                            "prompt_tokens": t.prompt_tokens,
                            "completion_tokens": t.completion_tokens,
                            "session_id": t.session_id,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&out)?);
                return Ok(());
            }

            println!("Turns ({} shown):", turns.len());
            for t in turns.iter().rev() {
                let status = if t.completed_at.is_some() {
                    "done"
                } else {
                    "open"
                };
                let duration = match (t.started_at, t.completed_at) {
                    (Some(s), Some(e)) => format!("{}s", (e - s).num_seconds()),
                    _ => "-".to_string(),
                };
                let output = t
                    .output_len
                    .map(|l| l.to_string())
                    .unwrap_or_else(|| "-".into());
                let harness = t.harness_name.clone().unwrap_or_else(|| "?".into());
                let start = t
                    .started_at
                    .map(|v| v.to_rfc3339())
                    .unwrap_or_else(|| "?".into());
                let tokens = match (t.prompt_tokens, t.completion_tokens) {
                    (Some(p), Some(c)) => format!("{}+{}", p, c),
                    _ => "-".to_string(),
                };
                println!(
                    "  {} [{}] {} duration={} output={} tokens={} errors={} harness={}",
                    short_id(&t.turn_id),
                    status,
                    start,
                    duration,
                    output,
                    tokens,
                    t.error_count,
                    harness,
                );
            }
        }
        TurnsCmd::Show {
            turn_id,
            limit,
            full,
            session,
        } => {
            let resp = api
                .get(&format!("/v1/events/recent?limit={}", limit))
                .await?;
            let events = resp["events"].as_array().cloned().unwrap_or_default();
            let mut matches: Vec<Value> = events
                .into_iter()
                .filter(|e| {
                    e.get("turn_id")
                        .and_then(|v| v.as_str())
                        .map(|id| id.starts_with(&turn_id))
                        .unwrap_or(false)
                })
                .collect();

            if let Some(sid) = resolve_session_filter(&api, &session).await? {
                matches.retain(|e| e.get("session_id").and_then(|v| v.as_str()) == Some(&sid));
            }

            if json || full {
                println!("{}", serde_json::to_string_pretty(&matches)?);
                return Ok(());
            }

            if matches.is_empty() {
                println!("No events for turn prefix: {}", turn_id);
                return Ok(());
            }

            println!("Turn events for {}:", turn_id);
            for event in matches {
                let ts = event["timestamp"].as_str().unwrap_or("?");
                let etype = event["type"].as_str().unwrap_or("?");
                println!("  {} {}", ts, etype);
            }
        }
    }

    Ok(())
}
