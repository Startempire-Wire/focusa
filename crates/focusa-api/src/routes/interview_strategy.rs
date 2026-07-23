use crate::server::AppState;
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use focusa_core::{
    runtime::interview_strategy::{
        GrillInterviewContext, GrillInterviewStrategyResult, generate_next_question,
    },
    tool_result::{FailureClass, ToolResultV1, ToolStatus},
    types::RoleProfileStatus,
};
use serde::Serialize;
use std::{collections::BTreeSet, sync::Arc};

type ApiError = (StatusCode, Json<Box<ToolResultV1>>);

#[derive(Debug, Serialize)]
pub struct GrillInterviewStrategyResponse {
    pub schema: &'static str,
    pub advisory_strategy: bool,
    pub canonical_inputs_verified: bool,
    pub interview_state_authority: &'static str,
    pub result: GrillInterviewStrategyResult,
    pub tool_result: ToolResultV1,
}

fn fail(
    status: StatusCode,
    tool_status: ToolStatus,
    class: FailureClass,
    summary: impl Into<String>,
) -> ApiError {
    let mut result = ToolResultV1::failure(tool_status, class, summary);
    result.tool = Some("focusa_interview_strategy_grill_with_docs_next_question".into());
    result.family = Some("interview_strategy".into());
    result.endpoint = Some("/v1/interview/strategy/grill-with-docs/next-question".into());
    result.next_tools = vec![
        "focusa_role_profiles_list".into(),
        "focusa_context_retrieve".into(),
        "focusa_evidence_capture".into(),
    ];
    (status, Json(Box::new(result)))
}

fn valid_text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max
}

async fn next_question(
    State(state): State<Arc<AppState>>,
    Json(context): Json<GrillInterviewContext>,
) -> Result<Json<GrillInterviewStrategyResponse>, ApiError> {
    if !valid_text(&context.project_root, 4096)
        || !valid_text(&context.continuity_id, 256)
        || !valid_text(&context.attachment_id, 256)
        || !valid_text(&context.session_id, 256)
        || context.gaps.len() > 256
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "exact scope, session_id, and at most 256 bounded gaps are required",
        ));
    }
    let snapshot = state.focusa.read().await;
    let approved_role = snapshot.project_role_profiles.iter().any(|profile| {
        profile.role_profile_id == context.approved_role_profile_ref
            && profile.project_root == context.project_root
            && profile.continuity_id == context.continuity_id
            && profile.attachment_id == context.attachment_id
            && matches!(profile.status, RoleProfileStatus::Approved)
    });
    if !approved_role {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ApprovalRequired,
            "Grill Interview strategy requires an approved Role Profile in exact scope",
        ));
    }
    let mut canonical_context_refs = BTreeSet::new();
    for source in snapshot.context_sources.iter().filter(|source| {
        source.project_root == context.project_root
            && source.continuity_id == context.continuity_id
            && source.attachment_id == context.attachment_id
    }) {
        canonical_context_refs.insert(source.source_id.as_str());
    }
    for artifact in snapshot.workspace_artifacts.iter().filter(|artifact| {
        artifact.scope.project_root == context.project_root
            && artifact.scope.continuity_id == context.continuity_id
            && artifact.origin.attachment_id == context.attachment_id
    }) {
        canonical_context_refs.insert(artifact.artifact_id.as_str());
    }
    for claim in snapshot.context_claims.iter().filter(|claim| {
        claim.project_root == context.project_root
            && claim.continuity_id == context.continuity_id
            && claim.attachment_id == context.attachment_id
    }) {
        canonical_context_refs.insert(claim.claim_id.as_str());
    }
    let contradiction_refs: BTreeSet<&str> = snapshot
        .context_contradictions
        .iter()
        .filter(|item| {
            item.project_root == context.project_root
                && item.continuity_id == context.continuity_id
                && item.attachment_id == context.attachment_id
        })
        .map(|item| item.contradiction_id.as_str())
        .collect();
    for gap in &context.gaps {
        let mut context_refs = gap
            .recommendation_basis_refs
            .iter()
            .chain(&gap.environment_facts_checked)
            .chain(&gap.linked_context_refs);
        if let Some(reference) =
            context_refs.find(|reference| !canonical_context_refs.contains(reference.as_str()))
        {
            return Err(fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::NotFound,
                format!("Interview strategy ref is not canonical in exact scope: {reference}"),
            ));
        }
        if let Some(reference) = gap
            .contradiction_refs
            .iter()
            .find(|reference| !contradiction_refs.contains(reference.as_str()))
        {
            return Err(fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::NotFound,
                format!("Interview contradiction ref is not canonical in exact scope: {reference}"),
            ));
        }
    }
    drop(snapshot);
    let strategy_result = generate_next_question(&context).map_err(|message| {
        fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            message,
        )
    })?;
    let mut tool_result = ToolResultV1::success(
        ToolStatus::Completed,
        if strategy_result.ready_for_spec {
            "All represented Grill branches are resolved; strategy proposes no further question"
        } else {
            "One retrieval-grounded Grill question proposed; operator answer remains authoritative"
        },
    );
    tool_result.tool = Some("focusa_interview_strategy_grill_with_docs_next_question".into());
    tool_result.family = Some("interview_strategy".into());
    tool_result.endpoint = Some("/v1/interview/strategy/grill-with-docs/next-question".into());
    tool_result.evidence_refs = strategy_result
        .proposal
        .as_ref()
        .map(|proposal| proposal.recommendation_basis_refs.clone())
        .unwrap_or_default();
    tool_result.next_tools = vec![
        "focusa_interview_answer_commit".into(),
        "focusa_evidence_capture".into(),
    ];
    Ok(Json(GrillInterviewStrategyResponse {
        schema: "focusa.grill_interview_strategy_response.v1",
        advisory_strategy: true,
        canonical_inputs_verified: true,
        interview_state_authority: "Focusa Interview Engine",
        result: strategy_result,
        tool_result,
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route(
        "/v1/interview/strategy/grill-with-docs/next-question",
        post(next_question),
    )
}
