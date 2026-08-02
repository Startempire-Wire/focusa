//! Spec 140 typed Runtime Constitution discovery, compilation, evaluation, and delivery API.

use crate::{routes::permissions::permission_context, server::AppState};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::{
    agent_runtime_constitution::{
        InstructionClaim, InstructionConflict, InstructionResolution, InstructionSource,
        PromptEpistemicOutcomeRecord, PromptEvaluation, RuntimeConstitutionEvent,
        RuntimeConstitutionVersion,
    },
    agent_runtime_constitution_authority::{
        default_authority_graph, detect_conflicts, discover_project_instructions, resolve_conflict,
    },
    agent_runtime_constitution_compiler::{
        CompiledPromptLayers, PromptCompileInput, compile_prompt, compile_prompt_with_safe_fallback,
    },
    agent_runtime_constitution_lifecycle::{
        bind_prompt_epistemic_outcome, evaluate_prompt_variant,
    },
    agent_runtime_constitution_orchestrator::{
        CristRuntimeInput, RuntimeConstitutionComposition, compose_runtime_constitution,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use uuid::Uuid;

const SCHEMA: &str = "focusa.agent_runtime_api.v1";
type ApiError = (StatusCode, Json<Value>);

#[derive(Debug, Deserialize)]
struct DraftRequest {
    #[serde(flatten)]
    input: CristRuntimeInput,
    idempotency_key: String,
}

#[derive(Debug, Serialize)]
struct DraftResponse {
    schema: &'static str,
    composition: RuntimeConstitutionComposition,
    event_hash: String,
}

#[derive(Debug, Deserialize)]
struct CompileRequest {
    #[serde(flatten)]
    input: PromptCompileInput,
    safe_fallback: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct EvaluationRequest {
    evaluation_id: String,
    variant_id: String,
    dimensions: BTreeMap<String, f64>,
    evidence_refs: Vec<String>,
    constitution_id: String,
    idempotency_key: String,
    epistemic_outcome: Option<PromptEpistemicOutcomeRecord>,
}

#[derive(Debug, Deserialize)]
struct InstructionQuery {
    project_root: String,
    max_source_bytes: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ReconcileRequest {
    constitution_id: String,
    conflict: InstructionConflict,
    claims: Vec<InstructionClaim>,
    sources: Vec<InstructionSource>,
    operator_winner: Option<String>,
    idempotency_key: String,
}

#[derive(Debug, Deserialize)]
struct SimulateRequest {
    project_root: String,
    path: Option<String>,
    profile: Option<String>,
    target: Option<String>,
    max_source_bytes: Option<u64>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/agent-runtime/instructions/scan", post(scan))
        .route("/v1/agent-runtime/instructions/sources", get(sources))
        .route("/v1/agent-runtime/instructions/claims", get(claims))
        .route("/v1/agent-runtime/instructions/conflicts", get(conflicts))
        .route("/v1/agent-runtime/instructions/reconcile", post(reconcile))
        .route("/v1/agent-runtime/instructions/simulate", post(simulate))
        .route("/v1/agent-runtime/instructions/effective", get(effective))
        .route("/v1/agent-runtime/instructions/drift", get(drift))
        .route("/v1/agent-runtime/constitutions/draft", post(draft))
        .route("/v1/agent-runtime/constitutions/{id}", get(show))
        .route(
            "/v1/agent-runtime/constitutions/{id}/preview",
            post(preview),
        )
        .route(
            "/v1/agent-runtime/compile/system-prompt",
            post(compile_system_prompt),
        )
        .route("/v1/agent-runtime/variants/{id}", get(variant_events))
        .route("/v1/agent-runtime/evaluations", post(evaluate))
        .route("/v1/agent-runtime/evaluations/{id}", get(evaluation_events))
        .route("/v1/agent-runtime/delivery/status", get(delivery_status))
}

async fn scan(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<SimulateRequest>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    let discovered = discover(&request.project_root, request.max_source_bytes)?;
    Ok(Json(
        json!({"schema":SCHEMA,"status":"completed","canonical":true,"scan":discovered}),
    ))
}

async fn sources(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<InstructionQuery>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    let discovered = discover(&query.project_root, query.max_source_bytes)?;
    Ok(Json(
        json!({"schema":SCHEMA,"status":"completed","canonical":true,"sources":discovered.sources,"findings":discovered.findings}),
    ))
}

async fn claims(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<InstructionQuery>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    let discovered = discover(&query.project_root, query.max_source_bytes)?;
    Ok(Json(
        json!({"schema":SCHEMA,"status":"completed","canonical":true,"claims":discovered.claims,"findings":discovered.findings}),
    ))
}

async fn conflicts(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<InstructionQuery>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    let discovered = discover(&query.project_root, query.max_source_bytes)?;
    let conflicts = detect_conflicts(&discovered.claims, &default_authority_graph());
    let degraded = !conflicts.is_empty();
    Ok(Json(json!({
        "schema":SCHEMA,
        "status":if degraded { "degraded" } else { "completed" },
        "canonical":!degraded,
        "degraded":degraded,
        "conflicts":conflicts
    })))
}

async fn reconcile(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<ReconcileRequest>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:write")?;
    let resolution: InstructionResolution = resolve_conflict(
        &request.conflict,
        &request.claims,
        &request.sources,
        request.operator_winner.as_deref(),
    )
    .map_err(|reason| unprocessable(&reason, Value::Null))?;
    let event = RuntimeConstitutionEvent::InstructionConflictResolved(resolution.clone());
    let event_id = Uuid::now_v7().to_string();
    let stored = state
        .persistence
        .append_runtime_constitution_event(
            &event_id,
            &request.constitution_id,
            &request.idempotency_key,
            &event,
        )
        .map_err(internal)?;
    Ok(Json(
        json!({"schema":SCHEMA,"resolution":resolution,"event_hash":stored.event_hash}),
    ))
}

async fn simulate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<SimulateRequest>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    let discovered = discover(&request.project_root, request.max_source_bytes)?;
    let conflicts = detect_conflicts(&discovered.claims, &default_authority_graph());
    Ok(Json(json!({
        "schema":SCHEMA,"status":"completed","canonical":false,
        "simulation":{"path":request.path,"profile":request.profile,"target":request.target},
        "sources":discovered.sources,"claims":discovered.claims,"conflicts":conflicts,
        "committed":false
    })))
}

async fn effective(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<InstructionQuery>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    let discovered = discover(&query.project_root, query.max_source_bytes)?;
    let conflicts = detect_conflicts(&discovered.claims, &default_authority_graph());
    let conflicted: std::collections::BTreeSet<_> = conflicts
        .iter()
        .flat_map(|conflict| conflict.claim_refs.iter().cloned())
        .collect();
    let effective: Vec<_> = discovered
        .claims
        .iter()
        .filter(|claim| !conflicted.contains(&claim.claim_id))
        .collect();
    let degraded = !conflicts.is_empty();
    Ok(Json(json!({
        "schema":SCHEMA,
        "status":if degraded { "degraded" } else { "completed" },
        "canonical":!degraded,
        "degraded":degraded,
        "effective_claims":effective,
        "unresolved_conflicts":conflicts
    })))
}

async fn drift(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<InstructionQuery>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    let discovered = discover(&query.project_root, query.max_source_bytes)?;
    let assessment = focusa_core::agent_runtime_constitution_lifecycle::assess_contract_impact(
        &format!("impact:{}", Uuid::now_v7()),
        discovered
            .sources
            .iter()
            .map(|source| source.source_ref.clone())
            .collect(),
        discovered.findings,
    );
    Ok(Json(
        json!({"schema":SCHEMA,"impact_assessment":assessment,"regenerated":false}),
    ))
}

fn discover(
    project_root: &str,
    max_source_bytes: Option<u64>,
) -> Result<focusa_core::agent_runtime_constitution_authority::DiscoveredInstructionSet, ApiError> {
    let root = PathBuf::from(project_root);
    if !root.is_absolute() {
        return Err(bad_request("absolute_project_root_required"));
    }
    discover_project_instructions(&root, max_source_bytes.unwrap_or(256 * 1024))
        .map_err(|reason| unprocessable(&reason, Value::Null))
}

async fn draft(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<DraftRequest>,
) -> Result<Json<DraftResponse>, ApiError> {
    require(&headers, &state, "work-loop:write")?;
    if request.idempotency_key.trim().is_empty() {
        return Err(bad_request("idempotency_key_required"));
    }
    let composition = compose_runtime_constitution(request.input)
        .map_err(|errors| unprocessable("constitution_composition_rejected", json!(errors)))?;
    state
        .persistence
        .save_runtime_constitution(&composition.constitution)
        .map_err(internal)?;
    let event = RuntimeConstitutionEvent::RuntimeConstitutionDrafted(RuntimeConstitutionVersion {
        version: composition.constitution.revision.to_string(),
        parent_version: None,
        content_sha256: composition
            .constitution
            .instruction_sources
            .iter()
            .map(|source| source.content_sha256.as_str())
            .collect::<Vec<_>>()
            .join(":"),
        lifecycle: composition.constitution.status,
        created_at: Utc::now(),
    });
    let event_id = Uuid::now_v7().to_string();
    let stored = state
        .persistence
        .append_runtime_constitution_event(
            &event_id,
            &composition.constitution.constitution_id,
            &request.idempotency_key,
            &event,
        )
        .map_err(internal)?;
    Ok(Json(DraftResponse {
        schema: SCHEMA,
        composition,
        event_hash: stored.event_hash,
    }))
}

async fn show(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    let constitution = state
        .persistence
        .load_runtime_constitution(&id)
        .map_err(internal)?;
    constitution
        .map(|value| Json(json!({"schema": SCHEMA, "constitution": value})))
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error":"constitution_not_found"})),
            )
        })
}

async fn preview(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<CompileRequest>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    if request.input.constitution.constitution_id != id {
        return Err(bad_request("constitution_id_mismatch"));
    }
    let compiled = compile_request(request)?;
    Ok(Json(
        json!({"schema":SCHEMA,"preview":compiled,"committed":false}),
    ))
}

async fn compile_system_prompt(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<CompileRequest>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    let compiled = compile_request(request)?;
    Ok(Json(json!({"schema":SCHEMA,"compiled":compiled})))
}

async fn evaluate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<EvaluationRequest>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:write")?;
    let evaluation = evaluate_prompt_variant(
        &request.evaluation_id,
        &request.variant_id,
        request.dimensions,
        request.evidence_refs,
    )
    .map_err(|reason| unprocessable(&reason, Value::Null))?;
    let event = RuntimeConstitutionEvent::PromptVariantEvaluated(evaluation.clone());
    let event_id = Uuid::now_v7().to_string();
    let stored = state
        .persistence
        .append_runtime_constitution_event(
            &event_id,
            &request.constitution_id,
            &request.idempotency_key,
            &event,
        )
        .map_err(internal)?;
    let transfer_event_hash = if let Some(record) = request.epistemic_outcome {
        let record = bind_prompt_epistemic_outcome(record)
            .map_err(|reason| unprocessable(&reason, Value::Null))?;
        let event = RuntimeConstitutionEvent::PromptOutcomeTransferred(record);
        Some(
            state
                .persistence
                .append_runtime_constitution_event(
                    &Uuid::now_v7().to_string(),
                    &request.constitution_id,
                    &format!("{}:spec138", request.idempotency_key),
                    &event,
                )
                .map_err(internal)?
                .event_hash,
        )
    } else {
        None
    };
    Ok(Json(json!({
        "schema":SCHEMA,
        "evaluation":evaluation,
        "event_hash":stored.event_hash,
        "spec138_transfer_event_hash":transfer_event_hash
    })))
}

async fn variant_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    let events = state
        .persistence
        .runtime_constitution_events(&id)
        .map_err(internal)?;
    let variants: Vec<Value> = events
        .iter()
        .filter(|event| event.kind == "prompt.variant_compiled")
        .filter_map(|event| serde_json::from_str(&event.payload_json).ok())
        .collect();
    Ok(Json(json!({"schema":SCHEMA,"variants":variants})))
}

async fn evaluation_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    let events = state
        .persistence
        .runtime_constitution_events(&id)
        .map_err(internal)?;
    let evaluations: Vec<PromptEvaluation> = events
        .iter()
        .filter(|event| event.kind == "prompt.variant_evaluated")
        .filter_map(|event| serde_json::from_str(&event.payload_json).ok())
        .collect();
    Ok(Json(json!({"schema":SCHEMA,"evaluations":evaluations})))
}

async fn delivery_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    require(&headers, &state, "work-loop:read")?;
    Ok(Json(
        json!({"schema":SCHEMA,"status":"available","commit_requires_operator_confirmation":true,"receipt_required":true}),
    ))
}

fn compile_request(request: CompileRequest) -> Result<CompiledPromptLayers, ApiError> {
    if request.safe_fallback.unwrap_or(false) {
        Ok(compile_prompt_with_safe_fallback(request.input))
    } else {
        compile_prompt(request.input)
            .map_err(|errors| unprocessable("prompt_compilation_rejected", json!(errors)))
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
