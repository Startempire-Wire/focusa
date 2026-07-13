//! Spec 130 compaction packet inspection, fidelity, replay, and diff CLI.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::{Value, json};

#[derive(Subcommand, Debug)]
pub enum CompactionCmd {
    /// Inspect kept/omitted context, authority, evidence, and exact next tool.
    Inspect {
        #[arg(long)]
        packet_id: String,
    },
    /// Evaluate required-field and authority fidelity for a packet.
    Evaluate {
        #[arg(long)]
        packet_id: String,
    },
    /// Explain why context was kept/omitted and which authority supports continuation.
    Why {
        #[arg(long)]
        packet_id: String,
    },
    /// Replay a stored packet as advisory context; never restores authority.
    Replay {
        #[arg(long)]
        packet_id: String,
    },
    /// Rehydrate an omitted ECS handle with a bounded token limit.
    RestoreContext {
        #[arg(long)]
        handle: String,
        #[arg(long, default_value_t = 2000)]
        token_limit: usize,
    },
    /// Compare two bounded compaction packets.
    Diff {
        #[arg(long)]
        before: String,
        #[arg(long)]
        after: String,
    },
}

fn text_at<'a>(response: &'a Value, pointers: &[&str]) -> Option<&'a str> {
    pointers
        .iter()
        .find_map(|pointer| response.pointer(pointer).and_then(Value::as_str))
}

fn print_human(label: &str, response: &Value) {
    let status = response
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("completed");
    let packet_id = response
        .get("packet_id")
        .or_else(|| response.pointer("/packet/packet_id"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    println!("Focusa compaction {label}: status={status} packet_id={packet_id}");
    for (name, pointers) in [
        (
            "scope",
            &["/kept/scope/project_root", "/packet/scope/project_root"][..],
        ),
        (
            "continuity",
            &["/kept/scope/continuity_id", "/packet/scope/continuity_id"],
        ),
        (
            "session",
            &["/kept/scope/session_id", "/packet/scope/session_id"],
        ),
        (
            "hlt_status",
            &["/hlt_posture", "/packet/trajectory/hlt_status"],
        ),
        (
            "fallback",
            &["/kept/trajectory/fallback", "/packet/trajectory/fallback"],
        ),
        (
            "workpoint_status",
            &["/kept/workpoint/status", "/packet/workpoint/status"],
        ),
        ("resume_state", &["/resume_state", "/packet/resume_state"]),
        (
            "next_tool",
            &["/exact_next_tool", "/packet/next/exact_next_tool"],
        ),
    ] {
        if let Some(value) = text_at(response, pointers) {
            println!("{name}={value}");
        }
    }
    let warnings = response
        .pointer("/kept/trajectory/warnings")
        .or_else(|| response.pointer("/packet/trajectory/warnings"))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" | ")
        })
        .unwrap_or_else(|| "none".into());
    println!("warnings={warnings}");
    let omitted = response
        .get("omitted")
        .or_else(|| response.pointer("/packet/bloatgaurd/omitted_sections"))
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    println!("omitted_count={omitted}");
    if let Some(changed) = response.get("changed_fields").and_then(Value::as_array) {
        let fields = changed.iter().filter_map(Value::as_str).collect::<Vec<_>>();
        println!("changed_fields={}", fields.join(","));
    }
}

pub async fn run(command: CompactionCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let (label, response) = match command {
        CompactionCmd::Inspect { packet_id } => {
            let path = format!("/v1/compaction/inspect/{}", urlencoding::encode(&packet_id));
            ("inspect", api.get(&path).await?)
        }
        CompactionCmd::Why { packet_id } => {
            let path = format!("/v1/compaction/inspect/{}", urlencoding::encode(&packet_id));
            ("why", api.get(&path).await?)
        }
        CompactionCmd::Evaluate { packet_id } => (
            "evaluate",
            api.post("/v1/compaction/evaluate", &json!({"packet_id": packet_id}))
                .await?,
        ),
        CompactionCmd::Replay { packet_id } => (
            "replay",
            api.post("/v1/compaction/replay", &json!({"packet_id": packet_id}))
                .await?,
        ),
        CompactionCmd::RestoreContext {
            handle,
            token_limit,
        } => (
            "restore-context",
            api.post(
                &format!("/v1/ecs/rehydrate/{}", urlencoding::encode(&handle)),
                &json!({"token_limit": token_limit}),
            )
            .await?,
        ),
        CompactionCmd::Diff { before, after } => (
            "diff",
            api.post(
                "/v1/compaction/diff",
                &json!({"before": before, "after": after}),
            )
            .await?,
        ),
    };
    if json_mode {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        print_human(label, &response);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_render_accepts_bounded_diff() {
        print_human(
            "diff",
            &json!({
                "status": "completed",
                "packet_id": "packet-1",
                "changed_fields": ["workpoint", "trajectory"]
            }),
        );
    }
}
