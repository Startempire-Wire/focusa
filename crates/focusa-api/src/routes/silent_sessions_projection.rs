use std::{collections::BTreeSet, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::get,
};
use focusa_core::silent_sessions::{
    CompletionEvaluation, RunGeneration, SilentSessionAction, SilentSessionId, SilentSessionRun,
    SilentSessionRunId, list_checkpoint_values, list_completion_evaluations, load_run,
    load_session, load_usage_summary,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{middleware::principal::ApiRequestPrincipal, server::AppState};

use super::{
    silent_sessions::{
        ApiResponse, authorized_projection, disclose_principal_side_effect,
        durable_request_principal, failure, persistence_failure,
    },
    silent_sessions_contract::{
        ExactSessionRunTarget, SilentSessionApiEnvelope, guard_exact_target,
    },
};

#[derive(Debug, Deserialize)]
struct ProjectionQuery {
    run_id: SilentSessionRunId,
    generation: RunGeneration,
}

struct ProjectionContext {
    principal: ApiRequestPrincipal,
    run: SilentSessionRun,
    redacted: bool,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/silent-sessions/{session_id}/usage", get(usage))
        .route(
            "/v1/silent-sessions/{session_id}/checkpoints",
            get(checkpoints),
        )
        .route("/v1/silent-sessions/{session_id}/artifacts", get(artifacts))
        .route("/v1/silent-sessions/{session_id}/receipts", get(receipts))
}

async fn usage(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
    Query(query): Query<ProjectionQuery>,
) -> ApiResponse {
    let context = match context(&state, &headers, session_id, &query).await {
        Ok(context) => context,
        Err(response) => return *response,
    };
    match load_usage_summary(&state.persistence, session_id, context.run.id) {
        Ok(summary) => success("usage", json!(summary), &context.principal),
        Err(error) => after(persistence_failure(error), &context.principal),
    }
}

async fn checkpoints(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
    Query(query): Query<ProjectionQuery>,
) -> ApiResponse {
    let context = match context(&state, &headers, session_id, &query).await {
        Ok(context) => context,
        Err(response) => return *response,
    };
    match list_checkpoint_values(&state.persistence, session_id, context.run.id) {
        Ok(values) => {
            let data = if context.redacted {
                values.iter().map(redacted_checkpoint).collect::<Vec<_>>()
            } else {
                values
            };
            success("checkpoints", json!(data), &context.principal)
        }
        Err(error) => after(persistence_failure(error), &context.principal),
    }
}

async fn artifacts(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
    Query(query): Query<ProjectionQuery>,
) -> ApiResponse {
    let context = match context(&state, &headers, session_id, &query).await {
        Ok(context) => context,
        Err(response) => return *response,
    };
    let checkpoints = match list_checkpoint_values(&state.persistence, session_id, context.run.id) {
        Ok(values) => values,
        Err(error) => return after(persistence_failure(error), &context.principal),
    };
    let evaluations =
        match list_completion_evaluations(&state.persistence, session_id, context.run.id) {
            Ok(values) => values,
            Err(error) => return after(persistence_failure(error), &context.principal),
        };
    let refs = artifact_refs(&checkpoints, &evaluations);
    let data = artifact_projection(
        session_id,
        context.run.id,
        context.run.generation,
        refs,
        context.redacted,
    );
    success("artifacts", data, &context.principal)
}

async fn receipts(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<SilentSessionId>,
    Query(query): Query<ProjectionQuery>,
) -> ApiResponse {
    let context = match context(&state, &headers, session_id, &query).await {
        Ok(context) => context,
        Err(response) => return *response,
    };
    match list_completion_evaluations(&state.persistence, session_id, context.run.id) {
        Ok(values) => {
            let ready_ids = values
                .into_iter()
                .filter(|value| value.receipt_ready)
                .map(|value| value.id.to_string())
                .collect::<Vec<_>>();
            let data = receipt_projection(
                session_id,
                context.run.id,
                context.run.generation,
                ready_ids,
                context.redacted,
            );
            success("receipts", data, &context.principal)
        }
        Err(error) => after(persistence_failure(error), &context.principal),
    }
}

async fn context(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    session_id: SilentSessionId,
    query: &ProjectionQuery,
) -> Result<ProjectionContext, Box<ApiResponse>> {
    let principal = durable_request_principal(state, headers).await?;
    let session = match load_session(&state.persistence, session_id) {
        Ok(Some(session)) => session,
        Ok(None) => return Err(Box::new(after(not_found("session_id"), &principal))),
        Err(error) => return Err(Box::new(after(persistence_failure(error), &principal))),
    };
    let run = match load_run(&state.persistence, query.run_id) {
        Ok(Some(run)) => run,
        Ok(None) => return Err(Box::new(after(not_found("run_id"), &principal))),
        Err(error) => return Err(Box::new(after(persistence_failure(error), &principal))),
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
        return Err(Box::new(after(stale_target(), &principal)));
    }
    let Some(projection) = authorized_projection(&principal, &session, SilentSessionAction::Show)
    else {
        return Err(Box::new(after(
            failure(
                StatusCode::FORBIDDEN,
                "forbidden",
                "authorization_denied",
                "The authenticated principal cannot observe this Silent Session.",
            ),
            &principal,
        )));
    };
    let redacted = projection.get("projection") == Some(&Value::String("redacted_summary".into()));
    Ok(ProjectionContext {
        principal,
        run,
        redacted,
    })
}

fn artifact_refs(checkpoints: &[Value], evaluations: &[CompletionEvaluation]) -> Vec<String> {
    let mut refs = BTreeSet::new();
    for checkpoint in checkpoints {
        if let Some(values) = checkpoint.get("evidence_refs").and_then(Value::as_array) {
            refs.extend(
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned),
            );
        }
    }
    for evaluation in evaluations {
        refs.extend(evaluation.verified_evidence_refs.iter().cloned());
    }
    refs.into_iter().collect()
}

fn artifact_projection(
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
    generation: RunGeneration,
    refs: Vec<String>,
    redacted: bool,
) -> Value {
    if redacted {
        json!({
            "session_id": session_id,
            "run_id": run_id,
            "generation": generation,
            "artifact_count": refs.len(),
            "projection": "redacted_summary"
        })
    } else {
        json!({
            "session_id": session_id,
            "run_id": run_id,
            "generation": generation,
            "artifact_refs": refs
        })
    }
}

fn receipt_projection(
    session_id: SilentSessionId,
    run_id: SilentSessionRunId,
    generation: RunGeneration,
    ready_ids: Vec<String>,
    redacted: bool,
) -> Value {
    if redacted {
        json!({
            "session_id": session_id,
            "run_id": run_id,
            "generation": generation,
            "receipt_count": 0,
            "ready_evaluation_count": ready_ids.len(),
            "materialization_pending": !ready_ids.is_empty(),
            "projection": "redacted_summary"
        })
    } else {
        json!({
            "session_id": session_id,
            "run_id": run_id,
            "generation": generation,
            "receipt_refs": [],
            "ready_evaluation_ids": ready_ids,
            "materialization_pending": !ready_ids.is_empty()
        })
    }
}

fn redacted_checkpoint(value: &Value) -> Value {
    json!({
        "id": value.get("id"),
        "schema_version": value.get("schema_version"),
        "created_at": value.get("created_at"),
        "projection": "redacted_summary"
    })
}

fn success(status: &str, data: Value, principal: &ApiRequestPrincipal) -> ApiResponse {
    after(
        (
            StatusCode::OK,
            Json(SilentSessionApiEnvelope::canonical(status, data)),
        ),
        principal,
    )
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
        "Refresh status and retry with the current exact target.",
    );
    response.1.0.stale = true;
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_projection_is_deduplicated_and_sorted() {
        let checkpoints = vec![json!({"evidence_refs": ["z", "a", "z"]})];
        let refs = artifact_refs(&checkpoints, &[]);
        assert_eq!(refs, vec!["a", "z"]);
    }

    #[test]
    fn proof_projections_bind_exact_session_run_and_generation() {
        let session_id = SilentSessionId::new();
        let run_id = SilentSessionRunId::new();
        let generation = RunGeneration::first();

        for projection in [
            artifact_projection(
                session_id,
                run_id,
                generation,
                vec!["artifact:1".into()],
                false,
            ),
            receipt_projection(
                session_id,
                run_id,
                generation,
                vec!["evaluation:1".into()],
                false,
            ),
        ] {
            assert_eq!(projection["session_id"], json!(session_id));
            assert_eq!(projection["run_id"], json!(run_id));
            assert_eq!(projection["generation"], json!(generation));
        }
    }

    #[test]
    fn checkpoint_redaction_drops_mission_and_evidence() {
        let redacted = redacted_checkpoint(&json!({
            "id": "checkpoint-1",
            "schema_version": 1,
            "created_at": "now",
            "mission": "secret",
            "evidence_refs": ["secret-ref"]
        }));
        assert!(redacted.get("mission").is_none());
        assert!(redacted.get("evidence_refs").is_none());
    }
}
