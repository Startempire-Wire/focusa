use std::{convert::Infallible, path::PathBuf, sync::Arc, time::Duration};

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
};
use chrono::Utc;
use focusa_core::silent_sessions::{
    AuthorizationTarget, ContextAuthorityVerdict, OutputChannel, RunGeneration, SecureStreamStore,
    SilentSession, SilentSessionAction, SilentSessionAuthorizationRequest, SilentSessionId,
    SilentSessionRole, SilentSessionRunId, StreamCursor, VerifiedAuthorityFacts,
    authorize_silent_session_action, load_run, load_session, load_session_events,
};
use serde::Deserialize;
use serde_json::json;

use crate::{middleware::principal::ApiRequestPrincipal, server::AppState};

use super::{
    silent_sessions::{
        ApiResponse, disclose_principal_side_effect, durable_request_principal, failure,
        persistence_failure,
    },
    silent_sessions_contract::{
        ApiSideEffect, ExactSessionRunTarget, RetryDirective, SilentSessionApiEnvelope,
        guard_exact_target, resume_sequence,
    },
};

#[derive(Debug, Deserialize)]
pub(super) struct StreamQuery {
    pub run_id: SilentSessionRunId,
    pub generation: RunGeneration,
}

#[derive(Debug, Deserialize)]
pub(super) struct OutputQuery {
    pub run_id: SilentSessionRunId,
    pub generation: RunGeneration,
    #[serde(default = "default_output_channel")]
    pub channel: OutputChannel,
}

pub(super) async fn events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
    Query(query): Query<StreamQuery>,
) -> Response {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return (*response).into_response(),
    };
    let session = match load_session(&state.persistence, session_id) {
        Ok(Some(session)) => session,
        Ok(None) => return after(not_found("session_id"), &principal).into_response(),
        Err(error) => return after(persistence_failure(error), &principal).into_response(),
    };
    let run = match load_run(&state.persistence, query.run_id) {
        Ok(Some(run)) => run,
        Ok(None) => return after(not_found("run_id"), &principal).into_response(),
        Err(error) => return after(persistence_failure(error), &principal).into_response(),
    };
    if session.current_run_generation != query.generation
        || guard_exact_target(
            ExactSessionRunTarget {
                session_id,
                run_id: query.run_id,
                generation: query.generation,
            },
            &run,
        )
        .is_err()
    {
        return after(stale_target(), &principal).into_response();
    }
    if let Err(response) = authorize_stream(&principal, &session) {
        return after(*response, &principal).into_response();
    }
    let last_event_id = headers
        .get("last-event-id")
        .and_then(|value| value.to_str().ok());
    let resume_after = match resume_sequence(last_event_id, query.run_id) {
        Ok(sequence) => sequence,
        Err(error) => {
            let mut response = failure(
                StatusCode::BAD_REQUEST,
                "invalid_cursor",
                "resume_cursor_invalid",
                "Discard Last-Event-ID and reconnect from the current exact target.",
            );
            response.1.0.misuse_hint = Some(format!("cursor rejected: {error:?}"));
            return after(response, &principal).into_response();
        }
    };
    let stream_state = state.clone();
    let run_id = query.run_id;
    let generation = query.generation;
    let principal_id = principal.principal.principal_id.clone();
    let stream = async_stream::stream! {
        let mut opened = SilentSessionApiEnvelope::canonical(
            "stream_open",
            json!({"session_id": session_id, "run_id": run_id, "generation": generation}),
        );
        opened.side_effects.push(ApiSideEffect {
            effect: "authorization_principal_upsert".into(),
            status: "completed".into(),
            target_ref: Some(principal_id),
        });
        yield Ok::<Event, Infallible>(Event::default()
            .event("silent_session_stream_open")
            .data(serde_json::to_string(&opened).unwrap_or_else(|_| "{\"ok\":false}".into())));
        let mut delivered = resume_after;
        loop {
            match load_session(&stream_state.persistence, session_id) {
                Ok(Some(current)) if current.current_run_generation == generation => {}
                Ok(Some(_)) | Ok(None) => {
                    yield Ok::<Event, Infallible>(sse_failure(
                        "stale_target",
                        "The session generation changed; refresh status before reconnecting.",
                    ));
                    break;
                }
                Err(_) => {
                    yield Ok::<Event, Infallible>(sse_failure(
                        "persistence_failure",
                        "Observation persistence is temporarily unavailable.",
                    ));
                    break;
                }
            }
            match load_session_events(&stream_state.persistence, session_id) {
                Ok(events) => {
                    for observed in events {
                        if observed.run_id != Some(run_id) || observed.sequence <= delivered {
                            continue;
                        }
                        delivered = observed.sequence;
                        let cursor = StreamCursor::new(run_id, observed.sequence)
                            .encode()
                            .unwrap_or_else(|_| observed.sequence.to_string());
                        let envelope = SilentSessionApiEnvelope::canonical("event", observed);
                        let data = serde_json::to_string(&envelope)
                            .unwrap_or_else(|_| "{\"ok\":false}".into());
                        yield Ok::<Event, Infallible>(Event::default()
                            .event("silent_session_event")
                            .id(cursor)
                            .data(data));
                    }
                }
                Err(_) => {
                    yield Ok::<Event, Infallible>(sse_failure(
                        "persistence_failure",
                        "Observation persistence is temporarily unavailable.",
                    ));
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    };
    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keepalive"),
        )
        .into_response()
}

pub(super) async fn output(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
    Query(query): Query<OutputQuery>,
) -> Response {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return (*response).into_response(),
    };
    let session = match load_session(&state.persistence, session_id) {
        Ok(Some(session)) => session,
        Ok(None) => return after(not_found("session_id"), &principal).into_response(),
        Err(error) => return after(persistence_failure(error), &principal).into_response(),
    };
    let run = match load_run(&state.persistence, query.run_id) {
        Ok(Some(run)) => run,
        Ok(None) => return after(not_found("run_id"), &principal).into_response(),
        Err(error) => return after(persistence_failure(error), &principal).into_response(),
    };
    if session.current_run_generation != query.generation
        || guard_exact_target(
            ExactSessionRunTarget {
                session_id,
                run_id: query.run_id,
                generation: query.generation,
            },
            &run,
        )
        .is_err()
    {
        return after(stale_target(), &principal).into_response();
    }
    if let Err(response) = authorize_stream(&principal, &session) {
        return after(*response, &principal).into_response();
    }
    let last_event_id = headers
        .get("last-event-id")
        .and_then(|value| value.to_str().ok());
    if let Err(error) = resume_sequence(last_event_id, query.run_id) {
        let mut response = failure(
            StatusCode::BAD_REQUEST,
            "invalid_cursor",
            "resume_cursor_invalid",
            "Discard Last-Event-ID and reconnect from the current exact target.",
        );
        response.1.0.misuse_hint = Some(format!("cursor rejected: {error:?}"));
        return after(response, &principal).into_response();
    }
    let configured = PathBuf::from(&state.config.data_dir);
    let data_root = if configured.is_absolute() {
        configured
    } else {
        match std::env::current_dir() {
            Ok(current) => current.join(configured),
            Err(error) => return after(persistence_failure(error), &principal).into_response(),
        }
    };
    let stream_root = data_root.join("silent-sessions");
    let store = match SecureStreamStore::new(stream_root.clone(), state.persistence.clone()) {
        Ok(store) => store,
        Err(error) => return after(persistence_failure(error), &principal).into_response(),
    };
    let run_id = query.run_id;
    let generation = query.generation;
    let channel = query.channel;
    let principal_id = principal.principal.principal_id.clone();
    let mut initial_cursor = last_event_id.map(ToOwned::to_owned);
    let stream_state = state.clone();
    let stream = async_stream::stream! {
        let mut opened = SilentSessionApiEnvelope::canonical(
            "stream_open",
            json!({
                "session_id": session_id,
                "run_id": run_id,
                "generation": generation,
                "channel": channel,
            }),
        );
        opened.side_effects.extend([
            ApiSideEffect {
                effect: "authorization_principal_upsert".into(),
                status: "completed".into(),
                target_ref: Some(principal_id),
            },
            ApiSideEffect {
                effect: "stream_store_root_ensure".into(),
                status: "completed".into(),
                target_ref: Some(stream_root.display().to_string()),
            },
        ]);
        yield Ok::<Event, Infallible>(Event::default()
            .event("silent_session_output_open")
            .data(serde_json::to_string(&opened).unwrap_or_else(|_| "{\"ok\":false}".into())));
        loop {
            match load_session(&stream_state.persistence, session_id) {
                Ok(Some(current)) if current.current_run_generation == generation => {}
                Ok(Some(_)) | Ok(None) => {
                    yield Ok::<Event, Infallible>(sse_failure(
                        "stale_target",
                        "The session generation changed; refresh status before reconnecting.",
                    ));
                    break;
                }
                Err(_) => {
                    yield Ok::<Event, Infallible>(sse_failure(
                        "persistence_failure",
                        "Observation persistence is temporarily unavailable.",
                    ));
                    break;
                }
            }
            match store.read_after(
                session_id,
                run_id,
                channel,
                initial_cursor.as_deref(),
                256,
            ) {
                Ok((events, next_cursor)) => {
                    for observed in events {
                        let cursor = StreamCursor::new(run_id, observed.seq)
                            .encode()
                            .unwrap_or_else(|_| observed.seq.to_string());
                        let envelope = SilentSessionApiEnvelope::canonical("output", observed);
                        let data = serde_json::to_string(&envelope)
                            .unwrap_or_else(|_| "{\"ok\":false}".into());
                        yield Ok::<Event, Infallible>(Event::default()
                            .event("silent_session_output")
                            .id(cursor)
                            .data(data));
                    }
                    initial_cursor = next_cursor;
                }
                Err(_) => {
                    yield Ok::<Event, Infallible>(sse_failure(
                        "stream_storage_failure",
                        "Verify secure stream storage and reconnect with the last valid cursor.",
                    ));
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    };
    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keepalive"),
        )
        .into_response()
}

fn default_output_channel() -> OutputChannel {
    OutputChannel::Stdout
}

fn authorize_stream(
    request_principal: &ApiRequestPrincipal,
    session: &SilentSession,
) -> Result<(), Box<ApiResponse>> {
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
        writer_principal_id: Some(session.controller_principal_id.clone()),
        config_hash: String::new(),
        model_binding: String::new(),
        workspace: session.authority.project_root.clone(),
    };
    let authority = VerifiedAuthorityFacts {
        project_permission: permission,
        continuity_permission: permission,
        work_item_permission: permission,
        writer_ownership: controller,
        authorized_project_root: target.project_root.clone(),
        authorized_continuity_id: target.continuity_id.clone(),
        authorized_work_item_ref: target.work_item_ref.clone(),
        writer_principal_id: controller.then(|| principal.principal_id.clone()),
        context_authority: ContextAuthorityVerdict::Allowed,
    };
    let decision = authorize_silent_session_action(&SilentSessionAuthorizationRequest {
        principal: principal.clone(),
        action: SilentSessionAction::FollowStream,
        target,
        authority,
        approval: None,
        approval_durably_verified: false,
        legacy_approved: false,
        requested_side_effects: Vec::new(),
        now: Utc::now(),
    });
    if decision.allowed {
        Ok(())
    } else {
        Err(Box::new(failure(
            StatusCode::FORBIDDEN,
            "forbidden",
            "authorization_denied",
            &decision.reason,
        )))
    }
}

fn sse_failure(class: &str, recovery: &str) -> Event {
    let mut envelope = SilentSessionApiEnvelope::<serde_json::Value>::failure(
        "stream_closed",
        class,
        RetryDirective {
            retryable: true,
            after_ms: Some(1_000),
            idempotency_key_required: false,
        },
    );
    envelope.recovery_hint = Some(recovery.into());
    Event::default()
        .event("silent_session_error")
        .data(serde_json::to_string(&envelope).unwrap_or_else(|_| "{\"ok\":false}".into()))
}

fn after(response: ApiResponse, principal: &ApiRequestPrincipal) -> ApiResponse {
    disclose_principal_side_effect(response, principal)
}

fn not_found(target: &str) -> ApiResponse {
    failure(
        StatusCode::NOT_FOUND,
        "not_found",
        "not_found",
        &format!("No canonical record exists for {target}."),
    )
}

fn stale_target() -> ApiResponse {
    let mut response = failure(
        StatusCode::CONFLICT,
        "stale_target",
        "exact_target_mismatch",
        "Refresh status and reconnect with the current exact target.",
    );
    response.1.0.stale = true;
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_defaults_to_redacted_stdout_channel() {
        assert_eq!(default_output_channel(), OutputChannel::Stdout);
    }

    #[test]
    fn stale_stream_target_is_canonical_and_non_retrying() {
        let response = stale_target();
        assert!(response.1.0.stale);
        assert_eq!(
            response.1.0.failure_class.as_deref(),
            Some("exact_target_mismatch")
        );
        assert!(!response.1.0.retry.retryable);
    }
}
