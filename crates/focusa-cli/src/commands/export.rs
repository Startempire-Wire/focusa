//! Export CLI — docs/21-data-export-cli.md
//!
//! focusa export <dataset_type> --output <path> [options]

use crate::api_client::ApiClient;
use anyhow::Context;
use clap::{Subcommand, ValueEnum};
use parquet::data_type::{ByteArray, ByteArrayType};
use parquet::file::properties::WriterProperties;
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::parser::parse_message_type;
use serde_json::{Value, json};
use std::fs;
use std::sync::Arc;

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

fn write_jsonl(path: &str, records: &[Value]) -> anyhow::Result<()> {
    let mut out = String::new();
    for record in records {
        out.push_str(&serde_json::to_string(record)?);
        out.push('\n');
    }
    fs::write(path, out).with_context(|| format!("failed to write dataset: {path}"))?;
    Ok(())
}

fn write_parquet(path: &str, records: &[Value]) -> anyhow::Result<()> {
    let file = fs::File::create(path)
        .with_context(|| format!("failed to create parquet dataset: {path}"))?;

    let schema = Arc::new(parse_message_type(
        "message focusa_export { REQUIRED BINARY record_json (UTF8); }",
    )?);
    let props = Arc::new(WriterProperties::builder().build());
    let mut writer = SerializedFileWriter::new(file, schema, props)?;

    let mut row_group = writer.next_row_group()?;
    if let Some(mut col_writer) = row_group.next_column()? {
        let typed = col_writer.typed::<ByteArrayType>();
        let json_rows = records
            .iter()
            .map(|r| serde_json::to_string(r).map(|s| ByteArray::from(s.as_str())))
            .collect::<Result<Vec<_>, _>>()?;
        typed.write_batch(&json_rows, None, None)?;
        col_writer.close()?;
    }

    row_group.close()?;
    writer.close()?;
    Ok(())
}

fn write_manifest(path: &str, manifest: &Value) -> anyhow::Result<String> {
    let manifest_path = format!("{}.manifest.json", path);
    fs::write(&manifest_path, serde_json::to_string_pretty(manifest)?)
        .with_context(|| format!("failed to write manifest: {manifest_path}"))?;
    Ok(manifest_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_parquet_emits_valid_magic_header() {
        let out_path = std::env::temp_dir().join(format!(
            "focusa-export-{}.parquet",
            uuid::Uuid::now_v7()
        ));
        let records = vec![json!({"dataset_type": "sft", "turn_id": "t1"})];

        write_parquet(out_path.to_str().expect("utf8 path"), &records).expect("parquet write");

        let bytes = fs::read(&out_path).expect("read parquet");
        assert!(bytes.len() >= 4);
        assert_eq!(&bytes[..4], b"PAR1");

        let _ = fs::remove_file(out_path);
    }
}

async fn run_export(
    api: &ApiClient,
    json_mode: bool,
    dataset_type: &str,
    options: &CommonExportOptions,
    dataset_flags: Value,
) -> anyhow::Result<()> {
    let body = json!({
        "dataset_type": dataset_type,
        "output": options.output,
        "format": options.format.as_str(),
        "filters": options.filters_json(),
        "dataset_flags": dataset_flags,
        "dry_run": options.dry_run,
        "explain": options.explain,
    });

    let resp = api.post("/v1/export/run", &body).await?;

    if !options.dry_run {
        let records = resp
            .get("records")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        match options.format {
            ExportFormat::Jsonl => write_jsonl(&options.output, &records)?,
            ExportFormat::Parquet => write_parquet(&options.output, &records)?,
        }

        if let Some(manifest) = resp.get("manifest") {
            let manifest_path = write_manifest(&options.output, manifest)?;
            if !json_mode {
                println!("Manifest: {manifest_path}");
            }
        }
    }

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }

    println!("Export completed:");
    println!("  Dataset: {}", dataset_type);
    println!(
        "  Mode: {}",
        if options.dry_run { "dry-run" } else { "write" }
    );
    println!("  Output: {}", options.output);
    println!("  Format: {}", options.format.as_str());
    println!(
        "  Eligible records: {}",
        resp.get("eligible_records")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
    );
    println!(
        "  Excluded records: {}",
        resp.get("excluded_records")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
    );

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
                println!("  Status: {}", resp["status"].as_str().unwrap_or("unknown"));
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
                        .map(|items| items
                            .iter()
                            .filter_map(|v| v.as_str())
                            .collect::<Vec<_>>()
                            .join(", "))
                        .unwrap_or_default()
                );
                println!(
                    "  Supported formats: {}",
                    resp["supported_formats"]
                        .as_array()
                        .map(|items| items
                            .iter()
                            .filter_map(|v| v.as_str())
                            .collect::<Vec<_>>()
                            .join(", "))
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
            run_export(
                &api,
                json_mode,
                "sft",
                &options,
                json!({
                    "require_success": require_success,
                    "min_turns": min_turns,
                }),
            )
            .await?;
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
            run_export(
                &api,
                json_mode,
                "preference",
                &options,
                json!({
                    "min_delta": min_delta,
                    "require_user_correction": require_user_correction,
                }),
            )
            .await?;
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
            run_export(
                &api,
                json_mode,
                "contrastive",
                &options,
                json!({
                    "require_abandoned_branch": require_abandoned_branch,
                    "max_path_length": max_path_length,
                }),
            )
            .await?;
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
            run_export(
                &api,
                json_mode,
                "long-horizon",
                &options,
                json!({
                    "min_session_length": min_session_length,
                    "min_state_transitions": min_state_transitions,
                }),
            )
            .await?;
        }
    }
    Ok(())
}
