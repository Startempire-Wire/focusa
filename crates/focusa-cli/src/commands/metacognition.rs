//! Metacognition CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand)]
pub enum MetacognitionCmd {
    /// Capture a metacognitive signal packet.
    Capture {
        #[arg(long)]
        kind: String,
        #[arg(long)]
        content: String,
        #[arg(long)]
        rationale: Option<String>,
        #[arg(long)]
        confidence: Option<f64>,
        #[arg(long = "strategy-class")]
        strategy_class: Option<String>,
    },
    /// Retrieve prior metacognitive candidates.
    Retrieve {
        #[arg(long = "current-ask")]
        current_ask: String,
        #[arg(long = "scope-tag")]
        scope_tags: Vec<String>,
        #[arg(long, default_value_t = 5)]
        k: u32,
    },
    /// Reflect over a turn range.
    Reflect {
        #[arg(long = "turn-range")]
        turn_range: String,
        #[arg(long = "failure-class")]
        failure_classes: Vec<String>,
    },
    /// Apply selected strategy updates.
    Adjust {
        #[arg(long = "reflection-id")]
        reflection_id: String,
        #[arg(long = "selected-update")]
        selected_updates: Vec<String>,
    },
    /// Evaluate outcome deltas.
    Evaluate {
        #[arg(long = "adjustment-id")]
        adjustment_id: String,
        #[arg(long = "observed-metric")]
        observed_metrics: Vec<String>,
    },
}

pub async fn run(cmd: MetacognitionCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    let (command, path, body) = match cmd {
        MetacognitionCmd::Capture {
            kind,
            content,
            rationale,
            confidence,
            strategy_class,
        } => (
            "capture",
            "/v1/metacognition/capture",
            json!({
                "kind": kind,
                "content": content,
                "rationale": rationale,
                "confidence": confidence,
                "strategy_class": strategy_class,
            }),
        ),
        MetacognitionCmd::Retrieve {
            current_ask,
            scope_tags,
            k,
        } => (
            "retrieve",
            "/v1/metacognition/retrieve",
            json!({
                "current_ask": current_ask,
                "scope_tags": scope_tags,
                "k": k,
            }),
        ),
        MetacognitionCmd::Reflect {
            turn_range,
            failure_classes,
        } => (
            "reflect",
            "/v1/metacognition/reflect",
            json!({
                "turn_range": turn_range,
                "failure_classes": failure_classes,
            }),
        ),
        MetacognitionCmd::Adjust {
            reflection_id,
            selected_updates,
        } => (
            "adjust",
            "/v1/metacognition/adjust",
            json!({
                "reflection_id": reflection_id,
                "selected_updates": selected_updates,
            }),
        ),
        MetacognitionCmd::Evaluate {
            adjustment_id,
            observed_metrics,
        } => (
            "evaluate",
            "/v1/metacognition/evaluate",
            json!({
                "adjustment_id": adjustment_id,
                "observed_metrics": observed_metrics,
            }),
        ),
    };

    let resp = api.post(path, &body).await?;

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&resp)?);
    } else {
        println!("metacognition {}: ok", command);
        println!("{}", serde_json::to_string_pretty(&resp)?);
    }

    Ok(())
}
