use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::mission_canvas::{
    CompositionEvent, DOMAIN_PACK_INSTALL_CAPABILITY, DomainPackInstallCommand,
    DomainPackInstallError, DomainPackInstallService, HostPlatform, HostRendererResolutionError,
    HostRendererResolutionService, MissionCanvasScope, MissionCanvasStore, ResolveProjectionInput,
    StoredDocument, resolve_projection,
};
use focusa_core::workstream_context::{
    ActorRef, ActorType, AuthorityContext, WorkstreamContext, WorkstreamContextError,
    WorkstreamRequestEnvelope,
};
use focusa_core::workstream_identity::{
    AttachmentKey, RuntimeObjectRef, WorkSurfaceId, WorkstreamKey,
};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::routes::permissions::permission_context;
use crate::server::AppState;

type ApiResult = Result<Json<Value>, (StatusCode, Json<Value>)>;

#[derive(Clone, Debug, Deserialize)]
pub struct ScopeQuery {
    /// Query transport encodes the generated WorkstreamKey as one JSON value.
    pub workstream: String,
    pub continuity_id: Option<String>,
    pub attachment: Option<String>,
    pub workspace_binding_id: Option<String>,
    pub runtime_object: Option<String>,
    pub work_surface_id: Option<String>,
    /// Durable composition-event cursor.  It is deliberately separate from
    /// the Workstream identity and is only used to replay events after the
    /// caller's last confirmed event.
    pub after_cursor: Option<String>,
}

impl ScopeQuery {
    fn scope(&self) -> Result<MissionCanvasScope, (StatusCode, Json<Value>)> {
        let workstream = parse_query_json::<WorkstreamKey>(&self.workstream, "workstream")?;
        let attachment =
            parse_optional_query_json::<AttachmentKey>(&self.attachment, "attachment")?;
        let runtime_object =
            parse_optional_query_json::<RuntimeObjectRef>(&self.runtime_object, "runtime_object")?;
        let work_surface_id = self
            .work_surface_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| WorkSurfaceId::parse(value.to_owned()))
            .transpose()
            .map_err(|_| {
                error(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "work_surface_invalid",
                    "work_surface_id must be non-empty",
                )
            })?;
        let continuity_id = self
            .continuity_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| focusa_core::workstream_identity::ContinuityId::parse(value.to_owned()))
            .transpose()
            .map_err(|_| {
                error(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "continuity_invalid",
                    "continuity_id must be non-empty when provided",
                )
            })?;
        let workspace_binding_id = self
            .workspace_binding_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| {
                focusa_core::workstream_identity::WorkspaceBindingId::parse(value.to_owned())
            })
            .transpose()
            .map_err(|_| {
                error(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "workspace_binding_invalid",
                    "workspace_binding_id must be non-empty when provided",
                )
            })?;
        MissionCanvasScope::from_parts(
            workstream,
            continuity_id,
            attachment,
            workspace_binding_id,
            runtime_object,
            work_surface_id,
        )
        .map_err(|reason| error(StatusCode::UNPROCESSABLE_ENTITY, "scope_incomplete", reason))
    }
}

fn parse_query_json<T: DeserializeOwned>(
    value: &str,
    field: &str,
) -> Result<T, (StatusCode, Json<Value>)> {
    if value.trim().is_empty() {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "workstream_missing",
            &format!("{field} is required"),
        ));
    }
    serde_json::from_str(value).map_err(|_| {
        error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "identity_invalid",
            &format!("{field} must be generated identity JSON"),
        )
    })
}

fn parse_optional_query_json<T: DeserializeOwned>(
    value: &Option<String>,
    field: &str,
) -> Result<Option<T>, (StatusCode, Json<Value>)> {
    value
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| parse_query_json(value, field))
        .transpose()
}

fn validate_authority(scope: &MissionCanvasScope) -> Result<(), (StatusCode, Json<Value>)> {
    scope
        .validate()
        .map_err(|reason| error(StatusCode::CONFLICT, "workstream_identity_mismatch", reason))
}

#[derive(Debug, Deserialize)]
struct DocumentWriteRequest {
    #[serde(flatten)]
    scope: MissionCanvasScope,
    document_id: String,
    revision: u64,
    expected_revision: Option<u64>,
    payload: Value,
    idempotency_key: String,
}

#[derive(Debug, Deserialize)]
struct RecipientResolveRequest {
    #[serde(flatten)]
    scope: MissionCanvasScope,
    recipient_ref: String,
}

#[derive(Debug, Deserialize)]
struct CompositionSelectionRequest {
    #[serde(flatten)]
    scope: MissionCanvasScope,
    selection_id: String,
    expected_projection_revision: u64,
    idempotency_key: String,
}

#[derive(Debug, Deserialize)]
struct DomainPackInstallRequest {
    #[serde(flatten)]
    scope: MissionCanvasScope,
    pack: focusa_core::mission_canvas::DomainPack,
    idempotency_key: String,
    #[serde(default)]
    confirmation: Option<String>,
    #[serde(default)]
    confirmed: bool,
    #[serde(default)]
    actor_id: Option<String>,
    #[serde(default)]
    authority_ref: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PiSessionEventRequest {
    #[serde(flatten)]
    scope: MissionCanvasScope,
    event_id: String,
    event_kind: String,
    projection_revision: u64,
    layout_revision: u64,
    payload: Value,
    occurred_at: String,
}

#[derive(Debug, Deserialize)]
struct LayoutMutationRequest {
    command_id: String,
    #[serde(flatten)]
    scope: MissionCanvasScope,
    action: String,
    target_contribution_id: Option<String>,
    secondary_work_surface_id: Option<String>,
    target_layout_node_id: Option<String>,
    split_ratio: Option<f64>,
    expected_projection_revision: u64,
    expected_layout_revision: u64,
    idempotency_key: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/mission-canvas/projection", get(get_projection))
        .route("/v1/mission-canvas/projection/resolve", post(resolve))
        .route("/v1/mission-canvas/profiles", get(list_profiles))
        .route("/v1/mission-canvas/profiles/select", post(select_profile))
        .route("/v1/mission-canvas/profiles/{profile_id}", get(get_profile))
        .route("/v1/mission-canvas/activities", get(list_activities))
        .route(
            "/v1/mission-canvas/activities/select",
            post(select_activity),
        )
        .route(
            "/v1/mission-canvas/domain-packs/install",
            post(install_domain_pack),
        )
        .route(
            "/v1/mission-canvas/registries/{registry_kind}",
            get(list_registry),
        )
        .route(
            "/v1/mission-canvas/layout-memory",
            get(get_layout_memory).post(put_layout_memory),
        )
        .route("/v1/mission-canvas/layout/mutations", post(mutate_layout))
        .route(
            "/v1/mission-canvas/rich-host/resolution",
            get(resolve_host_renderer),
        )
        .route(
            "/v1/mission-canvas/rich-host/{action}",
            post(update_host_lifecycle),
        )
        .route("/v1/mission-canvas/drafts/{draft_id}", get(get_draft))
        .route("/v1/mission-canvas/drafts/sync", post(sync_draft))
        .route(
            "/v1/mission-canvas/recipients/resolve",
            post(resolve_recipient),
        )
        .route(
            "/v1/mission-canvas/recompositions/{revision}/{proof_kind}",
            get(get_recomposition_proof),
        )
        .route("/v1/mission-canvas/events", get(list_events))
        .route(
            "/v1/mission-canvas/pi-session/events",
            post(append_pi_session_event),
        )
}

async fn get_projection(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    require_permission(&headers, "mission_canvas:read")?;
    let scope = query.scope()?;
    let store = store(&state)?;
    match store.get_projection(&scope).map_err(store_error)? {
        Some(projection) => Ok(Json(serde_json::to_value(projection).map_err(json_error)?)),
        None => Err(error(
            StatusCode::NOT_FOUND,
            "projection_not_found",
            "No resolved projection exists for this exact scope",
        )),
    }
}

async fn resolve(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(input): Json<ResolveProjectionInput>,
) -> ApiResult {
    require_permission(&headers, "mission_canvas:write")?;
    validate_authority(&input.eligibility.scope)?;
    let store = store(&state)?;
    let previous = store
        .get_projection(&input.eligibility.scope)
        .map_err(store_error)?;
    if previous.is_none() && input.previous_projection_revision != 0 {
        return Err(error(
            StatusCode::CONFLICT,
            "projection_revision_conflict",
            "A new scope must start at revision zero",
        ));
    }
    ensure_resolver_catalog(&store, &input)?;
    let previous_digest = previous
        .as_ref()
        .map(|value| value.projection_digest.clone());
    let expected_revision = previous
        .as_ref()
        .map(|_| input.previous_projection_revision);
    let result = resolve_projection(input, previous_digest).map_err(|error_value| {
        error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "projection_resolution_failed",
            &error_value.to_string(),
        )
    })?;
    store
        .put_projection(&result.projection, expected_revision, &result.event)
        .map_err(store_error)?;
    Ok(Json(json!({
        "schema": "focusa.mission_canvas.resolve_result.v1",
        "projection": result.projection,
        "evidence": result.evidence,
        "receipt": result.receipt,
    })))
}

fn ensure_resolver_catalog(
    store: &MissionCanvasStore,
    input: &ResolveProjectionInput,
) -> Result<(), (StatusCode, Json<Value>)> {
    let now = Utc::now().to_rfc3339();
    let candidates = input
        .candidates
        .iter()
        .map(|candidate| candidate.contribution_id.clone())
        .collect::<Vec<_>>();
    let documents = [
        (
            "mission_canvas_profiles",
            format!("profile:{}", input.eligibility.profile_id),
            json!({
                "profile_id":input.eligibility.profile_id,
                "revision":input.workspace_profile_revision,
                "display_name":input.eligibility.profile_id,
                "candidate_contribution_ids":candidates.clone(),
                "density":"standard",
                "terminology_registry_ref":"registry:terminology:default",
                "renderer_registry_ref":"registry:renderer:default",
                "domain_semantic_binding_registry_ref":null,
                "viability_rule_revision":"profile-viability:v1",
                "installed":true,
            }),
            "profile_changed",
        ),
        (
            "mission_canvas_activity_modes",
            format!("activity:{}", input.eligibility.activity_mode_id),
            json!({
                "activity_mode_id":input.eligibility.activity_mode_id,
                "revision":input.activity_mode_revision,
                "display_name":input.eligibility.activity_mode_id,
                "candidate_contribution_ids":candidates.clone(),
                "terminology_overrides_ref":null,
                "viability_rule_revision":"activity-viability:v1",
            }),
            "activity_mode_changed",
        ),
    ];
    for (table, document_id, payload, event_kind) in documents {
        if store
            .get_document(table, &input.eligibility.scope, &document_id)
            .map_err(store_error)?
            .is_some()
        {
            continue;
        }
        let document = StoredDocument {
            document_id: document_id.clone(),
            scope: input.eligibility.scope.clone(),
            revision: 1,
            payload,
            updated_at: now.clone(),
        };
        let event = CompositionEvent {
            event_id: format!("projection-event:{event_kind}:{}", input.idempotency_key),
            event_kind: event_kind.into(),
            scope: input.eligibility.scope.clone(),
            projection_revision: input.previous_projection_revision,
            layout_revision: input.previous_layout_revision,
            causation_id: Some(input.idempotency_key.clone()),
            correlation_id: None,
            occurred_at: now.clone(),
            payload: json!({"document_id":document_id}),
            evidence_refs: vec![],
            receipt_refs: vec![],
        };
        store
            .put_document(table, &document, None, &event)
            .map_err(store_error)?;
    }
    let builtin = focusa_core::mission_canvas::CompositionRegistry::builtin();
    for profile in builtin.profiles.values() {
        ensure_catalog_document(
            store,
            input,
            CatalogDocumentSeed {
                table: "mission_canvas_profiles",
                document_id: &format!("profile:{}", profile.profile_id),
                scope: &input.eligibility.scope,
                payload: serde_json::to_value(profile).map_err(json_error)?,
                event_kind: "profile_changed",
                now: &now,
            },
        )?;
    }
    for activity in builtin.activities.values() {
        ensure_catalog_document(
            store,
            input,
            CatalogDocumentSeed {
                table: "mission_canvas_activity_modes",
                document_id: &format!("activity:{}", activity.activity_mode_id),
                scope: &input.eligibility.scope,
                payload: serde_json::to_value(activity).map_err(json_error)?,
                event_kind: "activity_mode_changed",
                now: &now,
            },
        )?;
    }
    for entry in builtin
        .panels
        .values()
        .chain(builtin.home_canvases.values())
        .chain(builtin.work_surface_renderers.values())
        .chain(builtin.artifact_renderers.values())
        .chain(builtin.terminology.values())
        .chain(builtin.domain_semantics.values())
    {
        ensure_catalog_document(
            store,
            input,
            CatalogDocumentSeed {
                table: "mission_canvas_registry_entries",
                document_id: &format!("registry:{}", entry.entry_id),
                scope: &input.eligibility.scope,
                payload: serde_json::to_value(entry).map_err(json_error)?,
                event_kind: "candidate_discovered",
                now: &now,
            },
        )?;
    }
    for candidate in &input.candidates {
        let document_id = format!("candidate:{}", candidate.contribution_id);
        if store
            .get_document(
                "mission_canvas_registry_entries",
                &input.eligibility.scope,
                &document_id,
            )
            .map_err(store_error)?
            .is_some()
        {
            continue;
        }
        let document = StoredDocument {
            document_id: document_id.clone(),
            scope: input.eligibility.scope.clone(),
            revision: 1,
            payload: json!({"registry_kind":"PanelRegistry","entry_id":candidate.contribution_id,"revision":1,"schema_ref":"candidate-contribution.schema.json","payload_ref":candidate.contribution_id,"required_capabilities":candidate.required_capabilities,"required_permissions":candidate.required_permissions,"enabled":true,"candidate":candidate}),
            updated_at: now.clone(),
        };
        let event = CompositionEvent {
            event_id: format!(
                "projection-event:registry:{}:{}",
                candidate.contribution_id, input.idempotency_key
            ),
            event_kind: "candidate_discovered".into(),
            scope: input.eligibility.scope.clone(),
            projection_revision: input.previous_projection_revision,
            layout_revision: input.previous_layout_revision,
            causation_id: Some(input.idempotency_key.clone()),
            correlation_id: None,
            occurred_at: now.clone(),
            payload: json!({"document_id":document_id}),
            evidence_refs: vec![],
            receipt_refs: vec![],
        };
        store
            .put_document("mission_canvas_registry_entries", &document, None, &event)
            .map_err(store_error)?;
    }
    Ok(())
}

struct CatalogDocumentSeed<'a> {
    table: &'a str,
    document_id: &'a str,
    scope: &'a MissionCanvasScope,
    payload: Value,
    event_kind: &'a str,
    now: &'a str,
}

fn ensure_catalog_document(
    store: &MissionCanvasStore,
    input: &ResolveProjectionInput,
    seed: CatalogDocumentSeed<'_>,
) -> Result<(), (StatusCode, Json<Value>)> {
    let CatalogDocumentSeed {
        table,
        document_id,
        scope,
        payload,
        event_kind,
        now,
    } = seed;
    if store
        .get_document(table, scope, document_id)
        .map_err(store_error)?
        .is_some()
    {
        return Ok(());
    }
    let document = StoredDocument {
        document_id: document_id.into(),
        scope: scope.clone(),
        revision: 1,
        payload,
        updated_at: now.into(),
    };
    let event = CompositionEvent {
        event_id: format!(
            "projection-event:{event_kind}:{}:{}",
            document_id.replace(':', "-"),
            input.idempotency_key
        ),
        event_kind: event_kind.into(),
        scope: scope.clone(),
        projection_revision: input.previous_projection_revision,
        layout_revision: input.previous_layout_revision,
        causation_id: Some(input.idempotency_key.clone()),
        correlation_id: None,
        occurred_at: now.into(),
        payload: json!({"document_id":document_id}),
        evidence_refs: vec![],
        receipt_refs: vec![],
    };
    store
        .put_document(table, &document, None, &event)
        .map_err(store_error)?;
    Ok(())
}

async fn select_profile(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<CompositionSelectionRequest>,
) -> ApiResult {
    switch_composition(&state, &headers, request, true)
}

async fn select_activity(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<CompositionSelectionRequest>,
) -> ApiResult {
    switch_composition(&state, &headers, request, false)
}

fn switch_composition(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    request: CompositionSelectionRequest,
    profile_selection: bool,
) -> ApiResult {
    require_permission(headers, "mission_canvas:write")?;
    validate_authority(&request.scope)?;
    let store = store(state)?;
    let mut projection = store
        .get_projection(&request.scope)
        .map_err(store_error)?
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "projection_not_found",
                "No projection exists for composition switching",
            )
        })?;
    if projection.projection_revision != request.expected_projection_revision {
        return Err(error(
            StatusCode::CONFLICT,
            "projection_revision_conflict",
            "Projection revision is stale",
        ));
    }
    let registry = focusa_core::mission_canvas::CompositionRegistry::builtin();
    let profile_id = if profile_selection {
        request.selection_id.as_str()
    } else {
        projection.workspace_profile_id.as_str()
    };
    let activity_id = if profile_selection {
        projection.activity_mode_id.as_str()
    } else {
        request.selection_id.as_str()
    };
    let available = projection
        .candidate_contribution_ids
        .iter()
        .cloned()
        .collect();
    let candidates = registry
        .compose_candidate_ids(profile_id, activity_id, &available)
        .map_err(|error_value| {
            error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "composition_selection_unknown",
                &error_value.to_string(),
            )
        })?;
    if candidates.is_empty() {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "composition_not_viable",
            "Selected profile/activity has no meaningful contribution in this scope",
        ));
    }
    let now = Utc::now().to_rfc3339();
    let memory_id = format!(
        "layout-memory:{}:{}:switch",
        projection.workspace_profile_id, projection.activity_mode_id
    );
    let memory = StoredDocument {
        document_id: memory_id.clone(),
        scope: request.scope.clone(),
        revision: projection.layout_revision + 1,
        payload: json!({"memory_id":memory_id,"profile_id":projection.workspace_profile_id,"activity_mode_id":projection.activity_mode_id,"layout_tree":projection.layout_tree.clone(),"focused_semantic_target":projection.focused_semantic_target.clone(),"candidate_contribution_ids":projection.candidate_contribution_ids.clone()}),
        updated_at: now.clone(),
    };
    let prior_memory = store
        .get_document(
            "mission_canvas_layout_memory",
            &request.scope,
            &memory.document_id,
        )
        .map_err(store_error)?;
    let memory_event = CompositionEvent {
        event_id: format!("projection-event:layout-memory:{}", request.idempotency_key),
        event_kind: "layout_changed".into(),
        scope: request.scope.clone(),
        projection_revision: projection.projection_revision,
        layout_revision: projection.layout_revision,
        causation_id: Some(request.idempotency_key.clone()),
        correlation_id: None,
        occurred_at: now.clone(),
        payload: json!({"memory_id":memory.document_id}),
        evidence_refs: vec![],
        receipt_refs: vec![],
    };
    store
        .put_document(
            "mission_canvas_layout_memory",
            &memory,
            prior_memory.map(|value| value.revision),
            &memory_event,
        )
        .map_err(store_error)?;
    if profile_selection {
        let selected = registry.profiles.get(profile_id).unwrap();
        projection.workspace_profile_id = selected.profile_id.clone();
        projection.workspace_profile_revision = selected.revision;
    } else {
        let selected = registry.activities.get(activity_id).unwrap();
        projection.activity_mode_id = selected.activity_mode_id.clone();
        projection.activity_mode_revision = selected.revision;
    }
    projection.projection_revision += 1;
    projection.layout_revision += 1;
    projection.durable_event_cursor = format!("mission-canvas:{}", projection.projection_revision);
    projection.resolved_at = Some(now.clone());
    projection.projection_digest =
        focusa_core::mission_canvas::reducer::projection_digest(&projection).map_err(json_error)?;
    let event = CompositionEvent {
        event_id: format!(
            "projection-event:composition-switch:{}",
            request.idempotency_key
        ),
        event_kind: if profile_selection {
            "profile_changed".into()
        } else {
            "activity_mode_changed".into()
        },
        scope: request.scope,
        projection_revision: projection.projection_revision,
        layout_revision: projection.layout_revision,
        causation_id: Some(request.idempotency_key),
        correlation_id: None,
        occurred_at: now,
        payload: json!({"profile_id":projection.workspace_profile_id,"activity_mode_id":projection.activity_mode_id,"candidate_contribution_ids":candidates}),
        evidence_refs: vec![format!(
            "evidence:composition-switch:{}",
            projection.projection_revision
        )],
        receipt_refs: vec![format!(
            "receipt:composition-switch:{}",
            projection.projection_revision
        )],
    };
    store
        .put_projection(
            &projection,
            Some(request.expected_projection_revision),
            &event,
        )
        .map_err(store_error)?;
    Ok(Json(
        json!({"projection":projection,"evidence_ref":event.evidence_refs[0],"receipt_ref":event.receipt_refs[0]}),
    ))
}

async fn install_domain_pack(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<DomainPackInstallRequest>,
) -> ApiResult {
    require_permission_with_state(&state, &headers, "mission_canvas:write")?;
    validate_authority(&request.scope)?;

    let context = domain_pack_workstream_context(&request, &headers).map_err(|error_value| {
        domain_pack_install_error(DomainPackInstallError::Context(error_value))
    })?;
    let permissions = permission_context(&headers, token_enabled(&state))
        .list()
        .into_iter()
        .collect();
    let capabilities = header_values(&headers, "x-focusa-capabilities");
    let capabilities = if capabilities.is_empty() {
        [DOMAIN_PACK_INSTALL_CAPABILITY.to_owned()]
            .into_iter()
            .collect()
    } else {
        capabilities
    };
    let confirmation = request
        .confirmation
        .or_else(|| request.confirmed.then(|| "confirm".to_owned()));
    let command = DomainPackInstallCommand {
        context,
        scope: request.scope,
        pack: request.pack,
        idempotency_key: request.idempotency_key,
        confirmation,
        capabilities,
        permissions,
    };
    let receipt = DomainPackInstallService
        .install(&store(&state)?, &command)
        .map_err(domain_pack_install_error)?;
    Ok(Json(serde_json::to_value(receipt).map_err(json_error)?))
}

async fn list_profiles(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    list_viable_documents(&state, &headers, query, "mission_canvas_profiles")
}

async fn get_profile(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(profile_id): Path<String>,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    get_document(
        &state,
        &headers,
        query,
        "mission_canvas_profiles",
        &format!("profile:{profile_id}"),
    )
}

async fn list_activities(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    list_viable_documents(&state, &headers, query, "mission_canvas_activity_modes")
}

async fn list_registry(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(registry_kind): Path<String>,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    let mut response = list_documents(&state, &headers, query, "mission_canvas_registry_entries")?;
    if let Some(items) = response.0.as_array_mut() {
        items.retain(|item| {
            item.pointer("/payload/registry_kind")
                .and_then(Value::as_str)
                == Some(registry_kind.as_str())
        });
    }
    Ok(response)
}

async fn get_layout_memory(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    let scope = query.scope()?;
    let document_id = scope
        .attachment
        .as_ref()
        .map(|attachment| format!("layout-memory:{}", attachment.attachment_id))
        .unwrap_or_else(|| format!("layout-memory:{}", scope.workstream.workstream_id));
    get_document(
        &state,
        &headers,
        query,
        "mission_canvas_layout_memory",
        &document_id,
    )
}

async fn put_layout_memory(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<DocumentWriteRequest>,
) -> ApiResult {
    write_document(
        &state,
        &headers,
        "mission_canvas_layout_memory",
        request,
        "layout_changed",
        "mission_canvas:write",
    )
}

async fn mutate_layout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<LayoutMutationRequest>,
) -> ApiResult {
    require_permission(&headers, "mission_canvas:write")?;
    if request.idempotency_key.trim().is_empty() || request.command_id.trim().is_empty() {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "layout_command_invalid",
            "command_id and idempotency_key are required",
        ));
    }
    request
        .scope
        .validate()
        .map_err(|reason| error(StatusCode::CONFLICT, "attachment_scope_mismatch", reason))?;
    if request.scope.attachment.is_none() {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "attachment_missing",
            "Layout mutation requires an exact generated AttachmentKey",
        ));
    }
    let store = store(&state)?;
    let mut projection = store
        .get_projection(&request.scope)
        .map_err(store_error)?
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "projection_not_found",
                "No projection exists for layout mutation",
            )
        })?;
    if projection.projection_revision != request.expected_projection_revision
        || projection.layout_revision != request.expected_layout_revision
    {
        return Err(error(
            StatusCode::CONFLICT,
            "layout_revision_conflict",
            "Projection or layout revision is stale",
        ));
    }
    let target = request
        .target_contribution_id
        .as_deref()
        .unwrap_or_default();
    let secondary = request
        .secondary_work_surface_id
        .as_deref()
        .unwrap_or_default();
    let changed = if request.action == "focus" {
        projection.focused_semantic_target = target.to_owned();
        true
    } else if matches!(
        request.action.as_str(),
        "open" | "pin" | "unpin" | "rehydrate"
    ) {
        true
    } else {
        mutate_layout_value(
            &mut projection.layout_tree,
            &request.action,
            target,
            secondary,
            request.target_layout_node_id.as_deref(),
            request.split_ratio,
        )
    };
    if !changed {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "layout_mutation_not_applicable",
            "Layout command did not match the current projection",
        ));
    }
    projection.projection_revision += 1;
    projection.layout_revision += 1;
    projection.durable_event_cursor = format!("mission-canvas:{}", projection.projection_revision);
    projection.resolved_at = Some(Utc::now().to_rfc3339());
    projection.projection_digest =
        focusa_core::mission_canvas::reducer::projection_digest(&projection).map_err(json_error)?;
    let event = CompositionEvent {
        event_id: format!("projection-event:layout:{}", request.idempotency_key),
        event_kind: "layout_changed".into(),
        scope: request.scope,
        projection_revision: projection.projection_revision,
        layout_revision: projection.layout_revision,
        causation_id: Some(request.command_id.clone()),
        correlation_id: Some(request.idempotency_key),
        occurred_at: Utc::now().to_rfc3339(),
        payload: json!({"action":request.action,"target_contribution_id":target,"projection_digest":projection.projection_digest}),
        evidence_refs: vec![format!("evidence:layout:{}", projection.layout_revision)],
        receipt_refs: vec![format!("receipt:layout:{}", projection.layout_revision)],
    };
    store
        .put_projection(
            &projection,
            Some(request.expected_projection_revision),
            &event,
        )
        .map_err(store_error)?;
    Ok(Json(json!({
        "workstream": projection.scope.workstream,
        "continuity_id": projection.scope.continuity_id,
        "attachment": projection.scope.attachment,
        "workspace_binding_id": projection.scope.workspace_binding_id,
        "runtime_object": projection.scope.runtime_object,
        "work_surface_id": projection.scope.work_surface_id,
        "command_id": request.command_id,
        "accepted": true,
        "projection_revision": projection.projection_revision,
        "layout_revision": projection.layout_revision,
        "projection_digest": projection.projection_digest,
        "event_cursor": projection.durable_event_cursor,
        "error_ref": null,
        "evidence_ref": event.evidence_refs[0],
        "receipt_ref": event.receipt_refs[0],
    })))
}

fn mutate_layout_value(
    node: &mut Value,
    action: &str,
    target: &str,
    secondary: &str,
    target_node: Option<&str>,
    split_ratio: Option<f64>,
) -> bool {
    if action == "set_active_tab" && node.get("kind").and_then(Value::as_str) == Some("tabs") {
        let contains = node
            .get("contribution_ids")
            .and_then(Value::as_array)
            .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(target)));
        if contains {
            node["active_contribution_id"] = json!(target);
            return true;
        }
    }
    if action == "resize_split"
        && node.get("kind").and_then(Value::as_str) == Some("split")
        && target_node
            .is_none_or(|expected| node.get("node_id").and_then(Value::as_str) == Some(expected))
    {
        let ratio = split_ratio.unwrap_or(0.67);
        if (0.1..=0.9).contains(&ratio) {
            node["ratio"] = json!(ratio);
            return true;
        }
    }
    if action == "ungroup" && node.get("kind").and_then(Value::as_str) == Some("tabs") {
        if let Some(active) = node
            .get("active_contribution_id")
            .and_then(Value::as_str)
            .map(str::to_owned)
        {
            *node = json!({"kind":"single","node_id":target_node.unwrap_or("layout:ungrouped"),"contribution_id":active});
            return true;
        }
    }
    if action == "reorder" {
        let mut target_path = None;
        let mut secondary_path = None;
        find_contribution_paths(
            node,
            &mut vec![],
            target,
            secondary,
            &mut target_path,
            &mut secondary_path,
        );
        if let (Some(left), Some(right)) = (target_path, secondary_path) {
            let left_value = node.pointer(&left).cloned();
            let right_value = node.pointer(&right).cloned();
            if let (Some(left_value), Some(right_value)) = (left_value, right_value) {
                if let Some(slot) = node.pointer_mut(&left) {
                    *slot = right_value;
                }
                if let Some(slot) = node.pointer_mut(&right) {
                    *slot = left_value;
                }
                return true;
            }
        }
    }
    if action == "group"
        && node.get("kind").and_then(Value::as_str) == Some("single")
        && node.get("contribution_id").and_then(Value::as_str) == Some(target)
        && !secondary.is_empty()
    {
        *node = json!({
            "kind":"tabs",
            "node_id":target_node.unwrap_or("layout:grouped-tabs"),
            "contribution_ids":[target,secondary],
            "active_contribution_id":target,
        });
        return true;
    }
    if matches!(action, "split_horizontal" | "split_vertical" | "compare")
        && node.get("kind").and_then(Value::as_str) == Some("single")
        && node.get("contribution_id").and_then(Value::as_str) == Some(target)
        && !secondary.is_empty()
    {
        let original = node.clone();
        *node = json!({
            "kind":"split",
            "node_id":target_node.unwrap_or("layout:mutated-split"),
            "orientation":if action == "split_vertical" { "vertical" } else { "horizontal" },
            "ratio":split_ratio.unwrap_or(0.67),
            "children":[original,{"kind":"single","node_id":format!("layout:{}",secondary),"contribution_id":secondary}],
        });
        return true;
    }
    if matches!(action, "suspend_projection" | "close_projection") {
        if let Some(children) = node.get_mut("children").and_then(Value::as_array_mut) {
            let before = children.len();
            children.retain(|child| !layout_contains_contribution(child, target));
            if children.len() != before {
                if children.len() == 1 {
                    *node = children.remove(0);
                }
                return true;
            }
        }
    }
    if let Some(children) = node.get_mut("children").and_then(Value::as_array_mut) {
        for child in children {
            if mutate_layout_value(child, action, target, secondary, target_node, split_ratio) {
                return true;
            }
        }
    }
    if let Some(primary) = node.get_mut("primary") {
        if mutate_layout_value(primary, action, target, secondary, target_node, split_ratio) {
            return true;
        }
    }
    false
}

fn layout_contains_contribution(value: &Value, target: &str) -> bool {
    match value {
        Value::String(item) => item == target,
        Value::Array(items) => items
            .iter()
            .any(|item| layout_contains_contribution(item, target)),
        Value::Object(items) => items
            .values()
            .any(|item| layout_contains_contribution(item, target)),
        _ => false,
    }
}

fn find_contribution_paths(
    value: &Value,
    path: &mut Vec<String>,
    target: &str,
    secondary: &str,
    target_path: &mut Option<String>,
    secondary_path: &mut Option<String>,
) {
    match value {
        Value::String(item) if item == target || item == secondary => {
            let pointer = format!(
                "/{}",
                path.iter()
                    .map(|part| part.replace('~', "~0").replace('/', "~1"))
                    .collect::<Vec<_>>()
                    .join("/")
            );
            if item == target {
                *target_path = Some(pointer);
            } else {
                *secondary_path = Some(pointer);
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                path.push(index.to_string());
                find_contribution_paths(item, path, target, secondary, target_path, secondary_path);
                path.pop();
            }
        }
        Value::Object(items) => {
            for (key, item) in items {
                path.push(key.clone());
                find_contribution_paths(item, path, target, secondary, target_path, secondary_path);
                path.pop();
            }
        }
        _ => {}
    }
}

async fn resolve_host_renderer(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    require_permission_with_state(&state, &headers, "mission_canvas:host")?;
    let scope = query.scope()?;
    let context =
        host_renderer_workstream_context(&scope, &headers).map_err(host_renderer_context_error)?;
    let capabilities = header_values(&headers, "x-focusa-capabilities");
    let resolution = HostRendererResolutionService
        .resolve(&context, &scope, &capabilities, HostPlatform::current())
        .map_err(host_renderer_resolution_error)?;
    Ok(Json(serde_json::to_value(resolution).map_err(json_error)?))
}

fn host_renderer_workstream_context(
    scope: &MissionCanvasScope,
    headers: &HeaderMap,
) -> Result<WorkstreamContext, WorkstreamContextError> {
    let actor_id = headers
        .get("x-focusa-actor-id")
        .and_then(|value| value.to_str().ok())
        .ok_or(WorkstreamContextError::MissingActor)?;
    let authority_ref = headers
        .get("x-focusa-authority-ref")
        .and_then(|value| value.to_str().ok())
        .ok_or(WorkstreamContextError::MissingAuthority)?;
    let actor = ActorRef::new(ActorType::Desktop, actor_id.to_owned())?;
    let authority = AuthorityContext::canonical(
        authority_ref.to_owned(),
        "mission_canvas:host permission established for the exact Workstream",
    );
    let mut envelope = WorkstreamRequestEnvelope::new(
        Some(scope.workstream.clone()),
        scope.attachment.clone(),
        actor,
        authority,
    );
    envelope.continuity_id = scope.continuity_id.clone();
    envelope.workspace_binding_id = scope.workspace_binding_id.clone();
    WorkstreamContext::extract(envelope)
}

fn host_renderer_context_error(error_value: WorkstreamContextError) -> (StatusCode, Json<Value>) {
    let status = match &error_value {
        WorkstreamContextError::WorkstreamMismatch
        | WorkstreamContextError::ContinuityMismatch
        | WorkstreamContextError::WorkspaceBindingMismatch
        | WorkstreamContextError::AuthorityDenied => StatusCode::CONFLICT,
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    };
    error(
        status,
        "workstream_context_invalid",
        &error_value.to_string(),
    )
}

fn host_renderer_resolution_error(
    error_value: HostRendererResolutionError,
) -> (StatusCode, Json<Value>) {
    match error_value {
        HostRendererResolutionError::CapabilityUnavailable(capability) => error(
            StatusCode::FORBIDDEN,
            "capability_unavailable",
            &format!("Missing required host capability: {capability}"),
        ),
        HostRendererResolutionError::Context(context_error) => {
            host_renderer_context_error(context_error)
        }
        HostRendererResolutionError::Scope(reason) => {
            error(StatusCode::UNPROCESSABLE_ENTITY, "scope_incomplete", reason)
        }
    }
}

async fn update_host_lifecycle(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(action): Path<String>,
    Json(mut request): Json<DocumentWriteRequest>,
) -> ApiResult {
    if !matches!(action.as_str(), "launch" | "focus" | "hide" | "close") {
        return Err(error(
            StatusCode::NOT_FOUND,
            "host_action_unknown",
            "Unknown rich-host lifecycle action",
        ));
    }
    request.payload["state"] = json!(match action.as_str() {
        "launch" => "launching",
        "focus" => "focused",
        "hide" => "hidden",
        _ => "closing",
    });
    write_document(
        &state,
        &headers,
        "mission_canvas_host_lifecycle",
        request,
        &format!("host_{action}"),
        "mission_canvas:host",
    )
}

async fn get_draft(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(draft_id): Path<String>,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    require_permission(&headers, "mission_canvas:draft")?;
    get_document_with_permission(&state, query, "mission_canvas_drafts", &draft_id)
}

async fn sync_draft(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<DocumentWriteRequest>,
) -> ApiResult {
    write_document(
        &state,
        &headers,
        "mission_canvas_drafts",
        request,
        "draft_synchronized",
        "mission_canvas:draft",
    )
}

async fn resolve_recipient(
    headers: HeaderMap,
    Json(request): Json<RecipientResolveRequest>,
) -> ApiResult {
    require_permission(&headers, "mission_canvas:draft")?;
    validate_authority(&request.scope)?;
    if request.recipient_ref.trim().is_empty() {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "recipient_missing",
            "recipient_ref is required",
        ));
    }
    let scope = request.scope;
    Ok(Json(json!({
        "schema": "focusa.mission_canvas.recipient_resolution.v1",
        "workstream": scope.workstream,
        "continuity_id": scope.continuity_id,
        "attachment": scope.attachment,
        "workspace_binding_id": scope.workspace_binding_id,
        "runtime_object": scope.runtime_object,
        "work_surface_id": scope.work_surface_id,
        "recipient_ref": request.recipient_ref,
        "routable": true
    })))
}

async fn get_recomposition_proof(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((revision, proof_kind)): Path<(u64, String)>,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    require_permission(&headers, "mission_canvas:read")?;
    let scope = query.scope()?;
    let events = store(&state)?
        .events_after(&scope, 0, 10_000)
        .map_err(store_error)?;
    let event = events
        .into_iter()
        .map(|(_, event)| event)
        .find(|event| {
            event.projection_revision == revision && event.event_kind == "projection_resolved"
        })
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "recomposition_not_found",
                "No recomposition proof exists for this revision",
            )
        })?;
    let key = match proof_kind.as_str() {
        "evidence" => "evidence",
        "receipt" => "receipt",
        "diagnostics" => "omission_diagnostics",
        _ => {
            return Err(error(
                StatusCode::NOT_FOUND,
                "proof_kind_unknown",
                "Unknown recomposition proof kind",
            ));
        }
    };
    Ok(Json(event.payload.get(key).cloned().unwrap_or(Value::Null)))
}

async fn append_pi_session_event(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<PiSessionEventRequest>,
) -> ApiResult {
    require_permission(&headers, "mission_canvas:write")?;
    validate_authority(&request.scope)?;
    if !matches!(
        request.event_kind.as_str(),
        "pi_turn_started"
            | "pi_turn_completed"
            | "pi_message_updated"
            | "pi_tool_started"
            | "pi_tool_completed"
    ) {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "pi_event_kind_unknown",
            "Unknown Pi session event kind",
        ));
    }
    let event = CompositionEvent {
        event_id: request.event_id,
        event_kind: request.event_kind,
        scope: request.scope,
        projection_revision: request.projection_revision,
        layout_revision: request.layout_revision,
        causation_id: None,
        correlation_id: None,
        occurred_at: request.occurred_at,
        payload: request.payload,
        evidence_refs: vec![],
        receipt_refs: vec![],
    };
    let sequence = store(&state)?.append_event(&event).map_err(store_error)?;
    Ok(Json(json!({
        "schema": "focusa.mission_canvas.pi_session_event_receipt.v1",
        "workstream": event.scope.workstream,
        "event_id": event.event_id,
        "accepted": true,
        "receipt_ref": format!("receipt:pi-session:{}", sequence)
    })))
}

async fn list_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    require_permission(&headers, "mission_canvas:read")?;
    let scope = query.scope()?;
    let requested_cursor = requested_event_cursor(&query, &headers)?;
    let after_sequence = parse_event_cursor(requested_cursor.as_deref())?;
    let store = store(&state)?;
    let latest_sequence = store.latest_event_sequence(&scope).map_err(store_error)?;
    if after_sequence > latest_sequence {
        return Err(error(
            StatusCode::CONFLICT,
            "event_cursor_stale",
            "The requested composition-event cursor is ahead of this exact Workstream history",
        ));
    }

    // The generated `eventsStream` method is a bounded replay/tail read: the
    // Desktop event client calls this endpoint again with the last confirmed
    // cursor, so replay and subsequent tail reads use the same exact scope and
    // durable sequence rather than a tab, CWD, or latest-record fallback.
    let events = store
        .events_after(&scope, after_sequence, 1_000)
        .map_err(store_error)?
        .into_iter()
        .map(|(sequence, event)| projection_lifecycle_event(sequence, event))
        .collect::<Vec<_>>();
    Ok(Json(Value::Array(events)))
}

fn requested_event_cursor(
    query: &ScopeQuery,
    headers: &HeaderMap,
) -> Result<Option<String>, (StatusCode, Json<Value>)> {
    let query_cursor = query
        .after_cursor
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let header_cursor = headers
        .get("last-event-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    if let (Some(query_cursor), Some(header_cursor)) = (&query_cursor, &header_cursor) {
        if query_cursor != header_cursor {
            return Err(error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "event_cursor_mismatch",
                "after_cursor and Last-Event-ID must identify the same durable cursor",
            ));
        }
    }
    Ok(query_cursor.or(header_cursor))
}

fn parse_event_cursor(cursor: Option<&str>) -> Result<u64, (StatusCode, Json<Value>)> {
    let Some(cursor) = cursor else {
        return Ok(0);
    };
    let cursor = cursor.trim();
    let sequence = cursor
        .strip_prefix("event:")
        .or_else(|| cursor.strip_prefix("cursor:"))
        .unwrap_or(cursor);
    if sequence.is_empty() || !sequence.chars().all(|value| value.is_ascii_digit()) {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "event_cursor_invalid",
            "event cursor must be a durable numeric cursor",
        ));
    }
    sequence.parse::<u64>().map_err(|_| {
        error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "event_cursor_invalid",
            "event cursor is outside the supported durable sequence range",
        )
    })
}

fn projection_lifecycle_event(sequence: u64, event: CompositionEvent) -> Value {
    let event_id = if generated_projection_event_id(&event.event_id) {
        event.event_id
    } else {
        // Older appenders used a broader event-id namespace.  Keep those
        // events replayable while emitting the generated DTO's constrained,
        // deterministic identifier; the durable sequence remains the cursor.
        format!("projection-event:sequence:{sequence}")
    };
    json!({
        "event_id": event_id,
        "event_kind": event.event_kind,
        "workstream": event.scope.workstream,
        "continuity_id": event.scope.continuity_id,
        "attachment": event.scope.attachment,
        "workspace_binding_id": event.scope.workspace_binding_id,
        "runtime_object": event.scope.runtime_object,
        "work_surface_id": event.scope.work_surface_id,
        "projection_revision": event.projection_revision,
        "layout_revision": event.layout_revision,
        "event_cursor": format!("event:{sequence}"),
        "occurred_at": event.occurred_at,
        "payload_ref": format!("mission-canvas:composition-event:{sequence}"),
        "evidence_refs": event.evidence_refs,
        "receipt_refs": event.receipt_refs,
        "causation_id": event.causation_id,
        "correlation_id": event.correlation_id,
    })
}

fn generated_projection_event_id(value: &str) -> bool {
    let Some(suffix) = value.strip_prefix("projection-event:") else {
        return false;
    };
    !suffix.is_empty()
        && suffix.chars().all(|value| {
            value.is_ascii_lowercase() || value.is_ascii_digit() || "._:-".contains(value)
        })
}

fn list_viable_documents(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    query: ScopeQuery,
    table: &str,
) -> ApiResult {
    require_permission(headers, "mission_canvas:read")?;
    let scope = query.scope()?;
    let store = store(state)?;
    let candidates = store
        .get_projection(&scope)
        .map_err(store_error)?
        .map(|projection| {
            projection
                .candidate_contribution_ids
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>()
        })
        .unwrap_or_default();
    let documents = store.list_documents(table, &scope).map_err(store_error)?;
    let viable = documents
        .into_iter()
        .filter(|document| {
            document
                .payload
                .get("candidate_contribution_ids")
                .and_then(Value::as_array)
                .is_some_and(|ids| {
                    ids.iter()
                        .filter_map(Value::as_str)
                        .any(|id| candidates.contains(id))
                })
        })
        .collect::<Vec<_>>();
    Ok(Json(serde_json::to_value(viable).map_err(json_error)?))
}

fn list_documents(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    query: ScopeQuery,
    table: &str,
) -> ApiResult {
    require_permission(headers, "mission_canvas:read")?;
    let documents = store(state)?
        .list_documents(table, &query.scope()?)
        .map_err(store_error)?;
    Ok(Json(serde_json::to_value(documents).map_err(json_error)?))
}

fn get_document(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    query: ScopeQuery,
    table: &str,
    document_id: &str,
) -> ApiResult {
    require_permission(headers, "mission_canvas:read")?;
    get_document_with_permission(state, query, table, document_id)
}

fn get_document_with_permission(
    state: &Arc<AppState>,
    query: ScopeQuery,
    table: &str,
    document_id: &str,
) -> ApiResult {
    let document = store(state)?
        .get_document(table, &query.scope()?, document_id)
        .map_err(store_error)?
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "document_not_found",
                "No document exists for this exact scope",
            )
        })?;
    Ok(Json(serde_json::to_value(document).map_err(json_error)?))
}

fn write_document(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    table: &str,
    request: DocumentWriteRequest,
    event_kind: &str,
    permission: &str,
) -> ApiResult {
    require_permission(headers, permission)?;
    validate_authority(&request.scope)?;
    if request.idempotency_key.trim().is_empty() {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "idempotency_key_missing",
            "idempotency_key is required",
        ));
    }
    let now = Utc::now().to_rfc3339();
    let document = StoredDocument {
        document_id: request.document_id,
        scope: request.scope.clone(),
        revision: request.revision,
        payload: request.payload,
        updated_at: now.clone(),
    };
    let event = CompositionEvent {
        event_id: format!(
            "projection-event:{}:{}",
            event_kind, request.idempotency_key
        ),
        event_kind: event_kind.into(),
        scope: request.scope,
        projection_revision: request.revision,
        layout_revision: request.revision,
        causation_id: Some(request.idempotency_key),
        correlation_id: None,
        occurred_at: now,
        payload: json!({"document_id":document.document_id,"revision":document.revision}),
        evidence_refs: vec![],
        receipt_refs: vec![format!("receipt:{}:{}", event_kind, document.revision)],
    };
    store(state)?
        .put_document(table, &document, request.expected_revision, &event)
        .map_err(store_error)?;
    Ok(Json(
        json!({"document":document,"event":event,"receipt_ref":event.receipt_refs[0]}),
    ))
}

fn store(state: &Arc<AppState>) -> Result<MissionCanvasStore, (StatusCode, Json<Value>)> {
    MissionCanvasStore::open(&state.config.data_dir).map_err(store_error)
}

fn token_enabled(state: &Arc<AppState>) -> bool {
    state.config.auth_token.is_some()
        || std::env::var("FOCUSA_AUTH_TOKEN")
            .ok()
            .is_some_and(|value| !value.trim().is_empty())
}

fn header_values(headers: &HeaderMap, name: &str) -> std::collections::BTreeSet<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .split([',', ' '])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

fn require_permission_with_state(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    permission: &str,
) -> Result<(), (StatusCode, Json<Value>)> {
    if permission_context(headers, token_enabled(state)).allows(permission) {
        Ok(())
    } else {
        Err(error(
            StatusCode::FORBIDDEN,
            "permission_denied",
            &format!("Missing required permission: {permission}"),
        ))
    }
}

fn domain_pack_workstream_context(
    request: &DomainPackInstallRequest,
    headers: &HeaderMap,
) -> Result<WorkstreamContext, WorkstreamContextError> {
    let actor_id = request
        .actor_id
        .as_deref()
        .or_else(|| {
            headers
                .get("x-focusa-actor-id")
                .and_then(|value| value.to_str().ok())
        })
        .ok_or(WorkstreamContextError::MissingActor)?;
    let authority_ref = request
        .authority_ref
        .as_deref()
        .or_else(|| {
            headers
                .get("x-focusa-authority-ref")
                .and_then(|value| value.to_str().ok())
        })
        .ok_or(WorkstreamContextError::MissingAuthority)?;
    let actor = ActorRef::new(ActorType::Desktop, actor_id.to_owned())?;
    let authority = AuthorityContext::canonical(
        authority_ref.to_owned(),
        "mission_canvas:write permission established for the exact Workstream",
    );
    let mut envelope = WorkstreamRequestEnvelope::new(
        Some(request.scope.workstream.clone()),
        request.scope.attachment.clone(),
        actor,
        authority,
    );
    envelope.continuity_id = request.scope.continuity_id.clone();
    envelope.workspace_binding_id = request.scope.workspace_binding_id.clone();
    WorkstreamContext::extract(envelope)
}

fn domain_pack_install_error(error_value: DomainPackInstallError) -> (StatusCode, Json<Value>) {
    let message = error_value.to_string();
    let (status, code) = match &error_value {
        DomainPackInstallError::CapabilityUnavailable(_) => {
            (StatusCode::FORBIDDEN, "capability_unavailable")
        }
        DomainPackInstallError::PermissionDenied(_) => (StatusCode::FORBIDDEN, "permission_denied"),
        DomainPackInstallError::ConfirmationRequired => {
            (StatusCode::PRECONDITION_REQUIRED, "confirmation_required")
        }
        DomainPackInstallError::IdempotencyKeyRequired => {
            (StatusCode::UNPROCESSABLE_ENTITY, "idempotency_key_missing")
        }
        DomainPackInstallError::Context(context_error) => match context_error {
            WorkstreamContextError::WorkstreamMismatch
            | WorkstreamContextError::ContinuityMismatch
            | WorkstreamContextError::WorkspaceBindingMismatch
            | WorkstreamContextError::AuthorityDenied => {
                (StatusCode::CONFLICT, "workstream_identity_mismatch")
            }
            _ => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "workstream_context_invalid",
            ),
        },
        DomainPackInstallError::Scope(_) => (StatusCode::UNPROCESSABLE_ENTITY, "scope_incomplete"),
        DomainPackInstallError::InvalidPack(_) => {
            (StatusCode::UNPROCESSABLE_ENTITY, "domain_pack_invalid")
        }
        DomainPackInstallError::Store(store_error) => {
            let conflict = matches!(
                store_error,
                focusa_core::mission_canvas::MissionCanvasStoreError::DomainPackAlreadyInstalled(_)
                    | focusa_core::mission_canvas::MissionCanvasStoreError::DomainPackIdempotencyConflict
                    | focusa_core::mission_canvas::MissionCanvasStoreError::DomainPackDocumentAlreadyExists(_)
            );
            if conflict {
                (StatusCode::CONFLICT, "domain_pack_conflict")
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "mission_canvas_store_error",
                )
            }
        }
        DomainPackInstallError::Serialization(_) => {
            (StatusCode::UNPROCESSABLE_ENTITY, "domain_pack_invalid")
        }
    };
    error(status, code, &message)
}

fn require_permission(
    headers: &HeaderMap,
    permission: &str,
) -> Result<(), (StatusCode, Json<Value>)> {
    let granted = headers
        .get("x-focusa-permissions")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .any(|value| value == permission || value == "mission_canvas:*");
    if granted {
        Ok(())
    } else {
        Err(error(
            StatusCode::FORBIDDEN,
            "permission_denied",
            &format!("Missing required permission: {permission}"),
        ))
    }
}

fn store_error(error_value: impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    let message = error_value.to_string();
    let status = if message.contains("revision conflict") || message.contains("already exists") {
        StatusCode::CONFLICT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    error(status, "mission_canvas_store_error", &message)
}

fn json_error(error_value: serde_json::Error) -> (StatusCode, Json<Value>) {
    error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "serialization_error",
        &error_value.to_string(),
    )
}

fn error(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(
            json!({"schema":"focusa.tool_result.v1","status":"blocked","error":{"code":code,"message":message}}),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn complete_scope() -> ScopeQuery {
        let legacy = focusa_core::scoped_state::ScopeRef::project(
            "project:focusa",
            "/workspace/focusa",
            "Focusa",
            "host-a:worktree-main",
        )
        .unwrap();
        let workstream = focusa_core::workstream_identity::WorkstreamKey::new(
            focusa_core::workstream_identity::ScopeRef::project(legacy).unwrap(),
            focusa_core::workstream_identity::WorkstreamId::parse("ws:mission-canvas").unwrap(),
        );
        let continuity =
            focusa_core::workstream_identity::ContinuityId::parse("continuity:mission-canvas")
                .unwrap();
        let attachment = focusa_core::workstream_identity::AttachmentKey::new(
            workstream.clone(),
            Some(continuity.clone()),
            focusa_core::workstream_identity::InstanceId::parse("instance:pi").unwrap(),
            focusa_core::workstream_identity::SessionId::parse("session:1").unwrap(),
            focusa_core::workstream_identity::AttachmentId::parse("attachment:1").unwrap(),
            focusa_core::workstream_identity::WorkspaceBindingId::parse("workspace:mission-canvas")
                .unwrap(),
        );
        ScopeQuery {
            workstream: serde_json::to_string(&workstream).unwrap(),
            continuity_id: Some(continuity.to_string()),
            attachment: Some(serde_json::to_string(&attachment).unwrap()),
            workspace_binding_id: Some("workspace:mission-canvas".into()),
            runtime_object: None,
            work_surface_id: Some("surface:pi".into()),
            after_cursor: None,
        }
    }

    #[test]
    fn exact_scope_rejects_missing_workstream_authority() {
        let mut query = complete_scope();
        query.workstream.clear();
        assert!(query.scope().is_err());
        assert!(complete_scope().scope().is_ok());
    }

    #[test]
    fn permission_header_fails_closed() {
        let headers = HeaderMap::new();
        assert!(require_permission(&headers, "mission_canvas:read").is_err());
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-focusa-permissions",
            HeaderValue::from_static("mission_canvas:read"),
        );
        assert!(require_permission(&headers, "mission_canvas:read").is_ok());
        assert!(require_permission(&headers, "mission_canvas:write").is_err());
    }

    #[test]
    fn layout_mutations_are_revision_payload_deterministic() {
        let mut layout = json!({
            "kind":"split","node_id":"layout:root","orientation":"horizontal","ratio":0.5,
            "children":[
                {"kind":"single","node_id":"layout:a","contribution_id":"contribution:a"},
                {"kind":"single","node_id":"layout:b","contribution_id":"contribution:b"}
            ]
        });
        assert!(mutate_layout_value(
            &mut layout,
            "reorder",
            "contribution:a",
            "contribution:b",
            None,
            None
        ));
        assert_eq!(
            layout
                .pointer("/children/0/contribution_id")
                .and_then(Value::as_str),
            Some("contribution:b")
        );
        let mut tab = json!({"kind":"tabs","node_id":"layout:tabs","contribution_ids":["contribution:a","contribution:b"],"active_contribution_id":"contribution:a"});
        assert!(mutate_layout_value(
            &mut tab,
            "set_active_tab",
            "contribution:b",
            "",
            None,
            None
        ));
        assert_eq!(tab["active_contribution_id"], "contribution:b");
    }
}
