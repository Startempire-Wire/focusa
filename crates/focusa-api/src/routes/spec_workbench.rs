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
        Action, CristSpecHandoff, FocusaEvent, FocusaState, ProjectInterviewSessionStatus,
        RoleProfileStatus, SpecAmendmentRecord, SpecGateDecision, SpecGroundingBlock,
        SpecObjectionRecord, SpecObjectionStatus, SpecOperatorGateRecord, SpecRoundRecord,
        SpecSectionRecord, SpecSectionStatus, SpecWorkbenchSessionRecord, SpecWorkbenchStatus,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, sync::Arc, time::Duration};

type ApiError = (StatusCode, Json<Box<ToolResultV1>>);
const TOOL: &str = "focusa_spec_workbench_mutate";
const ENDPOINT: &str = "/v1/spec-workbench/session/mutate";
#[derive(Debug, Deserialize)]
pub struct QueryInput {
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
    #[serde(default)]
    pub workbench_session_id: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationAction {
    Open,
    UpsertSection,
    AddRound,
    AddObjection,
    ResolveObjection,
    ApproveSection,
    RejectSection,
    AmendSection,
    Close,
    Reopen,
    FinalApprove,
}
#[derive(Debug, Clone, Deserialize)]
pub struct SectionInput {
    #[serde(default)]
    pub section_id: Option<String>,
    pub title: String,
    pub section_kind: String,
    #[serde(default)]
    pub reality_classification: String,
    pub order_index: u32,
    pub content: String,
    #[serde(default)]
    pub context_refs: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub codebase_refs: Vec<String>,
    #[serde(default)]
    pub research_refs: Vec<String>,
    #[serde(default)]
    pub docs_only: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct RoundInput {
    #[serde(default)]
    pub round_id: Option<String>,
    pub section_id: String,
    pub round_kind: String,
    #[serde(default)]
    pub output_refs: Vec<String>,
    pub transcript_ref: String,
    pub verdict: String,
    #[serde(default)]
    pub stop_reason: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct ObjectionInput {
    #[serde(default)]
    pub objection_id: Option<String>,
    pub section_id: String,
    pub round_id: String,
    pub actor_role: String,
    pub claim: String,
    pub reasoning_summary: String,
    pub evidence_refs: Vec<String>,
    pub confidence: f64,
}
#[derive(Debug, Clone, Deserialize)]
pub struct DecisionInput {
    pub section_id: String,
    pub rationale: String,
    pub decided_by: String,
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub approval_scope: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct AmendmentInput {
    pub section_id: String,
    pub content: String,
    pub reason: String,
    pub changed_by: String,
    pub evidence_refs: Vec<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct MutationRequest {
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
    pub idempotency_key: String,
    pub expected_state_version: u64,
    #[serde(default)]
    pub expected_session_revision: u64,
    pub action: MutationAction,
    #[serde(default)]
    pub workbench_session_id: Option<String>,
    #[serde(default)]
    pub current_ask: Option<String>,
    #[serde(default)]
    pub desired_spec_template: Option<String>,
    #[serde(default)]
    pub section: Option<SectionInput>,
    #[serde(default)]
    pub round: Option<RoundInput>,
    #[serde(default)]
    pub objection: Option<ObjectionInput>,
    #[serde(default)]
    pub objection_id: Option<String>,
    #[serde(default)]
    pub resolution: Option<String>,
    #[serde(default)]
    pub decision: Option<DecisionInput>,
    #[serde(default)]
    pub amendment: Option<AmendmentInput>,
}
#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub schema: &'static str,
    pub state_version: u64,
    pub sessions: Vec<SpecWorkbenchSessionRecord>,
}
#[derive(Debug, Serialize)]
pub struct MutationResponse {
    pub schema: &'static str,
    pub state_version: u64,
    pub replayed: bool,
    pub exact_resume: bool,
    pub session: SpecWorkbenchSessionRecord,
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
    let mut r = ToolResultV1::failure(tool_status, class, summary);
    r.tool = Some(TOOL.into());
    r.family = Some("spec_workbench".into());
    r.endpoint = Some(ENDPOINT.into());
    r.next_tools = vec![
        "focusa_spec_workbench_list".into(),
        "focusa_evidence_capture".into(),
    ];
    (status, Json(Box::new(r)))
}
fn bounded(v: &str, n: usize) -> bool {
    !v.trim().is_empty() && v.len() <= n
}
fn id(prefix: &str, values: &[&str]) -> String {
    let mut h = Sha256::new();
    for v in values {
        h.update(v.as_bytes());
        h.update([0]);
    }
    format!("{prefix}:{}", &hex::encode(h.finalize())[..24])
}
fn scoped(s: &SpecWorkbenchSessionRecord, p: &str, c: &str, a: &str) -> bool {
    s.project_root == p && s.continuity_id == c && s.attachment_id == a
}

const PROJECT_GENESIS_SECTIONS: [&str; 22] = [
    "Project title and one-line definition",
    "Problem or opportunity",
    "Project identity and current-state reality",
    "Long-term desired state / mandatory HLT",
    "Users and stakeholders",
    "Approved project agent role",
    "Context sources and provenance",
    "Scope",
    "Non-goals",
    "Constraints",
    "Risks",
    "Authority and approval boundaries",
    "Data, privacy, retention, and connector posture",
    "Workspace and visual profile",
    "Evidence and proof policy",
    "Core workflows",
    "Initial architecture or operating model",
    "Success criteria",
    "Milestones and Waypoints",
    "Known unknowns and open questions",
    "Initial task-decomposition policy",
    "Final approval record",
];

const REALITY_CLASSIFICATIONS: [&str; 9] = [
    "implemented",
    "partial",
    "docs_only",
    "normative_target",
    "planned",
    "speculative",
    "stale",
    "blocked",
    "unknown",
];

fn crist_handoff(
    snapshot: &FocusaState,
    request: &MutationRequest,
    current_ask: &str,
) -> CristSpecHandoff {
    let in_scope = |project_root: &str, continuity_id: &str, attachment_id: &str| {
        project_root == request.project_root
            && continuity_id == request.continuity_id
            && attachment_id == request.attachment_id
    };
    let sources: Vec<_> = snapshot
        .context_sources
        .iter()
        .filter(|source| {
            in_scope(
                &source.project_root,
                &source.continuity_id,
                &source.attachment_id,
            )
        })
        .collect();
    let context_pack_refs = sources
        .iter()
        .map(|source| {
            if source.artifact.artifact_id.is_empty() {
                source.source_id.clone()
            } else {
                source.artifact.artifact_id.clone()
            }
        })
        .collect();
    let active_domain_pack_refs = sources
        .iter()
        .flat_map(|source| source.artifact.semantic.domain_pack_refs.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let accepted_project_claim_refs = snapshot
        .context_claims
        .iter()
        .filter(|claim| {
            in_scope(
                &claim.project_root,
                &claim.continuity_id,
                &claim.attachment_id,
            ) && claim.status == "accepted"
        })
        .map(|claim| claim.claim_id.clone())
        .collect();
    let role_profile_ref = snapshot
        .project_role_profiles
        .iter()
        .filter(|role| {
            in_scope(&role.project_root, &role.continuity_id, &role.attachment_id)
                && role.status == RoleProfileStatus::Approved
        })
        .max_by_key(|role| role.revision)
        .map(|role| role.role_profile_id.clone())
        .unwrap_or_default();
    let interviews: Vec<_> = snapshot
        .project_interview_sessions
        .iter()
        .filter(|session| {
            in_scope(
                &session.project_root,
                &session.continuity_id,
                &session.attachment_id,
            ) && matches!(
                session.status,
                ProjectInterviewSessionStatus::Closed | ProjectInterviewSessionStatus::ReadyForSpec
            )
        })
        .collect();
    let interview_session_refs = interviews
        .iter()
        .map(|session| session.interview_session_id.clone())
        .collect();
    let unresolved_questions = interviews
        .iter()
        .flat_map(|session| {
            session.questions.iter().filter_map(|question| {
                let answered = session
                    .answers
                    .iter()
                    .any(|answer| answer.question_id == question.question_id);
                (!answered).then(|| question.question.clone())
            })
        })
        .collect();
    let known_contradictions = snapshot
        .context_contradictions
        .iter()
        .filter(|contradiction| {
            in_scope(
                &contradiction.project_root,
                &contradiction.continuity_id,
                &contradiction.attachment_id,
            ) && contradiction.status != "resolved"
        })
        .map(|contradiction| contradiction.contradiction_id.clone())
        .collect();
    CristSpecHandoff {
        schema: "focusa.crist_spec_handoff.v1".to_string(),
        project_root: request.project_root.clone(),
        continuity_id: request.continuity_id.clone(),
        current_ask: current_ask.to_string(),
        workspace_profile_ref: id(
            "workspace-profile",
            &[&request.project_root, &request.continuity_id],
        ),
        active_domain_pack_refs,
        semantic_registry_version: format!("focusa-state:r{}", snapshot.version),
        context_pack_refs,
        accepted_project_claim_refs,
        role_profile_ref,
        interview_session_refs,
        unresolved_questions,
        known_contradictions,
        desired_spec_template: "project_genesis".to_string(),
    }
}

fn response(s: SpecWorkbenchSessionRecord, v: u64, replayed: bool) -> MutationResponse {
    let e = format!(
        "evidence:spec-workbench:{}:r{}",
        s.workbench_session_id, s.state_revision
    );
    let r = format!(
        "receipt:spec-workbench:{}:{}",
        s.workbench_session_id, s.idempotency_key
    );
    let mut t = ToolResultV1::success(
        ToolStatus::Completed,
        if replayed {
            "Spec Workbench mutation replayed idempotently"
        } else {
            "Canonical Spec Workbench revision committed"
        },
    );
    t.tool = Some(TOOL.into());
    t.family = Some("spec_workbench".into());
    t.endpoint = Some(ENDPOINT.into());
    t.evidence_refs = vec![e.clone(), r.clone()];
    MutationResponse {
        schema: "focusa.spec_workbench_mutation_result.v1",
        state_version: v,
        replayed,
        exact_resume: true,
        session: s,
        evidence_ref: e,
        receipt_ref: r,
        tool_result: t,
    }
}
async fn list(
    State(state): State<Arc<AppState>>,
    Query(q): Query<QueryInput>,
) -> Result<Json<ListResponse>, ApiError> {
    if !bounded(&q.project_root, 4096)
        || !bounded(&q.continuity_id, 256)
        || !bounded(&q.attachment_id, 256)
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "exact scope required",
        ));
    }
    let s = state.focusa.read().await;
    let mut xs: Vec<_> = s
        .spec_workbench_sessions
        .iter()
        .filter(|x| {
            scoped(x, &q.project_root, &q.continuity_id, &q.attachment_id)
                && q.workbench_session_id
                    .as_deref()
                    .is_none_or(|id| x.workbench_session_id == id)
        })
        .cloned()
        .collect();
    xs.sort_by(|a, b| {
        a.workbench_session_id
            .cmp(&b.workbench_session_id)
            .then(a.state_revision.cmp(&b.state_revision))
    });
    Ok(Json(ListResponse {
        schema: "focusa.spec_workbench_session_list.v1",
        state_version: s.version,
        sessions: xs,
    }))
}
async fn mutate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MutationRequest>,
) -> Result<Json<MutationResponse>, ApiError> {
    if !bounded(&req.project_root, 4096)
        || !bounded(&req.continuity_id, 256)
        || !bounded(&req.attachment_id, 256)
        || !bounded(&req.idempotency_key, 256)
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "exact scope and idempotency_key required",
        ));
    }
    let snap = state.focusa.read().await;
    if let Some(x) = snap.spec_workbench_sessions.iter().find(|x| {
        scoped(x, &req.project_root, &req.continuity_id, &req.attachment_id)
            && x.idempotency_key == req.idempotency_key
    }) {
        return Ok(Json(response(x.clone(), snap.version, true)));
    }
    if snap.version != req.expected_state_version {
        return Err(fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            "stale canonical state version",
        ));
    }
    let now = Utc::now();
    let mut s = if matches!(req.action, MutationAction::Open) {
        if req.expected_session_revision != 0 {
            return Err(fail(
                StatusCode::CONFLICT,
                ToolStatus::Blocked,
                FailureClass::WriterConflict,
                "open requires revision zero",
            ));
        }
        let ask = req
            .current_ask
            .clone()
            .filter(|x| bounded(x, 2000))
            .ok_or_else(|| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "open requires current_ask",
                )
            })?;
        let desired_spec_template = req
            .desired_spec_template
            .clone()
            .unwrap_or_else(|| "adversarial".to_string());
        if !matches!(
            desired_spec_template.as_str(),
            "adversarial" | "project_genesis"
        ) {
            return Err(fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ValidationRejected,
                "desired_spec_template must be adversarial or project_genesis",
            ));
        }
        let crist_handoff = if desired_spec_template == "project_genesis" {
            let handoff = crist_handoff(&snap, &req, &ask);
            if handoff.context_pack_refs.is_empty()
                || handoff.role_profile_ref.is_empty()
                || handoff.interview_session_refs.is_empty()
            {
                return Err(fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::Blocked,
                    FailureClass::ApprovalRequired,
                    "project_genesis requires source-linked Context, an approved Role Profile, and a closed Interview",
                ));
            }
            handoff
        } else {
            CristSpecHandoff::default()
        };
        let sid = req.workbench_session_id.clone().unwrap_or_else(|| {
            id(
                "spec-workbench",
                &[
                    &req.project_root,
                    &req.continuity_id,
                    &req.attachment_id,
                    &req.idempotency_key,
                ],
            )
        });
        SpecWorkbenchSessionRecord {
            workbench_session_id: sid,
            project_root: req.project_root.clone(),
            continuity_id: req.continuity_id.clone(),
            attachment_id: req.attachment_id.clone(),
            current_ask: ask,
            desired_spec_template,
            crist_handoff,
            state_revision: 1,
            status: SpecWorkbenchStatus::Active,
            canonical: true,
            advisory_agents: true,
            operator_required: true,
            current_section_id: None,
            sections: vec![],
            rounds: vec![],
            objections: vec![],
            gates: vec![],
            amendments: vec![],
            receipt_refs: vec![],
            final_spec_id: None,
            idempotency_key: req.idempotency_key.clone(),
            created_at: now,
            updated_at: now,
            closed_at: None,
        }
    } else {
        let sid = req.workbench_session_id.as_deref().ok_or_else(|| {
            fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ValidationRejected,
                "workbench_session_id required",
            )
        })?;
        let latest = snap
            .spec_workbench_sessions
            .iter()
            .filter(|x| {
                scoped(x, &req.project_root, &req.continuity_id, &req.attachment_id)
                    && x.workbench_session_id == sid
            })
            .max_by_key(|x| x.state_revision)
            .cloned()
            .ok_or_else(|| {
                fail(
                    StatusCode::NOT_FOUND,
                    ToolStatus::Blocked,
                    FailureClass::NotFound,
                    "Workbench session not found",
                )
            })?;
        if latest.state_revision != req.expected_session_revision {
            return Err(fail(
                StatusCode::CONFLICT,
                ToolStatus::Blocked,
                FailureClass::WriterConflict,
                "stale Workbench revision",
            ));
        }
        if matches!(latest.status, SpecWorkbenchStatus::Closed)
            && !matches!(req.action, MutationAction::Reopen)
        {
            return Err(fail(
                StatusCode::CONFLICT,
                ToolStatus::Blocked,
                FailureClass::ValidationRejected,
                "closed Workbench must reopen first",
            ));
        }
        let mut n = latest;
        n.state_revision += 1;
        n.idempotency_key = req.idempotency_key.clone();
        n.updated_at = now;
        n
    };
    drop(snap);
    match req.action {
        MutationAction::Open => {}
        MutationAction::UpsertSection => {
            let x = req.section.ok_or_else(|| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "section required",
                )
            })?;
            let sid = x
                .section_id
                .unwrap_or_else(|| id("spec-section", &[&s.workbench_session_id, &x.title]));
            let existing = s.sections.iter().find(|z| z.section_id == sid).cloned();
            let rev = existing.as_ref().map_or(1, |z| z.revision);
            let created = existing.as_ref().map_or(now, |z| z.created_at);
            let reality_classification = if x.reality_classification.trim().is_empty() {
                if x.docs_only {
                    "docs_only".to_string()
                } else {
                    "unknown".to_string()
                }
            } else {
                x.reality_classification.trim().to_lowercase()
            };
            if !REALITY_CLASSIFICATIONS.contains(&reality_classification.as_str()) {
                return Err(fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "invalid reality_classification",
                ));
            }
            let status = if x.context_refs.is_empty() || x.evidence_refs.is_empty() {
                SpecSectionStatus::Draft
            } else {
                SpecSectionStatus::Grounded
            };
            let record = SpecSectionRecord {
                section_id: sid.clone(),
                title: x.title,
                section_kind: x.section_kind,
                reality_classification,
                status,
                order_index: x.order_index,
                revision: rev,
                content: x.content,
                grounding: SpecGroundingBlock {
                    context_refs: x.context_refs,
                    evidence_refs: x.evidence_refs,
                    codebase_refs: x.codebase_refs,
                    research_refs: x.research_refs,
                    docs_only: x.docs_only,
                },
                objection_ids: existing
                    .as_ref()
                    .map(|z| z.objection_ids.clone())
                    .unwrap_or_default(),
                approved_revision: None,
                operator_gate_id: None,
                amendment_ids: existing.map(|z| z.amendment_ids).unwrap_or_default(),
                created_at: created,
                updated_at: now,
            };
            if let Some(z) = s.sections.iter_mut().find(|z| z.section_id == sid) {
                *z = record
            } else {
                s.sections.push(record)
            }
            s.current_section_id = Some(sid);
        }
        MutationAction::AddRound => {
            let x = req.round.ok_or_else(|| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "round required",
                )
            })?;
            if !s.sections.iter().any(|z| z.section_id == x.section_id) {
                return Err(fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::NotFound,
                    "section missing",
                ));
            }
            let index = s
                .rounds
                .iter()
                .filter(|z| z.section_id == x.section_id)
                .count() as u32
                + 1;
            let rid = x.round_id.unwrap_or_else(|| {
                id(
                    "spec-round",
                    &[&s.workbench_session_id, &x.section_id, &index.to_string()],
                )
            });
            s.rounds.push(SpecRoundRecord {
                round_id: rid,
                section_id: x.section_id.clone(),
                round_index: index,
                round_kind: x.round_kind,
                output_refs: x.output_refs,
                transcript_ref: x.transcript_ref,
                verdict: x.verdict,
                stop_reason: x.stop_reason,
                created_at: now,
            });
            s.current_section_id = Some(x.section_id);
        }
        MutationAction::AddObjection => {
            let x = req.objection.ok_or_else(|| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "objection required",
                )
            })?;
            if !s
                .rounds
                .iter()
                .any(|z| z.round_id == x.round_id && z.section_id == x.section_id)
            {
                return Err(fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::NotFound,
                    "linked round missing",
                ));
            }
            let oid = x
                .objection_id
                .unwrap_or_else(|| id("spec-objection", &[&x.round_id, &req.idempotency_key]));
            s.objections.push(SpecObjectionRecord {
                objection_id: oid.clone(),
                section_id: x.section_id.clone(),
                round_id: x.round_id,
                actor_role: x.actor_role,
                claim: x.claim,
                reasoning_summary: x.reasoning_summary,
                evidence_refs: x.evidence_refs,
                confidence: x.confidence,
                status: SpecObjectionStatus::Open,
                resolution: None,
                created_at: now,
                updated_at: now,
            });
            let section = s
                .sections
                .iter_mut()
                .find(|z| z.section_id == x.section_id)
                .expect("round section checked");
            section.objection_ids.push(oid);
            section.status = SpecSectionStatus::Challenged;
            section.updated_at = now;
            s.current_section_id = Some(x.section_id);
        }
        MutationAction::ResolveObjection => {
            let oid = req.objection_id.as_deref().ok_or_else(|| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "objection_id required",
                )
            })?;
            let x = s
                .objections
                .iter_mut()
                .find(|z| z.objection_id == oid)
                .ok_or_else(|| {
                    fail(
                        StatusCode::NOT_FOUND,
                        ToolStatus::Blocked,
                        FailureClass::NotFound,
                        "objection missing",
                    )
                })?;
            x.status = SpecObjectionStatus::Resolved;
            x.resolution = req.resolution;
            x.updated_at = now;
            let sid = x.section_id.clone();
            if !s
                .objections
                .iter()
                .any(|z| z.section_id == sid && matches!(z.status, SpecObjectionStatus::Open))
            {
                let section = s
                    .sections
                    .iter_mut()
                    .find(|z| z.section_id == sid)
                    .expect("objection section retained");
                section.status = SpecSectionStatus::PendingApproval;
                section.updated_at = now;
            }
        }
        MutationAction::ApproveSection | MutationAction::RejectSection => {
            let x = req.decision.ok_or_else(|| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ApprovalRequired,
                    "operator decision required",
                )
            })?;
            let section = s
                .sections
                .iter_mut()
                .find(|z| z.section_id == x.section_id)
                .ok_or_else(|| {
                    fail(
                        StatusCode::NOT_FOUND,
                        ToolStatus::Blocked,
                        FailureClass::NotFound,
                        "section missing",
                    )
                })?;
            if matches!(req.action, MutationAction::ApproveSection)
                && s.objections.iter().any(|z| {
                    z.section_id == x.section_id && matches!(z.status, SpecObjectionStatus::Open)
                })
            {
                return Err(fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ApprovalRequired,
                    "resolve objections before approval",
                ));
            }
            let decision = if matches!(req.action, MutationAction::ApproveSection) {
                SpecGateDecision::Approve
            } else {
                SpecGateDecision::Reject
            };
            let gid = id(
                "spec-gate",
                &[&s.workbench_session_id, &x.section_id, &req.idempotency_key],
            );
            s.gates.push(SpecOperatorGateRecord {
                gate_id: gid.clone(),
                section_id: x.section_id.clone(),
                decision: decision.clone(),
                approval_scope: x
                    .approval_scope
                    .unwrap_or_else(|| "section_revision".into()),
                rationale: x.rationale,
                decided_by: x.decided_by,
                evidence_refs: x.evidence_refs,
                decided_at: now,
            });
            section.operator_gate_id = Some(gid);
            section.status = if matches!(decision, SpecGateDecision::Approve) {
                section.approved_revision = Some(section.revision);
                SpecSectionStatus::Approved
            } else {
                section.approved_revision = None;
                SpecSectionStatus::Rejected
            };
            section.updated_at = now;
        }
        MutationAction::AmendSection => {
            let x = req.amendment.ok_or_else(|| {
                fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ValidationRejected,
                    "amendment required",
                )
            })?;
            let section = s
                .sections
                .iter_mut()
                .find(|z| z.section_id == x.section_id)
                .ok_or_else(|| {
                    fail(
                        StatusCode::NOT_FOUND,
                        ToolStatus::Blocked,
                        FailureClass::NotFound,
                        "section missing",
                    )
                })?;
            let before = section.revision;
            let aid = id(
                "spec-amendment",
                &[&s.workbench_session_id, &x.section_id, &req.idempotency_key],
            );
            s.amendments.push(SpecAmendmentRecord {
                amendment_id: aid.clone(),
                section_id: x.section_id.clone(),
                before_revision: before,
                after_revision: before + 1,
                reason: x.reason,
                changed_by: x.changed_by,
                evidence_refs: x.evidence_refs,
                created_at: now,
            });
            section.revision += 1;
            section.content = x.content;
            section.status = SpecSectionStatus::Amended;
            section.approved_revision = None;
            section.operator_gate_id = None;
            section.amendment_ids.push(aid);
            section.updated_at = now;
            s.current_section_id = Some(x.section_id);
        }
        MutationAction::Close => {
            s.status = SpecWorkbenchStatus::Closed;
            s.closed_at = Some(now)
        }
        MutationAction::Reopen => {
            s.status = SpecWorkbenchStatus::Active;
            s.closed_at = None
        }
        MutationAction::FinalApprove => {
            if s.sections.is_empty()
                || s.sections
                    .iter()
                    .any(|z| !matches!(z.status, SpecSectionStatus::Approved))
            {
                return Err(fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ToolStatus::ValidationRejected,
                    FailureClass::ApprovalRequired,
                    "all sections require approval",
                ));
            }
            if s.desired_spec_template == "project_genesis" {
                let missing_sections: Vec<_> = PROJECT_GENESIS_SECTIONS
                    .iter()
                    .filter(|title| !s.sections.iter().any(|section| section.title == **title))
                    .copied()
                    .collect();
                if !missing_sections.is_empty() {
                    return Err(fail(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        ToolStatus::ValidationRejected,
                        FailureClass::ApprovalRequired,
                        format!(
                            "project_genesis final approval requires all 22 sections; missing: {}",
                            missing_sections.join(", ")
                        ),
                    ));
                }
            }
            s.status = SpecWorkbenchStatus::FinalApproved;
            s.final_spec_id = Some(id(
                "approved-spec",
                &[&s.workbench_session_id, &s.state_revision.to_string()],
            ));
            s.closed_at = None;
        }
    }
    let sid = s.workbench_session_id.clone();
    let key = s.idempotency_key.clone();
    let receipt = format!("receipt:spec-workbench:{sid}:{key}");
    s.receipt_refs.push(receipt);
    state
        .command_tx
        .send(Action::EmitEvent {
            event: FocusaEvent::SpecWorkbenchSessionRevised { session: s },
        })
        .await
        .map_err(|_| {
            fail(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Offline,
                FailureClass::DaemonUnavailable,
                "Workbench command channel unavailable",
            )
        })?;
    for _ in 0..100 {
        let current = state.focusa.read().await;
        if let Some(x) = current
            .spec_workbench_sessions
            .iter()
            .find(|x| x.workbench_session_id == sid && x.idempotency_key == key)
        {
            return Ok(Json(response(x.clone(), current.version, false)));
        }
        drop(current);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    Err(fail(
        StatusCode::SERVICE_UNAVAILABLE,
        ToolStatus::Degraded,
        FailureClass::ReadModelLag,
        "Workbench revision not visible",
    ))
}
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/spec-workbench/sessions", get(list))
        .route(ENDPOINT, post(mutate))
}
