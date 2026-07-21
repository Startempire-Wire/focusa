use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::{
    tool_result::{FailureClass, ToolResultV1, ToolStatus},
    types::{
        Action, FocusaEvent, ProjectInterviewAnswerRecord, ProjectInterviewAnswerStatus,
        ProjectInterviewBranchRecord, ProjectInterviewBranchStatus, ProjectInterviewQuestionRecord,
        ProjectInterviewQuestionStatus, ProjectInterviewSessionRecord,
        ProjectInterviewSessionStatus, RoleProfileStatus,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{sync::Arc, time::Duration};

type ApiError = (StatusCode, Json<Box<ToolResultV1>>);
const TOOL: &str = "focusa_interview_session_mutate";
const ENDPOINT: &str = "/v1/interviews/sessions/mutate";

#[derive(Debug, Clone, Deserialize)]
pub struct InterviewSessionQuery {
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
    #[serde(default)]
    pub interview_session_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterviewSessionAction {
    Open,
    UpsertBranch,
    QueueQuestion,
    RecordAnswer,
    Pause,
    Close,
    Reopen,
    DeferBranch,
    ResolveBranch,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InterviewBranchInput {
    pub decision_branch_id: String,
    #[serde(default)]
    pub parent_branch_id: Option<String>,
    pub tranche: String,
    pub label: String,
    #[serde(default)]
    pub deferred_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InterviewQuestionInput {
    #[serde(default)]
    pub question_id: Option<String>,
    pub decision_branch_id: String,
    #[serde(default)]
    pub parent_question_id: Option<String>,
    pub question: String,
    pub reason_for_asking: String,
    pub triggering_gap: String,
    pub recommendation: String,
    #[serde(default)]
    pub recommendation_basis_refs: Vec<String>,
    #[serde(default)]
    pub environment_facts_checked: Vec<String>,
    #[serde(default)]
    pub contradiction_refs: Vec<String>,
    #[serde(default)]
    pub linked_context_refs: Vec<String>,
    #[serde(default)]
    pub linked_spec_sections: Vec<String>,
    pub decision_required: bool,
    pub priority: String,
    pub answer_type: String,
    #[serde(default)]
    pub sensitivity: String,
    pub readiness_effect: String,
    pub stop_condition: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InterviewAnswerInput {
    #[serde(default)]
    pub answer_id: Option<String>,
    pub question_id: String,
    pub answer: Value,
    #[serde(default)]
    pub attachment_refs: Vec<String>,
    pub operator_id: String,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub supersedes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InterviewSessionMutationRequest {
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
    pub idempotency_key: String,
    pub expected_state_version: u64,
    #[serde(default)]
    pub expected_session_revision: u64,
    pub action: InterviewSessionAction,
    #[serde(default)]
    pub interview_session_id: Option<String>,
    #[serde(default)]
    pub approved_role_profile_ref: Option<String>,
    #[serde(default)]
    pub branch: Option<InterviewBranchInput>,
    #[serde(default)]
    pub question: Option<InterviewQuestionInput>,
    #[serde(default)]
    pub answer: Option<InterviewAnswerInput>,
    #[serde(default)]
    pub decision_branch_id: Option<String>,
    #[serde(default)]
    pub deferred_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InterviewSessionListResponse {
    pub schema: &'static str,
    pub state_version: u64,
    pub sessions: Vec<ProjectInterviewSessionRecord>,
}

#[derive(Debug, Serialize)]
pub struct InterviewSessionMutationResponse {
    pub schema: &'static str,
    pub state_version: u64,
    pub replayed: bool,
    pub exact_resume: bool,
    pub session: ProjectInterviewSessionRecord,
    pub evidence_ref: String,
    pub receipt_ref: String,
    pub tool_result: ToolResultV1,
}

fn fail(
    status: StatusCode,
    tool_status: ToolStatus,
    class: FailureClass,
    summary: impl Into<String>,
) -> ApiError {
    let mut result = ToolResultV1::failure(tool_status, class, summary);
    result.tool = Some(TOOL.into());
    result.family = Some("project_interview".into());
    result.endpoint = Some(ENDPOINT.into());
    result.next_tools = vec![
        "focusa_interview_sessions_list".into(),
        "focusa_interview_strategy_grill_with_docs_next_question".into(),
    ];
    (status, Json(Box::new(result)))
}

fn bounded(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max
}
fn stable_id(prefix: &str, values: &[&str]) -> String {
    let mut hash = Sha256::new();
    for value in values {
        hash.update(value.as_bytes());
        hash.update([0]);
    }
    format!("{prefix}:{}", &hex::encode(hash.finalize())[..24])
}
fn scoped(
    session: &ProjectInterviewSessionRecord,
    project_root: &str,
    continuity_id: &str,
    attachment_id: &str,
) -> bool {
    session.project_root == project_root
        && session.continuity_id == continuity_id
        && session.attachment_id == attachment_id
}

async fn list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<InterviewSessionQuery>,
) -> Result<Json<InterviewSessionListResponse>, ApiError> {
    if !bounded(&query.project_root, 4096)
        || !bounded(&query.continuity_id, 256)
        || !bounded(&query.attachment_id, 256)
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "exact project, continuity, and attachment scope are required",
        ));
    }
    let snapshot = state.focusa.read().await;
    let mut sessions: Vec<_> = snapshot
        .project_interview_sessions
        .iter()
        .filter(|session| {
            scoped(
                session,
                &query.project_root,
                &query.continuity_id,
                &query.attachment_id,
            ) && query
                .interview_session_id
                .as_deref()
                .is_none_or(|id| session.interview_session_id == id)
        })
        .cloned()
        .collect();
    sessions.sort_by(|left, right| {
        left.interview_session_id
            .cmp(&right.interview_session_id)
            .then(left.state_revision.cmp(&right.state_revision))
    });
    Ok(Json(InterviewSessionListResponse {
        schema: "focusa.project_interview_session_list.v1",
        state_version: snapshot.version,
        sessions,
    }))
}

fn mutation_response(
    session: ProjectInterviewSessionRecord,
    state_version: u64,
    replayed: bool,
) -> InterviewSessionMutationResponse {
    let evidence_ref = format!(
        "evidence:project-interview:{}:r{}",
        session.interview_session_id, session.state_revision
    );
    let receipt_ref = format!(
        "receipt:project-interview:{}:{}",
        session.interview_session_id, session.idempotency_key
    );
    let mut tool_result = ToolResultV1::success(
        ToolStatus::Completed,
        if replayed {
            "Interview mutation replayed idempotently"
        } else {
            "Canonical Interview revision committed"
        },
    );
    tool_result.tool = Some(TOOL.into());
    tool_result.family = Some("project_interview".into());
    tool_result.endpoint = Some(ENDPOINT.into());
    tool_result.evidence_refs = vec![evidence_ref.clone(), receipt_ref.clone()];
    InterviewSessionMutationResponse {
        schema: "focusa.project_interview_session_mutation_result.v1",
        state_version,
        replayed,
        exact_resume: true,
        session,
        evidence_ref,
        receipt_ref,
        tool_result,
    }
}

async fn mutate(
    State(state): State<Arc<AppState>>,
    Json(request): Json<InterviewSessionMutationRequest>,
) -> Result<Json<InterviewSessionMutationResponse>, ApiError> {
    if !bounded(&request.project_root, 4096)
        || !bounded(&request.continuity_id, 256)
        || !bounded(&request.attachment_id, 256)
        || !bounded(&request.idempotency_key, 256)
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "exact scope and bounded idempotency_key are required",
        ));
    }
    let snapshot = state.focusa.read().await;
    if let Some(existing) = snapshot.project_interview_sessions.iter().find(|session| {
        scoped(
            session,
            &request.project_root,
            &request.continuity_id,
            &request.attachment_id,
        ) && session.idempotency_key == request.idempotency_key
    }) {
        return Ok(Json(mutation_response(
            existing.clone(),
            snapshot.version,
            true,
        )));
    }
    if snapshot.version != request.expected_state_version {
        return Err(fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            format!(
                "expected_state_version={} does not match canonical version={}",
                request.expected_state_version, snapshot.version
            ),
        ));
    }
    let now = Utc::now();
    let mut session = if matches!(request.action, InterviewSessionAction::Open) {
        if request.expected_session_revision != 0 {
            return Err(fail(
                StatusCode::CONFLICT,
                ToolStatus::Blocked,
                FailureClass::WriterConflict,
                "open requires expected_session_revision=0",
            ));
        }
        let role_ref = request.approved_role_profile_ref.clone().ok_or_else(|| {
            fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ApprovalRequired,
                "open requires approved_role_profile_ref",
            )
        })?;
        if !snapshot.project_role_profiles.iter().any(|profile| {
            profile.role_profile_id == role_ref
                && profile.project_root == request.project_root
                && profile.continuity_id == request.continuity_id
                && profile.attachment_id == request.attachment_id
                && matches!(profile.status, RoleProfileStatus::Approved)
        }) {
            return Err(fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ApprovalRequired,
                "open requires an approved Role Profile in exact scope",
            ));
        }
        let id = request.interview_session_id.clone().unwrap_or_else(|| {
            stable_id(
                "project-interview",
                &[
                    &request.project_root,
                    &request.continuity_id,
                    &request.attachment_id,
                    &request.idempotency_key,
                ],
            )
        });
        ProjectInterviewSessionRecord {
            interview_session_id: id,
            project_root: request.project_root.clone(),
            continuity_id: request.continuity_id.clone(),
            attachment_id: request.attachment_id.clone(),
            strategy_id: "focusa.interview.strategy.grill-with-docs.v1".into(),
            strategy_version: 1,
            approved_role_profile_ref: role_ref,
            state_revision: 1,
            status: ProjectInterviewSessionStatus::Active,
            active_branch_id: None,
            current_question_id: None,
            branches: vec![],
            questions: vec![],
            answers: vec![],
            idempotency_key: request.idempotency_key.clone(),
            created_at: now,
            updated_at: now,
            closed_at: None,
        }
    } else {
        let id = request.interview_session_id.as_deref().ok_or_else(|| {
            fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ValidationRejected,
                "mutation requires interview_session_id",
            )
        })?;
        let latest = snapshot
            .project_interview_sessions
            .iter()
            .filter(|candidate| {
                scoped(
                    candidate,
                    &request.project_root,
                    &request.continuity_id,
                    &request.attachment_id,
                ) && candidate.interview_session_id == id
            })
            .max_by_key(|candidate| candidate.state_revision)
            .cloned()
            .ok_or_else(|| {
                fail(
                    StatusCode::NOT_FOUND,
                    ToolStatus::Blocked,
                    FailureClass::NotFound,
                    "Interview session not found in exact scope",
                )
            })?;
        if latest.state_revision != request.expected_session_revision {
            return Err(fail(
                StatusCode::CONFLICT,
                ToolStatus::Blocked,
                FailureClass::WriterConflict,
                format!(
                    "expected_session_revision={} does not match canonical revision={}",
                    request.expected_session_revision, latest.state_revision
                ),
            ));
        }
        if matches!(latest.status, ProjectInterviewSessionStatus::Closed)
            && !matches!(request.action, InterviewSessionAction::Reopen)
        {
            return Err(fail(
                StatusCode::CONFLICT,
                ToolStatus::Blocked,
                FailureClass::ValidationRejected,
                "closed Interview session must be reopened before mutation",
            ));
        }
        let mut next = latest;
        next.state_revision += 1;
        next.idempotency_key = request.idempotency_key.clone();
        next.updated_at = now;
        next
    };
    drop(snapshot);

    match request.action {
        InterviewSessionAction::Open => {}
        InterviewSessionAction::UpsertBranch => {
            let input = request.branch.ok_or_else(|| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "upsert_branch requires branch",
                )
            })?;
            let record = ProjectInterviewBranchRecord {
                decision_branch_id: input.decision_branch_id.clone(),
                parent_branch_id: input.parent_branch_id,
                tranche: input.tranche,
                label: input.label,
                status: ProjectInterviewBranchStatus::Active,
                question_ids: session
                    .branches
                    .iter()
                    .find(|b| b.decision_branch_id == input.decision_branch_id)
                    .map(|b| b.question_ids.clone())
                    .unwrap_or_default(),
                deferred_reason: input.deferred_reason,
                updated_at: now,
            };
            if let Some(existing) = session
                .branches
                .iter_mut()
                .find(|branch| branch.decision_branch_id == record.decision_branch_id)
            {
                *existing = record;
            } else {
                session.branches.push(record);
            }
            if session.active_branch_id.is_none() {
                session.active_branch_id = Some(input.decision_branch_id);
            }
        }
        InterviewSessionAction::QueueQuestion => {
            if session.current_question_id.as_ref().is_some_and(|current| {
                session.questions.iter().any(|q| {
                    &q.question_id == current
                        && matches!(q.status, ProjectInterviewQuestionStatus::Asked)
                })
            }) {
                return Err(fail(
                    StatusCode::CONFLICT,
                    ToolStatus::Blocked,
                    FailureClass::ValidationRejected,
                    "answer, defer, or skip the current question before queueing another",
                ));
            }
            let input = request.question.ok_or_else(|| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "queue_question requires question",
                )
            })?;
            if !session
                .branches
                .iter()
                .any(|branch| branch.decision_branch_id == input.decision_branch_id)
            {
                return Err(fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::NotFound,
                    "question branch is not retained in this session",
                ));
            }
            let id = input.question_id.unwrap_or_else(|| {
                stable_id(
                    "interview-question",
                    &[&session.interview_session_id, &request.idempotency_key],
                )
            });
            let record = ProjectInterviewQuestionRecord {
                question_id: id.clone(),
                session_id: session.interview_session_id.clone(),
                decision_branch_id: input.decision_branch_id.clone(),
                parent_question_id: input.parent_question_id,
                question: input.question,
                reason_for_asking: input.reason_for_asking,
                triggering_gap: input.triggering_gap,
                recommendation: input.recommendation,
                recommendation_basis_refs: input.recommendation_basis_refs,
                environment_facts_checked: input.environment_facts_checked,
                contradiction_refs: input.contradiction_refs,
                linked_context_refs: input.linked_context_refs,
                linked_spec_sections: input.linked_spec_sections,
                decision_required: input.decision_required,
                priority: input.priority,
                answer_type: input.answer_type,
                sensitivity: input.sensitivity,
                readiness_effect: input.readiness_effect,
                stop_condition: input.stop_condition,
                status: ProjectInterviewQuestionStatus::Asked,
                created_at: now,
                answered_at: None,
            };
            session
                .branches
                .iter_mut()
                .find(|branch| branch.decision_branch_id == input.decision_branch_id)
                .expect("branch checked")
                .question_ids
                .push(id.clone());
            session.active_branch_id = Some(input.decision_branch_id);
            session.current_question_id = Some(id);
            session.questions.push(record);
        }
        InterviewSessionAction::RecordAnswer => {
            let input = request.answer.ok_or_else(|| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "record_answer requires answer",
                )
            })?;
            let question = session
                .questions
                .iter_mut()
                .find(|question| question.question_id == input.question_id)
                .ok_or_else(|| {
                    fail(
                        StatusCode::NOT_FOUND,
                        ToolStatus::Blocked,
                        FailureClass::NotFound,
                        "answer question not found",
                    )
                })?;
            if !matches!(question.status, ProjectInterviewQuestionStatus::Asked) {
                return Err(fail(
                    StatusCode::CONFLICT,
                    ToolStatus::Blocked,
                    FailureClass::ValidationRejected,
                    "only the asked question can receive an answer",
                ));
            }
            let id = input.answer_id.unwrap_or_else(|| {
                stable_id(
                    "interview-answer",
                    &[
                        &session.interview_session_id,
                        &input.question_id,
                        &request.idempotency_key,
                    ],
                )
            });
            question.status = ProjectInterviewQuestionStatus::Answered;
            question.answered_at = Some(now);
            session.answers.push(ProjectInterviewAnswerRecord {
                answer_id: id,
                question_id: input.question_id,
                answer: input.answer,
                attachment_refs: input.attachment_refs,
                operator_id: input.operator_id,
                status: if input.supersedes.is_some() {
                    ProjectInterviewAnswerStatus::Amended
                } else {
                    ProjectInterviewAnswerStatus::Active
                },
                confidence: input.confidence,
                notes: input.notes,
                created_at: now,
                supersedes: input.supersedes,
            });
            session.current_question_id = None;
        }
        InterviewSessionAction::Pause => session.status = ProjectInterviewSessionStatus::Paused,
        InterviewSessionAction::Close => {
            session.status = ProjectInterviewSessionStatus::Closed;
            session.closed_at = Some(now);
        }
        InterviewSessionAction::Reopen => {
            session.status = ProjectInterviewSessionStatus::Active;
            session.closed_at = None;
        }
        InterviewSessionAction::DeferBranch | InterviewSessionAction::ResolveBranch => {
            let id = request.decision_branch_id.as_deref().ok_or_else(|| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "branch transition requires decision_branch_id",
                )
            })?;
            let branch = session
                .branches
                .iter_mut()
                .find(|branch| branch.decision_branch_id == id)
                .ok_or_else(|| {
                    fail(
                        StatusCode::NOT_FOUND,
                        ToolStatus::Blocked,
                        FailureClass::NotFound,
                        "branch not found",
                    )
                })?;
            if matches!(request.action, InterviewSessionAction::DeferBranch) {
                branch.status = ProjectInterviewBranchStatus::Deferred;
                branch.deferred_reason = request.deferred_reason;
            } else {
                branch.status = ProjectInterviewBranchStatus::Resolved;
                branch.deferred_reason = None;
            }
            branch.updated_at = now;
            if matches!(request.action, InterviewSessionAction::DeferBranch) {
                if let Some(question) = session
                    .current_question_id
                    .as_ref()
                    .and_then(|id| session.questions.iter_mut().find(|q| &q.question_id == id))
                {
                    question.status = ProjectInterviewQuestionStatus::Deferred;
                }
            }
        }
    }
    let id = session.interview_session_id.clone();
    let key = session.idempotency_key.clone();
    state
        .command_tx
        .send(Action::EmitEvent {
            event: FocusaEvent::ProjectInterviewSessionRevised { session },
        })
        .await
        .map_err(|_| {
            fail(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Offline,
                FailureClass::DaemonUnavailable,
                "canonical Interview command channel unavailable",
            )
        })?;
    for _ in 0..100 {
        let current = state.focusa.read().await;
        if let Some(visible) = current.project_interview_sessions.iter().find(|candidate| {
            candidate.interview_session_id == id && candidate.idempotency_key == key
        }) {
            return Ok(Json(mutation_response(
                visible.clone(),
                current.version,
                false,
            )));
        }
        drop(current);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    Err(fail(
        StatusCode::SERVICE_UNAVAILABLE,
        ToolStatus::Degraded,
        FailureClass::ReadModelLag,
        "Interview mutation dispatched but not visible",
    ))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/interviews/sessions", get(list))
        .route(ENDPOINT, post(mutate))
}
