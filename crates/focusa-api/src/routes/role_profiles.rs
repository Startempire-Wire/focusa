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
        Action, FocusaEvent, ProjectAgentRoleProfile, RoleAlternativeRecord, RoleAssumptionRecord,
        RoleProfileGrounding, RoleProfileStatus, RoleRedlineRecord, RoleReviewDecision,
        RoleReviewRecord,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{sync::Arc, time::Duration};

type ApiError = (StatusCode, Json<Box<ToolResultV1>>);

#[derive(Debug, Deserialize)]
pub struct RoleProfileQuery {
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RoleAssumptionInput {
    pub statement: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct RoleRedlineInput {
    pub field: String,
    pub before: String,
    pub after: String,
    pub rationale: String,
}

#[derive(Debug, Deserialize)]
pub struct RoleAlternativeInput {
    pub title: String,
    pub purpose: String,
    #[serde(default)]
    pub tradeoffs: Vec<String>,
    #[serde(default)]
    pub grounding_refs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RoleProfileDraftRequest {
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
    pub idempotency_key: String,
    pub expected_state_version: u64,
    pub original_seed: String,
    pub title: String,
    pub purpose: String,
    #[serde(default)]
    pub expertise: Vec<String>,
    #[serde(default)]
    pub primary_responsibilities: Vec<String>,
    #[serde(default)]
    pub secondary_responsibilities: Vec<String>,
    #[serde(default)]
    pub expected_deliverables: Vec<String>,
    #[serde(default)]
    pub quality_standards: Vec<String>,
    #[serde(default)]
    pub decision_principles: Vec<String>,
    #[serde(default)]
    pub evidence_expectations: Vec<String>,
    pub evidence_behavior: String,
    pub communication_posture: String,
    pub stakeholder_posture: String,
    #[serde(default)]
    pub non_responsibilities: Vec<String>,
    #[serde(default)]
    pub forbidden_assumptions: Vec<String>,
    #[serde(default)]
    pub escalation_triggers: Vec<String>,
    #[serde(default)]
    pub handoff_boundaries: Vec<String>,
    #[serde(default)]
    pub tool_preferences: Vec<String>,
    #[serde(default)]
    pub reviewer_lenses: Vec<String>,
    #[serde(default)]
    pub alternatives: Vec<RoleAlternativeInput>,
    #[serde(default)]
    pub context_artifact_refs: Vec<String>,
    #[serde(default)]
    pub context_claim_refs: Vec<String>,
    #[serde(default)]
    pub interview_answer_refs: Vec<String>,
    #[serde(default)]
    pub assumptions: Vec<RoleAssumptionInput>,
    #[serde(default)]
    pub unresolved_questions: Vec<String>,
    #[serde(default)]
    pub redlines: Vec<RoleRedlineInput>,
    #[serde(default)]
    pub permission_profile_refs: Vec<String>,
    /// Must remain empty. Role responsibility is not operational permission.
    #[serde(default)]
    pub permission_assertions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RoleProfileReviewRequest {
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
    pub role_profile_id: String,
    pub profile_revision: u64,
    pub idempotency_key: String,
    pub expected_state_version: u64,
    pub decision: String,
    pub reviewed_by: String,
    pub rationale: String,
}

#[derive(Debug, Serialize)]
pub struct RoleProfileListResponse {
    pub schema: &'static str,
    pub responsibility_is_not_permission: bool,
    pub state_version: u64,
    pub profiles: Vec<ProjectAgentRoleProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest: Option<ProjectAgentRoleProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved: Option<ProjectAgentRoleProfile>,
}

#[derive(Debug, Serialize)]
pub struct RoleProfileMutationResponse {
    pub schema: &'static str,
    pub canonical: bool,
    pub responsibility_is_not_permission: bool,
    pub replayed: bool,
    pub state_version: u64,
    pub profile: ProjectAgentRoleProfile,
    pub evidence_ref: String,
    pub receipt_ref: String,
    pub tool_result: ToolResultV1,
}

fn fail(
    status: StatusCode,
    tool_status: ToolStatus,
    class: FailureClass,
    summary: impl Into<String>,
    tool: &str,
    endpoint: &str,
) -> ApiError {
    let mut result = ToolResultV1::failure(tool_status, class, summary);
    result.tool = Some(tool.into());
    result.family = Some("project_role_profile".into());
    result.endpoint = Some(endpoint.into());
    result.next_tools = vec![
        "focusa_role_profiles_list".into(),
        "focusa_context_claims_list".into(),
        "focusa_evidence_capture".into(),
    ];
    (status, Json(Box::new(result)))
}

fn text(
    value: &str,
    field: &str,
    max: usize,
    tool: &str,
    endpoint: &str,
) -> Result<String, ApiError> {
    let value = value.trim();
    if value.is_empty() || value.len() > max {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            format!("{field} must contain 1-{max} characters"),
            tool,
            endpoint,
        ));
    }
    Ok(value.into())
}

fn strings(
    values: Vec<String>,
    field: &str,
    min: usize,
    max: usize,
    item_max: usize,
    tool: &str,
    endpoint: &str,
) -> Result<Vec<String>, ApiError> {
    if values.len() < min || values.len() > max {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            format!("{field} must contain {min}-{max} entries"),
            tool,
            endpoint,
        ));
    }
    let mut out = Vec::new();
    for value in values {
        out.push(text(&value, field, item_max, tool, endpoint)?);
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn stable(prefix: &str, parts: &[&str]) -> String {
    let mut hash = Sha256::new();
    for part in parts {
        hash.update((part.len() as u64).to_be_bytes());
        hash.update(part.as_bytes());
    }
    format!("{prefix}:{}", hex::encode(&hash.finalize()[..12]))
}

fn scoped(profile: &ProjectAgentRoleProfile, query: &RoleProfileQuery) -> bool {
    profile.project_root == query.project_root
        && profile.continuity_id == query.continuity_id
        && profile.attachment_id == query.attachment_id
}

fn contains_permission_grant(profile: &ProjectAgentRoleProfile) -> bool {
    let role_language = std::iter::once(profile.title.as_str())
        .chain(std::iter::once(profile.purpose.as_str()))
        .chain(profile.primary_responsibilities.iter().map(String::as_str))
        .chain(
            profile
                .secondary_responsibilities
                .iter()
                .map(String::as_str),
        )
        .chain(profile.expected_deliverables.iter().map(String::as_str));
    role_language.map(str::to_ascii_lowercase).any(|value| {
        [
            "permission to ",
            "authorized to ",
            "authority to ",
            "may file",
            "may send email",
            "may trade",
            "may modify production",
            "may access unapproved",
        ]
        .iter()
        .any(|pattern| value.contains(pattern))
    })
}

fn mutation_response(
    profile: ProjectAgentRoleProfile,
    state_version: u64,
    replayed: bool,
    tool_name: &str,
    endpoint: &str,
) -> RoleProfileMutationResponse {
    let evidence_ref = profile
        .grounding
        .context_claim_refs
        .first()
        .or_else(|| profile.grounding.context_artifact_refs.first())
        .or_else(|| profile.grounding.interview_answer_refs.first())
        .cloned()
        .unwrap_or_else(|| profile.grounding.operator_seed_ref.clone());
    let receipt_ref = stable(
        "receipt:project-role-profile",
        &[
            &profile.role_profile_id,
            &profile.revision.to_string(),
            &profile.idempotency_key,
        ],
    );
    let mut tool = ToolResultV1::success(
        if replayed {
            ToolStatus::NoOp
        } else {
            ToolStatus::Completed
        },
        if replayed {
            "Project role profile mutation replayed idempotently"
        } else {
            "Context-grounded project role profile revision committed without granting permission"
        },
    );
    tool.tool = Some(tool_name.into());
    tool.family = Some("project_role_profile".into());
    tool.endpoint = Some(endpoint.into());
    tool.evidence_refs = vec![evidence_ref.clone()];
    tool.side_effects = if replayed {
        vec![]
    } else {
        vec!["project_role_profile_revision_committed".into()]
    };
    tool.next_tools = vec![
        "focusa_role_profiles_list".into(),
        "focusa_evidence_capture".into(),
    ];
    RoleProfileMutationResponse {
        schema: "focusa.project_agent_role_profile_mutation_result.v1",
        canonical: true,
        responsibility_is_not_permission: true,
        replayed,
        state_version,
        profile,
        evidence_ref,
        receipt_ref,
        tool_result: tool,
    }
}

async fn list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<RoleProfileQuery>,
) -> Result<Json<RoleProfileListResponse>, ApiError> {
    const TOOL: &str = "focusa_role_profiles_list";
    const ENDPOINT: &str = "/v1/roles/profiles";
    let query = RoleProfileQuery {
        project_root: text(&query.project_root, "project_root", 4096, TOOL, ENDPOINT)?,
        continuity_id: text(&query.continuity_id, "continuity_id", 256, TOOL, ENDPOINT)?,
        attachment_id: text(&query.attachment_id, "attachment_id", 256, TOOL, ENDPOINT)?,
    };
    let snapshot = state.focusa.read().await;
    let profiles: Vec<ProjectAgentRoleProfile> = snapshot
        .project_role_profiles
        .iter()
        .filter(|profile| scoped(profile, &query))
        .cloned()
        .collect();
    let latest = profiles
        .iter()
        .max_by_key(|profile| profile.revision)
        .cloned();
    let approved = profiles
        .iter()
        .filter(|profile| matches!(profile.status, RoleProfileStatus::Approved))
        .max_by_key(|profile| profile.revision)
        .cloned();
    Ok(Json(RoleProfileListResponse {
        schema: "focusa.project_agent_role_profile_list.v1",
        responsibility_is_not_permission: true,
        state_version: snapshot.version,
        profiles,
        latest,
        approved,
    }))
}

async fn draft(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RoleProfileDraftRequest>,
) -> Result<Json<RoleProfileMutationResponse>, ApiError> {
    const TOOL: &str = "focusa_role_profile_draft";
    const ENDPOINT: &str = "/v1/roles/profiles/draft";
    let project_root = text(&request.project_root, "project_root", 4096, TOOL, ENDPOINT)?;
    let continuity_id = text(&request.continuity_id, "continuity_id", 256, TOOL, ENDPOINT)?;
    let attachment_id = text(&request.attachment_id, "attachment_id", 256, TOOL, ENDPOINT)?;
    let idempotency_key = text(
        &request.idempotency_key,
        "idempotency_key",
        256,
        TOOL,
        ENDPOINT,
    )?;
    if !request.permission_assertions.is_empty() {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::PermissionDenied,
            "role responsibility cannot grant file, email, trade, production, source, or other operational permission",
            TOOL,
            ENDPOINT,
        ));
    }
    let original_seed = text(
        &request.original_seed,
        "original_seed",
        2000,
        TOOL,
        ENDPOINT,
    )?;
    let context_artifact_refs = strings(
        request.context_artifact_refs,
        "context_artifact_refs",
        0,
        64,
        512,
        TOOL,
        ENDPOINT,
    )?;
    let context_claim_refs = strings(
        request.context_claim_refs,
        "context_claim_refs",
        0,
        64,
        512,
        TOOL,
        ENDPOINT,
    )?;
    let interview_answer_refs = strings(
        request.interview_answer_refs,
        "interview_answer_refs",
        0,
        64,
        512,
        TOOL,
        ENDPOINT,
    )?;
    if context_artifact_refs.is_empty()
        && context_claim_refs.is_empty()
        && interview_answer_refs.is_empty()
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "Role draft requires at least one Context artifact, claim, or interview-answer grounding ref",
            TOOL,
            ENDPOINT,
        ));
    }
    let role_profile_id = stable(
        "project-role-profile",
        &[&project_root, &continuity_id, &attachment_id],
    );
    let default_grounding_ref = context_artifact_refs
        .first()
        .or_else(|| context_claim_refs.first())
        .or_else(|| interview_answer_refs.first())
        .cloned()
        .unwrap_or_default();
    let alternative_inputs = if request.alternatives.is_empty() {
        vec![RoleAlternativeInput {
            title: format!("{} — narrow-scope alternative", request.title.trim()),
            purpose: format!(
                "Serve the same grounded purpose with reduced responsibilities: {}",
                request.purpose.trim()
            ),
            tradeoffs: vec![
                "Lower authority ambiguity, with narrower project coverage".to_string(),
            ],
            grounding_refs: vec![default_grounding_ref],
        }]
    } else {
        request.alternatives
    };
    let mut alternatives = Vec::with_capacity(alternative_inputs.len());
    for (index, alternative) in alternative_inputs.into_iter().enumerate() {
        let title = text(&alternative.title, "alternative.title", 200, TOOL, ENDPOINT)?;
        let grounding_refs = strings(
            alternative.grounding_refs,
            "alternative.grounding_refs",
            1,
            32,
            512,
            TOOL,
            ENDPOINT,
        )?;
        if grounding_refs.iter().any(|reference| {
            !context_artifact_refs.contains(reference)
                && !context_claim_refs.contains(reference)
                && !interview_answer_refs.contains(reference)
        }) {
            return Err(fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ValidationRejected,
                "role alternatives may cite only grounding refs accepted by the draft",
                TOOL,
                ENDPOINT,
            ));
        }
        alternatives.push(RoleAlternativeRecord {
            alternative_id: stable(
                "role-alternative",
                &[&role_profile_id, &index.to_string(), &title],
            ),
            title,
            purpose: text(
                &alternative.purpose,
                "alternative.purpose",
                2000,
                TOOL,
                ENDPOINT,
            )?,
            tradeoffs: strings(
                alternative.tradeoffs,
                "alternative.tradeoffs",
                1,
                16,
                512,
                TOOL,
                ENDPOINT,
            )?,
            grounding_refs,
        });
    }
    let now = Utc::now();
    let mut profile = ProjectAgentRoleProfile {
        role_profile_id: role_profile_id.clone(),
        project_root: project_root.clone(),
        continuity_id: continuity_id.clone(),
        attachment_id: attachment_id.clone(),
        revision: 1,
        original_seed: original_seed.clone(),
        title: text(&request.title, "title", 200, TOOL, ENDPOINT)?,
        purpose: text(&request.purpose, "purpose", 2000, TOOL, ENDPOINT)?,
        expertise: strings(request.expertise, "expertise", 1, 32, 256, TOOL, ENDPOINT)?,
        primary_responsibilities: strings(
            request.primary_responsibilities,
            "primary_responsibilities",
            1,
            32,
            512,
            TOOL,
            ENDPOINT,
        )?,
        secondary_responsibilities: strings(
            request.secondary_responsibilities,
            "secondary_responsibilities",
            0,
            32,
            512,
            TOOL,
            ENDPOINT,
        )?,
        expected_deliverables: strings(
            request.expected_deliverables,
            "expected_deliverables",
            1,
            32,
            512,
            TOOL,
            ENDPOINT,
        )?,
        quality_standards: strings(
            request.quality_standards,
            "quality_standards",
            1,
            32,
            512,
            TOOL,
            ENDPOINT,
        )?,
        decision_principles: strings(
            request.decision_principles,
            "decision_principles",
            1,
            32,
            512,
            TOOL,
            ENDPOINT,
        )?,
        evidence_expectations: strings(
            request.evidence_expectations,
            "evidence_expectations",
            1,
            32,
            512,
            TOOL,
            ENDPOINT,
        )?,
        evidence_behavior: text(
            &request.evidence_behavior,
            "evidence_behavior",
            1000,
            TOOL,
            ENDPOINT,
        )?,
        communication_posture: text(
            &request.communication_posture,
            "communication_posture",
            1000,
            TOOL,
            ENDPOINT,
        )?,
        stakeholder_posture: text(
            &request.stakeholder_posture,
            "stakeholder_posture",
            1000,
            TOOL,
            ENDPOINT,
        )?,
        non_responsibilities: strings(
            request.non_responsibilities,
            "non_responsibilities",
            1,
            32,
            512,
            TOOL,
            ENDPOINT,
        )?,
        forbidden_assumptions: strings(
            request.forbidden_assumptions,
            "forbidden_assumptions",
            1,
            32,
            512,
            TOOL,
            ENDPOINT,
        )?,
        escalation_triggers: strings(
            request.escalation_triggers,
            "escalation_triggers",
            1,
            32,
            512,
            TOOL,
            ENDPOINT,
        )?,
        handoff_boundaries: strings(
            request.handoff_boundaries,
            "handoff_boundaries",
            1,
            32,
            512,
            TOOL,
            ENDPOINT,
        )?,
        tool_preferences: strings(
            request.tool_preferences,
            "tool_preferences",
            0,
            32,
            256,
            TOOL,
            ENDPOINT,
        )?,
        reviewer_lenses: strings(
            request.reviewer_lenses,
            "reviewer_lenses",
            0,
            32,
            256,
            TOOL,
            ENDPOINT,
        )?,
        alternatives,
        grounding: RoleProfileGrounding {
            context_artifact_refs,
            context_claim_refs,
            interview_answer_refs,
            operator_seed_ref: stable("operator-role-seed", &[&project_root, &original_seed]),
        },
        assumptions: request
            .assumptions
            .into_iter()
            .enumerate()
            .map(|(index, assumption)| {
                let statement = text(
                    &assumption.statement,
                    "assumption.statement",
                    1000,
                    TOOL,
                    ENDPOINT,
                )?;
                let status = text(&assumption.status, "assumption.status", 32, TOOL, ENDPOINT)?;
                if !matches!(status.as_str(), "unverified" | "grounded" | "rejected") {
                    return Err(fail(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        ToolStatus::ValidationRejected,
                        FailureClass::ValidationRejected,
                        "assumption.status must be unverified, grounded, or rejected",
                        TOOL,
                        ENDPOINT,
                    ));
                }
                Ok(RoleAssumptionRecord {
                    assumption_id: stable(
                        "role-assumption",
                        &[&role_profile_id, &index.to_string(), &statement],
                    ),
                    statement,
                    source_refs: strings(
                        assumption.source_refs,
                        "assumption.source_refs",
                        0,
                        16,
                        512,
                        TOOL,
                        ENDPOINT,
                    )?,
                    status,
                })
            })
            .collect::<Result<Vec<_>, ApiError>>()?,
        unresolved_questions: strings(
            request.unresolved_questions,
            "unresolved_questions",
            0,
            32,
            1000,
            TOOL,
            ENDPOINT,
        )?,
        redlines: request
            .redlines
            .into_iter()
            .map(|redline| {
                Ok(RoleRedlineRecord {
                    field: text(&redline.field, "redline.field", 128, TOOL, ENDPOINT)?,
                    before: text(&redline.before, "redline.before", 2000, TOOL, ENDPOINT)?,
                    after: text(&redline.after, "redline.after", 2000, TOOL, ENDPOINT)?,
                    rationale: text(
                        &redline.rationale,
                        "redline.rationale",
                        1000,
                        TOOL,
                        ENDPOINT,
                    )?,
                })
            })
            .collect::<Result<Vec<_>, ApiError>>()?,
        grants_permissions: false,
        permission_profile_refs: strings(
            request.permission_profile_refs,
            "permission_profile_refs",
            0,
            32,
            512,
            TOOL,
            ENDPOINT,
        )?,
        status: RoleProfileStatus::PendingOperator,
        review: None,
        idempotency_key: idempotency_key.clone(),
        created_at: now,
        updated_at: now,
    };
    if contains_permission_grant(&profile) {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::PermissionDenied,
            "role title, purpose, responsibilities, and deliverables cannot contain operational permission grants",
            TOOL,
            ENDPOINT,
        ));
    }

    let writer = state.write_serial_lock.lock().await;
    let snapshot = state.focusa.read().await.clone();
    let scope = RoleProfileQuery {
        project_root,
        continuity_id,
        attachment_id,
    };
    if let Some(existing) = snapshot
        .project_role_profiles
        .iter()
        .find(|existing| scoped(existing, &scope) && existing.idempotency_key == idempotency_key)
    {
        return Ok(Json(mutation_response(
            existing.clone(),
            snapshot.version,
            true,
            TOOL,
            ENDPOINT,
        )));
    }
    let missing_artifact_ref = profile
        .grounding
        .context_artifact_refs
        .iter()
        .find(|reference| {
            !snapshot.context_sources.iter().any(|source| {
                source.source_id == **reference
                    && source.project_root == scope.project_root
                    && source.continuity_id == scope.continuity_id
                    && source.attachment_id == scope.attachment_id
            }) && !snapshot.workspace_artifacts.iter().any(|artifact| {
                artifact.artifact_id == **reference
                    && artifact.scope.project_root == scope.project_root
                    && artifact.scope.continuity_id == scope.continuity_id
                    && artifact.origin.attachment_id == scope.attachment_id
            })
        });
    let missing_claim_ref = profile
        .grounding
        .context_claim_refs
        .iter()
        .find(|reference| {
            !snapshot.context_claims.iter().any(|claim| {
                claim.claim_id == **reference
                    && claim.project_root == scope.project_root
                    && claim.continuity_id == scope.continuity_id
                    && claim.attachment_id == scope.attachment_id
            })
        });
    if let Some(reference) = missing_artifact_ref.or(missing_claim_ref) {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::NotFound,
            format!("Role grounding ref is not canonical in exact Focusa state: {reference}"),
            TOOL,
            ENDPOINT,
        ));
    }
    if profile.grounding.context_artifact_refs.is_empty()
        && profile.grounding.context_claim_refs.is_empty()
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "Role draft requires canonical Context source, Workspace Artifact, or claim grounding; interview refs are supplemental",
            TOOL,
            ENDPOINT,
        ));
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
            TOOL,
            ENDPOINT,
        ));
    }
    if let Some(latest) = snapshot
        .project_role_profiles
        .iter()
        .filter(|existing| existing.role_profile_id == role_profile_id)
        .max_by_key(|existing| existing.revision)
    {
        if profile.original_seed != latest.original_seed {
            return Err(fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ValidationRejected,
                "original_seed is immutable across role profile revisions",
                TOOL,
                ENDPOINT,
            ));
        }
        if profile.redlines.is_empty() {
            return Err(fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ValidationRejected,
                "revised role draft requires an explicit before/after redline",
                TOOL,
                ENDPOINT,
            ));
        }
        profile.revision = latest.revision + 1;
        profile.created_at = latest.created_at;
    }
    drop(writer);
    state
        .command_tx
        .send(Action::EmitEvent {
            event: FocusaEvent::ProjectRoleProfileRevised {
                profile: profile.clone(),
            },
        })
        .await
        .map_err(|_| {
            fail(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Offline,
                FailureClass::DaemonUnavailable,
                "canonical project role command channel unavailable",
                TOOL,
                ENDPOINT,
            )
        })?;
    wait_for_profile(&state, &role_profile_id, &idempotency_key, TOOL, ENDPOINT).await
}

async fn review(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RoleProfileReviewRequest>,
) -> Result<Json<RoleProfileMutationResponse>, ApiError> {
    const TOOL: &str = "focusa_role_profile_review";
    const ENDPOINT: &str = "/v1/roles/profiles/review";
    let scope = RoleProfileQuery {
        project_root: text(&request.project_root, "project_root", 4096, TOOL, ENDPOINT)?,
        continuity_id: text(&request.continuity_id, "continuity_id", 256, TOOL, ENDPOINT)?,
        attachment_id: text(&request.attachment_id, "attachment_id", 256, TOOL, ENDPOINT)?,
    };
    let role_profile_id = text(
        &request.role_profile_id,
        "role_profile_id",
        256,
        TOOL,
        ENDPOINT,
    )?;
    let idempotency_key = text(
        &request.idempotency_key,
        "idempotency_key",
        256,
        TOOL,
        ENDPOINT,
    )?;
    let decision_text = text(&request.decision, "decision", 32, TOOL, ENDPOINT)?;
    let (decision, status) = match decision_text.as_str() {
        "approve" => (RoleReviewDecision::Approve, RoleProfileStatus::Approved),
        "reject" => (RoleReviewDecision::Reject, RoleProfileStatus::Superseded),
        "defer" => (
            RoleReviewDecision::Defer,
            RoleProfileStatus::PendingOperator,
        ),
        _ => {
            return Err(fail(
                StatusCode::UNPROCESSABLE_ENTITY,
                ToolStatus::ValidationRejected,
                FailureClass::ValidationRejected,
                "decision must be approve, reject, or defer",
                TOOL,
                ENDPOINT,
            ));
        }
    };
    let writer = state.write_serial_lock.lock().await;
    let snapshot = state.focusa.read().await.clone();
    if let Some(existing) = snapshot
        .project_role_profiles
        .iter()
        .find(|existing| scoped(existing, &scope) && existing.idempotency_key == idempotency_key)
    {
        return Ok(Json(mutation_response(
            existing.clone(),
            snapshot.version,
            true,
            TOOL,
            ENDPOINT,
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
            TOOL,
            ENDPOINT,
        ));
    }
    let latest = snapshot
        .project_role_profiles
        .iter()
        .filter(|profile| profile.role_profile_id == role_profile_id && scoped(profile, &scope))
        .max_by_key(|profile| profile.revision)
        .cloned()
        .ok_or_else(|| {
            fail(
                StatusCode::NOT_FOUND,
                ToolStatus::Blocked,
                FailureClass::NotFound,
                "project role profile not found in exact scope",
                TOOL,
                ENDPOINT,
            )
        })?;
    if latest.revision != request.profile_revision {
        return Err(fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            format!(
                "profile_revision={} does not match latest revision={}",
                request.profile_revision, latest.revision
            ),
            TOOL,
            ENDPOINT,
        ));
    }
    if !matches!(
        latest.status,
        RoleProfileStatus::Draft | RoleProfileStatus::PendingOperator
    ) {
        return Err(fail(
            StatusCode::CONFLICT,
            ToolStatus::Blocked,
            FailureClass::WriterConflict,
            "only a draft or pending_operator role revision can be reviewed",
            TOOL,
            ENDPOINT,
        ));
    }
    if matches!(decision, RoleReviewDecision::Approve)
        && (!latest.unresolved_questions.is_empty()
            || latest
                .assumptions
                .iter()
                .any(|assumption| assumption.status == "unverified"))
    {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            ToolStatus::ValidationRejected,
            FailureClass::ValidationRejected,
            "approval requires resolved questions and grounded or rejected assumptions",
            TOOL,
            ENDPOINT,
        ));
    }
    let mut profile = latest;
    profile.revision += 1;
    profile.status = status;
    profile.idempotency_key = idempotency_key.clone();
    profile.updated_at = Utc::now();
    profile.review = Some(RoleReviewRecord {
        decision,
        reviewed_by: text(&request.reviewed_by, "reviewed_by", 256, TOOL, ENDPOINT)?,
        reviewed_at: profile.updated_at,
        rationale: text(&request.rationale, "rationale", 2000, TOOL, ENDPOINT)?,
    });
    drop(writer);
    state
        .command_tx
        .send(Action::EmitEvent {
            event: FocusaEvent::ProjectRoleProfileRevised {
                profile: profile.clone(),
            },
        })
        .await
        .map_err(|_| {
            fail(
                StatusCode::SERVICE_UNAVAILABLE,
                ToolStatus::Offline,
                FailureClass::DaemonUnavailable,
                "canonical project role review channel unavailable",
                TOOL,
                ENDPOINT,
            )
        })?;
    wait_for_profile(&state, &role_profile_id, &idempotency_key, TOOL, ENDPOINT).await
}

async fn wait_for_profile(
    state: &Arc<AppState>,
    role_profile_id: &str,
    idempotency_key: &str,
    tool: &str,
    endpoint: &str,
) -> Result<Json<RoleProfileMutationResponse>, ApiError> {
    for _ in 0..100 {
        let current = state.focusa.read().await;
        if let Some(profile) = current.project_role_profiles.iter().find(|profile| {
            profile.role_profile_id == role_profile_id && profile.idempotency_key == idempotency_key
        }) {
            return Ok(Json(mutation_response(
                profile.clone(),
                current.version,
                false,
                tool,
                endpoint,
            )));
        }
        drop(current);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    Err(fail(
        StatusCode::SERVICE_UNAVAILABLE,
        ToolStatus::Degraded,
        FailureClass::ReadModelLag,
        "project role profile mutation dispatched but not visible",
        tool,
        endpoint,
    ))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/roles/profiles", get(list))
        .route("/v1/roles/profiles/draft", post(draft))
        .route("/v1/roles/profiles/review", post(review))
}
