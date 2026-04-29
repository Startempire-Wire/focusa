//! Metacognition CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::{Value, json};
use std::collections::BTreeMap;

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
    /// Compound metacognition workflows.
    #[command(subcommand)]
    Loop(MetacognitionLoopCmd),
    /// Policy-gated promotion decision.
    Promote {
        #[arg(long = "adjustment-id")]
        adjustment_id: Option<String>,
        #[arg(long = "reflection-id")]
        reflection_id: Option<String>,
        #[arg(long = "selected-update")]
        selected_updates: Vec<String>,
        #[arg(long = "observed-metric")]
        observed_metrics: Vec<String>,
    },
    /// Signal quality + confidence diagnostics.
    Doctor {
        #[arg(long = "current-ask")]
        current_ask: String,
        #[arg(long = "scope-tag")]
        scope_tags: Vec<String>,
        #[arg(long, default_value_t = 5)]
        k: u32,
    },
}

#[derive(Subcommand)]
pub enum MetacognitionLoopCmd {
    /// Run capture -> retrieve -> reflect -> adjust -> evaluate.
    Run {
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
        #[arg(long = "current-ask")]
        current_ask: String,
        #[arg(long = "scope-tag")]
        scope_tags: Vec<String>,
        #[arg(long, default_value_t = 5)]
        k: u32,
        #[arg(long = "turn-range")]
        turn_range: String,
        #[arg(long = "failure-class")]
        failure_classes: Vec<String>,
        #[arg(long = "selected-update")]
        selected_updates: Vec<String>,
        #[arg(long = "observed-metric")]
        observed_metrics: Vec<String>,
    },
}

fn require_non_empty(name: &str, value: &str) -> anyhow::Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        anyhow::bail!("[CLI_INPUT_ERROR] {} is required", name);
    }
    Ok(trimmed.to_string())
}

fn value_string_array(value: &Value, field: &str) -> Vec<String> {
    value[field]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn print_json(value: &Value) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

struct LoopRunArgs {
    kind: String,
    content: String,
    rationale: Option<String>,
    confidence: Option<f64>,
    strategy_class: Option<String>,
    current_ask: String,
    scope_tags: Vec<String>,
    k: u32,
    turn_range: String,
    failure_classes: Vec<String>,
    selected_updates: Vec<String>,
    observed_metrics: Vec<String>,
}

async fn run_loop(api: &ApiClient, json_mode: bool, args: LoopRunArgs) -> anyhow::Result<()> {
    let LoopRunArgs {
        kind,
        content,
        rationale,
        confidence,
        strategy_class,
        current_ask,
        scope_tags,
        k,
        turn_range,
        failure_classes,
        selected_updates,
        observed_metrics,
    } = args;
    let kind = require_non_empty("kind", &kind)?;
    let content = require_non_empty("content", &content)?;
    let current_ask = require_non_empty("current_ask", &current_ask)?;
    let turn_range = require_non_empty("turn_range", &turn_range)?;

    let capture = api
        .post(
            "/v1/metacognition/capture",
            &json!({
                "kind": kind,
                "content": content,
                "rationale": rationale,
                "confidence": confidence,
                "strategy_class": strategy_class,
            }),
        )
        .await?;

    let retrieve = api
        .post(
            "/v1/metacognition/retrieve",
            &json!({
                "current_ask": current_ask,
                "scope_tags": scope_tags,
                "k": k,
            }),
        )
        .await?;

    let reflect = api
        .post(
            "/v1/metacognition/reflect",
            &json!({
                "turn_range": turn_range,
                "failure_classes": failure_classes,
            }),
        )
        .await?;

    let chosen_updates = if selected_updates.is_empty() {
        value_string_array(&reflect, "strategy_updates")
    } else {
        selected_updates
    };

    let reflection_id = require_non_empty(
        "reflection_id",
        reflect["reflection_id"].as_str().unwrap_or_default(),
    )?;

    let adjust = api
        .post(
            "/v1/metacognition/adjust",
            &json!({
                "reflection_id": reflection_id,
                "selected_updates": chosen_updates,
            }),
        )
        .await?;

    let adjustment_id = require_non_empty(
        "adjustment_id",
        adjust["adjustment_id"].as_str().unwrap_or_default(),
    )?;

    let evaluate = api
        .post(
            "/v1/metacognition/evaluate",
            &json!({
                "adjustment_id": adjustment_id,
                "observed_metrics": observed_metrics,
            }),
        )
        .await?;

    let result = json!({
        "status": "ok",
        "workflow": "metacognition_loop_run",
        "capture": capture,
        "retrieve": retrieve,
        "reflect": reflect,
        "adjust": adjust,
        "evaluate": evaluate,
    });

    if json_mode {
        print_json(&result)?;
    } else {
        println!(
            "metacognition loop: result={} promote={}",
            evaluate["result"].as_str().unwrap_or("unknown"),
            evaluate["promote_learning"].as_bool().unwrap_or(false)
        );
        println!(
            "  capture={} reflection={} adjustment={}",
            result["capture"]["capture_id"].as_str().unwrap_or("unknown"),
            result["reflect"]["reflection_id"].as_str().unwrap_or("unknown"),
            result["adjust"]["adjustment_id"].as_str().unwrap_or("unknown")
        );
        println!(
            "  retrieved_candidates={} selected_updates={}",
            result["retrieve"]["total_candidates"].as_u64().unwrap_or(0),
            result["adjust"]["next_step_policy"]
                .as_array()
                .map(|items| items.len())
                .unwrap_or(0)
        );
    }

    Ok(())
}

async fn run_promote(
    api: &ApiClient,
    json_mode: bool,
    adjustment_id: Option<String>,
    reflection_id: Option<String>,
    selected_updates: Vec<String>,
    observed_metrics: Vec<String>,
) -> anyhow::Result<()> {
    let mut adjustment = None;

    let adjustment_id = if let Some(id) = adjustment_id {
        require_non_empty("adjustment_id", &id)?
    } else {
        let reflection_id = require_non_empty(
            "reflection_id",
            reflection_id.as_deref().unwrap_or_default(),
        )?;
        let created_adjustment = api
            .post(
                "/v1/metacognition/adjust",
                &json!({
                    "reflection_id": reflection_id,
                    "selected_updates": selected_updates,
                }),
            )
            .await?;
        let id = require_non_empty(
            "adjustment_id",
            created_adjustment["adjustment_id"].as_str().unwrap_or_default(),
        )?;
        adjustment = Some(created_adjustment);
        id
    };

    let evaluation = api
        .post(
            "/v1/metacognition/evaluate",
            &json!({
                "adjustment_id": adjustment_id,
                "observed_metrics": observed_metrics,
            }),
        )
        .await?;

    let promote_learning = evaluation["promote_learning"].as_bool().unwrap_or(false);
    let result = json!({
        "status": "ok",
        "workflow": "metacognition_promote",
        "decision": if promote_learning { "promote" } else { "hold" },
        "adjustment": adjustment.unwrap_or_else(|| json!({ "adjustment_id": adjustment_id })),
        "evaluation": evaluation,
    });

    if json_mode {
        print_json(&result)?;
    } else {
        println!(
            "metacognition promote: {}",
            result["decision"].as_str().unwrap_or("hold")
        );
        println!(
            "  adjustment={} result={} promote={}",
            result["adjustment"]["adjustment_id"].as_str().unwrap_or("unknown"),
            result["evaluation"]["result"].as_str().unwrap_or("unknown"),
            result["evaluation"]["promote_learning"].as_bool().unwrap_or(false)
        );
    }

    Ok(())
}

async fn run_doctor(
    api: &ApiClient,
    json_mode: bool,
    current_ask: String,
    scope_tags: Vec<String>,
    k: u32,
) -> anyhow::Result<()> {
    let current_ask = require_non_empty("current_ask", &current_ask)?;
    let retrieve = api
        .post(
            "/v1/metacognition/retrieve",
            &json!({
                "current_ask": current_ask,
                "scope_tags": scope_tags,
                "k": k,
                "summary_only": true,
            }),
        )
        .await?;

    let candidates = retrieve["candidates"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let mut confidence_sum = 0.0_f64;
    let mut confidence_count = 0_u64;
    let mut kinds = BTreeMap::<String, u64>::new();

    for candidate in &candidates {
        if let Some(conf) = candidate["confidence"].as_f64() {
            confidence_sum += conf;
            confidence_count += 1;
        }
        if let Some(kind) = candidate["kind"].as_str() {
            *kinds.entry(kind.to_string()).or_insert(0) += 1;
        }
    }

    let avg_confidence = if confidence_count > 0 {
        Some(confidence_sum / confidence_count as f64)
    } else {
        None
    };

    let result = json!({
        "status": "ok",
        "workflow": "metacognition_doctor",
        "diagnostics": {
            "candidate_count": candidates.len(),
            "with_confidence": confidence_count,
            "without_confidence": candidates.len() as u64 - confidence_count,
            "avg_confidence": avg_confidence,
            "kind_breakdown": kinds,
            "truncated": retrieve["retrieval_budget"]["truncated"].as_bool().unwrap_or(false),
            "next_cursor_present": !retrieve["next_cursor"].is_null(),
        },
        "sample_candidates": candidates.into_iter().take(3).collect::<Vec<_>>(),
    });

    if json_mode {
        print_json(&result)?;
    } else {
        println!(
            "metacognition doctor: candidates={} avg_confidence={}",
            result["diagnostics"]["candidate_count"].as_u64().unwrap_or(0),
            result["diagnostics"]["avg_confidence"]
                .as_f64()
                .map(|v| format!("{v:.2}"))
                .unwrap_or_else(|| "n/a".to_string())
        );
        println!(
            "  with_confidence={} without_confidence={} truncated={}",
            result["diagnostics"]["with_confidence"].as_u64().unwrap_or(0),
            result["diagnostics"]["without_confidence"].as_u64().unwrap_or(0),
            result["diagnostics"]["truncated"].as_bool().unwrap_or(false)
        );
    }

    Ok(())
}

pub async fn run(cmd: MetacognitionCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        MetacognitionCmd::Capture {
            kind,
            content,
            rationale,
            confidence,
            strategy_class,
        } => {
            let resp = api
                .post(
                    "/v1/metacognition/capture",
                    &json!({
                        "kind": kind,
                        "content": content,
                        "rationale": rationale,
                        "confidence": confidence,
                        "strategy_class": strategy_class,
                    }),
                )
                .await?;
            if json_mode {
                print_json(&resp)?;
            } else {
                println!("metacognition capture: {}", resp["capture_id"].as_str().unwrap_or("ok"));
            }
        }
        MetacognitionCmd::Retrieve {
            current_ask,
            scope_tags,
            k,
        } => {
            let resp = api
                .post(
                    "/v1/metacognition/retrieve",
                    &json!({
                        "current_ask": current_ask,
                        "scope_tags": scope_tags,
                        "k": k,
                    }),
                )
                .await?;
            if json_mode {
                print_json(&resp)?;
            } else {
                println!(
                    "metacognition retrieve: candidates={}",
                    resp["total_candidates"].as_u64().unwrap_or(0)
                );
            }
        }
        MetacognitionCmd::Reflect {
            turn_range,
            failure_classes,
        } => {
            let resp = api
                .post(
                    "/v1/metacognition/reflect",
                    &json!({
                        "turn_range": turn_range,
                        "failure_classes": failure_classes,
                    }),
                )
                .await?;
            if json_mode {
                print_json(&resp)?;
            } else {
                println!("metacognition reflect: {}", resp["reflection_id"].as_str().unwrap_or("ok"));
            }
        }
        MetacognitionCmd::Adjust {
            reflection_id,
            selected_updates,
        } => {
            let resp = api
                .post(
                    "/v1/metacognition/adjust",
                    &json!({
                        "reflection_id": reflection_id,
                        "selected_updates": selected_updates,
                    }),
                )
                .await?;
            if json_mode {
                print_json(&resp)?;
            } else {
                println!("metacognition adjust: {}", resp["adjustment_id"].as_str().unwrap_or("ok"));
            }
        }
        MetacognitionCmd::Evaluate {
            adjustment_id,
            observed_metrics,
        } => {
            let resp = api
                .post(
                    "/v1/metacognition/evaluate",
                    &json!({
                        "adjustment_id": adjustment_id,
                        "observed_metrics": observed_metrics,
                    }),
                )
                .await?;
            if json_mode {
                print_json(&resp)?;
            } else {
                println!(
                    "metacognition evaluate: result={} promote={}",
                    resp["result"].as_str().unwrap_or("unknown"),
                    resp["promote_learning"].as_bool().unwrap_or(false)
                );
            }
        }
        MetacognitionCmd::Loop(MetacognitionLoopCmd::Run {
            kind,
            content,
            rationale,
            confidence,
            strategy_class,
            current_ask,
            scope_tags,
            k,
            turn_range,
            failure_classes,
            selected_updates,
            observed_metrics,
        }) => {
            run_loop(
                &api,
                json_mode,
                LoopRunArgs {
                    kind,
                    content,
                    rationale,
                    confidence,
                    strategy_class,
                    current_ask,
                    scope_tags,
                    k,
                    turn_range,
                    failure_classes,
                    selected_updates,
                    observed_metrics,
                },
            )
            .await?;
        }
        MetacognitionCmd::Promote {
            adjustment_id,
            reflection_id,
            selected_updates,
            observed_metrics,
        } => {
            run_promote(
                &api,
                json_mode,
                adjustment_id,
                reflection_id,
                selected_updates,
                observed_metrics,
            )
            .await?;
        }
        MetacognitionCmd::Doctor {
            current_ask,
            scope_tags,
            k,
        } => {
            run_doctor(&api, json_mode, current_ask, scope_tags, k).await?;
        }
    }

    Ok(())
}
