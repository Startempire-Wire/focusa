use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum PredictCmd {
    /// Record a bounded prediction.
    Record {
        #[arg(long)]
        prediction_type: String,
        #[arg(long)]
        predicted_outcome: String,
        #[arg(long, default_value_t = 0.5)]
        confidence: f64,
        #[arg(long)]
        recommended_action: String,
        #[arg(long)]
        why: String,
        #[arg(long, value_delimiter = ',')]
        context_refs: Vec<String>,
    },
    /// Evaluate a prediction by id.
    Evaluate {
        prediction_id: String,
        #[arg(long)]
        actual_outcome: String,
        #[arg(long)]
        score: Option<f64>,
        #[arg(long)]
        learning_signal_ref: Option<String>,
    },
    /// Recent predictions.
    Recent {
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
    /// Accuracy/calibration stats.
    Stats,
}

pub async fn run(cmd: PredictCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let resp = match cmd {
        PredictCmd::Record {
            prediction_type,
            predicted_outcome,
            confidence,
            recommended_action,
            why,
            context_refs,
        } => {
            api.post(
                "/v1/predictions",
                &json!({
                    "prediction_type": prediction_type,
                    "context_refs": context_refs,
                    "predicted_outcome": predicted_outcome,
                    "confidence": confidence,
                    "recommended_action": recommended_action,
                    "why": why,
                }),
            )
            .await?
        }
        PredictCmd::Evaluate {
            prediction_id,
            actual_outcome,
            score,
            learning_signal_ref,
        } => {
            api.post(
                &format!("/v1/predictions/{prediction_id}/evaluate"),
                &json!({
                    "actual_outcome": actual_outcome,
                    "score": score,
                    "learning_signal_ref": learning_signal_ref,
                }),
            )
            .await?
        }
        PredictCmd::Recent { limit } => {
            api.get(&format!("/v1/predictions/recent?limit={limit}"))
                .await?
        }
        PredictCmd::Stats => api.get("/v1/predictions/stats").await?,
    };
    if json_mode {
        println!("{}", serde_json::to_string_pretty(&resp)?);
    } else {
        println!("Status: {}", resp["status"].as_str().unwrap_or("completed"));
        println!(
            "Summary: {}",
            resp["summary"]
                .as_str()
                .unwrap_or("prediction command completed")
        );
        println!(
            "Next action: {}",
            resp["next_action"]
                .as_str()
                .unwrap_or("evaluate predictions after outcome is known")
        );
        println!(
            "Why: predictions make Focusa guidance measurable but do not override operator steering"
        );
        println!("Command: focusa predict stats");
        println!("Recovery: focusa predict recent --json");
        println!("Evidence: /v1/predictions/stats");
        println!("Docs: docs/current/PREDICTIVE_POWER_GUIDE.md");
    }
    Ok(())
}
