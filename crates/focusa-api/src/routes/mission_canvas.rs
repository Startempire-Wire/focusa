use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use focusa_core::mission_canvas::{
    resolve_projection, validate_profile_layout_memory, ActivityModeDefinition,
    ActivitySelectionCommand, ActivitySelectionError, ActivitySelectionService,
    CandidateContribution, CompositionEvent, CompositionRegistry, DomainPackInstallCommand,
    DomainPackInstallError, DomainPackInstallService, EligibilityContext,
    HostLifecycleCloseCommand, HostLifecycleError, HostLifecycleFocusCommand,
    HostLifecycleHideCommand, HostLifecycleLaunchCommand, HostLifecycleService, HostLifecycleState,
    HostPlatform, HostRendererResolutionError, HostRendererResolutionService,
    LayoutMemoryUpdateCommand, LayoutMemoryUpdateError, LayoutMemoryUpdateService,
    LayoutMutationCommand, LayoutMutationError, LayoutMutationExecution, LayoutMutationService,
    MissionCanvasScope, MissionCanvasStore, ProfileLayoutMemory, ProfileSelectionCommand,
    ProfileSelectionError, ProfileSelectionService, RegistryDefinition, ResolveProjectionInput,
    StoredDocument, WorkspaceProfileDefinition, DOMAIN_PACK_INSTALL_CAPABILITY,
    LAYOUT_MEMORY_UPDATE_PERMISSION,
};
use focusa_core::workstream_context::{
    ActorRef, ActorType, AuthorityContext, WorkstreamContext, WorkstreamContextError,
    WorkstreamRequestEnvelope,
};
use focusa_core::workstream_identity::{
    AttachmentKey, RuntimeObjectRef, WorkSurfaceId, WorkstreamKey,
};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{json, Value};

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
    /// Layout-memory selectors are generated operation input. They are kept
    /// separate from the Workstream authority so a profile, activity, or
    /// viewport can never replace the canonical owner.
    pub profile_id: Option<String>,
    pub activity_mode_id: Option<String>,
    pub viewport_class: Option<String>,
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
    /// The selected profile or activity is an operation input extension carried
    /// alongside the generated Workstream authority. The operation/path/response
    /// still come from the generated registry and the Core service owns its
    /// meaning; this field never replaces Workstream authority.
    selection_id: String,
    expected_projection_revision: u64,
    idempotency_key: String,
    #[serde(default)]
    event_cursor: Option<String>,
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
struct RichHostCommandRequest {
    #[serde(flatten)]
    scope: MissionCanvasScope,
    idempotency_key: String,
}

/// The generated ContributionEligibilityContext is the public resolve
/// request.  ResolveProjectionInput remains a compatibility adapter for
/// existing Core callers; it is not the Desktop transport DTO.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContributionEligibilityContextRequest {
    #[serde(flatten)]
    scope: MissionCanvasScope,
    workspace_profile_id: String,
    workspace_profile_revision: u64,
    activity_mode_id: String,
    activity_mode_revision: u64,
    focused_work_surface_id: Option<WorkSurfaceId>,
    #[serde(default)]
    open_work_surface_ids: Option<Vec<WorkSurfaceId>>,
    canonical_read_model_revision: u64,
    available_operations: Vec<String>,
    capabilities: Vec<String>,
    permissions: Vec<String>,
    viewport: ContributionViewportRequest,
    project_constraint_refs: Vec<String>,
    user_preference_ref: Option<String>,
    resolver_rule_revision: String,
    #[serde(default)]
    observed_at: Option<String>,
    #[serde(default)]
    pinned_work_surface_ids: Option<Vec<WorkSurfaceId>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContributionViewportRequest {
    class: String,
    css_height: u32,
    css_width: u32,
    device_pixel_ratio: f64,
    platform: String,
    #[serde(default)]
    high_contrast: Option<bool>,
    #[serde(default)]
    reduced_motion: Option<bool>,
    #[serde(default)]
    reduced_transparency: Option<bool>,
    #[serde(default)]
    text_scale_percent: Option<u32>,
    #[serde(default)]
    zoom_percent: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ProjectionResolveRequest {
    Generated(ContributionEligibilityContextRequest),
    Core(ResolveProjectionInput),
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
        .route("/v1/mission-canvas/rich-host/launch", post(launch_host))
        .route("/v1/mission-canvas/rich-host/focus", post(focus_host))
        .route("/v1/mission-canvas/rich-host/hide", post(hide_host))
        .route("/v1/mission-canvas/rich-host/close", post(close_host))
        .route("/v1/mission-canvas/drafts/{draft_id}", get(get_draft))
        .route("/v1/mission-canvas/drafts/sync", post(sync_draft))
        .route(
            "/v1/mission-canvas/recipients/resolve",
            post(resolve_recipient),
        )
        .route(
            "/v1/mission-canvas/recompositions/{revision}/evidence",
            get(get_recomposition_evidence),
        )
        .route(
            "/v1/mission-canvas/recompositions/{revision}/receipt",
            get(get_recomposition_receipt),
        )
        .route(
            "/v1/mission-canvas/recompositions/{revision}/diagnostics",
            get(get_recomposition_diagnostics),
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
        Some(projection) => {
            projection.validate_scope(&scope).map_err(|reason| {
                error(StatusCode::CONFLICT, "projection_scope_invalid", reason)
            })?;
            Ok(Json(serde_json::to_value(projection).map_err(json_error)?))
        }
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
    Json(request): Json<ProjectionResolveRequest>,
) -> ApiResult {
    require_permission(&headers, "mission_canvas:write")?;
    let request_scope = match &request {
        ProjectionResolveRequest::Generated(request) => &request.scope,
        ProjectionResolveRequest::Core(input) => &input.eligibility.scope,
    };
    validate_authority(request_scope)?;
    // Resolve and rich-host operations share the same exact Workstream
    // extraction; the presentation surface is never used as ownership.
    exact_workstream_context(request_scope, &headers).map_err(host_renderer_context_error)?;
    let store = store(&state)?;
    let input = match request {
        ProjectionResolveRequest::Generated(request) => {
            generated_projection_input(&store, &headers, request)?
        }
        ProjectionResolveRequest::Core(input) => input,
    };
    let previous = store
        .get_projection(&input.eligibility.scope)
        .map_err(store_error)?;
    if let Some(previous_projection) = previous.as_ref() {
        let replay = store
            .events_after(&input.eligibility.scope, 0, 10_000)
            .map_err(store_error)?
            .into_iter()
            .map(|(_, event)| event)
            .find(|event| {
                event.event_kind == "projection_resolved"
                    && (event.causation_id.as_deref() == Some(input.idempotency_key.as_str())
                        || event.payload["receipt"]["idempotency_key"]
                            .as_str()
                            .is_some_and(|key| key == input.idempotency_key))
            });
        if let Some(event) = replay {
            if event.projection_revision == previous_projection.projection_revision {
                return Ok(Json(
                    serde_json::to_value(previous_projection).map_err(json_error)?,
                ));
            }
            return Err(error(
                StatusCode::CONFLICT,
                "idempotency_conflict",
                "The idempotency key belongs to a different projection revision",
            ));
        }
    }
    if let Some(previous_projection) = previous.as_ref() {
        if previous_projection.projection_revision != input.previous_projection_revision {
            return Err(error(
                StatusCode::CONFLICT,
                "projection_revision_conflict",
                "The projection revision is stale",
            ));
        }
        if previous_projection.layout_revision != input.previous_layout_revision {
            return Err(error(
                StatusCode::CONFLICT,
                "layout_revision_conflict",
                "The layout revision is stale",
            ));
        }
        if previous_projection.durable_event_cursor != input.event_cursor {
            return Err(error(
                StatusCode::CONFLICT,
                "projection_cursor_conflict",
                "The durable event cursor is stale",
            ));
        }
    } else if input.previous_projection_revision != 0 || input.previous_layout_revision != 0 {
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
    Ok(Json(
        serde_json::to_value(result.projection).map_err(json_error)?,
    ))
}

fn generated_projection_input(
    store: &MissionCanvasStore,
    headers: &HeaderMap,
    request: ContributionEligibilityContextRequest,
) -> Result<ResolveProjectionInput, (StatusCode, Json<Value>)> {
    if request.workspace_profile_id.trim().is_empty() || request.activity_mode_id.trim().is_empty()
    {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "projection_context_invalid",
            "workspace profile and activity mode are required",
        ));
    }
    if request.resolver_rule_revision
        != focusa_core::mission_canvas::resolver::RESOLVER_RULE_REVISION
    {
        return Err(error(
            StatusCode::CONFLICT,
            "resolver_rule_revision_mismatch",
            "The requested resolver rule is not the Core-authorized revision",
        ));
    }
    validate_viewport(&request.viewport)?;
    let idempotency_key = required_header(
        headers,
        "idempotency-key",
        "idempotency_key_missing",
        "Idempotency-Key is required for projection resolution",
    )?;
    let granted_capabilities = header_values(headers, "x-focusa-capabilities");
    if request
        .capabilities
        .iter()
        .any(|capability| !granted_capabilities.contains(capability))
    {
        return Err(error(
            StatusCode::FORBIDDEN,
            "capability_context_mismatch",
            "The generated capability context exceeds the authenticated capability set",
        ));
    }
    let granted_permissions = header_values(headers, "x-focusa-permissions");
    if request
        .permissions
        .iter()
        .any(|permission| !granted_permissions.contains(permission))
    {
        return Err(error(
            StatusCode::FORBIDDEN,
            "permission_context_mismatch",
            "The generated permission context exceeds the authenticated permission set",
        ));
    }
    let expected_projection_revision = required_if_match_revision(headers)?;
    let previous = store.get_projection(&request.scope).map_err(store_error)?;
    let (previous_layout_revision, event_cursor, previously_eligible) =
        if let Some(projection) = previous.as_ref() {
            (
                projection.layout_revision,
                projection.durable_event_cursor.clone(),
                projection
                    .eligible_contributions
                    .iter()
                    .map(|contribution| contribution.contribution_id.clone())
                    .collect::<BTreeSet<_>>(),
            )
        } else {
            (
                0,
                format!(
                    "event:{}",
                    store
                        .latest_event_sequence(&request.scope)
                        .map_err(store_error)?
                ),
                BTreeSet::new(),
            )
        };
    let projection_revision = expected_projection_revision.checked_add(1).ok_or_else(|| {
        error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "projection_revision_overflow",
            "projection revision exceeds the supported range",
        )
    })?;
    let observed_at = match request.observed_at {
        Some(value) if DateTime::parse_from_rfc3339(&value).is_ok() => value,
        Some(_) => {
            return Err(error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "observed_at_invalid",
                "observed_at must be an RFC3339 timestamp",
            ));
        }
        None => Utc::now().to_rfc3339(),
    };
    let candidates = resolver_candidates(
        store,
        &request.scope,
        &request.workspace_profile_id,
        &request.activity_mode_id,
    )?;
    let eligibility = EligibilityContext {
        scope: request.scope.clone(),
        profile_id: request.workspace_profile_id.clone(),
        activity_mode_id: request.activity_mode_id.clone(),
        projection_revision,
        capabilities: granted_capabilities,
        permissions: granted_permissions,
        available_operations: request.available_operations.into_iter().collect(),
        meaningful_content: BTreeMap::new(),
        previously_eligible,
        observed_at,
    };
    Ok(ResolveProjectionInput {
        candidates,
        eligibility,
        workspace_profile_revision: request.workspace_profile_revision,
        activity_mode_revision: request.activity_mode_revision,
        focused_work_surface_id: request
            .focused_work_surface_id
            .map(|value| value.to_string()),
        canonical_read_model_revision: request.canonical_read_model_revision,
        viewport_width: request.viewport.css_width,
        viewport_height: request.viewport.css_height,
        viewport_class: request.viewport.class,
        focused_semantic_target: String::new(),
        previous_projection_revision: expected_projection_revision,
        previous_layout_revision,
        event_cursor,
        causation_id: Some(idempotency_key.clone()),
        idempotency_key,
    })
}

fn validate_viewport(
    viewport: &ContributionViewportRequest,
) -> Result<(), (StatusCode, Json<Value>)> {
    if viewport.css_width < 1024
        || viewport.css_height < 720
        || !viewport.device_pixel_ratio.is_finite()
        || !(1.0..=4.0).contains(&viewport.device_pixel_ratio)
    {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "viewport_invalid",
            "viewport dimensions and device pixel ratio are outside the generated bounds",
        ));
    }
    if !matches!(
        viewport.class.as_str(),
        "minimum" | "compact" | "standard" | "productive" | "wide" | "reference_capture"
    ) {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "viewport_class_invalid",
            "viewport class is not in the generated contract",
        ));
    }
    if !matches!(viewport.platform.as_str(), "macOS" | "Windows" | "Linux") {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "viewport_platform_invalid",
            "viewport platform is not in the generated contract",
        ));
    }
    if viewport
        .text_scale_percent
        .is_some_and(|value| !(100..=200).contains(&value))
        || viewport
            .zoom_percent
            .is_some_and(|value| !matches!(value, 100 | 125 | 150 | 200))
    {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "viewport_scale_invalid",
            "viewport text scale or zoom is not in the generated contract",
        ));
    }
    Ok(())
}

fn resolver_candidates(
    store: &MissionCanvasStore,
    scope: &MissionCanvasScope,
    profile_id: &str,
    activity_mode_id: &str,
) -> Result<Vec<CandidateContribution>, (StatusCode, Json<Value>)> {
    let profile_ids = resolver_profile_candidate_ids(store, scope, profile_id)?;
    let activity_ids = resolver_activity_candidate_ids(store, scope, activity_mode_id)?;
    let documents = store
        .list_documents("mission_canvas_registry_entries", scope)
        .map_err(store_error)?;
    let mut candidates = BTreeMap::new();
    for document in documents {
        let candidate_value = document
            .payload
            .get("candidate")
            .cloned()
            .unwrap_or_else(|| document.payload.clone());
        if candidate_value.get("contribution_id").is_none() {
            continue;
        }
        let candidate: CandidateContribution =
            serde_json::from_value(candidate_value).map_err(|_| {
                error(
                    StatusCode::CONFLICT,
                    "projection_catalog_invalid",
                    "A canonical candidate registry entry is malformed",
                )
            })?;
        candidates.insert(candidate.contribution_id.clone(), candidate);
    }
    let selected_ids = profile_ids
        .intersection(&activity_ids)
        .filter(|id| candidates.contains_key(*id))
        .cloned()
        .collect::<Vec<_>>();
    if selected_ids.is_empty() {
        return Err(error(
            StatusCode::CONFLICT,
            "projection_catalog_missing",
            "No canonical candidates are available for the requested profile and activity",
        ));
    }
    Ok(selected_ids
        .into_iter()
        .filter_map(|id| candidates.remove(&id))
        .collect())
}

fn resolver_profile_candidate_ids(
    store: &MissionCanvasStore,
    scope: &MissionCanvasScope,
    profile_id: &str,
) -> Result<BTreeSet<String>, (StatusCode, Json<Value>)> {
    let profile = store
        .get_document(
            "mission_canvas_profiles",
            scope,
            &format!("profile:{profile_id}"),
        )
        .map_err(store_error)?
        .map(|document| document.payload)
        .or_else(|| {
            focusa_core::mission_canvas::CompositionRegistry::builtin()
                .profiles
                .get(profile_id)
                .and_then(|profile| serde_json::to_value(profile).ok())
        })
        .ok_or_else(|| {
            error(
                StatusCode::CONFLICT,
                "profile_not_found",
                "The requested workspace profile is not in the canonical registry",
            )
        })?;
    let profile: focusa_core::mission_canvas::WorkspaceProfileDefinition =
        serde_json::from_value(profile).map_err(|_| {
            error(
                StatusCode::CONFLICT,
                "profile_invalid",
                "The canonical workspace profile is malformed",
            )
        })?;
    Ok(profile.candidate_contribution_ids.into_iter().collect())
}

fn resolver_activity_candidate_ids(
    store: &MissionCanvasStore,
    scope: &MissionCanvasScope,
    activity_mode_id: &str,
) -> Result<BTreeSet<String>, (StatusCode, Json<Value>)> {
    let activity = store
        .get_document(
            "mission_canvas_activity_modes",
            scope,
            &format!("activity:{activity_mode_id}"),
        )
        .map_err(store_error)?
        .map(|document| document.payload)
        .or_else(|| {
            focusa_core::mission_canvas::CompositionRegistry::builtin()
                .activities
                .get(activity_mode_id)
                .and_then(|activity| serde_json::to_value(activity).ok())
        })
        .ok_or_else(|| {
            error(
                StatusCode::CONFLICT,
                "activity_mode_not_found",
                "The requested activity mode is not in the canonical registry",
            )
        })?;
    let activity: focusa_core::mission_canvas::ActivityModeDefinition =
        serde_json::from_value(activity).map_err(|_| {
            error(
                StatusCode::CONFLICT,
                "activity_mode_invalid",
                "The canonical activity mode is malformed",
            )
        })?;
    Ok(activity.candidate_contribution_ids.into_iter().collect())
}

fn required_header(
    headers: &HeaderMap,
    name: &str,
    code: &'static str,
    message: &'static str,
) -> Result<String, (StatusCode, Json<Value>)> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| error(StatusCode::UNPROCESSABLE_ENTITY, code, message))
}

fn required_if_match_revision(headers: &HeaderMap) -> Result<u64, (StatusCode, Json<Value>)> {
    let raw = required_header(
        headers,
        "if-match",
        "if_match_revision_missing",
        "If-Match is required for projection resolution",
    )?;
    let value = raw.trim_matches('"');
    let value = value.strip_prefix("revision:").unwrap_or(value);
    value.parse::<u64>().map_err(|_| {
        error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "if_match_revision_invalid",
            "If-Match must contain a non-negative projection revision",
        )
    })
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
            event_id: format!(
                "projection-event:{event_kind}:{}:{}",
                input.eligibility.scope.workstream.storage_key(),
                input.idempotency_key
            ),
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
                "projection-event:registry:{}:{}:{}",
                input.eligibility.scope.workstream.storage_key(),
                candidate.contribution_id,
                input.idempotency_key
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
            "projection-event:{event_kind}:{}:{}:{}",
            scope.workstream.storage_key(),
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
    require_permission_with_state(&state, &headers, "mission_canvas:write")?;
    validate_authority(&request.scope)?;
    let context =
        exact_workstream_context(&request.scope, &headers).map_err(host_renderer_context_error)?;
    let store = store(&state)?;
    let current = store
        .get_projection(&request.scope)
        .map_err(store_error)?
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "projection_not_found",
                "No projection exists for profile selection in this exact Workstream",
            )
        })?;
    current
        .validate_scope(&request.scope)
        .map_err(|reason| error(StatusCode::CONFLICT, "projection_scope_invalid", reason))?;

    // Generated mutation controls are transport-owned.  The body copies them
    // only because the generated client keeps operation extensions in its
    // request object; the authenticated headers remain authoritative and must
    // agree exactly.
    let header_idempotency_key = required_header(
        &headers,
        "idempotency-key",
        "idempotency_key_missing",
        "Idempotency-Key is required for profile selection",
    )?;
    if header_idempotency_key != request.idempotency_key {
        return Err(error(
            StatusCode::CONFLICT,
            "idempotency_key_mismatch",
            "The generated request body and Idempotency-Key header disagree",
        ));
    }
    let header_revision = required_if_match_revision(&headers)?;
    if header_revision != request.expected_projection_revision {
        return Err(error(
            StatusCode::CONFLICT,
            "projection_revision_conflict",
            "The generated request body and If-Match revision disagree",
        ));
    }
    if request
        .event_cursor
        .as_deref()
        .is_some_and(|cursor| cursor != current.durable_event_cursor)
    {
        return Err(error(
            StatusCode::CONFLICT,
            "projection_cursor_conflict",
            "The durable event cursor is stale",
        ));
    }

    // A same-key retry is a replay of the already accepted Core event.  A key
    // reused for another revision or operation is never interpreted as a new
    // profile choice.
    if let Some(event) = store
        .events_after(&request.scope, 0, 10_000)
        .map_err(store_error)?
        .into_iter()
        .map(|(_, event)| event)
        .find(|event| event.causation_id.as_deref() == Some(request.idempotency_key.as_str()))
    {
        if event.event_kind == "profile_changed"
            && event.projection_revision == current.projection_revision
            && event
                .payload
                .pointer("/profile_selection/profile_id")
                .and_then(Value::as_str)
                == Some(request.selection_id.as_str())
        {
            return Ok(Json(serde_json::to_value(current).map_err(json_error)?));
        }
        return Err(error(
            StatusCode::CONFLICT,
            "idempotency_conflict",
            "The idempotency key belongs to a different Mission Canvas mutation",
        ));
    }
    if current.projection_revision != request.expected_projection_revision {
        return Err(error(
            StatusCode::CONFLICT,
            "projection_revision_conflict",
            "Projection revision is stale",
        ));
    }

    // The following helpers only read canonical documents from this exact
    // Workstream.  Profile eligibility, composition, layout, Evidence, and
    // Receipt construction remain inside ProfileSelectionService in Core.
    let profile = profile_for_selection(&store, &request.scope, &request.selection_id)?;
    let activity = profile_list_activity(&store, &request.scope, &current.activity_mode_id)?;
    let candidates = selection_candidates(&store, &request.scope)?;
    let permissions = permission_context(&headers, token_enabled(&state))
        .list()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let available_operations = current
        .operation_bindings
        .iter()
        .filter_map(|binding| binding.get("operation_id").and_then(Value::as_str))
        .map(str::to_owned)
        .chain(
            current
                .eligible_contributions
                .iter()
                .flat_map(|contribution| contribution.operation_ids.iter().cloned()),
        )
        .collect::<BTreeSet<_>>();
    let result = ProfileSelectionService
        .select(ProfileSelectionCommand {
            context,
            scope: request.scope.clone(),
            current_projection: current,
            profile,
            activity,
            candidates,
            capabilities: header_values(&headers, "x-focusa-capabilities"),
            permissions,
            available_operations,
            expected_projection_revision: request.expected_projection_revision,
            expected_event_cursor: request.event_cursor,
            idempotency_key: request.idempotency_key,
        })
        .map_err(profile_selection_error)?;
    store
        .put_projection(
            &result.projection,
            Some(request.expected_projection_revision),
            &result.event,
        )
        .map_err(store_error)?;
    // The generated response is the direct ResolvedWorkspaceProjection.  Its
    // receipt_refs/evidence_refs point at the Core-owned result carried by the
    // durable event; no route-local wrapper is allowed to reach Desktop.
    Ok(Json(
        serde_json::to_value(result.projection).map_err(json_error)?,
    ))
}

fn profile_for_selection(
    store: &MissionCanvasStore,
    scope: &MissionCanvasScope,
    profile_id: &str,
) -> Result<WorkspaceProfileDefinition, (StatusCode, Json<Value>)> {
    if profile_id.trim().is_empty() {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "profile_selection_invalid",
            "selection_id must name a workspace profile",
        ));
    }
    let payload = store
        .get_document(
            "mission_canvas_profiles",
            scope,
            &format!("profile:{profile_id}"),
        )
        .map_err(store_error)?
        .map(|document| document.payload)
        .or_else(|| {
            CompositionRegistry::builtin()
                .profiles
                .get(profile_id)
                .and_then(|profile| serde_json::to_value(profile).ok())
        })
        .ok_or_else(|| {
            error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "profile_selection_unknown",
                "The selected workspace profile is not in the canonical registry",
            )
        })?;
    let profile: WorkspaceProfileDefinition = serde_json::from_value(payload).map_err(|_| {
        error(
            StatusCode::CONFLICT,
            "profile_catalog_invalid",
            "The selected canonical workspace profile is malformed",
        )
    })?;
    if profile.profile_id != profile_id {
        return Err(error(
            StatusCode::CONFLICT,
            "profile_catalog_identity_mismatch",
            "The canonical profile document does not match the selected profile",
        ));
    }
    Ok(profile)
}

fn selection_candidates(
    store: &MissionCanvasStore,
    scope: &MissionCanvasScope,
) -> Result<Vec<CandidateContribution>, (StatusCode, Json<Value>)> {
    let mut candidates = Vec::new();
    for document in store
        .list_documents("mission_canvas_registry_entries", scope)
        .map_err(store_error)?
    {
        let candidate_value = document
            .payload
            .get("candidate")
            .cloned()
            .unwrap_or_else(|| document.payload.clone());
        if candidate_value.get("contribution_id").is_none() {
            continue;
        }
        let candidate = serde_json::from_value(candidate_value).map_err(|_| {
            error(
                StatusCode::CONFLICT,
                "profile_selection_catalog_invalid",
                "A canonical candidate contribution is malformed",
            )
        })?;
        candidates.push(candidate);
    }
    Ok(candidates)
}

fn profile_selection_error(error_value: ProfileSelectionError) -> (StatusCode, Json<Value>) {
    let message = error_value.to_string();
    match error_value {
        ProfileSelectionError::Context(context_error) => host_renderer_context_error(context_error),
        ProfileSelectionError::Scope(reason) => {
            error(StatusCode::CONFLICT, "workstream_identity_mismatch", reason)
        }
        ProfileSelectionError::PermissionDenied(permission) => error(
            StatusCode::FORBIDDEN,
            "permission_denied",
            &format!("Missing required permission: {permission}"),
        ),
        ProfileSelectionError::IdempotencyKeyRequired => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "idempotency_key_missing",
            &message,
        ),
        ProfileSelectionError::RevisionConflict | ProfileSelectionError::CursorConflict => error(
            StatusCode::CONFLICT,
            "projection_revision_conflict",
            &message,
        ),
        ProfileSelectionError::ProfileUnavailable(_) => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "profile_selection_unavailable",
            &message,
        ),
        ProfileSelectionError::ActivityUnavailable(_) => {
            error(StatusCode::CONFLICT, "activity_catalog_invalid", &message)
        }
        ProfileSelectionError::NoMeaningfulContribution => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "composition_not_viable",
            &message,
        ),
        ProfileSelectionError::Recomposition(_) => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "profile_recomposition_failed",
            &message,
        ),
    }
}

async fn select_activity(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<CompositionSelectionRequest>,
) -> ApiResult {
    require_permission_with_state(&state, &headers, "mission_canvas:write")?;
    validate_authority(&request.scope)?;
    let context =
        exact_workstream_context(&request.scope, &headers).map_err(host_renderer_context_error)?;
    let store = store(&state)?;
    let current = store
        .get_projection(&request.scope)
        .map_err(store_error)?
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "projection_not_found",
                "No projection exists for activity selection in this exact Workstream",
            )
        })?;
    current
        .validate_scope(&request.scope)
        .map_err(|reason| error(StatusCode::CONFLICT, "projection_scope_invalid", reason))?;

    // Idempotency and optimistic concurrency controls are generated mutation
    // metadata. The authenticated headers must agree with their body mirrors;
    // neither value is inferred from the selected activity or the current tab.
    let header_idempotency_key = required_header(
        &headers,
        "idempotency-key",
        "idempotency_key_missing",
        "Idempotency-Key is required for activity selection",
    )?;
    if header_idempotency_key != request.idempotency_key {
        return Err(error(
            StatusCode::CONFLICT,
            "idempotency_key_mismatch",
            "The generated request body and Idempotency-Key header disagree",
        ));
    }
    let header_revision = required_if_match_revision(&headers)?;
    if header_revision != request.expected_projection_revision {
        return Err(error(
            StatusCode::CONFLICT,
            "projection_revision_conflict",
            "The generated request body and If-Match revision disagree",
        ));
    }
    if request
        .event_cursor
        .as_deref()
        .is_some_and(|cursor| cursor != current.durable_event_cursor)
    {
        return Err(error(
            StatusCode::CONFLICT,
            "projection_cursor_conflict",
            "The durable event cursor is stale",
        ));
    }

    // Replay only the exact prior activity mutation. A key reused for another
    // activity, operation, or revision is a conflict, never a local fallback.
    if let Some(event) = store
        .events_after(&request.scope, 0, 10_000)
        .map_err(store_error)?
        .into_iter()
        .map(|(_, event)| event)
        .find(|event| event.causation_id.as_deref() == Some(request.idempotency_key.as_str()))
    {
        let selected_activity = event
            .payload
            .pointer("/activity_selection/activity_mode_id")
            .and_then(Value::as_str)
            .or_else(|| {
                event
                    .payload
                    .get("activity_mode_id")
                    .and_then(Value::as_str)
            });
        if event.event_kind == "activity_mode_changed"
            && event.projection_revision == current.projection_revision
            && selected_activity == Some(request.selection_id.as_str())
        {
            return Ok(Json(serde_json::to_value(current).map_err(json_error)?));
        }
        return Err(error(
            StatusCode::CONFLICT,
            "idempotency_conflict",
            "The idempotency key belongs to a different Mission Canvas mutation",
        ));
    }
    if current.projection_revision != request.expected_projection_revision {
        return Err(error(
            StatusCode::CONFLICT,
            "projection_revision_conflict",
            "Projection revision is stale",
        ));
    }

    // These helpers read only canonical documents from the exact Workstream.
    // Activity applicability, eligibility, layout, Evidence, and Receipt stay
    // in ActivitySelectionService rather than in this HTTP adapter.
    let profile = profile_for_selection(&store, &request.scope, &current.workspace_profile_id)?;
    let activity = activity_for_selection(&store, &request.scope, &request.selection_id)?;
    let candidates = selection_candidates(&store, &request.scope)?;
    let permissions = permission_context(&headers, token_enabled(&state))
        .list()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let available_operations = current
        .operation_bindings
        .iter()
        .filter_map(|binding| binding.get("operation_id").and_then(Value::as_str))
        .map(str::to_owned)
        .chain(
            current
                .eligible_contributions
                .iter()
                .flat_map(|contribution| contribution.operation_ids.iter().cloned()),
        )
        .collect::<BTreeSet<_>>();
    let result = ActivitySelectionService
        .select(ActivitySelectionCommand {
            context,
            scope: request.scope.clone(),
            current_projection: current,
            profile,
            activity,
            candidates,
            capabilities: header_values(&headers, "x-focusa-capabilities"),
            permissions,
            available_operations,
            expected_projection_revision: request.expected_projection_revision,
            expected_event_cursor: request.event_cursor,
            idempotency_key: request.idempotency_key,
        })
        .map_err(activity_selection_error)?;
    store
        .put_projection(
            &result.projection,
            Some(request.expected_projection_revision),
            &result.event,
        )
        .map_err(store_error)?;
    // The generated response is the direct Core-owned projection. Evidence and
    // Receipt references remain inside that DTO; no route-local wrapper reaches
    // MissionCanvasClient.activitySelect or the trusted recursive renderer.
    Ok(Json(
        serde_json::to_value(result.projection).map_err(json_error)?,
    ))
}

fn activity_selection_error(error_value: ActivitySelectionError) -> (StatusCode, Json<Value>) {
    let message = error_value.to_string();
    match error_value {
        ActivitySelectionError::Context(context_error) => {
            host_renderer_context_error(context_error)
        }
        ActivitySelectionError::Scope(reason) => {
            error(StatusCode::CONFLICT, "workstream_identity_mismatch", reason)
        }
        ActivitySelectionError::PermissionDenied(permission) => error(
            StatusCode::FORBIDDEN,
            "permission_denied",
            &format!("Missing required permission: {permission}"),
        ),
        ActivitySelectionError::IdempotencyKeyRequired => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "idempotency_key_missing",
            &message,
        ),
        ActivitySelectionError::RevisionConflict => error(
            StatusCode::CONFLICT,
            "projection_revision_conflict",
            &message,
        ),
        ActivitySelectionError::CursorConflict => {
            error(StatusCode::CONFLICT, "projection_cursor_conflict", &message)
        }
        ActivitySelectionError::ProfileUnavailable(_) => error(
            StatusCode::CONFLICT,
            "activity_selection_profile_invalid",
            &message,
        ),
        ActivitySelectionError::ActivityUnavailable(_) => {
            error(StatusCode::CONFLICT, "activity_catalog_invalid", &message)
        }
        ActivitySelectionError::NoMeaningfulContribution => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "composition_not_viable",
            &message,
        ),
        ActivitySelectionError::Recomposition(_) => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "activity_recomposition_failed",
            &message,
        ),
    }
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
    require_permission(&headers, "mission_canvas:read")?;
    let scope = query.scope()?;
    // Profile availability is an authority-bearing read: the actor and
    // authority envelope must be extracted for this exact Workstream before
    // Core-owned projection eligibility is consulted.
    exact_workstream_context(&scope, &headers).map_err(host_renderer_context_error)?;
    let store = store(&state)?;
    let Some(projection) = store.get_projection(&scope).map_err(store_error)? else {
        // No canonical projection means no profile can truthfully claim a
        // meaningful eligible projection. Do not expose a dead selector.
        return Ok(Json(Value::Array(Vec::new())));
    };
    projection
        .validate_scope(&scope)
        .map_err(|reason| error(StatusCode::CONFLICT, "projection_scope_invalid", reason))?;

    let activity = profile_list_activity(&store, &scope, &projection.activity_mode_id)?;
    let mut profiles = CompositionRegistry::builtin()
        .profiles
        .into_values()
        .collect::<Vec<_>>();
    for document in store
        .list_documents("mission_canvas_profiles", &scope)
        .map_err(store_error)?
    {
        let profile: WorkspaceProfileDefinition = serde_json::from_value(document.payload)
            .map_err(|_| {
                error(
                    StatusCode::CONFLICT,
                    "profile_catalog_invalid",
                    "A canonical workspace profile is malformed",
                )
            })?;
        profiles.retain(|candidate| candidate.profile_id != profile.profile_id);
        profiles.push(profile);
    }
    let eligible_contribution_ids = projection
        .eligible_contributions
        .iter()
        .map(|contribution| contribution.contribution_id.clone())
        .collect::<BTreeSet<_>>();
    let viable = focusa_core::mission_canvas::profiles::meaningful_profiles_for_projection(
        &profiles,
        &activity,
        &eligible_contribution_ids,
    );
    Ok(Json(serde_json::to_value(viable).map_err(json_error)?))
}

fn registered_profile(
    store: &MissionCanvasStore,
    scope: &MissionCanvasScope,
    profile_id: &str,
) -> Result<WorkspaceProfileDefinition, (StatusCode, Json<Value>)> {
    if profile_id.trim().is_empty() {
        return Err(error(
            StatusCode::NOT_FOUND,
            "profile_not_found",
            "The requested workspace profile is not in the canonical registry",
        ));
    }

    let payload = store
        .get_document(
            "mission_canvas_profiles",
            scope,
            &format!("profile:{profile_id}"),
        )
        .map_err(store_error)?
        .map(|document| document.payload)
        .or_else(|| {
            CompositionRegistry::builtin()
                .profiles
                .get(profile_id)
                .and_then(|profile| serde_json::to_value(profile).ok())
        })
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "profile_not_found",
                "The requested workspace profile is not in the canonical registry",
            )
        })?;

    let profile: WorkspaceProfileDefinition = serde_json::from_value(payload).map_err(|_| {
        error(
            StatusCode::CONFLICT,
            "profile_catalog_invalid",
            "The canonical workspace profile is malformed",
        )
    })?;
    if profile.profile_id != profile_id {
        return Err(error(
            StatusCode::CONFLICT,
            "profile_catalog_identity_mismatch",
            "The canonical profile document does not match the requested profile",
        ));
    }
    Ok(profile)
}

fn profile_list_activity(
    store: &MissionCanvasStore,
    scope: &MissionCanvasScope,
    activity_mode_id: &str,
) -> Result<ActivityModeDefinition, (StatusCode, Json<Value>)> {
    if let Some(document) = store
        .get_document(
            "mission_canvas_activity_modes",
            scope,
            &format!("activity:{activity_mode_id}"),
        )
        .map_err(store_error)?
    {
        let activity: ActivityModeDefinition =
            serde_json::from_value(document.payload).map_err(|_| {
                error(
                    StatusCode::CONFLICT,
                    "activity_catalog_invalid",
                    "The canonical activity mode is malformed",
                )
            })?;
        if activity.activity_mode_id != activity_mode_id {
            return Err(error(
                StatusCode::CONFLICT,
                "activity_catalog_identity_mismatch",
                "The canonical activity document does not match the requested activity",
            ));
        }
        return Ok(activity);
    }

    CompositionRegistry::builtin()
        .activities
        .get(activity_mode_id)
        .cloned()
        .ok_or_else(|| {
            error(
                StatusCode::CONFLICT,
                "activity_mode_not_found",
                "The current projection references an unknown activity mode",
            )
        })
}

fn activity_for_selection(
    store: &MissionCanvasStore,
    scope: &MissionCanvasScope,
    activity_mode_id: &str,
) -> Result<ActivityModeDefinition, (StatusCode, Json<Value>)> {
    if activity_mode_id.trim().is_empty() {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "activity_selection_invalid",
            "selection_id must name an activity mode",
        ));
    }
    let activity = profile_list_activity(store, scope, activity_mode_id)?;
    if activity.activity_mode_id != activity_mode_id
        || activity.display_name.trim().is_empty()
        || activity.viability_rule_revision.trim().is_empty()
    {
        return Err(error(
            StatusCode::CONFLICT,
            "activity_catalog_invalid",
            "The selected canonical activity mode is malformed or has mismatched identity",
        ));
    }
    Ok(activity)
}

async fn get_profile(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(profile_id): Path<String>,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    require_permission(&headers, "mission_canvas:read")?;
    let scope = query.scope()?;
    // Profile metadata is an authority-bearing read just like profile.list:
    // establish the actor and canonical authority for this exact Workstream
    // before consulting the profile registry or returning any DTO.
    exact_workstream_context(&scope, &headers).map_err(host_renderer_context_error)?;
    let profile = registered_profile(&store(&state)?, &scope, &profile_id)?;
    // The generated response is the exact WorkspaceProfile DTO.  Do not leak
    // the StoredDocument envelope or allow a client to infer registry state
    // from persistence metadata.
    Ok(Json(serde_json::to_value(profile).map_err(json_error)?))
}

fn registered_activity_modes(
    store: &MissionCanvasStore,
    scope: &MissionCanvasScope,
) -> Result<Vec<ActivityModeDefinition>, (StatusCode, Json<Value>)> {
    let mut activities = CompositionRegistry::builtin().activities;
    for document in store
        .list_documents("mission_canvas_activity_modes", scope)
        .map_err(store_error)?
    {
        let activity: ActivityModeDefinition =
            serde_json::from_value(document.payload).map_err(|_| {
                error(
                    StatusCode::CONFLICT,
                    "activity_catalog_invalid",
                    "A canonical activity mode is malformed",
                )
            })?;
        if activity.activity_mode_id.trim().is_empty()
            || activity.display_name.trim().is_empty()
            || activity.viability_rule_revision.trim().is_empty()
        {
            return Err(error(
                StatusCode::CONFLICT,
                "activity_catalog_invalid",
                "A canonical activity mode is missing required registry identity or viability metadata",
            ));
        }
        if document.document_id != format!("activity:{}", activity.activity_mode_id) {
            return Err(error(
                StatusCode::CONFLICT,
                "activity_catalog_identity_mismatch",
                "The canonical activity document does not match its registered activity identity",
            ));
        }
        activities.insert(activity.activity_mode_id.clone(), activity);
    }
    Ok(activities.into_values().collect())
}

async fn list_activities(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    require_permission(&headers, "mission_canvas:read")?;
    let scope = query.scope()?;
    // Activity availability is an authority-bearing read.  Establish the
    // authenticated actor and canonical authority for this exact Workstream
    // before reading either the projection or the activity registry.
    exact_workstream_context(&scope, &headers).map_err(host_renderer_context_error)?;
    let store = store(&state)?;
    let Some(projection) = store.get_projection(&scope).map_err(store_error)? else {
        // Without a canonical projection there is no Core-owned evidence that
        // any activity can produce meaningful content.  An empty array is the
        // truthful generated ActivityMode[] result; no activity is inferred.
        return Ok(Json(Value::Array(Vec::new())));
    };
    projection
        .validate_scope(&scope)
        .map_err(|reason| error(StatusCode::CONFLICT, "projection_scope_invalid", reason))?;
    let profile = registered_profile(&store, &scope, &projection.workspace_profile_id)?;
    let activities = registered_activity_modes(&store, &scope)?;
    let eligible_contribution_ids = projection
        .eligible_contributions
        .iter()
        .map(|contribution| contribution.contribution_id.clone())
        .collect::<BTreeSet<_>>();
    // Core owns registered applicability, meaningful-content eligibility, and
    // adaptive composition.  Desktop receives direct generated ActivityMode
    // DTOs, never persistence envelopes or a client-local activity resolver.
    let viable = focusa_core::mission_canvas::profiles::meaningful_activities_for_projection(
        &activities,
        &profile,
        &eligible_contribution_ids,
    );
    Ok(Json(serde_json::to_value(viable).map_err(json_error)?))
}

fn registered_registry_entries(
    store: &MissionCanvasStore,
    scope: &MissionCanvasScope,
    registry_kind: &str,
) -> Result<Vec<RegistryDefinition>, (StatusCode, Json<Value>)> {
    if !matches!(
        registry_kind,
        "WorkspaceProfileRegistry"
            | "ActivityModeRegistry"
            | "PanelRegistry"
            | "HomeCanvasRegistry"
            | "WorkSurfaceRendererRegistry"
            | "ArtifactRendererRegistry"
            | "TerminologyRegistry"
            | "DomainSemanticBindingRegistry"
    ) {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "registry_kind_unknown",
            "The requested registry kind is not supported",
        ));
    }
    let mut registry = CompositionRegistry::builtin();
    let entries = match registry_kind {
        "PanelRegistry" => &mut registry.panels,
        "HomeCanvasRegistry" => &mut registry.home_canvases,
        "WorkSurfaceRendererRegistry" => &mut registry.work_surface_renderers,
        "ArtifactRendererRegistry" => &mut registry.artifact_renderers,
        "TerminologyRegistry" => &mut registry.terminology,
        "DomainSemanticBindingRegistry" => &mut registry.domain_semantics,
        _ => return Ok(Vec::new()),
    };
    for document in store
        .list_documents("mission_canvas_registry_entries", scope)
        .map_err(store_error)?
    {
        let entry: RegistryDefinition = serde_json::from_value(document.payload).map_err(|_| {
            error(
                StatusCode::CONFLICT,
                "registry_catalog_invalid",
                "A canonical registry entry document is malformed",
            )
        })?;
        if entry.registry_kind != registry_kind {
            continue;
        }
        if entry.entry_id.trim().is_empty()
            || entry.schema_ref.trim().is_empty()
            || entry.payload_ref.trim().is_empty()
            || entry.revision == 0
        {
            return Err(error(
                StatusCode::CONFLICT,
                "registry_catalog_invalid",
                "A canonical registry entry is missing required metadata",
            ));
        }
        if document.document_id != format!("registry:{}", entry.entry_id) {
            return Err(error(
                StatusCode::CONFLICT,
                "registry_catalog_identity_mismatch",
                "The canonical registry document does not match its registered entry identity",
            ));
        }
        entries.insert(entry.entry_id.clone(), entry);
    }
    Ok(entries.values().cloned().collect())
}

async fn list_registry(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(registry_kind): Path<String>,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    require_permission(&headers, "mission_canvas:read")?;
    let scope = query.scope()?;
    // Registry list is authority-bearing.  Establish the exact Workstream
    // before reading profile/activity/bundle registries.
    exact_workstream_context(&scope, &headers).map_err(host_renderer_context_error)?;
    let entries = registered_registry_entries(&store(&state)?, &scope, &registry_kind)?;
    Ok(Json(serde_json::to_value(entries).map_err(json_error)?))
}

async fn get_layout_memory(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    // Layout memory is a generated read operation, not a generic document
    // read. Establish permission and the exact Workstream actor/authority
    // before selecting the profile-specific document. The profile/activity/
    // viewport tuple is a selector, never an alternate ownership key.
    require_permission(&headers, "mission_canvas:read")?;
    let scope = query.scope()?;
    validate_authority(&scope)?;
    exact_workstream_context(&scope, &headers).map_err(host_renderer_context_error)?;
    let (profile_id, activity_mode_id, viewport_class) = layout_memory_selector(&query)?;
    let document_id = layout_memory_document_id(&profile_id, &activity_mode_id, &viewport_class);
    let document = store(&state)?
        .get_document("mission_canvas_layout_memory", &scope, &document_id)
        .map_err(store_error)?
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "layout_memory_not_found",
                "No layout memory exists for this exact Workstream and profile",
            )
        })?;
    let memory: ProfileLayoutMemory = serde_json::from_value(document.payload).map_err(|_| {
        error(
            StatusCode::CONFLICT,
            "layout_memory_invalid",
            "The canonical layout memory document is malformed",
        )
    })?;
    validate_profile_layout_memory(
        &memory,
        &scope,
        &profile_id,
        &activity_mode_id,
        &viewport_class,
    )
    .map_err(layout_memory_validation_error)?;
    // The generated operation returns the direct ProfileLayoutMemory DTO. Do
    // not leak StoredDocument revision/table metadata or invent a wrapper.
    Ok(Json(serde_json::to_value(memory).map_err(json_error)?))
}

fn layout_memory_selector(
    query: &ScopeQuery,
) -> Result<(String, String, String), (StatusCode, Json<Value>)> {
    let profile_id = required_layout_memory_selector(
        query.profile_id.as_deref(),
        "profile_id",
        "layout_memory_profile_missing",
    )?;
    let activity_mode_id = required_layout_memory_selector(
        query.activity_mode_id.as_deref(),
        "activity_mode_id",
        "layout_memory_activity_missing",
    )?;
    let viewport_class = required_layout_memory_selector(
        query.viewport_class.as_deref(),
        "viewport_class",
        "layout_memory_viewport_missing",
    )?;
    if !matches!(
        viewport_class.as_str(),
        "minimum" | "compact" | "standard" | "productive" | "wide" | "reference_capture"
    ) {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "layout_memory_viewport_invalid",
            "viewport_class is not in the generated ProfileLayoutMemory contract",
        ));
    }
    Ok((profile_id, activity_mode_id, viewport_class))
}

fn required_layout_memory_selector(
    value: Option<&str>,
    field: &'static str,
    code: &'static str,
) -> Result<String, (StatusCode, Json<Value>)> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            error(
                StatusCode::UNPROCESSABLE_ENTITY,
                code,
                &format!("{field} is required"),
            )
        })
}

fn layout_memory_document_id(
    profile_id: &str,
    activity_mode_id: &str,
    viewport_class: &str,
) -> String {
    format!("layout-memory:{profile_id}:{activity_mode_id}:{viewport_class}")
}

fn layout_memory_validation_error(reason: &'static str) -> (StatusCode, Json<Value>) {
    let code = match reason {
        "scope_mismatch" => "layout_memory_scope_mismatch",
        "profile_mismatch" => "layout_memory_profile_mismatch",
        "activity_mode_mismatch" => "layout_memory_activity_mismatch",
        "viewport_class_mismatch" => "layout_memory_viewport_mismatch",
        "memory_id_mismatch" => "layout_memory_id_mismatch",
        "idempotency_key_missing" => "layout_memory_idempotency_missing",
        "placement_invalid"
        | "placement_duplicate"
        | "absent_contribution_invalid"
        | "placement_absent_overlap"
        | "updated_at_invalid" => "layout_memory_content_invalid",
        "attachment_missing" => "layout_memory_attachment_missing",
        _ => "layout_memory_invalid",
    };
    let status = if reason == "attachment_missing" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::CONFLICT
    };
    error(
        status,
        code,
        "The layout memory does not match the exact generated scope and profile",
    )
}

fn layout_memory_update_error(error_value: LayoutMemoryUpdateError) -> (StatusCode, Json<Value>) {
    let message = error_value.to_string();
    match error_value {
        LayoutMemoryUpdateError::Context(context_error) => {
            host_renderer_context_error(context_error)
        }
        LayoutMemoryUpdateError::Scope(reason) | LayoutMemoryUpdateError::MemoryInvalid(reason) => {
            layout_memory_validation_error(reason)
        }
        LayoutMemoryUpdateError::PermissionDenied(permission) => error(
            StatusCode::FORBIDDEN,
            "permission_denied",
            &format!("Missing required permission: {permission}"),
        ),
        LayoutMemoryUpdateError::IdempotencyKeyRequired => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "idempotency_key_missing",
            &message,
        ),
        LayoutMemoryUpdateError::RevisionConflict { .. } => error(
            StatusCode::CONFLICT,
            "layout_memory_revision_conflict",
            &message,
        ),
        LayoutMemoryUpdateError::RevisionOverflow => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "layout_memory_revision_overflow",
            &message,
        ),
        LayoutMemoryUpdateError::Serialization(serialization_error) => {
            json_error(serialization_error)
        }
        LayoutMemoryUpdateError::Store(store_error) => {
            let (status, code) = match &store_error {
                focusa_core::mission_canvas::MissionCanvasStoreError::RevisionConflict { .. } => {
                    (StatusCode::CONFLICT, "layout_memory_revision_conflict")
                }
                focusa_core::mission_canvas::MissionCanvasStoreError::LayoutMemoryIdempotencyConflict => {
                    (StatusCode::CONFLICT, "idempotency_conflict")
                }
                focusa_core::mission_canvas::MissionCanvasStoreError::InvalidLayoutMemory(_) => {
                    (StatusCode::CONFLICT, "layout_memory_invalid")
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "mission_canvas_store_error"),
            };
            error(status, code, &store_error.to_string())
        }
    }
}

async fn put_layout_memory(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(memory): Json<ProfileLayoutMemory>,
) -> ApiResult {
    // This is the generated layout_memory.update adapter. It intentionally
    // does not accept DocumentWriteRequest: ProfileLayoutMemory is the
    // published request DTO and Core owns its semantic identity/revision.
    require_permission_with_state(&state, &headers, LAYOUT_MEMORY_UPDATE_PERMISSION)?;
    validate_authority(&memory.scope)?;
    let context =
        exact_workstream_context(&memory.scope, &headers).map_err(host_renderer_context_error)?;
    let header_idempotency_key = required_header(
        &headers,
        "idempotency-key",
        "idempotency_key_missing",
        "Idempotency-Key is required for layout-memory update",
    )?;
    if header_idempotency_key != memory.idempotency_key {
        return Err(error(
            StatusCode::CONFLICT,
            "idempotency_key_mismatch",
            "The generated ProfileLayoutMemory and Idempotency-Key header disagree",
        ));
    }
    let expected_memory_revision = required_if_match_revision(&headers)?;
    let permissions = permission_context(&headers, token_enabled(&state))
        .list()
        .into_iter()
        .collect();
    let receipt = LayoutMemoryUpdateService
        .update(
            &store(&state)?,
            &LayoutMemoryUpdateCommand {
                context,
                scope: memory.scope.clone(),
                memory,
                expected_memory_revision,
                permissions,
            },
        )
        .map_err(layout_memory_update_error)?;
    // Return the direct generated RecompositionReceipt. StoredDocument and
    // route-local receipt wrappers are not transport contracts.
    Ok(Json(serde_json::to_value(receipt).map_err(json_error)?))
}

async fn mutate_layout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(command): Json<LayoutMutationCommand>,
) -> ApiResult {
    // The route is only a generated transport adapter. Permission, exact
    // Workstream extraction, and header/body mutation controls are established
    // before Core is allowed to inspect a layout or contribution ID.
    require_permission_with_state(&state, &headers, "mission_canvas:write")?;
    validate_authority(&command.scope)?;
    if command.scope.attachment.is_none() {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "attachment_missing",
            "Layout mutation requires an exact generated AttachmentKey",
        ));
    }
    let context =
        exact_workstream_context(&command.scope, &headers).map_err(host_renderer_context_error)?;
    let header_idempotency_key = required_header(
        &headers,
        "idempotency-key",
        "idempotency_key_missing",
        "Idempotency-Key is required for layout mutation",
    )?;
    if header_idempotency_key != command.idempotency_key {
        return Err(error(
            StatusCode::CONFLICT,
            "idempotency_key_mismatch",
            "The generated LayoutMutationCommand and Idempotency-Key header disagree",
        ));
    }
    let header_revision = required_if_match_revision(&headers)?;
    if header_revision != command.expected_projection_revision {
        return Err(error(
            StatusCode::CONFLICT,
            "projection_revision_conflict",
            "The generated command and If-Match revision disagree",
        ));
    }
    let permissions = permission_context(&headers, token_enabled(&state))
        .list()
        .into_iter()
        .collect();
    let result = LayoutMutationService
        .mutate(
            &store(&state)?,
            LayoutMutationExecution {
                context,
                command,
                permissions,
            },
        )
        .map_err(layout_mutation_error)?;
    // LayoutMutationResult is the direct generated DTO. The canonical layout
    // is read through projection.get; Desktop never receives or fabricates a
    // competing layout wrapper or local projection.
    Ok(Json(serde_json::to_value(result).map_err(json_error)?))
}

fn layout_mutation_error(error_value: LayoutMutationError) -> (StatusCode, Json<Value>) {
    let message = error_value.to_string();
    match error_value {
        LayoutMutationError::Context(context_error) => {
            host_renderer_context_error(context_error)
        }
        LayoutMutationError::Scope(reason) => {
            error(StatusCode::CONFLICT, "workstream_identity_mismatch", reason)
        }
        LayoutMutationError::PermissionDenied(permission) => error(
            StatusCode::FORBIDDEN,
            "permission_denied",
            &format!("Missing required permission: {permission}"),
        ),
        LayoutMutationError::CommandInvalid(reason) => {
            error(StatusCode::UNPROCESSABLE_ENTITY, "layout_command_invalid", reason)
        }
        LayoutMutationError::ProjectionNotFound => error(
            StatusCode::NOT_FOUND,
            "projection_not_found",
            "No projection exists for this exact Workstream",
        ),
        LayoutMutationError::RevisionConflict => error(
            StatusCode::CONFLICT,
            "layout_revision_conflict",
            &message,
        ),
        LayoutMutationError::IdempotencyConflict => {
            error(StatusCode::CONFLICT, "idempotency_conflict", &message)
        }
        LayoutMutationError::UnknownContribution(contribution) => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "unknown_contribution_id",
            &format!("Unknown canonical contribution ID: {contribution}"),
        ),
        LayoutMutationError::UnknownWorkSurface(surface) => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "unknown_work_surface_id",
            &format!("Unknown canonical Work Surface: {surface}"),
        ),
        LayoutMutationError::NotApplicable => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "layout_mutation_not_applicable",
            &message,
        ),
        LayoutMutationError::UnsupportedAction(action) => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "layout_action_unsupported",
            &format!("Canonical layout does not expose action: {action}"),
        ),
        LayoutMutationError::InvalidLayout(reason) => error(
            StatusCode::CONFLICT,
            "canonical_layout_invalid",
            &reason,
        ),
        LayoutMutationError::RevisionOverflow => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "layout_revision_overflow",
            &message,
        ),
        LayoutMutationError::Serialization(serialization_error) => {
            json_error(serialization_error)
        }
        LayoutMutationError::Store(stored_error) => match stored_error {
            focusa_core::mission_canvas::MissionCanvasStoreError::RevisionConflict { .. }
            | focusa_core::mission_canvas::MissionCanvasStoreError::LayoutMutationIdempotencyConflict => {
                error(StatusCode::CONFLICT, "layout_revision_conflict", &message)
            }
            other => store_error(other),
        },
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
    exact_workstream_context(scope, headers)
}

fn exact_workstream_context(
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

fn host_lifecycle_error(error_value: HostLifecycleError) -> (StatusCode, Json<Value>) {
    match error_value {
        HostLifecycleError::Context(context_error)
        | HostLifecycleError::Resolution(HostRendererResolutionError::Context(context_error)) => {
            host_renderer_context_error(context_error)
        }
        HostLifecycleError::Scope(reason)
        | HostLifecycleError::Resolution(HostRendererResolutionError::Scope(reason)) => {
            error(StatusCode::UNPROCESSABLE_ENTITY, "scope_incomplete", reason)
        }
        HostLifecycleError::CapabilityUnavailable(capability)
        | HostLifecycleError::Resolution(HostRendererResolutionError::CapabilityUnavailable(
            capability,
        )) => error(
            StatusCode::FORBIDDEN,
            "capability_unavailable",
            &format!("Missing required host capability: {capability}"),
        ),
        HostLifecycleError::PermissionDenied(permission) => error(
            StatusCode::FORBIDDEN,
            "permission_denied",
            &format!("Missing required permission: {permission}"),
        ),
        HostLifecycleError::IdempotencyKeyRequired => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "idempotency_key_missing",
            "idempotency_key is required for rich-host lifecycle mutation",
        ),
        HostLifecycleError::PresentationNotFound => error(
            StatusCode::NOT_FOUND,
            "host_presentation_not_found",
            "No existing Desktop presentation exists for this exact Workstream",
        ),
        HostLifecycleError::PresentationUnavailable(reason) => error(
            StatusCode::CONFLICT,
            "host_presentation_unavailable",
            &format!("Existing Desktop presentation cannot be focused: {reason}"),
        ),
        HostLifecycleError::RendererUnavailable(reason) => {
            error(StatusCode::FORBIDDEN, "host_renderer_unavailable", &reason)
        }
        HostLifecycleError::IdempotencyConflict => error(
            StatusCode::CONFLICT,
            "idempotency_conflict",
            "Idempotency key belongs to a different rich-host lifecycle action",
        ),
        HostLifecycleError::InvalidDocument(reason) => {
            error(StatusCode::CONFLICT, "host_lifecycle_invalid", &reason)
        }
        HostLifecycleError::Store(store) => store_error(store),
        HostLifecycleError::Serialization(serialization_error) => json_error(serialization_error),
    }
}

async fn launch_host(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RichHostCommandRequest>,
) -> ApiResult {
    require_permission_with_state(&state, &headers, "mission_canvas:host")?;
    validate_authority(&request.scope)?;
    let context = host_renderer_workstream_context(&request.scope, &headers)
        .map_err(host_renderer_context_error)?;
    let permissions = permission_context(&headers, token_enabled(&state))
        .list()
        .into_iter()
        .collect();
    let command = HostLifecycleLaunchCommand {
        context,
        scope: request.scope,
        idempotency_key: request.idempotency_key,
        capabilities: header_values(&headers, "x-focusa-capabilities"),
        permissions,
    };
    let lifecycle: HostLifecycleState = HostLifecycleService
        .launch(&store(&state)?, &command, HostPlatform::current())
        .map_err(host_lifecycle_error)?;
    Ok(Json(serde_json::to_value(lifecycle).map_err(json_error)?))
}

async fn focus_host(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RichHostCommandRequest>,
) -> ApiResult {
    require_permission_with_state(&state, &headers, "mission_canvas:host")?;
    validate_authority(&request.scope)?;
    let context = host_renderer_workstream_context(&request.scope, &headers)
        .map_err(host_renderer_context_error)?;
    let permissions = permission_context(&headers, token_enabled(&state))
        .list()
        .into_iter()
        .collect();
    let command = HostLifecycleFocusCommand {
        context,
        scope: request.scope,
        idempotency_key: request.idempotency_key,
        capabilities: header_values(&headers, "x-focusa-capabilities"),
        permissions,
    };
    let lifecycle: HostLifecycleState = HostLifecycleService
        .focus(&store(&state)?, &command, HostPlatform::current())
        .map_err(host_lifecycle_error)?;
    Ok(Json(serde_json::to_value(lifecycle).map_err(json_error)?))
}

async fn hide_host(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RichHostCommandRequest>,
) -> ApiResult {
    require_permission_with_state(&state, &headers, "mission_canvas:host")?;
    validate_authority(&request.scope)?;
    let context = host_renderer_workstream_context(&request.scope, &headers)
        .map_err(host_renderer_context_error)?;
    let permissions = permission_context(&headers, token_enabled(&state))
        .list()
        .into_iter()
        .collect();
    let command = HostLifecycleHideCommand {
        context,
        scope: request.scope,
        idempotency_key: request.idempotency_key,
        capabilities: header_values(&headers, "x-focusa-capabilities"),
        permissions,
    };
    let lifecycle: HostLifecycleState = HostLifecycleService
        .hide(&store(&state)?, &command, HostPlatform::current())
        .map_err(host_lifecycle_error)?;
    Ok(Json(serde_json::to_value(lifecycle).map_err(json_error)?))
}

async fn close_host(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RichHostCommandRequest>,
) -> ApiResult {
    require_permission_with_state(&state, &headers, "mission_canvas:host")?;
    validate_authority(&request.scope)?;
    let context = host_renderer_workstream_context(&request.scope, &headers)
        .map_err(host_renderer_context_error)?;
    let permissions = permission_context(&headers, token_enabled(&state))
        .list()
        .into_iter()
        .collect();
    let command = HostLifecycleCloseCommand {
        context,
        scope: request.scope,
        idempotency_key: request.idempotency_key,
        capabilities: header_values(&headers, "x-focusa-capabilities"),
        permissions,
    };
    let lifecycle: HostLifecycleState = HostLifecycleService
        .close(&store(&state)?, &command, HostPlatform::current())
        .map_err(host_lifecycle_error)?;
    Ok(Json(serde_json::to_value(lifecycle).map_err(json_error)?))
}

async fn get_draft(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(draft_id): Path<String>,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    require_permission(&headers, "mission_canvas:draft")?;
    let scope = query.scope()?;
    validate_authority(&scope)?;
    exact_workstream_context(&scope, &headers).map_err(host_renderer_context_error)?;
    let store = store(&state)?;
    match store.load_draft(&scope, &draft_id).map_err(store_error)? {
        Some(draft) => Ok(Json(draft)),
        None => Err(error(
            StatusCode::NOT_FOUND,
            "draft_not_found",
            "No CanvasDraftState exists for this exact scope and draft",
        )),
    }
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
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RecipientResolveRequest>,
) -> ApiResult {
    require_permission(&headers, "mission_canvas:draft")?;
    validate_authority(&request.scope)?;
    exact_workstream_context(&request.scope, &headers).map_err(host_renderer_context_error)?;
    let recipient_ref = request.recipient_ref.trim().to_owned();
    if recipient_ref.is_empty() {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "recipient_missing",
            "recipient_ref is required",
        ));
    }
    let parts: Vec<&str> = recipient_ref.splitn(3, ':').collect();
    if parts.len() != 3 || parts[0] != "recipient" || parts[1].is_empty() || parts[2].is_empty() {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "recipient_invalid",
            "recipient_ref must be recipient:{kind}:{id}",
        ));
    }
    let scope = &request.scope;
    let routable = match parts[1] {
        // Aggregate Workstream recipients are always routable under the exact scope.
        "workstream" => true,
        // Surface-bound session recipients require exact Attachment + Work Surface
        // authority; the recipient id must match the bound session.
        "session" => {
            if scope.attachment.is_none() || scope.work_surface_id.is_none() {
                return Err(error(
                    StatusCode::FORBIDDEN,
                    "recipient_blocked",
                    "session recipient requires exact Attachment and Work Surface authority",
                ));
            }
            let session_id = scope.attachment.as_ref().and_then(|attachment| {
                serde_json::to_value(attachment)
                    .ok()
                    .and_then(|value| value.get("session_id").and_then(serde_json::Value::as_str).map(str::to_owned))
            });
            session_id.as_deref() == Some(parts[2])
        }
        _ => {
            return Err(error(
                StatusCode::FORBIDDEN,
                "recipient_blocked",
                "recipient kind is not authorized under the exact scope",
            ));
        }
    };
    if !routable {
        return Err(error(
            StatusCode::FORBIDDEN,
            "recipient_blocked",
            "recipient is not authorized under the exact scope",
        ));
    }
    Ok(Json(json!({
        "schema": "focusa.mission_canvas.recipient_resolution.v1",
        "workstream": scope.workstream,
        "continuity_id": scope.continuity_id,
        "attachment": scope.attachment,
        "workspace_binding_id": scope.workspace_binding_id,
        "runtime_object": scope.runtime_object,
        "work_surface_id": scope.work_surface_id,
        "recipient_ref": recipient_ref,
        "routable": true
    })))
}

async fn get_recomposition_diagnostics(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(revision): Path<u64>,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    require_permission(&headers, "mission_canvas:read")?;
    let scope = query.scope()?;
    validate_authority(&scope)?;
    exact_workstream_context(&scope, &headers).map_err(host_renderer_context_error)?;
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
                "No recomposition diagnostics exist for this revision",
            )
        })?;
    let diagnostics = event
        .payload
        .get("omission_diagnostics")
        .cloned()
        .unwrap_or_else(|| json!([]));
    if !diagnostics.is_array() {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "recomposition_diagnostics_invalid",
            "Stored omission diagnostics are not a list",
        ));
    }
    Ok(Json(diagnostics))
}

async fn get_recomposition_receipt(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(revision): Path<u64>,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    require_permission(&headers, "mission_canvas:read")?;
    let scope = query.scope()?;
    validate_authority(&scope)?;
    exact_workstream_context(&scope, &headers).map_err(host_renderer_context_error)?;
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
                "No recomposition receipt exists for this revision",
            )
        })?;
    let receipt = event
        .payload
        .get("receipt")
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "recomposition_receipt_missing",
                "The resolved projection event carries no receipt payload",
            )
        })?;
    let parsed: focusa_core::mission_canvas::reducer::RecompositionReceipt =
        serde_json::from_value(receipt.clone()).map_err(json_error)?;
    if parsed.scope.workstream != scope.workstream {
        return Err(error(
            StatusCode::CONFLICT,
            "recomposition_scope_mismatch",
            "Stored recomposition receipt belongs to a different Workstream",
        ));
    }
    if parsed.projection_revision != revision {
        return Err(error(
            StatusCode::CONFLICT,
            "recomposition_revision_mismatch",
            "Stored recomposition receipt does not match the requested projection revision",
        ));
    }
    Ok(Json(receipt.clone()))
}

async fn get_recomposition_evidence(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(revision): Path<u64>,
    Query(query): Query<ScopeQuery>,
) -> ApiResult {
    require_permission(&headers, "mission_canvas:read")?;
    let scope = query.scope()?;
    validate_authority(&scope)?;
    exact_workstream_context(&scope, &headers).map_err(host_renderer_context_error)?;
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
                "No recomposition evidence exists for this revision",
            )
        })?;
    let evidence = event
        .payload
        .get("evidence")
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "recomposition_evidence_missing",
                "The resolved projection event carries no evidence payload",
            )
        })?;
    let parsed: focusa_core::mission_canvas::reducer::RecompositionEvidence =
        serde_json::from_value(evidence.clone()).map_err(json_error)?;
    if parsed.scope.workstream != scope.workstream {
        return Err(error(
            StatusCode::CONFLICT,
            "recomposition_scope_mismatch",
            "Stored recomposition evidence belongs to a different Workstream",
        ));
    }
    let expected_suffix = format!(":{revision}");
    if !parsed.evidence_id.ends_with(&expected_suffix) {
        return Err(error(
            StatusCode::CONFLICT,
            "recomposition_revision_mismatch",
            "Stored recomposition evidence does not match the requested projection revision",
        ));
    }
    Ok(Json(evidence.clone()))
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
            profile_id: None,
            activity_mode_id: None,
            viewport_class: None,
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
}
