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
    #[arg(long)]
    host_id: Option<String>,
    #[arg(long)]
    operator_id: Option<String>,
    #[arg(long)]
    workpoint_id: Option<String>,
    #[arg(long)]
    item_id: Option<String>,
    #[arg(long)]
    task_id: Option<String>,
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
        idempotency_key: String,
        #[arg(long)]
        phase: String,
        #[arg(long)]
        target_state: String,
        #[arg(long)]
        scope_revision: String,
        #[arg(long)]
        expires_at: String,
        #[arg(long)]
        estimator_version: String,
        #[arg(long)]
        cohort: String,
        #[arg(long = "evidence-basis")]
        evidence_basis: Vec<String>,
        #[arg(long)]
        comparable_sample_count: usize,
        #[arg(long)]
        all_attempt_sample_count: usize,
        #[arg(long)]
        censoring_method: String,
        #[arg(long)]
        correlation_method: String,
        #[arg(long)]
        calibration_profile: String,
        #[arg(long)]
        baseline_ref: String,
        #[arg(long)]
        drift_policy_ref: String,
        #[arg(long)]
        actual_ms: Option<u64>,
        #[arg(long)]
        evaluation_packet: Option<std::path::PathBuf>,
    },
    /// Commit a signed HumanCalendarContext, TemporalPriorityFrame, and execution guard.
    CommitPriority {
        #[command(flatten)]
        scope: TemporalScopeArgs,
        #[arg(long)]
        packet: std::path::PathBuf,
    },
    /// Resolve and persist a versioned civil-time intent, including DST fold/gap policy.
    ResolveCivilTime {
        #[command(flatten)]
        scope: TemporalScopeArgs,
        #[arg(long)]
        packet: std::path::PathBuf,
    },
    /// Capture and persist a signed platform clock capability sample.
    CaptureClock {
        #[command(flatten)]
        scope: TemporalScopeArgs,
        #[arg(long)]
        timezone: String,
        #[arg(long)]
        tzdb_version: Option<String>,
        #[arg(long)]
        idempotency_key: String,
    },
    /// Validate a complete high-consequence temporal control packet before dispatch.
    HighConsequencePreflight {
        #[command(flatten)]
        scope: TemporalScopeArgs,
        #[arg(long)]
        packet: std::path::PathBuf,
    },
    /// Append a signed attestation for legacy unsigned temporal events without rewriting history.
    MigrateSignatures {
        #[command(flatten)]
        scope: TemporalScopeArgs,
        #[arg(long)]
        idempotency_key: String,
        #[arg(long)]
        confirm: bool,
    },
    /// Settle completion, missed-target, lost-time, receipt, and learning evidence.
    SettleClosure {
        #[command(flatten)]
        scope: TemporalScopeArgs,
        #[arg(long)]
        packet: std::path::PathBuf,
    },
    /// Check temporal policy without inventing a deadline.
    Preflight {
        #[command(flatten)]
        scope: TemporalScopeArgs,
    },
}

fn scope_body(scope: &TemporalScopeArgs) -> Value {
    json!({
        "project_root":scope.project_root,"continuity_id":scope.continuity_id,
        "host_id":scope.host_id,"operator_id":scope.operator_id,
        "workpoint_id":scope.workpoint_id,"item_id":scope.item_id,"task_id":scope.task_id
    })
}

fn claim_body(args: &TemporalClaimArgs) -> Value {
    let now = Utc::now().to_rfc3339();
    json!({
        "project_root":args.scope.project_root,
        "continuity_id":args.scope.continuity_id,
        "host_id":args.scope.host_id,"operator_id":args.scope.operator_id,
        "workpoint_id":args.scope.workpoint_id,"item_id":args.scope.item_id,"task_id":args.scope.task_id,
        "idempotency_key":args.idempotency_key,
        "confirm":args.confirm,
        "claim":{
            "claim_id":args.claim_id,"revision":1,
            "scope":{
                "project_root":args.scope.project_root,"continuity_id":args.scope.continuity_id,
                "host_id":args.scope.host_id,"operator_id":args.scope.operator_id,
                "workpoint_id":args.scope.workpoint_id,"item_id":args.scope.item_id,"task_id":args.scope.task_id
            },
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
    for (key, value) in [
        ("host_id", scope.host_id.as_deref()),
        ("operator_id", scope.operator_id.as_deref()),
        ("workpoint_id", scope.workpoint_id.as_deref()),
        ("item_id", scope.item_id.as_deref()),
        ("task_id", scope.task_id.as_deref()),
    ] {
        if let Some(value) = value {
            query.push_str(&format!("&{}={}", key, urlencoding::encode(value)));
        }
    }
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
            idempotency_key,
            phase,
            target_state,
            scope_revision,
            expires_at,
            estimator_version,
            cohort,
            evidence_basis,
            comparable_sample_count,
            all_attempt_sample_count,
            censoring_method,
            correlation_method,
            calibration_profile,
            baseline_ref,
            drift_policy_ref,
            actual_ms,
            evaluation_packet,
        } => {
            ensure_project_root_scope_safe(Some(&scope.project_root), "temporal forecast")?;
            let mut body = scope_body(&scope);
            body["idempotency_key"] = json!(idempotency_key);
            body["phase"] = json!(phase);
            body["authority"] = json!({
                "claim_kind":"forecast", "target_state":target_state,
                "scope_revision":scope_revision, "expires_at":expires_at,
                "estimator_version":estimator_version, "cohort":cohort,
                "evidence_basis":evidence_basis,
                "comparable_sample_count":comparable_sample_count,
                "all_attempt_sample_count":all_attempt_sample_count,
                "censoring_method":censoring_method,"correlation_method":correlation_method,
                "calibration_profile":calibration_profile,"grounding_status":"grounded",
                "baseline_ref":baseline_ref,"drift_policy_ref":drift_policy_ref
            });
            body["actual_ms"] = json!(actual_ms);
            if let Some(packet) = evaluation_packet {
                body["evaluation"] = serde_json::from_str(&std::fs::read_to_string(packet)?)?;
            }
            (
                "temporal forecast",
                api.post("/v1/temporal/forecast", &body).await?,
            )
        }
        TemporalCmd::CommitPriority { scope, packet } => {
            ensure_project_root_scope_safe(Some(&scope.project_root), "temporal commit-priority")?;
            let mut body: Value = serde_json::from_str(&std::fs::read_to_string(packet)?)?;
            let scoped = scope_body(&scope);
            let object = body
                .as_object_mut()
                .ok_or_else(|| anyhow::anyhow!("temporal priority packet must be a JSON object"))?;
            for (key, value) in scoped.as_object().expect("scope body object") {
                object.insert(key.clone(), value.clone());
            }
            (
                "temporal commit-priority",
                api.post("/v1/temporal/priority/commit", &body).await?,
            )
        }
        TemporalCmd::ResolveCivilTime { scope, packet } => {
            ensure_project_root_scope_safe(
                Some(&scope.project_root),
                "temporal resolve-civil-time",
            )?;
            let mut body: Value = serde_json::from_str(&std::fs::read_to_string(packet)?)?;
            let scoped = scope_body(&scope);
            let object = body
                .as_object_mut()
                .ok_or_else(|| anyhow::anyhow!("civil-time packet must be a JSON object"))?;
            for (key, value) in scoped.as_object().expect("scope body object") {
                object.insert(key.clone(), value.clone());
            }
            (
                "temporal resolve-civil-time",
                api.post("/v1/temporal/civil/resolve", &body).await?,
            )
        }
        TemporalCmd::CaptureClock {
            scope,
            timezone,
            tzdb_version,
            idempotency_key,
        } => {
            ensure_project_root_scope_safe(Some(&scope.project_root), "temporal capture-clock")?;
            let mut body = scope_body(&scope);
            body["timezone"] = json!(timezone);
            body["tzdb_version"] = json!(tzdb_version);
            body["idempotency_key"] = json!(idempotency_key);
            (
                "temporal capture-clock",
                api.post("/v1/temporal/clock/capture", &body).await?,
            )
        }
        TemporalCmd::HighConsequencePreflight { scope, packet } => {
            ensure_project_root_scope_safe(
                Some(&scope.project_root),
                "temporal high-consequence-preflight",
            )?;
            let mut body: Value = serde_json::from_str(&std::fs::read_to_string(packet)?)?;
            let scoped = scope_body(&scope);
            let object = body
                .as_object_mut()
                .ok_or_else(|| anyhow::anyhow!("high-consequence packet must be a JSON object"))?;
            for (key, value) in scoped.as_object().expect("scope body object") {
                object.insert(key.clone(), value.clone());
            }
            (
                "temporal high-consequence-preflight",
                api.post("/v1/temporal/high-consequence/preflight", &body)
                    .await?,
            )
        }
        TemporalCmd::MigrateSignatures {
            scope,
            idempotency_key,
            confirm,
        } => {
            ensure_project_root_scope_safe(
                Some(&scope.project_root),
                "temporal migrate-signatures",
            )?;
            let mut body = scope_body(&scope);
            body["idempotency_key"] = json!(idempotency_key);
            body["confirm"] = json!(confirm);
            (
                "temporal migrate-signatures",
                api.post("/v1/temporal/migrate-signatures", &body).await?,
            )
        }
        TemporalCmd::SettleClosure { scope, packet } => {
            ensure_project_root_scope_safe(Some(&scope.project_root), "temporal settle-closure")?;
            let mut body: Value = serde_json::from_str(&std::fs::read_to_string(packet)?)?;
            let scoped = scope_body(&scope);
            let object = body.as_object_mut().ok_or_else(|| {
                anyhow::anyhow!("closure settlement packet must be a JSON object")
            })?;
            for (key, value) in scoped.as_object().expect("scope body object") {
                object.insert(key.clone(), value.clone());
            }
            (
                "temporal settle-closure",
                api.post("/v1/temporal/settle-closure", &body).await?,
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
