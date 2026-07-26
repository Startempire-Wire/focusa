//! Spec137 temporal authority CLI.

use crate::api_client::ApiClient;
use crate::commands::scope::ensure_project_root_scope_safe;
use chrono::Utc;
use clap::{Args, Subcommand};
use serde_json::{Value, json};

#[derive(Args, Clone)]
pub struct TemporalScopeArgs {
    #[arg(long)]
    project_root: String,
    #[arg(long)]
    continuity_id: String,
}

#[derive(Args, Clone)]
pub struct TemporalClaimArgs {
    #[command(flatten)]
    scope: TemporalScopeArgs,
    #[arg(long)]
    idempotency_key: String,
    #[arg(long)]
    claim_id: String,
    #[arg(long)]
    kind: String,
    #[arg(long)]
    subject_ref: String,
    #[arg(long)]
    target_at: Option<String>,
    #[arg(long)]
    duration_ms: Option<u64>,
    #[arg(long)]
    timezone: String,
    #[arg(long)]
    source: String,
    #[arg(long)]
    source_ref: Option<String>,
    #[arg(long)]
    operator_confirmed: bool,
    #[arg(long, default_value = "medium")]
    confidence: String,
    #[arg(long = "evidence-ref")]
    evidence_refs: Vec<String>,
    #[arg(long)]
    reason_code: Option<String>,
    #[arg(long)]
    confirm: bool,
}

#[derive(Subcommand)]
pub enum TemporalCmd {
    /// Read truthful deadline/forecast/urgency state.
    Status {
        #[command(flatten)]
        scope: TemporalScopeArgs,
        #[arg(long)]
        as_of: Option<String>,
    },
    /// Commit an authority-backed temporal claim.
    Commit {
        #[command(flatten)]
        args: TemporalClaimArgs,
    },
    /// Revise and supersede an existing claim.
    Revise {
        #[command(flatten)]
        args: TemporalClaimArgs,
    },
    /// Record one observed release-phase duration.
    Observe {
        #[command(flatten)]
        scope: TemporalScopeArgs,
        #[arg(long)]
        idempotency_key: String,
        #[arg(long)]
        phase: String,
        #[arg(long)]
        duration_ms: u64,
        #[arg(long, default_value = "success")]
        outcome: String,
        #[arg(long)]
        reason_code: Option<String>,
        #[arg(long = "evidence-ref")]
        evidence_refs: Vec<String>,
    },
    /// Compute an empirical forecast range for one phase.
    Forecast {
        #[command(flatten)]
        scope: TemporalScopeArgs,
        #[arg(long)]
        phase: String,
        #[arg(long)]
        actual_ms: Option<u64>,
    },
    /// Check temporal policy without inventing a deadline.
    Preflight {
        #[command(flatten)]
        scope: TemporalScopeArgs,
    },
}

fn scope_body(scope: &TemporalScopeArgs) -> Value {
    json!({"project_root":scope.project_root,"continuity_id":scope.continuity_id})
}

fn claim_body(args: &TemporalClaimArgs) -> Value {
    let now = Utc::now().to_rfc3339();
    json!({
        "project_root":args.scope.project_root,
        "continuity_id":args.scope.continuity_id,
        "idempotency_key":args.idempotency_key,
        "confirm":args.confirm,
        "claim":{
            "claim_id":args.claim_id,"revision":1,
            "scope":{"project_root":args.scope.project_root,"continuity_id":args.scope.continuity_id},
            "kind":args.kind,"status":"canonical","subject_ref":args.subject_ref,
            "target_at":args.target_at,"duration_ms":args.duration_ms,"timezone":args.timezone,
            "source":args.source,"source_ref":args.source_ref,
            "operator_confirmed":args.operator_confirmed,"confidence":args.confidence,
            "uncertainty":null,"observed_at":now,"effective_at":now,"expires_at":null,
            "supersedes_revision":null,"evidence_refs":args.evidence_refs,"reason_code":args.reason_code
        }
    })
}

fn query(scope: &TemporalScopeArgs, extra: Option<(&str, &str)>) -> String {
    let mut query = format!(
        "project_root={}&continuity_id={}",
        urlencoding::encode(&scope.project_root),
        urlencoding::encode(&scope.continuity_id)
    );
    if let Some((key, value)) = extra {
        query.push_str(&format!("&{}={}", key, urlencoding::encode(value)));
    }
    query
}

pub async fn run(cmd: TemporalCmd, json_output: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let (label, response) = match cmd {
        TemporalCmd::Status { scope, as_of } => {
            ensure_project_root_scope_safe(Some(&scope.project_root), "temporal status")?;
            let path = format!(
                "/v1/temporal/status?{}",
                query(&scope, as_of.as_deref().map(|value| ("as_of", value)))
            );
            ("temporal status", api.get(&path).await?)
        }
        TemporalCmd::Commit { args } => {
            ensure_project_root_scope_safe(Some(&args.scope.project_root), "temporal commit")?;
            (
                "temporal commit",
                api.post("/v1/temporal/commit", &claim_body(&args)).await?,
            )
        }
        TemporalCmd::Revise { args } => {
            ensure_project_root_scope_safe(Some(&args.scope.project_root), "temporal revise")?;
            (
                "temporal revise",
                api.post("/v1/temporal/revise", &claim_body(&args)).await?,
            )
        }
        TemporalCmd::Observe {
            scope,
            idempotency_key,
            phase,
            duration_ms,
            outcome,
            reason_code,
            evidence_refs,
        } => {
            ensure_project_root_scope_safe(Some(&scope.project_root), "temporal observe")?;
            let mut body = scope_body(&scope);
            body["idempotency_key"] = json!(idempotency_key);
            body["phase"] = json!(phase);
            body["duration_ms"] = json!(duration_ms);
            body["outcome"] = json!(outcome);
            body["reason_code"] = json!(reason_code);
            body["evidence_refs"] = json!(evidence_refs);
            (
                "temporal observe",
                api.post("/v1/temporal/observe", &body).await?,
            )
        }
        TemporalCmd::Forecast {
            scope,
            phase,
            actual_ms,
        } => {
            ensure_project_root_scope_safe(Some(&scope.project_root), "temporal forecast")?;
            let mut body = scope_body(&scope);
            body["phase"] = json!(phase);
            body["actual_ms"] = json!(actual_ms);
            (
                "temporal forecast",
                api.post("/v1/temporal/forecast", &body).await?,
            )
        }
        TemporalCmd::Preflight { scope } => {
            ensure_project_root_scope_safe(Some(&scope.project_root), "temporal preflight")?;
            (
                "temporal preflight",
                api.post("/v1/temporal/preflight", &scope_body(&scope))
                    .await?,
            )
        }
    };
    if json_output {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        println!(
            "{label}: {}",
            response
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        );
        if let Some(next) = response.get("next_action").and_then(Value::as_str) {
            println!("  next: {next}");
        }
    }
    Ok(())
}
