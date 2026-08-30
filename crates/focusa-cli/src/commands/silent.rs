//! Spec 133 §24 daemon-native Silent Session CLI.
//!
//! Thin client only: all authority, state transitions, idempotency, retention,
//! receipts, and completion truth remain in daemon routes.

use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use serde_json::{Map, Value, json};

use crate::api_client::ApiClient;

use super::silent_render::{CLI_SCHEMA, print_result};

#[derive(Subcommand, Debug)]
pub enum SilentCmd {
    /// Resolve and validate an exact SilentSessionConfig without creating a session.
    Preflight(ConfigInputArgs),
    Create(CreateArgs),
    Start(SessionMutationArgs),
    #[command(subcommand)]
    Approval(ApprovalCmd),
    List(ListArgs),
    Show(ShowArgs),
    Status(StatusArgs),
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
    #[command(subcommand)]
    Model(ModelCmd),
    Checkpoints(ExactSessionArgs),
    Evidence(ProofArgs),
    Receipt(ProofArgs),
    Export(ExportArgs),
    Hold(RetentionArgs),
    Delete(RetentionArgs),
    Purge(RetentionArgs),
    Doctor(DoctorArgs),
}

#[derive(Subcommand, Debug)]
pub enum ApprovalCmd {
    /// Preview the exact server-derived approval target and action digest.
    Preview(ApprovalFileArgs),
    /// Persist the exact previewed approval; the request must include expected_action_digest.
    Create(ApprovalFileArgs),
}

#[derive(Args, Debug)]
pub struct ApprovalFileArgs {
    /// Exact durable Silent Session id.
    pub session_id: String,
    /// JSON request bound to the exact session/run/generation/action.
    #[arg(long)]
    pub request_file: PathBuf,
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

#[derive(Subcommand, Debug)]
pub enum ModelCmd {
    /// List models explicitly allowed by the server-owned Pi runtime catalog.
    List(ModelListArgs),
    /// Resolve one exact model or return a typed unsupported_model result.
    Preflight(ModelPreflightArgs),
}

#[derive(Args, Debug)]
pub struct ModelListArgs {
    #[arg(long, default_value = "pi-runtime")]
    pub provider: String,
}

#[derive(Args, Debug)]
pub struct ModelPreflightArgs {
    #[arg(long, default_value = "pi-runtime")]
    pub provider: String,
    #[arg(long)]
    pub model: String,
    #[arg(long)]
    pub thinking: Option<String>,
    #[arg(long, default_value_t = true)]
    pub strict: bool,
    #[arg(long, default_value_t = false)]
    pub require_entitlement_preflight: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ShowArgs {
    pub session_id: String,
    /// Accepted for backward compatibility; show is session-scoped.
    #[arg(long, alias = "run")]
    pub run_id: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct StatusArgs {
    pub session_id: String,
    #[arg(long, alias = "run")]
    pub run_id: String,
    /// Exact current run generation; stale generations fail closed.
    #[arg(long)]
    pub generation: u64,
}

#[derive(Args, Debug, Clone)]
pub struct ExactSessionArgs {
    /// Exact durable Silent Session id.
    pub session_id: String,
    #[arg(long)]
    pub run_id: String,
    #[arg(long)]
    pub generation: u64,
}

#[derive(Args, Debug, Clone)]
pub struct SessionMutationArgs {
    /// Exact durable Silent Session id.
    pub session_id: String,
    /// Exact current run id; never inferred by the CLI.
    #[arg(long, alias = "run")]
    pub run_id: String,
    /// Exact current run generation; stale generations are rejected.
    #[arg(long)]
    pub generation: u64,
    #[arg(long)]
    pub actor_instance_ref: Option<String>,
    /// Explicit daemon approval id for the intended mutation.
    #[arg(long)]
    pub approval_id: String,
    /// Idempotency key for mutation replay safety.
    #[arg(long)]
    pub idempotency_key: Option<String>,
    #[arg(long)]
    pub lease_file: Option<PathBuf>,
    #[arg(long)]
    pub reason_code: Option<String>,
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
    pub config_file: Option<PathBuf>,
    /// Optional complete governed lifecycle request.
    #[arg(long)]
    pub request_file: Option<PathBuf>,
    /// Optional ConfigLayer JSON array.
    #[arg(long)]
    pub layers_file: Option<PathBuf>,
    /// Required for config-based create; governed requests carry their own replay contract.
    #[arg(long)]
    pub idempotency_key: Option<String>,
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
    #[arg(long, alias = "run")]
    pub run_id: String,
    #[arg(long)]
    pub generation: Option<u64>,
    #[arg(long, alias = "after")]
    pub cursor: Option<String>,
    #[arg(long)]
    pub tools: bool,
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
    #[arg(long, alias = "run")]
    pub run_id: String,
    #[arg(long)]
    pub generation: u64,
    #[arg(long, alias = "after")]
    pub cursor: Option<String>,
    #[arg(long, default_value_t = 200)]
    pub limit: usize,
    #[arg(long)]
    pub stream: Option<String>,
}

#[derive(Args, Debug)]
pub struct InputArgs {
    pub session_id: String,
    #[arg(long, alias = "run")]
    pub run_id: String,
    #[arg(long)]
    pub generation: u64,
    #[arg(long)]
    pub actor_instance_ref: String,
    #[arg(long)]
    pub approval_id: String,
    #[arg(long)]
    pub idempotency_key: String,
    #[arg(long)]
    pub lease_file: PathBuf,
    #[arg(long)]
    pub payload_file: PathBuf,
    /// Foreground input or steering text; never interpreted as a shell command by this CLI.
    #[arg(long)]
    pub text: String,
}

#[derive(Args, Debug)]
pub struct KeyArgs {
    pub session_id: String,
    #[arg(long, alias = "run")]
    pub run_id: String,
    #[arg(long)]
    pub generation: u64,
    #[arg(long)]
    pub actor_instance_ref: String,
    #[arg(long)]
    pub approval_id: String,
    #[arg(long)]
    pub idempotency_key: String,
    #[arg(long)]
    pub lease_file: PathBuf,
    #[arg(long)]
    pub payload_file: PathBuf,
    /// Named key, e.g. Enter, Escape, ArrowUp, Ctrl-C.
    #[arg(long = "key", required = true)]
    pub keys: Vec<String>,
}

#[derive(Args, Debug)]
pub struct ConfigSessionArgs {
    pub session_id: String,
    #[arg(long)]
    pub run_id: String,
    #[arg(long)]
    pub generation: u64,
    #[arg(long)]
    pub approval_id: Option<String>,
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
    pub run_id: String,
    #[arg(long)]
    pub generation: u64,
    #[arg(long)]
    pub approval_id: String,
    #[arg(long)]
    pub revision: String,
    #[arg(long)]
    pub idempotency_key: String,
}

#[derive(Args, Debug)]
pub struct ProofArgs {
    pub session_id: String,
    #[arg(long, alias = "run")]
    pub run_id: String,
    /// Exact current run generation; stale generations fail closed.
    #[arg(long)]
    pub generation: u64,
    #[arg(long)]
    pub after: Option<String>,
    #[arg(long, default_value_t = 200)]
    pub limit: usize,
}

#[derive(Args, Debug)]
pub struct ExportArgs {
    pub session_id: String,
    #[arg(long, alias = "run")]
    pub run_id: String,
    #[arg(long)]
    pub generation: Option<u64>,
    #[arg(long)]
    pub after: Option<String>,
    #[arg(long, default_value_t = 200)]
    pub limit: usize,
    #[arg(long, default_value = "json")]
    pub format: String,
    #[arg(long)]
    pub include_output: bool,
    #[arg(long)]
    pub idempotency_key: Option<String>,
}

#[derive(Args, Debug)]
pub struct RetentionArgs {
    pub session_id: String,
    #[arg(long, alias = "run")]
    pub run_id: String,
    #[arg(long)]
    pub generation: u64,
    #[arg(long)]
    pub actor_instance_ref: String,
    #[arg(long)]
    pub approval_id: String,
    #[arg(long)]
    pub context_authority_file: PathBuf,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub apply: bool,
    #[arg(long)]
    pub reason_code: String,
    #[arg(long)]
    pub impact_preview_ref: Option<String>,
    #[arg(long)]
    pub confirm_delete: Option<String>,
    #[arg(long)]
    pub confirm_irreversible_purge: Option<String>,
}

#[derive(Args, Debug)]
pub struct HoldArgs {
    pub session_id: String,
    #[arg(long)]
    pub run_id: String,
    #[arg(long)]
    pub generation: u64,
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
    pub run_id: String,
    #[arg(long)]
    pub generation: u64,
    #[arg(long)]
    pub reason: String,
    #[arg(long)]
    pub idempotency_key: String,
}

#[derive(Args, Debug)]
pub struct PurgeArgs {
    pub session_id: String,
    #[arg(long)]
    pub run_id: String,
    #[arg(long)]
    pub generation: u64,
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
    body["run_id"] = Value::String(args.run_id.clone());
    body["generation"] = json!(args.generation);
    if let Some(approval_id) = &args.approval_id {
        body["approval_id"] = Value::String(approval_id.clone());
    }
    if let Some(key) = &args.idempotency_key {
        body["idempotency_key"] = Value::String(key.clone());
    }
    Ok(body)
}

fn config_apply_body(args: &ConfigSessionArgs) -> Result<Value> {
    anyhow::ensure!(
        args.approval_id.is_some(),
        "--approval-id is required for config apply"
    );
    anyhow::ensure!(
        args.idempotency_key.is_some(),
        "--idempotency-key is required for config apply"
    );
    config_session_body(args)
}

fn create_body(args: &CreateArgs) -> Result<Value> {
    if let Some(path) = &args.request_file {
        let request = read_json(path)?;
        let authority = request
            .pointer("/context_authority/project_identity_ref")
            .and_then(Value::as_str);
        let session = request
            .pointer("/session/project_identity_ref")
            .and_then(Value::as_str);
        let config = request
            .pointer("/initial_config/identity/project_identity_ref")
            .and_then(Value::as_str);
        anyhow::ensure!(
            authority.is_some() && authority == session && authority == config,
            "[CLI_STALE_SCOPE] create request context authority does not match the exact project scope"
        );
        return Ok(request);
    }
    let config_file = args
        .config_file
        .clone()
        .context("--config-file or --request-file is required")?;
    let idempotency_key = args
        .idempotency_key
        .clone()
        .context("--idempotency-key is required with --config-file")?;
    let input = ConfigInputArgs {
        config_file,
        layers_file: args.layers_file.clone(),
    };
    let mut body = config_body(&input)?;
    body["idempotency_key"] = Value::String(idempotency_key);
    Ok(body)
}

fn mutation_body(args: &SessionMutationArgs) -> Result<Value> {
    if let Some(path) = &args.lease_file {
        let lease = read_json(path)?;
        let actor = args
            .actor_instance_ref
            .as_deref()
            .context("--actor-instance-ref is required with --lease-file")?;
        let expires_at = lease
            .get("expires_at")
            .and_then(Value::as_str)
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok());
        anyhow::ensure!(
            lease.get("session_id").and_then(Value::as_str) == Some(args.session_id.as_str())
                && lease
                    .get("owner_actor_instance_ref")
                    .and_then(Value::as_str)
                    == Some(actor)
                && expires_at.is_some_and(|value| value > chrono::Utc::now()),
            "[CLI_STALE_SCOPE] lifecycle lease is stale or not bound to the exact session and actor"
        );
        return Ok(json!({
            "actor_instance_ref": actor,
            "approval_id": args.approval_id,
            "legacy_approved": false,
            "lease": lease,
            "reason_code": args.reason_code,
            "idempotency_key": args.idempotency_key,
        }));
    }
    let idempotency_key = args
        .idempotency_key
        .clone()
        .context("--idempotency-key or --lease-file is required")?;
    Ok(json!({
        "run_id": args.run_id,
        "generation": args.generation,
        "approval_id": args.approval_id,
        "idempotency_key": idempotency_key,
    }))
}

async fn lifecycle_call(
    client: &ApiClient,
    args: &SessionMutationArgs,
    operation: &str,
) -> Result<Value> {
    let path = if args.lease_file.is_some() {
        format!(
            "/v1/silent-sessions/{}/{operation}?run_id={}&expected_generation={}",
            args.session_id, args.run_id, args.generation
        )
    } else {
        format!("/v1/silent-sessions/{}/{operation}", args.session_id)
    };
    let result = client.post(&path, &mutation_body(args)?).await?;
    if result.get("ok").and_then(Value::as_bool) == Some(false)
        || result.get("stale").and_then(Value::as_bool) == Some(true)
    {
        let failure = result
            .get("failure_class")
            .and_then(Value::as_str)
            .unwrap_or("lifecycle_rejected");
        bail!("lifecycle mutation rejected: {failure}");
    }
    Ok(result)
}

fn validate_session_read(result: Value, session_id: &str) -> Result<Value> {
    let response_session = result
        .pointer("/data/id")
        .and_then(Value::as_str)
        .or_else(|| result.pointer("/data/session_id").and_then(Value::as_str))
        .or_else(|| result.pointer("/data/session/id").and_then(Value::as_str))
        .or_else(|| {
            result
                .pointer("/data/session/session_id")
                .and_then(Value::as_str)
        });
    anyhow::ensure!(
        response_session == Some(session_id),
        "[API_DECODE_ERROR] daemon response did not match the exact requested session"
    );
    Ok(result)
}

fn validate_exact_read(result: Value, session_id: &str, run_id: &str) -> Result<Value> {
    let response_session = result
        .pointer("/data/session_id")
        .and_then(Value::as_str)
        .or_else(|| {
            result
                .pointer("/data/session/session_id")
                .and_then(Value::as_str)
        });
    let response_run = result
        .pointer("/data/run_id")
        .and_then(Value::as_str)
        .or_else(|| result.pointer("/data/run/run_id").and_then(Value::as_str));
    anyhow::ensure!(
        response_session.is_none_or(|value| value == session_id) && response_run == Some(run_id),
        "[API_DECODE_ERROR] daemon response did not match the exact requested session and run"
    );
    Ok(result)
}

async fn watch_call(client: &ApiClient, args: &WatchArgs) -> Result<Value> {
    anyhow::ensure!(
        (1..=500).contains(&args.limit),
        "--limit must be between 1-500"
    );
    let generation = args
        .generation
        .context("--generation is required for exact-run watch")?;
    let max_polls = args.max_polls.clamp(1, 10_000);
    let poll_count = if args.follow { max_polls } else { 1 };
    let mut events = Vec::new();
    let mut next_cursor = args.cursor.clone();
    for poll in 0..poll_count {
        let path = format!(
            "/v1/silent-sessions/{}/events{}",
            args.session_id,
            query(&[
                ("run_id", Some(args.run_id.clone())),
                ("generation", Some(generation.to_string())),
                ("cursor", next_cursor.clone()),
                ("limit", Some(args.limit.to_string())),
                ("follow", Some("false".into())),
            ])
        );
        let page = client.get(&path).await?;
        let page_events = page
            .pointer("/data/events")
            .and_then(Value::as_array)
            .context("watch response omitted event page")?;
        events.extend(
            page_events
                .iter()
                .filter(|event| {
                    if !args.tools {
                        return true;
                    }
                    event
                        .get("kind")
                        .and_then(Value::as_str)
                        .is_some_and(|kind| kind.starts_with("tool."))
                })
                .cloned(),
        );
        next_cursor = page
            .pointer("/data/next_cursor")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or(next_cursor);
        if args.follow && poll + 1 < poll_count {
            tokio::time::sleep(std::time::Duration::from_millis(
                args.interval_ms.clamp(10, 60_000),
            ))
            .await;
        }
    }
    Ok(json!({
        "ok": true, "status": "observed", "canonical": true, "advisory": false,
        "degraded": false, "stale": false, "failure_class": Value::Null,
        "retry": {"safe": true, "posture": "idempotent_read"}, "side_effects": [],
        "evidence_refs": [], "receipt_refs": [], "next_tools": [],
        "recovery_hint": Value::Null, "misuse_hint": Value::Null,
        "data": {"session_id": args.session_id, "run_id": args.run_id, "generation": generation,
                 "after_cursor": args.cursor, "next_cursor": next_cursor,
                 "event_count": events.len(), "events": events}
    }))
}

async fn proof_call(client: &ApiClient, args: &ProofArgs, collection: &str) -> Result<Value> {
    anyhow::ensure!(
        (1..=1000).contains(&args.limit),
        "--limit must be between 1-1000"
    );
    let route = match collection {
        "artifacts" => "/artifacts",
        "receipts" => "/receipts",
        _ => anyhow::bail!("unsupported proof collection"),
    };
    let mut path = format!(
        "/v1/silent-sessions/{}{}?run_id={}&generation={}&limit={}",
        args.session_id,
        route,
        urlencoding::encode(&args.run_id),
        args.generation,
        args.limit
    );
    if let Some(after) = &args.after {
        path.push_str("&after=");
        path.push_str(&urlencoding::encode(after));
    }
    let result = client.get(&path).await?;
    anyhow::ensure!(
        result.pointer("/data/session_id").and_then(Value::as_str)
            == Some(args.session_id.as_str())
            && result.pointer("/data/run_id").and_then(Value::as_str) == Some(args.run_id.as_str())
            && result.pointer("/data/generation").and_then(Value::as_u64) == Some(args.generation)
            && result
                .pointer("/data/limit")
                .and_then(Value::as_u64)
                .is_none_or(|limit| limit <= args.limit as u64),
        "proof response violated exact session/run/generation or bounded-limit authority"
    );
    Ok(result)
}

async fn export_call(client: &ApiClient, args: &ExportArgs) -> Result<Value> {
    anyhow::ensure!(
        (1..=1000).contains(&args.limit),
        "--limit must be between 1-1000"
    );
    let path = format!(
        "/v1/silent-sessions/{}/export?run_id={}",
        args.session_id,
        urlencoding::encode(&args.run_id)
    );
    let body = json!({
        "schema": "focusa.silent_session_export_request.v1",
        "run_id": args.run_id,
        "generation": args.generation,
        "after_cursor": args.after,
        "event_limit": args.limit,
        "redaction_required": true,
        "format": args.format,
        "include_output": args.include_output,
        "idempotency_key": args.idempotency_key,
    });
    let result = client.post(&path, &body).await?;
    anyhow::ensure!(
        result.pointer("/data/run_id").and_then(Value::as_str) == Some(args.run_id.as_str())
            && result.pointer("/data/redacted").and_then(Value::as_bool) == Some(true),
        "export response violated exact-run or redaction authority"
    );
    Ok(result)
}

fn retention_body(args: &RetentionArgs, operation: &str) -> Result<Value> {
    anyhow::ensure!(
        args.dry_run ^ args.apply,
        "choose exactly one of --dry-run or --apply"
    );
    let context_authority = read_json(&args.context_authority_file)?;
    anyhow::ensure!(
        context_authority.get("allowed").and_then(Value::as_bool) == Some(true),
        "retention Context Authority is not allowed"
    );
    let scope = if operation == "purge" {
        "silent_sessions:forensics"
    } else {
        "silent_sessions:admin"
    };
    let confirmation = if args.dry_run {
        anyhow::ensure!(
            args.impact_preview_ref.is_none()
                && args.confirm_delete.is_none()
                && args.confirm_irreversible_purge.is_none(),
            "dry-run cannot accept apply confirmations"
        );
        Value::Null
    } else if operation == "delete" {
        let preview = args
            .impact_preview_ref
            .as_deref()
            .context("--impact-preview-ref is required for delete apply")?;
        let confirmation = args
            .confirm_delete
            .as_deref()
            .context("--confirm-delete is required for delete apply")?;
        anyhow::ensure!(
            confirmation == args.session_id,
            "--confirm-delete must exactly equal the canonical session_id"
        );
        json!({"session_id": confirmation, "active_projection_removal_acknowledged": true, "irreversible_forensic_loss_acknowledged": false, "impact_preview_ref": preview})
    } else if operation == "purge" {
        let preview = args
            .impact_preview_ref
            .as_deref()
            .context("--impact-preview-ref is required for purge apply")?;
        let confirmation = args
            .confirm_irreversible_purge
            .as_deref()
            .context("--confirm-irreversible-purge is required for purge apply")?;
        anyhow::ensure!(
            confirmation == args.session_id,
            "--confirm-irreversible-purge must exactly equal the canonical session_id"
        );
        json!({"session_id": confirmation, "active_projection_removal_acknowledged": false, "irreversible_forensic_loss_acknowledged": true, "impact_preview_ref": preview})
    } else {
        Value::Null
    };
    Ok(json!({
        "operation": operation, "session_id": args.session_id, "run_id": args.run_id,
        "expected_generation": args.generation, "actor_instance_ref": args.actor_instance_ref,
        "approval_id": args.approval_id, "legacy_approved": false,
        "required_authority_scope": scope, "context_authority": context_authority,
        "dry_run": args.dry_run, "side_effect_policy": if args.dry_run { "preview" } else { "commit" },
        "reason_code": args.reason_code, "impact_preview_ref": args.impact_preview_ref,
        "confirmation": confirmation, "evidence_hold": operation == "hold",
        "process_termination_allowed": false, "completion_transition_allowed": false,
        "lifecycle_transition_allowed": false
    }))
}

fn validate_retention_result(
    result: Value,
    args: &RetentionArgs,
    operation: &str,
) -> Result<Value> {
    let data = result.get("data").unwrap_or(&Value::Null);
    let unsafe_transition = data
        .get("process_termination_performed")
        .and_then(Value::as_bool)
        == Some(true)
        || data
            .get("completion_transition_performed")
            .and_then(Value::as_bool)
            == Some(true);
    let preview_effect = args.dry_run
        && result
            .get("side_effects")
            .and_then(Value::as_array)
            .is_some_and(|items| !items.is_empty());
    let held_purge = operation == "purge"
        && (data.get("evidence_hold_active").and_then(Value::as_bool) == Some(true)
            || data.get("purge_eligible").and_then(Value::as_bool) == Some(false));
    anyhow::ensure!(
        !unsafe_transition && !preview_effect && !held_purge,
        "retention response violated status separation, preview safety, or Evidence hold"
    );
    Ok(result)
}

async fn retention_call(
    client: &ApiClient,
    args: &RetentionArgs,
    operation: &str,
) -> Result<Value> {
    let path = if operation == "delete" {
        format!(
            "/v1/silent-sessions/{}?run_id={}&expected_generation={}",
            args.session_id,
            urlencoding::encode(&args.run_id),
            args.generation
        )
    } else {
        let route = match operation {
            "hold" => "/evidence-hold",
            "purge" => "/purge",
            _ => anyhow::bail!("unsupported retention operation"),
        };
        format!(
            "/v1/silent-sessions/{}{}?run_id={}&expected_generation={}",
            args.session_id,
            route,
            urlencoding::encode(&args.run_id),
            args.generation
        )
    };
    let body = retention_body(args, operation)?;
    let result = if operation == "delete" {
        delete(client, &path, &body).await?
    } else {
        client.post(&path, &body).await?
    };
    validate_retention_result(result, args, operation)
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

#[allow(clippy::too_many_arguments)]
fn interaction_body(
    session_id: &str,
    actor_instance_ref: &str,
    approval_id: &str,
    idempotency_key: &str,
    lease_file: &PathBuf,
    payload_file: &PathBuf,
    text: Option<&str>,
    keys: Option<&[String]>,
) -> Result<Value> {
    let lease = read_json(lease_file)?;
    let payload = read_json(payload_file)?;
    let lease_session = lease.get("session_id").and_then(Value::as_str);
    let lease_actor = lease
        .get("owner_actor_instance_ref")
        .and_then(Value::as_str);
    let expires_at = lease
        .get("expires_at")
        .and_then(Value::as_str)
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok());
    let lease_current = expires_at.is_some_and(|value| value > chrono::Utc::now());
    anyhow::ensure!(
        lease_session == Some(session_id)
            && lease_actor == Some(actor_instance_ref)
            && lease_current,
        "[CLI_STALE_SCOPE] interaction lease is stale or not bound to the exact session and actor"
    );
    Ok(json!({
        "actor_instance_ref": actor_instance_ref,
        "approval_id": approval_id,
        "legacy_approved": false,
        "idempotency_key": idempotency_key,
        "lease": lease,
        "payload": payload,
        "text": text,
        "keys": keys,
    }))
}

fn validate_interaction_replay(result: &Value) -> Result<()> {
    let replayed = result.pointer("/data/replayed").and_then(Value::as_bool) == Some(true);
    let reason = result.pointer("/retry/reason").and_then(Value::as_str);
    anyhow::ensure!(
        !replayed || reason == Some("idempotent_replay"),
        "interaction response lacks unambiguous replay safety"
    );
    Ok(())
}

fn doctor_check(
    component: &str,
    ok: bool,
    failure_class: Option<&str>,
    summary: &str,
    recovery_hint: &str,
) -> Value {
    json!({
        "component": component,
        "status": if ok { "ok" } else { "blocked" },
        "summary": summary,
        "failure_class": failure_class,
        "retry": {
            "safe": ok,
            "posture": if ok { "idempotent_recheck" } else { "recover_then_recheck" }
        },
        "recovery_hint": recovery_hint
    })
}

fn blocked_doctor_report(failure_class: &str, recovery_hint: &str) -> Value {
    let checks = ["daemon", "harness", "provider", "config"]
        .into_iter()
        .map(|component| {
            doctor_check(
                component,
                false,
                Some(failure_class),
                "Probe unavailable.",
                recovery_hint,
            )
        })
        .collect::<Vec<_>>();
    json!({
        "schema": "focusa.silent_cli_doctor.v1",
        "status": "blocked",
        "ok": false,
        "canonical": false,
        "degraded": true,
        "failure_class": failure_class,
        "retry": {"safe": false, "posture": "recover_then_recheck"},
        "side_effects": [],
        "data": {"read_only": true, "checks": checks}
    })
}

async fn silent_doctor_report(client: &ApiClient) -> Value {
    let (_, health) = match client.get_probe("/v1/health").await {
        Ok(response) => response,
        Err(_) => {
            return blocked_doctor_report(
                "daemon_unreachable",
                "Restore daemon connectivity, then repeat `focusa silent doctor`.",
            );
        }
    };
    let harness = client.get_probe("/v1/harnesses").await;
    let provider = client.get_probe("/v1/providers").await;
    let profiles = client.get_probe("/v1/silent-sessions/profiles").await;
    let presets = client.get_probe("/v1/silent-sessions/presets").await;
    let capabilities = client.get_probe("/v1/silent-sessions/capabilities").await;

    let capabilities_ok = capabilities.as_ref().is_ok_and(|(status, value)| {
        (200..300).contains(status)
            && value.get("ok").and_then(Value::as_bool) == Some(true)
            && value.get("canonical").and_then(Value::as_bool) == Some(true)
            && value.get("degraded").and_then(Value::as_bool) == Some(false)
    });
    let daemon_ok = health.get("status").and_then(Value::as_str) == Some("ok");
    let harness_ok = harness.as_ref().is_ok_and(|(status, value)| {
        (200..300).contains(status)
            && value
                .pointer("/data/harnesses/0/availability")
                .and_then(Value::as_str)
                == Some("available")
    });
    let provider_ok = provider.as_ref().is_ok_and(|(status, value)| {
        (200..300).contains(status)
            && value.get("ok").and_then(Value::as_bool) == Some(true)
            && value
                .pointer("/data/providers/0/catalog_status")
                .and_then(Value::as_str)
                == Some("ready")
    });
    let config_ok = profiles.as_ref().is_ok_and(|(status, value)| {
        (200..300).contains(status)
            && value
                .pointer("/data/profiles")
                .and_then(Value::as_array)
                .is_some_and(|items| !items.is_empty())
    }) && presets.as_ref().is_ok_and(|(status, value)| {
        (200..300).contains(status)
            && value
                .pointer("/data/presets")
                .and_then(Value::as_array)
                .is_some_and(|items| !items.is_empty())
    });

    let harness_failure = harness
        .as_ref()
        .ok()
        .and_then(|(_, value)| value.get("failure_class"))
        .and_then(Value::as_str)
        .unwrap_or("transport_degraded");
    let checks = vec![
        doctor_check(
            "daemon",
            daemon_ok,
            (!daemon_ok).then_some("daemon_unhealthy"),
            "Daemon health probe completed.",
            "Restore daemon health, then repeat `focusa silent doctor`.",
        ),
        doctor_check(
            "harness",
            harness_ok,
            (!harness_ok).then_some(harness_failure),
            "Harness catalog probe completed.",
            "Reconnect the harness transport, then repeat `focusa silent doctor`.",
        ),
        doctor_check(
            "provider",
            provider_ok,
            (!provider_ok).then_some("provider_unverified"),
            "Provider catalog probe completed.",
            "Verify provider auth and entitlement, then repeat `focusa silent doctor`.",
        ),
        doctor_check(
            "config",
            config_ok,
            (!config_ok).then_some("config_catalog_invalid"),
            "Profile and preset probes completed.",
            "Repair profile and preset catalogs, then repeat `focusa silent doctor`.",
        ),
        doctor_check(
            "capabilities",
            capabilities_ok,
            (!capabilities_ok).then_some("capability_catalog_unavailable"),
            "Silent Session capability probe completed.",
            "Restore the Silent Session capability catalog, then repeat `focusa silent doctor`.",
        ),
    ];
    let ready = daemon_ok && harness_ok && provider_ok && config_ok && capabilities_ok;
    json!({
        "schema": "focusa.silent_cli_doctor.v1",
        "status": if ready { "ready" } else { "blocked" },
        "ok": ready,
        "canonical": ready,
        "degraded": !ready,
        "failure_class": if ready { Value::Null } else { json!("doctor_readiness_blocked") },
        "retry": {
            "safe": ready,
            "posture": if ready { "idempotent_recheck" } else { "recover_then_recheck" }
        },
        "side_effects": [],
        "data": {"read_only": true, "checks": checks}
    })
}

async fn execute(client: &ApiClient, command: SilentCmd, json_output: bool) -> Result<()> {
    let (name, result) = match command {
        SilentCmd::Preflight(args) => ("preflight", client.post("/v1/silent-sessions/preflight", &config_body(&args)?).await?),
        SilentCmd::Create(args) => ("create", client.post("/v1/silent-sessions", &create_body(&args)?).await?),
        SilentCmd::Approval(ApprovalCmd::Preview(args)) => (
            "approval preview",
            client.post(
                &format!("/v1/silent-sessions/{}/approvals/preview", args.session_id),
                &read_json(&args.request_file)?,
            ).await?,
        ),
        SilentCmd::Approval(ApprovalCmd::Create(args)) => (
            "approval create",
            client.post(
                &format!("/v1/silent-sessions/{}/approvals", args.session_id),
                &read_json(&args.request_file)?,
            ).await?,
        ),
        SilentCmd::Start(args) => ("start", lifecycle_call(client, &args, "start").await?),
        SilentCmd::List(args) => {
            let query = query(&[
                ("project_root", args.project_root),
                ("continuity_id", args.continuity_id),
                ("status", args.status),
                ("limit", Some(args.limit.clamp(1, 200).to_string())),
            ]);
            ("list", client.get(&format!("/v1/silent-sessions{query}")).await?)
        }
        SilentCmd::Show(args) => {
            drop(args.run_id); // compatibility-only; canonical show is session-scoped.
            let result = client
                .get(&format!("/v1/silent-sessions/{}", args.session_id))
                .await?;
            ("show", validate_session_read(result, &args.session_id)?)
        }
        SilentCmd::Status(args) => {
            let result = client.get(&format!("/v1/silent-sessions/{}/status?run_id={}&generation={}", args.session_id, urlencoding::encode(&args.run_id), args.generation)).await?;
            ("status", validate_exact_read(result, &args.session_id, &args.run_id)?)
        }
        SilentCmd::Watch(args) => ("watch", watch_call(client, &args).await?),
        SilentCmd::Output(args) => {
            anyhow::ensure!((1..=1000).contains(&args.limit), "--limit must be between 1-1000");
            let mut path = format!("/v1/silent-sessions/{}/output?run_id={}&generation={}&follow=false&limit={}", args.session_id, urlencoding::encode(&args.run_id), args.generation, args.limit);
            if let Some(after) = &args.cursor { path.push_str("&after="); path.push_str(&urlencoding::encode(after)); }
            let result = client.get(&path).await?;
            ("output", validate_exact_read(result, &args.session_id, &args.run_id)?)
        }
        SilentCmd::Send(args) => {
            let body = interaction_body(&args.session_id, &args.actor_instance_ref, &args.approval_id, &args.idempotency_key, &args.lease_file, &args.payload_file, Some(&args.text), None)?;
            let path = format!("/v1/silent-sessions/{}/input?run_id={}&expected_generation={}", args.session_id, args.run_id, args.generation);
            let result = client.post(&path, &body).await?;
            validate_interaction_replay(&result)?;
            ("send", result)
        }
        SilentCmd::Steer(args) => {
            let body = interaction_body(&args.session_id, &args.actor_instance_ref, &args.approval_id, &args.idempotency_key, &args.lease_file, &args.payload_file, Some(&args.text), None)?;
            let path = format!("/v1/silent-sessions/{}/steer?run_id={}&expected_generation={}", args.session_id, args.run_id, args.generation);
            let result = client.post(&path, &body).await?;
            validate_interaction_replay(&result)?;
            ("steer", result)
        }
        SilentCmd::FollowUp(args) => {
            let body = interaction_body(&args.session_id, &args.actor_instance_ref, &args.approval_id, &args.idempotency_key, &args.lease_file, &args.payload_file, Some(&args.text), None)?;
            let path = format!("/v1/silent-sessions/{}/follow-up?run_id={}&expected_generation={}", args.session_id, args.run_id, args.generation);
            let result = client.post(&path, &body).await?;
            validate_interaction_replay(&result)?;
            ("follow-up", result)
        }
        SilentCmd::Key(args) => {
            let body = interaction_body(&args.session_id, &args.actor_instance_ref, &args.approval_id, &args.idempotency_key, &args.lease_file, &args.payload_file, None, Some(&args.keys))?;
            let path = format!("/v1/silent-sessions/{}/keys?run_id={}&expected_generation={}", args.session_id, args.run_id, args.generation);
            let result = client.post(&path, &body).await?;
            validate_interaction_replay(&result)?;
            ("key", result)
        },
        SilentCmd::Pause(args) => ("pause", lifecycle_call(client, &args, "pause").await?),
        SilentCmd::Resume(args) => ("resume", lifecycle_call(client, &args, "resume").await?),
        SilentCmd::Interrupt(args) => ("interrupt", lifecycle_call(client, &args, "interrupt").await?),
        SilentCmd::Cancel(args) => ("cancel", lifecycle_call(client, &args, "cancel").await?),
        SilentCmd::Restart(args) => ("restart", lifecycle_call(client, &args, "restart").await?),
        SilentCmd::Adopt(args) => ("adopt", lifecycle_call(client, &args, "adopt").await?),
        SilentCmd::Config(command) => match command {
            ConfigCmd::Resolve(args) => ("config resolve", client.post("/v1/silent-sessions/config/resolve", &config_body(&args)?).await?),
            ConfigCmd::Diff(args) => ("config diff", client.post(&format!("/v1/silent-sessions/{}/config/preview", args.session_id), &config_session_body(&args)?).await?),
            ConfigCmd::Apply(args) => ("config apply", client.post(&format!("/v1/silent-sessions/{}/config/revisions", args.session_id), &config_apply_body(&args)?).await?),
            ConfigCmd::Rollback(args) => ("config rollback", client.post(&format!("/v1/silent-sessions/{}/config/rollback", args.session_id), &json!({"run_id":args.run_id,"generation":args.generation,"approval_id":args.approval_id,"target_revision_id":args.revision,"idempotency_key":args.idempotency_key})).await?),
        },
        SilentCmd::Profile(ProfileCmd::List) => ("profile list", client.get("/v1/silent-sessions/profiles").await?),
        SilentCmd::Preset(PresetCmd::List) => ("preset list", client.get("/v1/silent-sessions/presets").await?),
        SilentCmd::Model(ModelCmd::List(args)) => (
            "model list",
            client
                .get(&format!(
                    "/v1/providers/{}/models",
                    urlencoding::encode(&args.provider)
                ))
                .await?,
        ),
        SilentCmd::Model(ModelCmd::Preflight(args)) => (
            "model preflight",
            client
                .post(
                    &format!(
                        "/v1/providers/{}/models/preflight",
                        urlencoding::encode(&args.provider)
                    ),
                    &json!({
                        "model": args.model,
                        "thinking": args.thinking,
                        "strict": args.strict,
                        "require_entitlement_preflight": args.require_entitlement_preflight,
                    }),
                )
                .await?,
        ),
        SilentCmd::Checkpoints(args) => ("checkpoints", client.get(&format!("/v1/silent-sessions/{}/checkpoints?run_id={}&generation={}", args.session_id, args.run_id, args.generation)).await?),
        SilentCmd::Evidence(args) => ("evidence", proof_call(client, &args, "artifacts").await?),
        SilentCmd::Receipt(args) => ("receipt", proof_call(client, &args, "receipts").await?),
        SilentCmd::Export(args) => ("export", export_call(client, &args).await?),
        SilentCmd::Hold(args) => ("hold", retention_call(client, &args, "hold").await?),
        SilentCmd::Delete(args) => ("delete", retention_call(client, &args, "delete").await?),
        SilentCmd::Purge(args) => ("purge", retention_call(client, &args, "purge").await?),
        SilentCmd::Doctor(_args) => ("doctor", silent_doctor_report(client).await),
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
