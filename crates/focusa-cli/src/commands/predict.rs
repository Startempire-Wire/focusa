use crate::api_client::ApiClient;
use crate::commands::scope_resolver::resolve_project_scope;
use anyhow::{Context, anyhow};
use clap::{Args, Subcommand};
use focusa_core::{
    scoped_state::{ScopeRef, WorkstreamKey},
    spec138_operations::spec138_operation,
};
use serde_json::{Value, json};
use std::path::Path;

#[derive(Args, Debug, Clone)]
pub struct PredictionScopeArgs {
    /// Explicit verified project root. Otherwise use the CLI project-scope resolver.
    #[arg(long)]
    project_root: Option<String>,
    /// Explicit project alias for the CLI project-scope resolver.
    #[arg(long)]
    project_alias: Option<String>,
    /// Required workstream continuity id; may also come from FOCUSA_CONTINUITY_ID.
    #[arg(long, env = "FOCUSA_CONTINUITY_ID")]
    continuity_id: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum PredictCmd {
    /// Record a bounded prediction.
    Record {
        #[command(flatten)]
        scope: PredictionScopeArgs,
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
        #[arg(long)]
        ontology_context: Option<String>,
    },
    /// Evaluate a prediction by id.
    Evaluate {
        #[command(flatten)]
        scope: PredictionScopeArgs,
        prediction_id: String,
        #[arg(long)]
        actual_outcome: String,
        #[arg(long)]
        score: Option<f64>,
        #[arg(long)]
        learning_signal_ref: Option<String>,
    },
    /// Auto-capture an outcome across recent matching unevaluated predictions.
    CaptureOutcome {
        #[command(flatten)]
        scope: PredictionScopeArgs,
        #[arg(long)]
        actual_outcome: String,
        #[arg(long)]
        prediction_type: Option<String>,
        #[arg(long, value_delimiter = ',')]
        context_refs: Vec<String>,
        #[arg(long)]
        ontology_context: Option<String>,
        #[arg(long)]
        score: Option<f64>,
        #[arg(long)]
        learning_signal_ref: Option<String>,
        #[arg(long)]
        limit: Option<u32>,
    },
    /// Recent predictions from one typed project/workstream scope.
    Recent {
        #[command(flatten)]
        scope: PredictionScopeArgs,
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
    /// Accuracy/calibration stats for one typed project/workstream scope.
    Stats {
        #[command(flatten)]
        scope: PredictionScopeArgs,
    },
    /// Invoke one canonical Spec 138/138A operation by exact operation id.
    Operation {
        #[command(flatten)]
        scope: PredictionScopeArgs,
        #[arg(long)]
        operation: String,
        /// Value for canonical {id} path segments.
        #[arg(long)]
        id: Option<String>,
        /// ScopedAuthorityEvent JSON; required for mutation operations.
        #[arg(long)]
        event_json: Option<String>,
    },
    /// Append one immutable Spec 138 authority event from JSON.
    AuthorityAppend {
        #[command(flatten)]
        scope: PredictionScopeArgs,
        #[arg(long)]
        event_json: String,
    },
    /// Read the durable Spec 138 authority projection.
    AuthorityProjection {
        #[command(flatten)]
        scope: PredictionScopeArgs,
    },
}

fn parse_ontology_context(raw: Option<String>) -> anyhow::Result<Value> {
    match raw {
        Some(s) if !s.trim().is_empty() => Ok(serde_json::from_str(&s)?),
        _ => Ok(Value::Null),
    }
}

fn resolve_prediction_scope(args: &PredictionScopeArgs) -> anyhow::Result<WorkstreamKey> {
    let cwd = std::env::current_dir()
        .context("prediction scope requires a readable current directory")?
        .to_string_lossy()
        .to_string();
    let resolved = resolve_project_scope(
        args.project_root.as_deref(),
        args.project_alias.as_deref(),
        Some(&cwd),
    )?;
    let continuity_id = args
        .continuity_id
        .clone()
        .or(resolved.continuity_id)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            anyhow!("typed prediction scope requires --continuity-id or FOCUSA_CONTINUITY_ID")
        })?;
    let fingerprint = resolved
        .fingerprint
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow!("resolved prediction scope lacks project fingerprint"))?;
    let canonical_name = Path::new(&resolved.project_root)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("project")
        .to_string();
    let scope_id = resolved
        .project_id
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("project:{}", fingerprint));
    let root_scope =
        ScopeRef::project(scope_id, resolved.project_root, canonical_name, fingerprint)?;
    Ok(WorkstreamKey::new(root_scope, continuity_id)?)
}

fn scoped_query(scope: &WorkstreamKey, limit: Option<u32>) -> String {
    let root = &scope.root_scope;
    let mut fields = vec![
        (
            "scope_kind",
            match root.scope_kind {
                focusa_core::scoped_state::ScopeKind::Project => "project".to_string(),
                focusa_core::scoped_state::ScopeKind::Host => "host".to_string(),
            },
        ),
        ("scope_id", root.scope_id.clone()),
        ("root_path", root.root_path.to_string_lossy().to_string()),
        ("canonical_name", root.canonical_name.clone()),
        ("fingerprint", root.fingerprint.clone()),
        ("continuity_id", scope.continuity_id.clone()),
    ];
    if let Some(limit) = limit {
        fields.push(("limit", limit.to_string()));
    }
    fields
        .into_iter()
        .map(|(key, value)| format!("{key}={}", urlencoding::encode(&value)))
        .collect::<Vec<_>>()
        .join("&")
}

pub async fn run(cmd: PredictCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let resp = match cmd {
        PredictCmd::Record {
            scope,
            prediction_type,
            predicted_outcome,
            confidence,
            recommended_action,
            why,
            context_refs,
            ontology_context,
        } => {
            let scope = resolve_prediction_scope(&scope)?;
            let ontology_context = parse_ontology_context(ontology_context)?;
            api.post(
                "/v1/predictions",
                &json!({
                    "scope": scope,
                    "prediction_type": prediction_type,
                    "context_refs": context_refs,
                    "ontology_context": ontology_context,
                    "predicted_outcome": predicted_outcome,
                    "confidence": confidence,
                    "recommended_action": recommended_action,
                    "why": why,
                }),
            )
            .await?
        }
        PredictCmd::Evaluate {
            scope,
            prediction_id,
            actual_outcome,
            score,
            learning_signal_ref,
        } => {
            let scope = resolve_prediction_scope(&scope)?;
            api.post(
                &format!("/v1/predictions/{prediction_id}/evaluate"),
                &json!({
                    "scope": scope,
                    "actual_outcome": actual_outcome,
                    "score": score,
                    "learning_signal_ref": learning_signal_ref,
                }),
            )
            .await?
        }
        PredictCmd::CaptureOutcome {
            scope,
            actual_outcome,
            prediction_type,
            context_refs,
            ontology_context,
            score,
            learning_signal_ref,
            limit,
        } => {
            let scope = resolve_prediction_scope(&scope)?;
            api.post(
                "/v1/predictions/capture-outcome",
                &json!({
                    "scope": scope,
                    "actual_outcome": actual_outcome,
                    "prediction_type": prediction_type,
                    "context_refs": context_refs,
                    "ontology_context": parse_ontology_context(ontology_context)?,
                    "score": score,
                    "learning_signal_ref": learning_signal_ref,
                    "limit": limit,
                }),
            )
            .await?
        }
        PredictCmd::Recent { scope, limit } => {
            let scope = resolve_prediction_scope(&scope)?;
            api.get(&format!(
                "/v1/predictions/recent?{}",
                scoped_query(&scope, Some(limit))
            ))
            .await?
        }
        PredictCmd::Stats { scope } => {
            let scope = resolve_prediction_scope(&scope)?;
            api.get(&format!(
                "/v1/predictions/stats?{}",
                scoped_query(&scope, None)
            ))
            .await?
        }
        PredictCmd::Operation {
            scope,
            operation,
            id,
            event_json,
        } => {
            let scope = resolve_prediction_scope(&scope)?;
            let descriptor = spec138_operation(&operation)
                .ok_or_else(|| anyhow!("unknown canonical Spec 138 operation: {operation}"))?;
            let path = if descriptor.path.contains("{id}") {
                let id = id
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| anyhow!("{} requires --id", descriptor.operation_id))?;
                descriptor.path.replace("{id}", &urlencoding::encode(id))
            } else {
                descriptor.path.to_string()
            };
            if descriptor.method == "GET" {
                api.get(&format!("{path}?{}", scoped_query(&scope, None)))
                    .await?
            } else {
                let raw = event_json
                    .as_deref()
                    .ok_or_else(|| anyhow!("{} requires --event-json", descriptor.operation_id))?;
                let event: Value = serde_json::from_str(raw)
                    .context("--event-json must be a typed Spec 138 ScopedAuthorityEvent")?;
                api.post(
                    &path,
                    &json!({
                        "operation_id":descriptor.operation_id, "scope":scope, "event":event
                    }),
                )
                .await?
            }
        }
        PredictCmd::AuthorityAppend { scope, event_json } => {
            let scope = resolve_prediction_scope(&scope)?;
            let event: Value = serde_json::from_str(&event_json)
                .context("--event-json must be a Spec 138 ScopedAuthorityEvent")?;
            api.post(
                "/v1/prediction-authority/events",
                &json!({"scope":scope,"event":event}),
            )
            .await?
        }
        PredictCmd::AuthorityProjection { scope } => {
            let scope = resolve_prediction_scope(&scope)?;
            api.post(
                "/v1/prediction-authority/projection",
                &json!({"scope":scope}),
            )
            .await?
        }
    };
    if json_mode {
        println!("{}", serde_json::to_string_pretty(&resp)?);
    } else {
        let data = resp.get("data").unwrap_or(&resp);
        println!(
            "Status: {}",
            resp.pointer("/authority/status")
                .and_then(Value::as_str)
                .unwrap_or("completed")
        );
        println!(
            "Summary: {}",
            resp.pointer("/human/summary")
                .and_then(Value::as_str)
                .unwrap_or("typed scoped prediction command completed")
        );
        if let Some(record_id) = data.pointer("/record/record_id").and_then(Value::as_str) {
            println!("Prediction record: {record_id}");
        }
        println!("Next action: evaluate predictions after outcome is known");
        println!("Why: predictions remain scoped and never override operator steering");
        println!("Command: focusa predict stats --continuity-id <id>");
        println!("Recovery: focusa predict recent --continuity-id <id> --json");
        println!("Evidence: /v1/predictions/stats");
        println!("Docs: docs/current/PREDICTIVE_POWER_GUIDE.md");
    }
    Ok(())
}
