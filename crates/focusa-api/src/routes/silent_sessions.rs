use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::silent_sessions::{
    AuthorizationTarget, AuthorizedProjection, ContextAuthorityVerdict, RunGeneration,
    SilentSession, SilentSessionAction, SilentSessionAuthorizationRequest, SilentSessionConfig,
    SilentSessionId, SilentSessionLifecycle, SilentSessionRole, SilentSessionRouteScope,
    SilentSessionRun, SilentSessionRunId, VerifiedAuthorityFacts, authorize_silent_session_action,
    list_sessions, load_config_revision, load_retention_record, load_run, load_session,
    load_session_events, save_authorization_principal,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    middleware::principal::{ApiRequestPrincipal, request_principal},
    server::AppState,
};

use super::silent_sessions_contract::{
    ApiSideEffect, ExactSessionRunTarget, RetryDirective, SilentSessionApiEnvelope,
    guard_exact_target,
};

pub(super) type ApiResponse = (StatusCode, Json<SilentSessionApiEnvelope<Value>>);

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/v1/silent-sessions/preflight",
            post(super::silent_sessions_create::preflight),
        )
        .route(
            "/v1/silent-sessions",
            get(list).post(super::silent_sessions_create::create),
        )
        .route(
            "/v1/silent-sessions/{session_id}",
            get(show).delete(super::silent_sessions_retention::ordinary_delete),
        )
        .route("/v1/silent-sessions/{session_id}/status", get(status))
        .route(
            "/v1/silent-sessions/{session_id}/start",
            post(super::silent_sessions_lifecycle::start),
        )
        .route(
            "/v1/silent-sessions/{session_id}/approvals",
            post(super::silent_sessions_approvals::create),
        )
        .route(
            "/v1/silent-sessions/{session_id}/pause",
            post(super::silent_sessions_control::pause),
        )
        .route(
            "/v1/silent-sessions/{session_id}/resume",
            post(super::silent_sessions_control::resume),
        )
        .route(
            "/v1/silent-sessions/{session_id}/interrupt",
            post(super::silent_sessions_control::interrupt),
        )
        .route(
            "/v1/silent-sessions/{session_id}/cancel",
            post(super::silent_sessions_control::cancel),
        )
        .route(
            "/v1/silent-sessions/{session_id}/restart",
            post(super::silent_sessions_restart::restart),
        )
        .route(
            "/v1/silent-sessions/{session_id}/adopt",
            post(super::silent_sessions_adopt::adopt),
        )
        .route(
            "/v1/silent-sessions/{session_id}/events",
            get(super::silent_sessions_observe::events),
        )
        .route(
            "/v1/silent-sessions/{session_id}/output",
            get(super::silent_sessions_observe::output),
        )
        .merge(super::silent_sessions_projection::router())
        .merge(super::silent_sessions_input::router())
        .merge(super::silent_sessions_config_read::router())
        .merge(super::silent_sessions_config_mutation::router())
        .merge(super::silent_sessions_capabilities::router())
        .merge(super::silent_sessions_retention::router())
}

async fn list(State(state): State<Arc<AppState>>, headers: HeaderMap) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    if !principal
        .principal
        .scopes
        .contains(&SilentSessionRouteScope::Read)
    {
        return disclose_principal_side_effect(
            failure(
                StatusCode::FORBIDDEN,
                "forbidden",
                "authorization_denied",
                "The authenticated principal lacks silent_sessions:read.",
            ),
            &principal,
        );
    }
    let sessions = match list_sessions(&state.persistence) {
        Ok(sessions) => sessions,
        Err(error) => {
            return disclose_principal_side_effect(persistence_failure(error), &principal);
        }
    };
    let data = sessions
        .iter()
        .filter_map(|session| authorized_projection(&principal, session, SilentSessionAction::List))
        .collect::<Vec<_>>();
    success_with_principal("listed", json!(data), &principal)
}

async fn show(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    let session = match load_session(&state.persistence, session_id) {
        Ok(Some(session)) => session,
        Ok(None) => {
            return disclose_principal_side_effect(
                failure(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "not_found",
                    "No Silent Session exists for the requested session_id.",
                ),
                &principal,
            );
        }
        Err(error) => {
            return disclose_principal_side_effect(persistence_failure(error), &principal);
        }
    };
    match load_retention_record(&state.persistence, session_id) {
        Ok(Some(record)) if record.deleted_at.is_some() => {
            return disclose_principal_side_effect(
                failure(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "ordinary_delete_hidden",
                    "The Silent Session is no longer present in ordinary views.",
                ),
                &principal,
            );
        }
        Ok(_) => {}
        Err(error) => {
            return disclose_principal_side_effect(persistence_failure(error), &principal);
        }
    }
    match authorized_projection(&principal, &session, SilentSessionAction::Show) {
        Some(data) => success_with_principal("found", data, &principal),
        None => disclose_principal_side_effect(
            failure(
                StatusCode::FORBIDDEN,
                "forbidden",
                "authorization_denied",
                "The authenticated principal is not authorized for this Silent Session.",
            ),
            &principal,
        ),
    }
}

#[derive(Debug, Deserialize)]
struct StatusQuery {
    run_id: SilentSessionRunId,
    generation: RunGeneration,
}

async fn status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
    Query(query): Query<StatusQuery>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    let session = match load_session(&state.persistence, session_id) {
        Ok(Some(session)) => session,
        Ok(None) => {
            return disclose_principal_side_effect(
                failure(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "not_found",
                    "No Silent Session exists for the requested session_id.",
                ),
                &principal,
            );
        }
        Err(error) => {
            return disclose_principal_side_effect(persistence_failure(error), &principal);
        }
    };
    match load_retention_record(&state.persistence, session_id) {
        Ok(Some(record)) if record.deleted_at.is_some() => {
            return disclose_principal_side_effect(
                failure(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "ordinary_delete_hidden",
                    "The Silent Session is no longer present in ordinary views.",
                ),
                &principal,
            );
        }
        Ok(_) => {}
        Err(error) => {
            return disclose_principal_side_effect(persistence_failure(error), &principal);
        }
    }
    let run = match load_run(&state.persistence, query.run_id) {
        Ok(Some(run)) => run,
        Ok(None) => {
            return disclose_principal_side_effect(
                failure(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "run_not_found",
                    "Refresh the session and use its current run_id.",
                ),
                &principal,
            );
        }
        Err(error) => {
            return disclose_principal_side_effect(persistence_failure(error), &principal);
        }
    };
    if session.current_run_generation != query.generation {
        let mut response = failure(
            StatusCode::CONFLICT,
            "stale_target",
            "stale_generation",
            "Refresh status and retry with the current run generation.",
        );
        response.1.0.stale = true;
        response.1.0.misuse_hint = Some(format!(
            "requested generation {}; current generation {}",
            query.generation.get(),
            session.current_run_generation.get()
        ));
        return disclose_principal_side_effect(response, &principal);
    }
    if let Err(error) = guard_exact_target(
        ExactSessionRunTarget {
            session_id,
            run_id: query.run_id,
            generation: query.generation,
        },
        &run,
    ) {
        let mut response = failure(
            StatusCode::CONFLICT,
            "stale_target",
            "exact_target_mismatch",
            "Refresh status and retry with the current session_id, run_id and generation.",
        );
        response.1.0.stale = true;
        response.1.0.misuse_hint = Some(format!("exact target guard rejected: {error:?}"));
        return disclose_principal_side_effect(response, &principal);
    }
    let Some(session_projection) =
        authorized_projection(&principal, &session, SilentSessionAction::Show)
    else {
        return disclose_principal_side_effect(
            failure(
                StatusCode::FORBIDDEN,
                "forbidden",
                "authorization_denied",
                "The authenticated principal is not authorized for this Silent Session.",
            ),
            &principal,
        );
    };
    let run_projection = if session_projection.get("projection")
        == Some(&Value::String("redacted_summary".into()))
    {
        json!({
            "id": run.id,
            "silent_session_id": run.silent_session_id,
            "generation": run.generation,
            "started_at": run.started_at,
            "ended_at": run.ended_at,
            "projection": "redacted_summary"
        })
    } else {
        json!(run)
    };
    let config = load_config_revision(&state.persistence, run.config_revision_id)
        .ok()
        .flatten();
    let event_count = load_session_events(&state.persistence, session.id)
        .ok()
        .map(|events| events.len());
    let temporal_context = silent_session_temporal_context(
        &session,
        Some(&run),
        config.as_ref().map(|revision| &revision.config),
        event_count,
    );
    success_with_principal(
        "status",
        json!({"session": session_projection, "run": run_projection, "temporal_context": temporal_context}),
        &principal,
    )
}

fn silent_session_temporal_projection(
    session: &SilentSession,
) -> Result<focusa_core::temporal::TemporalProjection, Box<ApiResponse>> {
    let mut scope = focusa_core::temporal::TemporalScope::project(
        session.authority.project_root.clone(),
        session.authority.continuity_id.clone(),
    );
    scope.item_id = session.work_item_ref.clone();
    let ledger = focusa_core::temporal::TemporalLedger::for_project(scope.clone()).map_err(|_| {
        Box::new(failure(
            StatusCode::PRECONDITION_FAILED,
            "temporal_scope_unavailable",
            "temporal_scope_unavailable",
            "Repair the exact project and continuity scope before Silent Session temporal action.",
        ))
    })?;
    let events = ledger.read_all().map_err(|_| {
        Box::new(failure(
            StatusCode::PRECONDITION_FAILED,
            "temporal_ledger_unavailable",
            "temporal_ledger_unavailable",
            "Repair temporal ledger integrity before Silent Session temporal action.",
        ))
    })?;
    Ok(focusa_core::temporal::project_temporal(
        scope,
        &events,
        Utc::now(),
    ))
}

pub(super) fn silent_session_temporal_context(
    session: &SilentSession,
    run: Option<&SilentSessionRun>,
    config: Option<&SilentSessionConfig>,
    event_count: Option<usize>,
) -> Value {
    let projection = match silent_session_temporal_projection(session) {
        Ok(projection) => projection,
        Err(response) => {
            return json!({
                "schema":"focusa.silent_session_temporal_context.v1",
                "status":"unavailable",
                "canonical":false,
                "scope":{
                    "project_root":session.authority.project_root,
                    "continuity_id":session.authority.continuity_id,
                    "item_id":session.work_item_ref
                },
                "failure_class":response.1.0.failure_class,
                "recovery_hint":response.1.0.recovery_hint
            });
        }
    };
    let now = Utc::now();
    let elapsed_ms = run.map(|run| {
        run.ended_at
            .unwrap_or(now)
            .signed_duration_since(run.started_at)
            .num_milliseconds()
            .max(0) as u64
    });
    let max_wall_clock_ms = config
        .and_then(|config| config.resources.max_wall_clock_seconds)
        .map(|seconds| seconds.saturating_mul(1_000));
    let remaining_wall_clock_ms = max_wall_clock_ms
        .zip(elapsed_ms)
        .map(|(maximum, elapsed)| maximum.saturating_sub(elapsed));
    let timeout_state = match (
        run.and_then(|run| run.ended_at),
        max_wall_clock_ms,
        elapsed_ms,
    ) {
        (Some(_), _, _) => "settled",
        (None, Some(maximum), Some(elapsed)) if elapsed >= maximum => "exceeded",
        (None, Some(_), Some(_)) => "within_budget",
        _ => "not_configured",
    };
    let cancellation_state = match session.lifecycle {
        SilentSessionLifecycle::Cancelling => "requested",
        SilentSessionLifecycle::Cancelled => "acknowledged",
        _ => "not_requested",
    };
    let settlement_status = match session.lifecycle {
        SilentSessionLifecycle::Completed
        | SilentSessionLifecycle::Failed
        | SilentSessionLifecycle::Cancelled => "terminal_pending_receipt",
        _ => "pending",
    };
    json!({
        "schema":"focusa.silent_session_temporal_context.v1",
        "status":"completed",
        "canonical":true,
        "scope":projection.scope,
        "observed_at":now,
        "deadline_status":projection.deadline_status,
        "active_claim_ref":projection.active_commitment.as_ref().map(|claim| json!({
            "claim_id":claim.claim_id,"revision":claim.revision,"kind":claim.kind,"target_at":claim.target_at
        })),
        "forecast_range":projection.authorized_forecast_range,
        "human_calendar_context":projection.human_calendar_context,
        "temporal_priority_frame":projection.temporal_priority_frame,
        "temporal_execution_guard":projection.temporal_execution_guard,
        "urgency":projection.urgency,
        "warnings":projection.warnings.into_iter().take(8).collect::<Vec<_>>(),
        "run_timing":{
            "started_at":run.map(|run| run.started_at),
            "ended_at":run.and_then(|run| run.ended_at),
            "elapsed_ms":elapsed_ms,
            "max_wall_clock_ms":max_wall_clock_ms,
            "remaining_wall_clock_ms":remaining_wall_clock_ms,
            "timeout_state":timeout_state
        },
        "progress":{
            "event_count":event_count,
            "lifecycle":session.lifecycle,
            "health":session.health,
            "semantic_activity":session.semantic_activity,
            "session_updated_at":session.updated_at
        },
        "cancellation_state":cancellation_state,
        "settlement":{
            "status":settlement_status,
            "ended_at":run.and_then(|run| run.ended_at),
            "completion_receipt_required":config.map(|config| config.governance.completion_receipt_required),
            "receipt_refs":[]
        }
    })
}

pub(super) fn ensure_silent_session_temporal_guard(
    session: &SilentSession,
    action_ref: &str,
) -> Result<Value, Box<ApiResponse>> {
    let mut scope = focusa_core::temporal::TemporalScope::project(
        session.authority.project_root.clone(),
        session.authority.continuity_id.clone(),
    );
    scope.item_id = session.work_item_ref.clone();
    let events = focusa_core::temporal::TemporalLedger::for_project(scope.clone())
        .and_then(|ledger| ledger.read_all())
        .map_err(|error| {
            Box::new(failure(
                StatusCode::PRECONDITION_FAILED,
                "temporal_priority_unavailable",
                "temporal_priority_unavailable",
                "Check the scoped temporal ledger and retry with a fresh priority packet.",
            ))
        })?;
    let projection = focusa_core::temporal::project_temporal(scope.clone(), &events, Utc::now());
    let calendar = projection.human_calendar_context.as_ref().ok_or_else(|| {
        Box::new(failure(
            StatusCode::PRECONDITION_FAILED,
            "temporal_priority_missing",
            "temporal_priority_missing",
            "HumanCalendarContext is required for Silent Session mutation.",
        ))
    })?;
    let frame = projection.temporal_priority_frame.as_ref().ok_or_else(|| {
        Box::new(failure(
            StatusCode::PRECONDITION_FAILED,
            "temporal_priority_missing",
            "temporal_priority_missing",
            "TemporalPriorityFrame is required for Silent Session mutation.",
        ))
    })?;
    let guard = projection
        .temporal_execution_guard
        .as_ref()
        .ok_or_else(|| {
            Box::new(failure(
                StatusCode::PRECONDITION_FAILED,
                "temporal_guard_missing",
                "temporal_guard_missing",
                "TemporalExecutionGuard is required for Silent Session mutation.",
            ))
        })?;
    focusa_core::temporal_operations::authorize_temporal_action(
        calendar,
        frame,
        Some(guard),
        &scope,
        &frame.operator_ask_digest,
        action_ref,
        Utc::now(),
    )
    .map_err(|error| {
        Box::new(failure(
            StatusCode::PRECONDITION_FAILED,
            "temporal_guard_rejected",
            "temporal_guard_rejected",
            "Refresh the scoped priority frame and execution guard before retrying.",
        ))
    })?;
    Ok(json!({
        "schema":"focusa.silent_session_temporal_guard.v1",
        "status":"completed",
        "canonical":true,
        "scope":projection.scope,
        "action_ref":action_ref,
        "deadline_status":projection.deadline_status,
        "priority_frame_ref":frame.frame_id,
        "guard_ref":guard.guard_id,
        "receipt_ref":guard.receipt_ref,
        "urgency":projection.urgency
    }))
}

pub(super) async fn durable_request_principal(
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Result<ApiRequestPrincipal, Box<ApiResponse>> {
    let principal = request_principal(headers).await.ok_or_else(|| {
        Box::new(failure(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "authentication_required",
            "Present a valid admin or paired-device bearer token.",
        ))
    })?;
    save_authorization_principal(&state.persistence, &principal.principal, Utc::now())
        .map_err(|error| Box::new(persistence_failure(error)))?;
    Ok(principal)
}

pub(super) fn authorized_projection(
    request_principal: &ApiRequestPrincipal,
    session: &SilentSession,
    action: SilentSessionAction,
) -> Option<Value> {
    let principal = &request_principal.principal;
    let administrator = principal.role == SilentSessionRole::Administrator;
    let creator = !session.creator_principal_id.is_empty()
        && session.creator_principal_id == principal.principal_id;
    let controller = !session.controller_principal_id.is_empty()
        && session.controller_principal_id == principal.principal_id;
    let permission = creator || controller || administrator;
    let target = AuthorizationTarget {
        project_root: session.authority.project_root.clone(),
        continuity_id: session.authority.continuity_id.clone(),
        work_item_ref: session.work_item_ref.clone(),
        session_id: Some(session.id),
        run_id: None,
        owner_os_user: session.owner_os_user.clone(),
        writer_principal_id: None,
        config_hash: String::new(),
        model_binding: String::new(),
        workspace: session.authority.project_root.clone(),
    };
    let authority = VerifiedAuthorityFacts {
        project_permission: permission,
        continuity_permission: permission,
        work_item_permission: permission,
        writer_ownership: false,
        authorized_project_root: target.project_root.clone(),
        authorized_continuity_id: target.continuity_id.clone(),
        authorized_work_item_ref: target.work_item_ref.clone(),
        writer_principal_id: None,
        context_authority: ContextAuthorityVerdict::Allowed,
    };
    let decision = authorize_silent_session_action(&SilentSessionAuthorizationRequest {
        principal: principal.clone(),
        action,
        target,
        authority,
        approval: None,
        approval_durably_verified: false,
        legacy_approved: false,
        requested_side_effects: Vec::new(),
        now: Utc::now(),
    });
    match decision.projection? {
        AuthorizedProjection::Full => serde_json::to_value(session).ok(),
        AuthorizedProjection::RedactedSummary => Some(json!({
            "silent_session_schema_version": session.silent_session_schema_version,
            "id": session.id,
            "display_name": session.display_name,
            "lifecycle": session.lifecycle,
            "health": session.health,
            "semantic_activity": session.semantic_activity,
            "current_run_generation": session.current_run_generation,
            "created_at": session.created_at,
            "updated_at": session.updated_at,
            "projection": "redacted_summary"
        })),
        AuthorizedProjection::RawForensics => None,
    }
}

pub(super) fn success_with_principal(
    status: &str,
    data: Value,
    principal: &ApiRequestPrincipal,
) -> ApiResponse {
    disclose_principal_side_effect(
        (
            StatusCode::OK,
            Json(SilentSessionApiEnvelope::canonical(status, data)),
        ),
        principal,
    )
}

pub(super) fn disclose_principal_side_effect(
    mut response: ApiResponse,
    principal: &ApiRequestPrincipal,
) -> ApiResponse {
    response.1.0.side_effects.push(ApiSideEffect {
        effect: "authorization_principal_upsert".into(),
        status: "completed".into(),
        target_ref: Some(principal.principal.principal_id.clone()),
    });
    response
}

pub(super) fn persistence_failure(error: impl std::fmt::Display) -> ApiResponse {
    tracing::error!(error = %error, "Silent Session persistence operation failed");
    failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        "persistence_error",
        "persistence_failure",
        "Retry after checking daemon persistence health.",
    )
}

pub(super) fn failure(
    status_code: StatusCode,
    status: &str,
    failure_class: &str,
    recovery_hint: &str,
) -> ApiResponse {
    let mut envelope = SilentSessionApiEnvelope::failure(
        status,
        failure_class,
        RetryDirective {
            retryable: status_code.is_server_error(),
            after_ms: status_code.is_server_error().then_some(1_000),
            idempotency_key_required: false,
        },
    );
    envelope.recovery_hint = Some(recovery_hint.into());
    (status_code, Json(envelope))
}

#[cfg(test)]
mod tests {
    use super::*;
    use focusa_core::silent_sessions::{
        ActorInstanceId, ConfigRevisionId, HarnessConfig, HarnessKind, IdentityConfig, ModelConfig,
        ModelFallbackPolicy, ModelSelectionPolicy, NativeResumePolicy, ProtocolVersions,
        SILENT_SESSION_SCHEMA_VERSION, SilentSessionAuthority, SilentSessionLifecycle,
    };

    fn request_principal(role: SilentSessionRole, os_user: &str) -> ApiRequestPrincipal {
        ApiRequestPrincipal {
            principal: focusa_core::silent_sessions::AuthenticatedPrincipal {
                principal_id: "principal:test".into(),
                actor: "actor:test".into(),
                role,
                os_user: os_user.into(),
                scopes: [
                    SilentSessionRouteScope::Read,
                    SilentSessionRouteScope::Admin,
                ]
                .into_iter()
                .collect(),
                authenticated: true,
            },
            source: crate::middleware::principal::ApiPrincipalSource::AdminToken,
        }
    }

    fn owned_session(owner: &str) -> SilentSession {
        SilentSession::draft_owned(
            SilentSessionAuthority::new("/repo/focusa", "continuity:test").unwrap(),
            "principal:test",
            owner,
            "session",
            "secret mission",
            ConfigRevisionId::new(),
            Utc::now(),
        )
        .unwrap()
    }

    fn temporal_test_config(project_root: &str) -> SilentSessionConfig {
        SilentSessionConfig::new(
            IdentityConfig {
                display_name: "temporal-proof".into(),
                project_root: project_root.into(),
                continuity_id: "continuity:temporal-proof".into(),
                work_item_ref: Some("focusa-vbcqu.9.2.3.2".into()),
                mission: "prove silent session temporal context".into(),
                agent_identity_ref: "agent:pi".into(),
                role_profile_ref: None,
            },
            HarnessConfig {
                kind: HarnessKind::Pi,
                adapter_version: "1".into(),
                native_resume_policy: NativeResumePolicy::Prefer,
            },
            ModelConfig {
                provider: "test".into(),
                model: "test".into(),
                thinking: None,
                selection_policy: ModelSelectionPolicy::Exact,
                fallback_policy: ModelFallbackPolicy::Disabled,
                allowed_fallbacks: Vec::new(),
                auth_profile_ref: "auth:test".into(),
                require_entitlement_preflight: false,
                require_runtime_model_confirmation: false,
            },
        )
    }

    #[test]
    fn silent_session_temporal_context_is_scoped_bounded_and_tracks_timeout_settlement() {
        let root =
            std::env::temp_dir().join(format!("focusa-silent-temporal-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&root).unwrap();
        let authority =
            SilentSessionAuthority::new(root.display().to_string(), "continuity:temporal-proof")
                .unwrap();
        let mut session = SilentSession::draft_owned(
            authority,
            "principal:test",
            "wirebot",
            "temporal-proof",
            "prove temporal context",
            ConfigRevisionId::new(),
            Utc::now(),
        )
        .unwrap();
        session.lifecycle = SilentSessionLifecycle::Running;
        let mut run = SilentSessionRun {
            silent_session_schema_version: SILENT_SESSION_SCHEMA_VERSION,
            id: SilentSessionRunId::new(),
            silent_session_id: session.id,
            generation: session.current_run_generation,
            actor_instance_id: ActorInstanceId::new(),
            config_revision_id: session.active_config_revision_id,
            protocol_versions: ProtocolVersions::default(),
            started_at: Utc::now() - chrono::Duration::seconds(10),
            ended_at: None,
        };
        let mut config = temporal_test_config(root.to_string_lossy().as_ref());
        config.resources.max_wall_clock_seconds = Some(5);

        let running = silent_session_temporal_context(&session, Some(&run), Some(&config), Some(4));
        assert_eq!(running["status"], "completed");
        assert_eq!(running["scope"]["project_root"], root.display().to_string());
        assert_eq!(
            running["scope"]["continuity_id"],
            "continuity:temporal-proof"
        );
        assert_eq!(running["run_timing"]["timeout_state"], "exceeded");
        assert_eq!(running["cancellation_state"], "not_requested");
        assert_eq!(running["progress"]["event_count"], 4);
        assert!(running["urgency"].is_null());

        session.lifecycle = SilentSessionLifecycle::Cancelling;
        let cancelling =
            silent_session_temporal_context(&session, Some(&run), Some(&config), Some(5));
        assert_eq!(cancelling["cancellation_state"], "requested");

        session.lifecycle = SilentSessionLifecycle::Completed;
        run.ended_at = Some(Utc::now());
        let completed =
            silent_session_temporal_context(&session, Some(&run), Some(&config), Some(6));
        assert_eq!(completed["run_timing"]["timeout_state"], "settled");
        assert_eq!(
            completed["settlement"]["status"],
            "terminal_pending_receipt"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn silent_session_mutation_without_temporal_priority_fails_closed() {
        let root = std::env::temp_dir().join(format!(
            "focusa-silent-temporal-guard-{}",
            uuid::Uuid::now_v7()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let session = SilentSession::draft_owned(
            SilentSessionAuthority::new(root.display().to_string(), "continuity:guard-proof")
                .unwrap(),
            "principal:test",
            "wirebot",
            "guard-proof",
            "prove guard rejection",
            ConfigRevisionId::new(),
            Utc::now(),
        )
        .unwrap();

        let response = ensure_silent_session_temporal_guard(&session, "silent-session:start")
            .expect_err("missing temporal priority must block mutation");
        assert_eq!(response.0, StatusCode::PRECONDITION_FAILED);
        assert_eq!(
            response.1.0.failure_class.as_deref(),
            Some("temporal_priority_missing")
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn creator_receives_full_projection() {
        let session = owned_session("wirebot");
        let value = authorized_projection(
            &request_principal(SilentSessionRole::Operator, "wirebot"),
            &session,
            SilentSessionAction::Show,
        )
        .unwrap();
        assert_eq!(value["mission"], "secret mission");
    }

    #[test]
    fn cross_user_admin_receives_redacted_projection() {
        let session = owned_session("wirebot");
        let value = authorized_projection(
            &request_principal(SilentSessionRole::Administrator, "root"),
            &session,
            SilentSessionAction::Show,
        )
        .unwrap();
        assert_eq!(value["projection"], "redacted_summary");
        assert!(value.get("mission").is_none());
    }

    #[test]
    fn legacy_unknown_owner_never_receives_full_projection() {
        let mut session = owned_session("wirebot");
        session.creator_principal_id.clear();
        session.controller_principal_id.clear();
        session.owner_os_user.clear();
        let value = authorized_projection(
            &request_principal(SilentSessionRole::Administrator, "root"),
            &session,
            SilentSessionAction::Show,
        )
        .unwrap();
        assert_eq!(value["projection"], "redacted_summary");
        assert_eq!(session.lifecycle, SilentSessionLifecycle::Draft);
    }
}
