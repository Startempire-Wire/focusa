//! Rust-authoritative compaction policy resolution.

use crate::{scope::ScopeContext, server::AppState};
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use chrono::Utc;
use focusa_core::compaction_policy::{
    CompactionPolicyLease, CompactionRuntimeFacts, PolicyMode, PolicySelectionContext,
    compile_policy_lattice, legal_action_mask, resolve_policy, resolve_runtime_fingerprint,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct ResolveRequest {
    schema: String,
    runtime_facts: CompactionRuntimeFacts,
    #[serde(default = "default_mode")]
    mode: PolicyMode,
    objective_profile: Option<String>,
    predicted_safe_tokens: Option<u64>,
    sample_size: Option<u64>,
    confidence: Option<f64>,
}

fn default_mode() -> PolicyMode {
    PolicyMode::Shadow
}

async fn resolve(
    State(_state): State<Arc<AppState>>,
    scope: ScopeContext,
    Json(mut req): Json<ResolveRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if req.schema != "focusa.compaction_policy_resolve_request.v1" {
        return Err(blocked(StatusCode::BAD_REQUEST, "invalid_resolve_schema"));
    }
    let workstream = scope
        .require_workstream_key()
        .map_err(|error| blocked(StatusCode::UNPROCESSABLE_ENTITY, &error))?;
    req.runtime_facts.project_root = Some(
        workstream
            .root_scope
            .root_path
            .to_string_lossy()
            .to_string(),
    );
    req.runtime_facts.continuity_id = Some(workstream.continuity_id);
    let fingerprint = resolve_runtime_fingerprint(req.runtime_facts);
    // Capability evidence is daemon-registry authority, never caller input.
    // Until a matching proven registry generation exists, native provider
    // strategies remain masked and the exact legacy baseline is selected.
    let legal_actions = legal_action_mask(&fingerprint, &[], Utc::now());
    let context_window = fingerprint.context_window.unwrap_or(256_000);
    let candidates = compile_policy_lattice(
        context_window,
        &legal_actions,
        req.objective_profile.as_deref().unwrap_or("daily_driver"),
        req.predicted_safe_tokens,
    );
    let selection_context = PolicySelectionContext {
        mode: req.mode,
        context_window,
        sample_size: req.sample_size.unwrap_or(0),
        measured_confidence: req.confidence,
        minimum_samples: 20,
        required_confidence: 0.95,
        // Canary authority requires a persisted operator enrollment receipt;
        // a caller cannot self-enroll through resolve.
        dev_fleet_enrolled: false,
    };
    let resolution = resolve_policy(&selection_context, &candidates);
    let lease = CompactionPolicyLease::freeze(
        &resolution,
        &fingerprint.segment_key,
        &fingerprint.capability_evidence_revision,
        "sha256:runtime-facts-v1",
    );
    Ok(Json(json!({
        "schema": "focusa.compaction_policy_resolve_result.v1",
        "status": "resolved",
        "runtime_fingerprint": fingerprint,
        "legal_actions": legal_actions,
        "resolution": resolution,
        "lease": lease,
        "candidate_count": candidates.len(),
        "capability_posture": "fallback_until_daemon_registry_proof"
    })))
}

fn blocked(status: StatusCode, error: &str) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({
            "schema":"focusa.compaction_policy_resolve_result.v1",
            "status":"blocked",
            "error":error.chars().take(240).collect::<String>()
        })),
    )
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/compaction/policy/resolve", post(resolve))
}
