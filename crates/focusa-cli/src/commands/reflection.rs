use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum ReflectionCmd {
    /// Run one reflection iteration.
    Run {
        /// Window hint (e.g. 1h, 24h)
        #[arg(long)]
        window: Option<String>,
        /// Token budget for reflection iteration
        #[arg(long)]
        budget: Option<u32>,
        /// Idempotency key for dedupe
        #[arg(long = "idempotency-key")]
        idempotency_key: Option<String>,
        /// Mode: manual|scheduled
        #[arg(long, default_value = "manual")]
        mode: String,
    },
    /// Show recent reflection history.
    History {
        /// Max items
        #[arg(long, default_value_t = 20)]
        limit: u32,
        /// Optional stop_reason filter
        #[arg(long)]
        stop_reason: Option<String>,
        /// Optional mode filter (manual|scheduled)
        #[arg(long)]
        mode: Option<String>,
        /// RFC3339 lower bound for created_at
        #[arg(long)]
        since: Option<String>,
        /// RFC3339 upper bound for created_at
        #[arg(long)]
        until: Option<String>,
        /// RFC3339 cursor to page older rows (created_at < cursor_before)
        #[arg(long)]
        cursor_before: Option<String>,
    },
    /// Show reflection scheduler/guardrail status.
    Status,
    /// Scheduler management operations.
    #[command(subcommand)]
    Scheduler(SchedulerCmd),
}

#[derive(Subcommand, Debug)]
pub enum SchedulerCmd {
    /// Read scheduler config.
    Status,
    /// Enable scheduler.
    Enable,
    /// Disable scheduler.
    Disable,
    /// Update scheduler limits.
    Set {
        #[arg(long)]
        interval_seconds: Option<u64>,
        #[arg(long)]
        max_iterations_per_window: Option<u32>,
        #[arg(long)]
        cooldown_seconds: Option<u64>,
        #[arg(long)]
        low_confidence_threshold: Option<f64>,
        #[arg(long)]
        no_delta_min_event_delta: Option<u32>,
    },
    /// Trigger one scheduler tick.
    Tick {
        #[arg(long)]
        window: Option<String>,
        #[arg(long)]
        budget: Option<u32>,
    },
}

pub async fn run(cmd: ReflectionCmd, json_output: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        ReflectionCmd::Run {
            window,
            budget,
            idempotency_key,
            mode,
        } => {
            let body = json!({
                "mode": mode,
                "window": window,
                "budget": budget,
                "idempotency_key": idempotency_key,
            });
            let resp = api.post("/v1/reflect/run", &body).await?;
            if json_output {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!(
                    "reflection: status={} duplicate={}",
                    resp["status"].as_str().unwrap_or("unknown"),
                    resp["duplicate"].as_bool().unwrap_or(false)
                );
                let r = &resp["result"];
                println!(
                    "  iteration_id: {}",
                    r["iteration_id"].as_str().unwrap_or("n/a")
                );
                println!("  mode:         {}", r["mode"].as_str().unwrap_or("n/a"));
                println!(
                    "  confidence:   {}",
                    r["confidence"].as_f64().unwrap_or_default()
                );
                println!(
                    "  stop_reason:  {}",
                    r["stop_reason"].as_str().unwrap_or("n/a")
                );
            }
        }
        ReflectionCmd::History {
            limit,
            stop_reason,
            mode,
            since,
            until,
            cursor_before,
        } => {
            let mut path = format!("/v1/reflect/history?limit={}", limit);
            if let Some(reason) = stop_reason.as_ref() {
                path.push_str("&stop_reason=");
                path.push_str(reason);
            }
            if let Some(v) = mode.as_ref() {
                path.push_str("&mode=");
                path.push_str(v);
            }
            if let Some(v) = since.as_ref() {
                path.push_str("&since=");
                path.push_str(v);
            }
            if let Some(v) = until.as_ref() {
                path.push_str("&until=");
                path.push_str(v);
            }
            if let Some(v) = cursor_before.as_ref() {
                path.push_str("&cursor_before=");
                path.push_str(v);
            }
            let resp = api.get(&path).await?;
            if json_output {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!(
                    "reflection history: {} item(s)",
                    resp["count"].as_u64().unwrap_or(0)
                );
                if let Some(items) = resp["items"].as_array() {
                    for it in items {
                        println!(
                            "  - {} [{}] key={}",
                            it["iteration_id"].as_str().unwrap_or("n/a"),
                            it["mode"].as_str().unwrap_or("n/a"),
                            it["idempotency_key"].as_str().unwrap_or("-")
                        );
                    }
                }
                println!(
                    "next_cursor: {}",
                    resp["next_cursor"].as_str().unwrap_or("-")
                );
            }
        }
        ReflectionCmd::Status => {
            let resp = api.get("/v1/reflect/status").await?;
            if json_output {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!(
                    "reflection status: enabled={} scheduler_active={} low_conf={} no_delta_min={}",
                    resp["enabled"].as_bool().unwrap_or(false),
                    resp["scheduler"]["active"].as_bool().unwrap_or(false),
                    resp["guardrails"]["low_confidence_threshold"]
                        .as_f64()
                        .unwrap_or(0.0),
                    resp["guardrails"]["no_delta_min_event_delta"]
                        .as_u64()
                        .unwrap_or(0)
                );
                println!(
                    "  telemetry: window_key={} run_count={} last_run_at={}",
                    resp["telemetry"]["latest_window_key"]
                        .as_str()
                        .unwrap_or("-"),
                    resp["telemetry"]["latest_window_run_count"]
                        .as_u64()
                        .unwrap_or(0),
                    resp["telemetry"]["last_scheduler_run_at"]
                        .as_str()
                        .unwrap_or("-")
                );
                println!(
                    "  stop_reason_counts: {}",
                    resp["telemetry"]["stop_reason_counts"]
                );
            }
        }
        ReflectionCmd::Scheduler(sub) => match sub {
            SchedulerCmd::Status => {
                let resp = api.get("/v1/reflect/scheduler").await?;
                if json_output {
                    println!("{}", serde_json::to_string_pretty(&resp)?);
                } else {
                    println!(
                        "scheduler: enabled={} interval={}s max_iter={} cooldown={}s low_conf={} no_delta_min={}",
                        resp["enabled"].as_bool().unwrap_or(false),
                        resp["interval_seconds"].as_u64().unwrap_or(0),
                        resp["max_iterations_per_window"].as_u64().unwrap_or(0),
                        resp["cooldown_seconds"].as_u64().unwrap_or(0),
                        resp["low_confidence_threshold"].as_f64().unwrap_or(0.0),
                        resp["no_delta_min_event_delta"].as_u64().unwrap_or(0)
                    );
                    println!(
                        "  telemetry: window_key={} run_count={} last_run_at={}",
                        resp["telemetry"]["latest_window_key"]
                            .as_str()
                            .unwrap_or("-"),
                        resp["telemetry"]["latest_window_run_count"]
                            .as_u64()
                            .unwrap_or(0),
                        resp["telemetry"]["last_scheduler_run_at"]
                            .as_str()
                            .unwrap_or("-")
                    );
                }
            }
            SchedulerCmd::Enable => {
                let resp = api
                    .post("/v1/reflect/scheduler", &json!({"enabled": true}))
                    .await?;
                if json_output {
                    println!("{}", serde_json::to_string_pretty(&resp)?);
                } else {
                    println!("scheduler: enabled");
                }
            }
            SchedulerCmd::Disable => {
                let resp = api
                    .post("/v1/reflect/scheduler", &json!({"enabled": false}))
                    .await?;
                if json_output {
                    println!("{}", serde_json::to_string_pretty(&resp)?);
                } else {
                    println!("scheduler: disabled");
                }
            }
            SchedulerCmd::Set {
                interval_seconds,
                max_iterations_per_window,
                cooldown_seconds,
                low_confidence_threshold,
                no_delta_min_event_delta,
            } => {
                let body = json!({
                    "interval_seconds": interval_seconds,
                    "max_iterations_per_window": max_iterations_per_window,
                    "cooldown_seconds": cooldown_seconds,
                    "low_confidence_threshold": low_confidence_threshold,
                    "no_delta_min_event_delta": no_delta_min_event_delta,
                });
                let resp = api.post("/v1/reflect/scheduler", &body).await?;
                if json_output {
                    println!("{}", serde_json::to_string_pretty(&resp)?);
                } else {
                    println!("scheduler: updated");
                }
            }
            SchedulerCmd::Tick { window, budget } => {
                let body = json!({"window": window, "budget": budget});
                let resp = api.post("/v1/reflect/scheduler/tick", &body).await?;
                if json_output {
                    println!("{}", serde_json::to_string_pretty(&resp)?);
                } else {
                    println!(
                        "scheduler tick: status={} reason={}",
                        resp["status"].as_str().unwrap_or("unknown"),
                        resp["reason"].as_str().unwrap_or("-")
                    );
                }
            }
        },
    }

    Ok(())
}
