//! Spec 133 §24 daemon-native Silent Session CLI.
//!
//! Thin client only: all authority, state transitions, idempotency, retention,
//! receipts, and completion truth remain in daemon routes.

use std::{fs, path::PathBuf, time::Duration};

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use serde_json::{Map, Value, json};

use crate::api_client::ApiClient;

const CLI_SCHEMA: &str = "focusa.silent_cli.v1";

#[derive(Subcommand, Debug)]
pub enum SilentCmd {
    Preflight(ConfigInputArgs),
    Create(CreateArgs),
    Start(SessionMutationArgs),
    List(ListArgs),
    Show(SessionArgs),
    Watch(WatchArgs),
    Output(OutputArgs),
    Send(InputArgs),
    Steer(InputArgs),
    FollowUp(InputArgs),
    Key(KeyArgs),
    Pause(SessionMutationArgs),
    Resume(SessionMutationArgs),
    Interrupt(SessionMutationArgs),
    Cancel(SessionMutationArgs),
    Restart(SessionMutationArgs),
    Adopt(SessionMutationArgs),
    #[command(subcommand)]
    Config(ConfigCmd),
    #[command(subcommand)]
    Profile(ProfileCmd),
    #[command(subcommand)]
    Preset(PresetCmd),
    Checkpoints(SessionArgs),
    Evidence(SessionArgs),
    Receipt(SessionArgs),
    Export(ExportArgs),
    Hold(HoldArgs),
    Delete(DeleteArgs),
    Purge(PurgeArgs),
    Doctor(DoctorArgs),
}

#[derive(Subcommand, Debug)]
pub enum ConfigCmd {
    Resolve(ConfigInputArgs),
    Diff(ConfigSessionArgs),
    Apply(ConfigSessionArgs),
    Rollback(RollbackArgs),
}

#[derive(Subcommand, Debug)]
pub enum ProfileCmd {
    List,
}

#[derive(Subcommand, Debug)]
pub enum PresetCmd {
    List,
}

#[derive(Args, Debug, Clone)]
pub struct SessionArgs {
    /// Exact durable Silent Session id.
    pub session_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct SessionMutationArgs {
    /// Exact durable Silent Session id.
    pub session_id: String,
    /// Idempotency key for mutation replay safety.
    #[arg(long)]
    pub idempotency_key: String,
    /// Human/operator reason recorded with the mutation.
    #[arg(long)]
    pub reason: Option<String>,
}

#[derive(Args, Debug)]
pub struct ConfigInputArgs {
    /// Complete SilentSessionConfig JSON object or envelope containing `config`.
    #[arg(long)]
    pub config_file: PathBuf,
    /// Optional ConfigLayer JSON array.
    #[arg(long)]
    pub layers_file: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Complete SilentSessionConfig JSON object or envelope containing `config`.
    #[arg(long)]
    pub config_file: PathBuf,
    /// Optional ConfigLayer JSON array.
    #[arg(long)]
    pub layers_file: Option<PathBuf>,
    /// Required create idempotency key; safe retries must reuse it unchanged.
    #[arg(long)]
    pub idempotency_key: String,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(long)]
    pub project_root: Option<String>,
    #[arg(long)]
    pub continuity_id: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: usize,
}

#[derive(Args, Debug)]
pub struct WatchArgs {
    pub session_id: String,
    #[arg(long)]
    pub cursor: Option<String>,
    #[arg(long, default_value_t = false)]
    pub follow: bool,
    /// Explicit finite bound; prevents accidental unbounded automation.
    #[arg(long, default_value_t = 1)]
    pub max_polls: usize,
    #[arg(long, default_value_t = 1000)]
    pub interval_ms: u64,
    #[arg(long, default_value_t = 100)]
    pub limit: usize,
}

#[derive(Args, Debug)]
pub struct OutputArgs {
    pub session_id: String,
    #[arg(long)]
    pub cursor: Option<String>,
    #[arg(long, default_value_t = 200)]
    pub limit: usize,
    #[arg(long)]
    pub stream: Option<String>,
}

#[derive(Args, Debug)]
pub struct InputArgs {
    pub session_id: String,
    /// Foreground input or steering text; never interpreted as a shell command by this CLI.
    #[arg(long)]
    pub text: String,
    #[arg(long)]
    pub idempotency_key: String,
}

#[derive(Args, Debug)]
pub struct KeyArgs {
    pub session_id: String,
    /// Named key, e.g. Enter, Escape, ArrowUp, Ctrl-C.
    #[arg(long = "key", required = true)]
    pub keys: Vec<String>,
    #[arg(long)]
    pub idempotency_key: String,
}

#[derive(Args, Debug)]
pub struct ConfigSessionArgs {
    pub session_id: String,
    #[arg(long)]
    pub config_file: PathBuf,
    #[arg(long)]
    pub layers_file: Option<PathBuf>,
    #[arg(long)]
    pub idempotency_key: Option<String>,
}

#[derive(Args, Debug)]
pub struct RollbackArgs {
    pub session_id: String,
    #[arg(long)]
    pub revision: u64,
    #[arg(long)]
    pub idempotency_key: String,
}

#[derive(Args, Debug)]
pub struct ExportArgs {
    pub session_id: String,
    #[arg(long, default_value = "json")]
    pub format: String,
    #[arg(long)]
    pub include_output: bool,
    #[arg(long)]
    pub idempotency_key: String,
}

#[derive(Args, Debug)]
pub struct HoldArgs {
    pub session_id: String,
    #[arg(long)]
    pub reason: String,
    #[arg(long)]
    pub expires_at: Option<String>,
    #[arg(long)]
    pub idempotency_key: String,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    pub session_id: String,
    #[arg(long)]
    pub reason: String,
    #[arg(long)]
    pub idempotency_key: String,
}

#[derive(Args, Debug)]
pub struct PurgeArgs {
    pub session_id: String,
    /// Preview is the default; commit requires both this flag and daemon authorization.
    #[arg(long, default_value_t = false)]
    pub commit: bool,
    #[arg(long)]
    pub reason: String,
    #[arg(long)]
    pub idempotency_key: String,
}

#[derive(Args, Debug)]
pub struct DoctorArgs {
    pub session_id: Option<String>,
    #[arg(long, default_value_t = false)]
    pub deep: bool,
}

fn read_json(path: &PathBuf) -> Result<Value> {
    let body =
        fs::read_to_string(path).with_context(|| format!("read JSON input {}", path.display()))?;
    serde_json::from_str(&body).with_context(|| format!("parse JSON input {}", path.display()))
}

fn config_body(args: &ConfigInputArgs) -> Result<Value> {
    let value = read_json(&args.config_file)?;
    let config = value.get("config").cloned().unwrap_or(value);
    let layers = match &args.layers_file {
        Some(path) => read_json(path)?,
        None => json!([]),
    };
    Ok(json!({"config": config, "layers": layers}))
}

fn config_session_body(args: &ConfigSessionArgs) -> Result<Value> {
    let input = ConfigInputArgs {
        config_file: args.config_file.clone(),
        layers_file: args.layers_file.clone(),
    };
    let mut body = config_body(&input)?;
    if let Some(key) = &args.idempotency_key {
        body["idempotency_key"] = Value::String(key.clone());
    }
    Ok(body)
}

fn mutation_body(args: &SessionMutationArgs) -> Value {
    json!({
        "idempotency_key": args.idempotency_key,
        "reason": args.reason,
    })
}

fn query(items: &[(&str, Option<String>)]) -> String {
    let encoded: Vec<String> = items
        .iter()
        .filter_map(|(key, value)| {
            value.as_ref().map(|value| {
                format!(
                    "{}={}",
                    urlencoding::encode(key),
                    urlencoding::encode(value)
                )
            })
        })
        .collect();
    if encoded.is_empty() {
        String::new()
    } else {
        format!("?{}", encoded.join("&"))
    }
}

fn redact(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                let sensitive = ["secret", "token", "credential", "authorization", "api_key"]
                    .iter()
                    .any(|needle| key.to_ascii_lowercase().contains(needle));
                if sensitive {
                    *value = Value::String("[REDACTED]".into());
                } else {
                    redact(value);
                }
            }
        }
        Value::Array(values) => values.iter_mut().for_each(redact),
        _ => {}
    }
}

fn wrap(command: &str, mut result: Value) -> Value {
    redact(&mut result);
    json!({
        "schema": CLI_SCHEMA,
        "command": command,
        "status": result.get("status").cloned().unwrap_or_else(|| json!("completed")),
        "canonical": result.get("canonical").cloned().unwrap_or(Value::Null),
        "side_effects": result.get("side_effects").cloned().unwrap_or_else(|| json!([])),
        "session_id": result.pointer("/data/session/id").or_else(|| result.pointer("/session/id")).or_else(|| result.get("session_id")).cloned(),
        "run_id": result.pointer("/data/run/id").or_else(|| result.pointer("/run/id")).or_else(|| result.get("run_id")).cloned(),
        "process_status": result.pointer("/data/process_status").or_else(|| result.get("process_status")).cloned(),
        "completion_status": result.pointer("/data/completion_status").or_else(|| result.get("completion_status")).cloned(),
        "result": result,
    })
}

fn print_result(command: &str, result: Value, json_output: bool) -> Result<()> {
    let envelope = wrap(command, result);
    if json_output {
        println!("{}", serde_json::to_string_pretty(&envelope)?);
        return Ok(());
    }
    println!(
        "silent {command} → {}",
        envelope["status"].as_str().unwrap_or("completed")
    );
    for key in [
        "session_id",
        "run_id",
        "process_status",
        "completion_status",
    ] {
        if !envelope[key].is_null() {
            println!("{key}={}", envelope[key]);
        }
    }
    if let Some(side_effects) = envelope["side_effects"].as_array()
        && !side_effects.is_empty()
    {
        println!("side_effects={}", serde_json::to_string(side_effects)?);
    }
    if command == "list" || command == "show" || command == "doctor" {
        println!("{}", serde_json::to_string_pretty(&envelope["result"])?);
    }
    Ok(())
}

async fn delete(client: &ApiClient, path: &str, body: &Value) -> Result<Value> {
    let url = format!("{}{}", client.base_url(), path);
    let response = client.http_client().delete(url).json(body).send().await?;
    let status = response.status();
    let body: Value = response
        .json()
        .await
        .context("decode daemon delete response")?;
    if !status.is_success() {
        bail!("[API_HTTP_ERROR] status={status} body={body}");
    }
    Ok(body)
}

async fn execute(client: &ApiClient, command: SilentCmd, json_output: bool) -> Result<()> {
    let (name, result) = match command {
        SilentCmd::Preflight(args) => ("preflight", client.post("/silent-sessions/preflight", &config_body(&args)?).await?),
        SilentCmd::Create(args) => {
            let input = ConfigInputArgs { config_file: args.config_file, layers_file: args.layers_file };
            let mut body = config_body(&input)?;
            body["idempotency_key"] = Value::String(args.idempotency_key);
            ("create", client.post("/silent-sessions", &body).await?)
        }
        SilentCmd::Start(args) => ("start", client.post(&format!("/silent-sessions/{}/start", args.session_id), &mutation_body(&args)).await?),
        SilentCmd::List(args) => {
            let query = query(&[
                ("project_root", args.project_root),
                ("continuity_id", args.continuity_id),
                ("status", args.status),
                ("limit", Some(args.limit.clamp(1, 200).to_string())),
            ]);
            ("list", client.get(&format!("/silent-sessions{query}")).await?)
        }
        SilentCmd::Show(args) => ("show", client.get(&format!("/silent-sessions/{}", args.session_id)).await?),
        SilentCmd::Watch(args) => {
            let polls = if args.follow { args.max_polls.clamp(1, 10_000) } else { 1 };
            let mut cursor = args.cursor;
            let mut responses = Vec::new();
            for index in 0..polls {
                let suffix = query(&[("cursor", cursor.clone()), ("limit", Some(args.limit.clamp(1, 500).to_string()))]);
                let value = client.get(&format!("/silent-sessions/{}/events{suffix}", args.session_id)).await?;
                cursor = value.get("next_cursor").and_then(Value::as_str).map(str::to_string).or(cursor);
                responses.push(value);
                if index + 1 < polls { tokio::time::sleep(Duration::from_millis(args.interval_ms.max(50))).await; }
            }
            ("watch", json!({"status":"completed","cursor":cursor,"polls":responses.len(),"events":responses}))
        }
        SilentCmd::Output(args) => {
            let suffix = query(&[("cursor", args.cursor), ("limit", Some(args.limit.clamp(1, 1000).to_string())), ("stream", args.stream)]);
            ("output", client.get(&format!("/silent-sessions/{}/output{suffix}", args.session_id)).await?)
        }
        SilentCmd::Send(args) => ("send", client.post(&format!("/silent-sessions/{}/input", args.session_id), &json!({"text":args.text,"idempotency_key":args.idempotency_key})).await?),
        SilentCmd::Steer(args) => ("steer", client.post(&format!("/silent-sessions/{}/steer", args.session_id), &json!({"text":args.text,"idempotency_key":args.idempotency_key})).await?),
        SilentCmd::FollowUp(args) => ("follow-up", client.post(&format!("/silent-sessions/{}/follow-up", args.session_id), &json!({"text":args.text,"idempotency_key":args.idempotency_key})).await?),
        SilentCmd::Key(args) => ("key", client.post(&format!("/silent-sessions/{}/keys", args.session_id), &json!({"keys":args.keys,"idempotency_key":args.idempotency_key})).await?),
        SilentCmd::Pause(args) => ("pause", client.post(&format!("/silent-sessions/{}/pause", args.session_id), &mutation_body(&args)).await?),
        SilentCmd::Resume(args) => ("resume", client.post(&format!("/silent-sessions/{}/resume", args.session_id), &mutation_body(&args)).await?),
        SilentCmd::Interrupt(args) => ("interrupt", client.post(&format!("/silent-sessions/{}/interrupt", args.session_id), &mutation_body(&args)).await?),
        SilentCmd::Cancel(args) => ("cancel", client.post(&format!("/silent-sessions/{}/cancel", args.session_id), &mutation_body(&args)).await?),
        SilentCmd::Restart(args) => ("restart", client.post(&format!("/silent-sessions/{}/restart", args.session_id), &mutation_body(&args)).await?),
        SilentCmd::Adopt(args) => ("adopt", client.post(&format!("/silent-sessions/{}/adopt", args.session_id), &mutation_body(&args)).await?),
        SilentCmd::Config(command) => match command {
            ConfigCmd::Resolve(args) => ("config resolve", client.post("/silent-sessions/config/resolve", &config_body(&args)?).await?),
            ConfigCmd::Diff(args) => ("config diff", client.post(&format!("/silent-sessions/{}/config/preview", args.session_id), &config_session_body(&args)?).await?),
            ConfigCmd::Apply(args) => ("config apply", client.post(&format!("/silent-sessions/{}/config/revisions", args.session_id), &config_session_body(&args)?).await?),
            ConfigCmd::Rollback(args) => ("config rollback", client.post(&format!("/silent-sessions/{}/config/rollback", args.session_id), &json!({"revision":args.revision,"idempotency_key":args.idempotency_key})).await?),
        },
        SilentCmd::Profile(ProfileCmd::List) => ("profile list", client.get("/silent-sessions/profiles").await?),
        SilentCmd::Preset(PresetCmd::List) => ("preset list", client.get("/silent-sessions/presets").await?),
        SilentCmd::Checkpoints(args) => ("checkpoints", client.get(&format!("/silent-sessions/{}/checkpoints", args.session_id)).await?),
        SilentCmd::Evidence(args) => ("evidence", client.get(&format!("/silent-sessions/{}/artifacts", args.session_id)).await?),
        SilentCmd::Receipt(args) => ("receipt", client.get(&format!("/silent-sessions/{}/receipts", args.session_id)).await?),
        SilentCmd::Export(args) => ("export", client.post(&format!("/silent-sessions/{}/export", args.session_id), &json!({"format":args.format,"include_output":args.include_output,"idempotency_key":args.idempotency_key})).await?),
        SilentCmd::Hold(args) => ("hold", client.post(&format!("/silent-sessions/{}/evidence-hold", args.session_id), &json!({"reason":args.reason,"expires_at":args.expires_at,"idempotency_key":args.idempotency_key})).await?),
        SilentCmd::Delete(args) => ("delete", delete(client, &format!("/silent-sessions/{}", args.session_id), &json!({"reason":args.reason,"idempotency_key":args.idempotency_key})).await?),
        SilentCmd::Purge(args) => ("purge", client.post(&format!("/silent-sessions/{}/purge", args.session_id), &json!({"commit":args.commit,"reason":args.reason,"idempotency_key":args.idempotency_key})).await?),
        SilentCmd::Doctor(args) => {
            let capabilities = client.get("/silent-sessions/capabilities").await?;
            let session = match args.session_id {
                Some(id) => Some(client.get(&format!("/silent-sessions/{id}/status")).await?),
                None => None,
            };
            ("doctor", json!({"status":"completed","deep":args.deep,"capabilities":capabilities,"session":session,"checks":{"daemon":true,"catalog":true},"next_tools":["focusa_tool_doctor"]}))
        }
    };
    print_result(name, result, json_output)
}

pub async fn run(command: SilentCmd, json_output: bool) -> Result<()> {
    let client = ApiClient::new();
    if let Err(error) = execute(&client, command, json_output).await {
        if json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schema": CLI_SCHEMA,
                    "status": "error",
                    "failure_class": "command_failed",
                    "message": error.to_string(),
                    "retry": {"safe": false, "posture": "inspect_side_effects_first"},
                    "recovery": ["focusa silent doctor", "inspect receipts and exact session/run ids before retry"]
                }))?
            );
        }
        return Err(error);
    }
    Ok(())
}
