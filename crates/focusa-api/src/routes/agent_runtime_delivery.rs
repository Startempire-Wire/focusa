//! Spec 140 governed lifecycle, target compilation, and artifact delivery routes.

use crate::{routes::permissions::permission_context, server::AppState};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::post,
};
use chrono::Utc;
use focusa_core::{
    agent_runtime_constitution::{
        AgentRuntimeDeliveryManifest, PromptRevocation, RuntimeArtifactProjection,
        RuntimeConstitutionEvent, RuntimeConstitutionLifecycleState, RuntimeConstitutionVersion,
        SkillBinding,
    },
    agent_runtime_constitution_compiler::{
        PromptCompileInput, compile_agents_artifacts, compile_cross_harness_artifact,
        compile_prompt,
    },
    agent_runtime_constitution_enforcement::compile_skill_activation_plan,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use uuid::Uuid;

type ApiError = (StatusCode, Json<Value>);
const SCHEMA: &str = "focusa.agent_runtime_delivery.v1";

#[derive(Debug, Deserialize)]
struct LifecycleRequest {
    operator_confirmed: bool,
    evidence_refs: Vec<String>,
    idempotency_key: String,
    reason_code: Option<String>,
    target_version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AgentsCompileRequest {
    #[serde(flatten)]
    input: PromptCompileInput,
    nested_deltas: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct TargetCompileRequest {
    #[serde(flatten)]
    input: PromptCompileInput,
    target: String,
}

#[derive(Debug, Deserialize)]
struct SkillsCompileRequest {
    plan_id: String,
    candidates: Vec<SkillBinding>,
    applicable_skill_ids: BTreeSet<String>,
}

#[derive(Debug, Deserialize)]
struct DeliveryRequest {
    manifest_id: String,
    constitution_id: String,
    constitution_version: String,
    artifacts: Vec<RuntimeArtifactProjection>,
    evidence_refs: Vec<String>,
    receipt_ref: Option<String>,
    operator_confirmed: Option<bool>,
    idempotency_key: Option<String>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/v1/agent-runtime/constitutions/{id}/approve",
            post(approve),
        )
        .route(
            "/v1/agent-runtime/constitutions/{id}/activate",
            post(activate),
        )
        .route("/v1/agent-runtime/constitutions/{id}/revoke", post(revoke))
        .route(
            "/v1/agent-runtime/constitutions/{id}/rollback",
            post(rollback),
        )
        .route("/v1/agent-runtime/compile/agents-md", post(compile_agents))
        .route("/v1/agent-runtime/compile/skills", post(compile_skills))
        .route("/v1/agent-runtime/compile/target", post(compile_target))
        .route("/v1/agent-runtime/delivery/preview", post(delivery_preview))
        .route("/v1/agent-runtime/delivery/commit", post(delivery_commit))
        .route("/v1/agent-runtime/delivery/verify", post(delivery_verify))
}

async fn approve(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<LifecycleRequest>,
) -> Result<Json<Value>, ApiError> {
    lifecycle(&state, &headers, &id, request, "approve").await
}

async fn activate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<LifecycleRequest>,
) -> Result<Json<Value>, ApiError> {
    lifecycle(&state, &headers, &id, request, "activate").await
}

async fn revoke(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<LifecycleRequest>,
) -> Result<Json<Value>, ApiError> {
    lifecycle(&state, &headers, &id, request, "revoke").await
}

async fn rollback(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<LifecycleRequest>,
) -> Result<Json<Value>, ApiError> {
    lifecycle(&state, &headers, &id, request, "rollback").await
}

async fn lifecycle(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    id: &str,
    request: LifecycleRequest,
    action: &str,
) -> Result<Json<Value>, ApiError> {
    require(headers, state, "work-loop:write")?;
    if !request.operator_confirmed || request.evidence_refs.is_empty() {
        return Err(precondition("operator_confirmation_and_evidence_required"));
    }
    let constitution = state
        .persistence
        .load_runtime_constitution(id)
        .map_err(internal)?
        .ok_or_else(|| not_found("constitution_not_found"))?;
    let version = RuntimeConstitutionVersion {
        version: request
            .target_version
            .clone()
            .unwrap_or_else(|| constitution.revision.to_string()),
        parent_version: None,
        content_sha256: constitution
            .instruction_sources
            .iter()
            .map(|source| source.content_sha256.as_str())
            .collect::<Vec<_>>()
            .join(":"),
        lifecycle: match action {
            "approve" => RuntimeConstitutionLifecycleState::Approved,
            "activate" | "rollback" => RuntimeConstitutionLifecycleState::Active,
            "revoke" => RuntimeConstitutionLifecycleState::Revoked,
            _ => return Err(bad_request("unknown_lifecycle_action")),
        },
        created_at: Utc::now(),
    };
    let event = match action {
        "approve" => RuntimeConstitutionEvent::RuntimeConstitutionApproved(version.clone()),
        "activate" => RuntimeConstitutionEvent::RuntimeConstitutionActivated(version.clone()),
        "rollback" => RuntimeConstitutionEvent::ContractRollbackActivated(version.clone()),
        "revoke" => RuntimeConstitutionEvent::RuntimeConstitutionRevoked(PromptRevocation {
            revocation_id: format!("revocation:{}", Uuid::now_v7()),
            version: version.version.clone(),
            reason_code: request
                .reason_code
                .clone()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| bad_request("revocation_reason_required"))?,
            effective_at: Utc::now(),
        }),
        _ => unreachable!(),
    };
    let event_id = Uuid::now_v7().to_string();
    let stored = state
        .persistence
        .append_runtime_constitution_event(&event_id, id, &request.idempotency_key, &event)
        .map_err(internal)?;
    Ok(Json(
        json!({"schema":SCHEMA,"action":action,"version":version,"event_hash":stored.event_hash,"activated_by_runtime_agent":false}),
    ))
}

async fn compile_agents(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<AgentsCompileRequest>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    let compiled = compile_prompt(request.input)
        .map_err(|errors| unprocessable("prompt_compilation_rejected", json!(errors)))?;
    let artifacts = compile_agents_artifacts(&compiled, &request.nested_deltas)
        .map_err(|reason| unprocessable(&reason, Value::Null))?;
    Ok(Json(
        json!({"schema":SCHEMA,"artifacts":artifacts,"committed":false}),
    ))
}

async fn compile_target(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<TargetCompileRequest>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    let compiled = compile_prompt(request.input)
        .map_err(|errors| unprocessable("prompt_compilation_rejected", json!(errors)))?;
    let artifact = compile_cross_harness_artifact(&request.target, &compiled)
        .map_err(|reason| unprocessable(&reason, Value::Null))?;
    Ok(Json(
        json!({"schema":SCHEMA,"artifact":artifact,"committed":false}),
    ))
}

async fn compile_skills(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<SkillsCompileRequest>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    let plan = compile_skill_activation_plan(
        request.plan_id,
        request.candidates,
        &request.applicable_skill_ids,
    );
    Ok(Json(
        json!({"schema":SCHEMA,"skill_activation_plan":plan,"committed":false}),
    ))
}

async fn delivery_preview(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<DeliveryRequest>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    Ok(Json(
        json!({"schema":SCHEMA,"manifest":manifest(&request),"committed":false,"receipt_required":true}),
    ))
}

async fn delivery_commit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<DeliveryRequest>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:write")?;
    if request.operator_confirmed != Some(true)
        || request.receipt_ref.as_deref().is_none_or(str::is_empty)
    {
        return Err(precondition("operator_confirmation_and_receipt_required"));
    }
    if request.artifacts.iter().any(|artifact| !artifact.verified) {
        return Err(unprocessable(
            "unverified_artifact_delivery_forbidden",
            Value::Null,
        ));
    }
    let key = request
        .idempotency_key
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| bad_request("idempotency_key_required"))?;
    let mut event_hashes = Vec::new();
    for (index, artifact) in request.artifacts.iter().enumerate() {
        let event = RuntimeConstitutionEvent::ArtifactDeliveryVerified(artifact.clone());
        let stored = state
            .persistence
            .append_runtime_constitution_event(
                &Uuid::now_v7().to_string(),
                &request.constitution_id,
                &format!("{key}:{index}"),
                &event,
            )
            .map_err(internal)?;
        event_hashes.push(stored.event_hash);
    }
    Ok(Json(
        json!({"schema":SCHEMA,"manifest":manifest(&request),"committed":true,"event_hashes":event_hashes,"receipt_ref":request.receipt_ref}),
    ))
}

async fn delivery_verify(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<DeliveryRequest>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    let verified = !request.artifacts.is_empty()
        && request
            .artifacts
            .iter()
            .all(|artifact| artifact.verified && !artifact.content_sha256.is_empty())
        && request
            .evidence_refs
            .iter()
            .all(|evidence| !evidence.trim().is_empty());
    Ok(Json(
        json!({"schema":SCHEMA,"manifest_id":request.manifest_id,"verified":verified}),
    ))
}

fn manifest(request: &DeliveryRequest) -> AgentRuntimeDeliveryManifest {
    AgentRuntimeDeliveryManifest {
        manifest_id: request.manifest_id.clone(),
        constitution_version: request.constitution_version.clone(),
        artifacts: request.artifacts.clone(),
        evidence_refs: request.evidence_refs.clone(),
    }
}

fn require(headers: &HeaderMap, state: &AppState, permission: &str) -> Result<(), ApiError> {
    if permission_context(headers, state.config.auth_token.is_some()).allows(permission) {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error":"permission_denied","required":permission})),
        ))
    }
}
fn bad_request(reason: &str) -> ApiError {
    (StatusCode::BAD_REQUEST, Json(json!({"error":reason})))
}
fn not_found(reason: &str) -> ApiError {
    (StatusCode::NOT_FOUND, Json(json!({"error":reason})))
}
fn precondition(reason: &str) -> ApiError {
    (
        StatusCode::PRECONDITION_REQUIRED,
        Json(json!({"error":reason})),
    )
}
fn unprocessable(reason: &str, details: Value) -> ApiError {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(json!({"error":reason,"details":details})),
    )
}
fn internal(error: anyhow::Error) -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error":"agent_runtime_persistence_failed","detail":error.to_string()})),
    )
}
