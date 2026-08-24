use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use focusa_core::silent_sessions::{
    ConfigLayer, RunGeneration, SilentSessionAction, SilentSessionConfig, SilentSessionId,
    SilentSessionRunId, load_config_revision, load_run, load_session, preview_config_revision,
    resolve_silent_session_config,
};
use serde::Deserialize;
use serde_json::json;

use crate::{middleware::principal::ApiRequestPrincipal, server::AppState};

use super::{
    silent_sessions::{
        ApiResponse, disclose_principal_side_effect, durable_request_principal, failure,
        persistence_failure,
    },
    silent_sessions_authorize::authorize_mutation,
    silent_sessions_contract::{
        ExactSessionRunTarget, SilentSessionApiEnvelope, guard_exact_target,
    },
};

#[derive(Debug, Deserialize)]
pub(super) struct ResolveBody {
    requested_config: SilentSessionConfig,
    #[serde(default)]
    layers: Vec<ConfigLayer>,
}

#[derive(Debug, Deserialize)]
pub(super) struct PreviewBody {
    run_id: SilentSessionRunId,
    generation: RunGeneration,
    requested_config: SilentSessionConfig,
    #[serde(default)]
    layers: Vec<ConfigLayer>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/silent-sessions/profiles", get(profiles))
        .route("/v1/silent-sessions/presets", get(presets))
        .route("/v1/silent-sessions/config/resolve", post(resolve))
        .route(
            "/v1/silent-sessions/{session_id}/config/preview",
            post(preview),
        )
}

async fn profiles(State(state): State<Arc<AppState>>, headers: HeaderMap) -> ApiResponse {
    authenticated_catalog(
        &state,
        &headers,
        "profiles",
        json!({
            "profiles": [
                {
                    "profile_id": "local_pi_isolated",
                    "role_id": "operator",
                    "description": "Legacy alias for the bounded operator profile.",
                    "capability_refs": ["focusa_work_loop_status"],
                    "approval": {"status": "operator_approval_required", "grants_permissions": false},
                    "persistent_defaults": {"harness": {"kind": "pi"}, "workspace": {"mode": "isolated_worktree"}, "governance": {"destructive_actions_allowed": false}}
                },
                {
                    "profile_id": "workforce_planner",
                    "role_id": "planner",
                    "description": "Plans bounded work and maintains scoped continuity.",
                    "capability_refs": ["focusa_trajectory_assess", "focusa_trajectory_propose_workpoint", "focusa_workpoint_checkpoint"],
                    "approval": {"status": "operator_approval_required", "grants_permissions": false},
                    "persistent_defaults": {"harness": {"kind": "pi"}, "workspace": {"mode": "isolated_worktree"}, "governance": {"destructive_actions_allowed": false}}
                },
                {
                    "profile_id": "workforce_researcher",
                    "role_id": "researcher",
                    "description": "Collects bounded evidence and records source-linked findings.",
                    "capability_refs": ["focusa_context_cognition", "focusa_context_cognition_curate", "focusa_evidence_capture"],
                    "approval": {"status": "operator_approval_required", "grants_permissions": false},
                    "persistent_defaults": {"harness": {"kind": "pi"}, "workspace": {"mode": "isolated_worktree"}, "governance": {"destructive_actions_allowed": false}}
                },
                {
                    "profile_id": "workforce_builder",
                    "role_id": "builder",
                    "description": "Executes approved implementation work through bounded sessions.",
                    "capability_refs": ["focusa_silent_sessions", "focusa_call_stack_design", "focusa_work_loop_status"],
                    "approval": {"status": "operator_approval_required", "grants_permissions": false},
                    "persistent_defaults": {"harness": {"kind": "pi"}, "workspace": {"mode": "isolated_worktree"}, "governance": {"destructive_actions_allowed": false}}
                },
                {
                    "profile_id": "workforce_reviewer",
                    "role_id": "reviewer",
                    "description": "Checks evidence, contracts, and implementation drift.",
                    "capability_refs": ["focusa_call_stack_verify", "focusa_metacog_doctor", "focusa_workpoint_resume"],
                    "approval": {"status": "operator_approval_required", "grants_permissions": false},
                    "persistent_defaults": {"harness": {"kind": "pi"}, "workspace": {"mode": "isolated_worktree"}, "governance": {"destructive_actions_allowed": false}}
                },
                {
                    "profile_id": "workforce_operator",
                    "role_id": "operator",
                    "description": "Coordinates approved workers and observes durable completion state.",
                    "capability_refs": ["focusa_bg_status", "focusa_work_loop_control", "focusa_work_loop_select_next"],
                    "approval": {"status": "operator_approval_required", "grants_permissions": false},
                    "persistent_defaults": {"harness": {"kind": "pi"}, "workspace": {"mode": "isolated_worktree"}, "governance": {"destructive_actions_allowed": false}}
                }
            ],
            "catalog_source": "generated_assignable_capability_registry",
            "catalog_revision": "workforce-role-profiles-v1"
        }),
    )
    .await
}

async fn presets(State(state): State<Arc<AppState>>, headers: HeaderMap) -> ApiResponse {
    authenticated_catalog(
        &state,
        &headers,
        "presets",
        json!({
            "presets": [
                {"preset_id": "conservative", "role_id": "operator", "description": "Legacy bounded single-worker alias", "capability_refs": ["focusa_work_loop_status"], "approval_required": true, "invocation_patch": {"governance": {"destructive_actions_allowed": false}, "concurrency": {"max_workers": 1}}},
                {"preset_id": "planner", "role_id": "planner", "description": "Bounded planning worker", "capability_refs": ["focusa_trajectory_assess", "focusa_trajectory_propose_workpoint"], "approval_required": true, "invocation_patch": {"governance": {"destructive_actions_allowed": false}, "concurrency": {"max_workers": 1}}},
                {"preset_id": "researcher", "role_id": "researcher", "description": "Source-linked evidence worker", "capability_refs": ["focusa_context_cognition", "focusa_evidence_capture"], "approval_required": true, "invocation_patch": {"governance": {"destructive_actions_allowed": false}, "concurrency": {"max_workers": 1}}},
                {"preset_id": "builder", "role_id": "builder", "description": "Approved implementation worker", "capability_refs": ["focusa_silent_sessions", "focusa_call_stack_design"], "approval_required": true, "invocation_patch": {"governance": {"destructive_actions_allowed": false}, "concurrency": {"max_workers": 1}}},
                {"preset_id": "reviewer", "role_id": "reviewer", "description": "Evidence and drift review worker", "capability_refs": ["focusa_call_stack_verify", "focusa_metacog_doctor"], "approval_required": true, "invocation_patch": {"governance": {"destructive_actions_allowed": false}, "concurrency": {"max_workers": 1}}},
                {"preset_id": "operator", "role_id": "operator", "description": "Durable coordination observer", "capability_refs": ["focusa_bg_status", "focusa_work_loop_status"], "approval_required": true, "invocation_patch": {"governance": {"destructive_actions_allowed": false}, "concurrency": {"max_workers": 1}}}
            ],
            "catalog_source": "generated_assignable_capability_registry",
            "catalog_revision": "workforce-role-presets-v1"
        }),
    )
    .await
}

async fn resolve(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<ResolveBody>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    match resolve_silent_session_config(body.requested_config, body.layers) {
        Ok(config) => after(
            (
                StatusCode::OK,
                Json(SilentSessionApiEnvelope::canonical(
                    "config_resolved",
                    json!(config),
                )),
            ),
            &principal,
        ),
        Err(error) => after(config_failure(error.to_string()), &principal),
    }
}

async fn preview(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
    Json(body): Json<PreviewBody>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    let session = match load_session(&state.persistence, session_id) {
        Ok(Some(session)) => session,
        Ok(None) => return after(not_found("session_id"), &principal),
        Err(error) => return after(persistence_failure(error), &principal),
    };
    let run = match load_run(&state.persistence, body.run_id) {
        Ok(Some(run)) => run,
        Ok(None) => return after(not_found("run_id"), &principal),
        Err(error) => return after(persistence_failure(error), &principal),
    };
    if session.current_run_generation != body.generation
        || guard_exact_target(
            ExactSessionRunTarget {
                session_id,
                run_id: body.run_id,
                generation: body.generation,
            },
            &run,
        )
        .is_err()
    {
        return after(stale_target(), &principal);
    }
    let current = match load_config_revision(&state.persistence, session.active_config_revision_id)
    {
        Ok(Some(config)) => config,
        Ok(None) => return after(not_found("active_config_revision_id"), &principal),
        Err(error) => return after(persistence_failure(error), &principal),
    };
    if let Err(response) = authorize_mutation(
        &principal,
        &session,
        &run,
        &current,
        SilentSessionAction::PreviewConfig,
        Vec::new(),
        None,
    ) {
        return after(*response, &principal);
    }
    match preview_config_revision(current.config, body.requested_config, body.layers) {
        Ok(plan) => after(
            (
                StatusCode::OK,
                Json(SilentSessionApiEnvelope::canonical(
                    "config_previewed",
                    json!(plan),
                )),
            ),
            &principal,
        ),
        Err(error) => after(config_failure(error.to_string()), &principal),
    }
}

async fn authenticated_catalog(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    status: &str,
    data: serde_json::Value,
) -> ApiResponse {
    let principal = match durable_request_principal(state, headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    after(
        (
            StatusCode::OK,
            Json(SilentSessionApiEnvelope::canonical(status, data)),
        ),
        &principal,
    )
}

fn config_failure(reason: String) -> ApiResponse {
    failure(
        StatusCode::BAD_REQUEST,
        "config_rejected",
        "validation_rejected",
        &reason,
    )
}

fn stale_target() -> ApiResponse {
    let mut response = failure(
        StatusCode::CONFLICT,
        "stale_target",
        "exact_target_mismatch",
        "Refresh status and retry with the current exact target.",
    );
    response.1.0.stale = true;
    response
}

fn not_found(target: &str) -> ApiResponse {
    failure(
        StatusCode::NOT_FOUND,
        "not_found",
        "not_found",
        &format!("No canonical record exists for {target}."),
    )
}

fn after(response: ApiResponse, principal: &ApiRequestPrincipal) -> ApiResponse {
    disclose_principal_side_effect(response, principal)
}
