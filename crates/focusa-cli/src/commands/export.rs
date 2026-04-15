//! Export CLI — docs/21-data-export-cli.md
//!
//! focusa export <dataset_type> --output <path> [options]

use crate::api_client::ApiClient;
use anyhow::bail;
use clap::{Subcommand, ValueEnum};
use serde_json::json;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ExportFormat {
    Jsonl,
    Parquet,
}

impl ExportFormat {
    fn as_str(self) -> &'static str {
        match self {
            Self::Jsonl => "jsonl",
            Self::Parquet => "parquet",
        }
    }
}

#[derive(Debug, Clone)]
struct CommonExportOptions {
    output: String,
    format: ExportFormat,
    min_uxp: f64,
    max_ufi: f64,
    min_autonomy: i32,
    agent: String,
    task: String,
    since: Option<String>,
    until: Option<String>,
    dry_run: bool,
    explain: bool,
}

impl CommonExportOptions {
    fn filters_json(&self) -> serde_json::Value {
        json!({
            "min_uxp": self.min_uxp,
            "max_ufi": self.max_ufi,
            "min_autonomy": self.min_autonomy,
            "agent": self.agent,
            "task": self.task,
            "since": self.since,
            "until": self.until,
        })
    }
}

#[derive(Subcommand)]
pub enum ExportCmd {
    /// Export SFT dataset.
    Sft {
        #[arg(long)]
        output: String,
        #[arg(long, value_enum, default_value = "jsonl")]
        format: ExportFormat,
        #[arg(long, default_value = "0.7")]
        min_uxp: f64,
        #[arg(long, default_value = "0.3")]
        max_ufi: f64,
        #[arg(long, default_value = "0")]
        min_autonomy: i32,
        #[arg(long, default_value = "all")]
        agent: String,
        #[arg(long, default_value = "all")]
        task: String,
        #[arg(long)]
        since: Option<String>,
        #[arg(long)]
        until: Option<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        explain: bool,
        #[arg(long)]
        require_success: bool,
        #[arg(long, default_value = "3")]
        min_turns: u32,
    },
    /// Export preference dataset.
    Preference {
        #[arg(long)]
        output: String,
        #[arg(long, value_enum, default_value = "jsonl")]
        format: ExportFormat,
        #[arg(long, default_value = "0.7")]
        min_uxp: f64,
        #[arg(long, default_value = "0.3")]
        max_ufi: f64,
        #[arg(long, default_value = "0")]
        min_autonomy: i32,
        #[arg(long, default_value = "all")]
        agent: String,
        #[arg(long, default_value = "all")]
        task: String,
        #[arg(long)]
        since: Option<String>,
        #[arg(long)]
        until: Option<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        explain: bool,
        #[arg(long, default_value = "0.15")]
        min_delta: f64,
        #[arg(long)]
        require_user_correction: bool,
    },
    /// Export contrastive dataset.
    Contrastive {
        #[arg(long)]
        output: String,
        #[arg(long, value_enum, default_value = "jsonl")]
        format: ExportFormat,
        #[arg(long, default_value = "0.7")]
        min_uxp: f64,
        #[arg(long, default_value = "0.3")]
        max_ufi: f64,
        #[arg(long, default_value = "0")]
        min_autonomy: i32,
        #[arg(long, default_value = "all")]
        agent: String,
        #[arg(long, default_value = "all")]
        task: String,
        #[arg(long)]
        since: Option<String>,
        #[arg(long)]
        until: Option<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        explain: bool,
        #[arg(long)]
        require_abandoned_branch: bool,
        #[arg(long, default_value = "20")]
        max_path_length: u32,
    },
    /// Export long-horizon dataset.
    LongHorizon {
        #[arg(long)]
        output: String,
        #[arg(long, value_enum, default_value = "jsonl")]
        format: ExportFormat,
        #[arg(long, default_value = "0.7")]
        min_uxp: f64,
        #[arg(long, default_value = "0.3")]
        max_ufi: f64,
        #[arg(long, default_value = "0")]
        min_autonomy: i32,
        #[arg(long, default_value = "all")]
        agent: String,
        #[arg(long, default_value = "all")]
        task: String,
        #[arg(long)]
        since: Option<String>,
        #[arg(long)]
        until: Option<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        explain: bool,
        #[arg(long)]
        min_session_length: Option<String>,
        #[arg(long, default_value = "5")]
        min_state_transitions: u32,
    },
    /// Show export pipeline status.
    Status,
}

fn print_not_implemented_json(value: serde_json::Value) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

fn emit_not_implemented(
    json_mode: bool,
    dataset_type: &str,
    options: &CommonExportOptions,
    dataset_flags: serde_json::Value,
) -> anyhow::Result<()> {
    let payload = json!({
        "status": "not_implemented",
        "dataset_type": dataset_type,
        "dry_run": options.dry_run,
        "explain": options.explain,
        "output": options.output,
        "format": options.format.as_str(),
        "filters": options.filters_json(),
        "dataset_flags": dataset_flags,
        "eligible_records": 0,
        "estimated_dataset_size_bytes": 0,
        "sample_schema_preview": serde_json::Value::Null,
        "exclusion_reasons": ["session replay export pipeline not implemented yet"],
        "manifest": serde_json::Value::Null,
        "reason": "docs/21 export execution phases are not implemented yet",
    });

    if json_mode {
        return print_not_implemented_json(payload);
    }

    println!("Export request not implemented:");
    println!("  Dataset: {}", dataset_type);
    println!("  Mode: {}", if options.dry_run { "dry-run" } else { "write" });
    println!("  Output: {}", options.output);
    println!("  Format: {}", options.format.as_str());
    println!("  Reason: docs/21 export execution phases are not implemented yet");
    if options.explain {
        println!("  Eligible records: 0");
        println!("  Exclusion reasons: session replay export pipeline not implemented yet");
    }
    Ok(())
}

pub async fn run(cmd: ExportCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        ExportCmd::Status => {
            let resp = api.get("/v1/export/status").await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("Export Pipeline Status:");
                println!(
                    "  Status: {}",
                    resp["status"].as_str().unwrap_or("unknown")
                );
                println!(
                    "  Implemented: {}",
                    if resp["implemented"].as_bool().unwrap_or(false) {
                        "yes"
                    } else {
                        "no"
                    }
                );
                println!(
                    "  Dataset types: {}",
                    resp["dataset_types"]
                        .as_array()
                        .map(|items| items.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
                        .unwrap_or_default()
                );
                println!(
                    "  Supported formats: {}",
                    resp["supported_formats"]
                        .as_array()
                        .map(|items| items.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
                        .unwrap_or_default()
                );
                if let Some(reason) = resp["reason"].as_str() {
                    println!("  Reason: {}", reason);
                }
            }
        }
        ExportCmd::Sft {
            output,
            format,
            min_uxp,
            max_ufi,
            min_autonomy,
            agent,
            task,
            since,
            until,
            dry_run,
            explain,
            require_success,
            min_turns,
        } => {
            let options = CommonExportOptions {
                output,
                format,
                min_uxp,
                max_ufi,
                min_autonomy,
                agent,
                task,
                since,
                until,
                dry_run,
                explain,
            };
            emit_not_implemented(
                json_mode,
                "sft",
                &options,
                json!({
                    "require_success": require_success,
                    "min_turns": min_turns,
                }),
            )?;
            if !options.dry_run {
                bail!("export pipeline not implemented yet");
            }
        }
        ExportCmd::Preference {
            output,
            format,
            min_uxp,
            max_ufi,
            min_autonomy,
            agent,
            task,
            since,
            until,
            dry_run,
            explain,
            min_delta,
            require_user_correction,
        } => {
            let options = CommonExportOptions {
                output,
                format,
                min_uxp,
                max_ufi,
                min_autonomy,
                agent,
                task,
                since,
                until,
                dry_run,
                explain,
            };
            emit_not_implemented(
                json_mode,
                "preference",
                &options,
                json!({
                    "min_delta": min_delta,
                    "require_user_correction": require_user_correction,
                }),
            )?;
            if !options.dry_run {
                bail!("export pipeline not implemented yet");
            }
        }
        ExportCmd::Contrastive {
            output,
            format,
            min_uxp,
            max_ufi,
            min_autonomy,
            agent,
            task,
            since,
            until,
            dry_run,
            explain,
            require_abandoned_branch,
            max_path_length,
        } => {
            let options = CommonExportOptions {
                output,
                format,
                min_uxp,
                max_ufi,
                min_autonomy,
                agent,
                task,
                since,
                until,
                dry_run,
                explain,
            };
            emit_not_implemented(
                json_mode,
                "contrastive",
                &options,
                json!({
                    "require_abandoned_branch": require_abandoned_branch,
                    "max_path_length": max_path_length,
                }),
            )?;
            if !options.dry_run {
                bail!("export pipeline not implemented yet");
            }
        }
        ExportCmd::LongHorizon {
            output,
            format,
            min_uxp,
            max_ufi,
            min_autonomy,
            agent,
            task,
            since,
            until,
            dry_run,
            explain,
            min_session_length,
            min_state_transitions,
        } => {
            let options = CommonExportOptions {
                output,
                format,
                min_uxp,
                max_ufi,
                min_autonomy,
                agent,
                task,
                since,
                until,
                dry_run,
                explain,
            };
            emit_not_implemented(
                json_mode,
                "long-horizon",
                &options,
                json!({
                    "min_session_length": min_session_length,
                    "min_state_transitions": min_state_transitions,
                }),
            )?;
            if !options.dry_run {
                bail!("export pipeline not implemented yet");
            }
        }
    }
    Ok(())
}
