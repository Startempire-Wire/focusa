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
    /// Replay a stored packet as advisory context; never restores authority.
    Replay {
        #[arg(long)]
        packet_id: String,
    },
    /// Compare two bounded compaction packets.
    Diff {
        #[arg(long)]
        before: String,
        #[arg(long)]
        after: String,
    },
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
    if let Some(next) = response
        .get("exact_next_tool")
        .or_else(|| response.pointer("/packet/next/exact_next_tool"))
        .and_then(Value::as_str)
    {
        println!("next_tool={next}");
    }
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
