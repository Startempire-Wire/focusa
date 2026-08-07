use std::collections::{BTreeMap, BTreeSet};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::workstream_context::{WorkstreamContext, WorkstreamContextError};

use super::{
    layout::{
        resolve_layout, validate_no_dead_chrome, InspectorSide, LayoutConstraints, LayoutError,
    },
    model::{
        CandidateContribution, CompositionEvent, ResolvedContribution, ResolvedWorkspaceProjection,
        ScopedCandidateContribution,
    },
    profiles::{ActivityModeDefinition, WorkspaceProfileDefinition},
    resolver::{collect_candidates, resolve_eligibility, EligibilityContext, EligibilityDecision},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResolveProjectionInput {
    pub candidates: Vec<CandidateContribution>,
    pub eligibility: EligibilityContext,
    pub workspace_profile_revision: u64,
    pub activity_mode_revision: u64,
    pub focused_work_surface_id: Option<String>,
    pub canonical_read_model_revision: u64,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub viewport_class: String,
    pub focused_semantic_target: String,
    pub previous_projection_revision: u64,
    pub previous_layout_revision: u64,
    pub event_cursor: String,
    pub causation_id: Option<String>,
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecompositionEvidence {
    pub evidence_id: String,
    #[serde(flatten)]
    pub scope: super::model::MissionCanvasScope,
    pub trigger: String,
    pub input_projection_digest: Option<String>,
    pub output_projection_digest: String,
    pub rule_revision: String,
    pub candidate_contribution_ids: Vec<String>,
    pub eligibility_decisions: Vec<EligibilityDecision>,
    pub observed_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecompositionReceipt {
    pub receipt_id: String,
    #[serde(flatten)]
    pub scope: super::model::MissionCanvasScope,
    pub accepted: bool,
    pub projection_revision: u64,
    pub layout_revision: u64,
    pub projection_digest: String,
    pub event_cursor: String,
    pub evidence_id: String,
    pub idempotency_key: String,
    pub issued_at: String,
}

#[derive(Clone, Debug)]
pub struct RecompositionResult {
    pub projection: ResolvedWorkspaceProjection,
    pub evidence: RecompositionEvidence,
    pub receipt: RecompositionReceipt,
    pub event: CompositionEvent,
}

pub const PROFILE_SELECT_OPERATION: &str = "focusa.mission_canvas.profile.select";
pub const PROFILE_SELECT_PERMISSION: &str = "mission_canvas:write";
pub const ACTIVITY_SELECT_OPERATION: &str = "focusa.mission_canvas.activity.select";
pub const ACTIVITY_SELECT_PERMISSION: &str = "mission_canvas:write";

/// Core-owned command for selecting a canonical Activity Mode. The API
/// adapter supplies the exact Workstream context, current profile, selected
/// activity, scoped candidate registry, and capability/permission projection;
/// this service owns eligibility, layout recomposition, Evidence, Receipt, and
/// the direct projection returned to the generated client.
#[derive(Clone, Debug)]
pub struct ActivitySelectionCommand {
    pub context: WorkstreamContext,
    pub scope: super::model::MissionCanvasScope,
    pub current_projection: ResolvedWorkspaceProjection,
    pub profile: WorkspaceProfileDefinition,
    pub activity: ActivityModeDefinition,
    pub candidates: Vec<CandidateContribution>,
    pub capabilities: BTreeSet<String>,
    pub permissions: BTreeSet<String>,
    pub available_operations: BTreeSet<String>,
    pub expected_projection_revision: u64,
    pub expected_event_cursor: Option<String>,
    pub idempotency_key: String,
}

#[derive(Debug, Error)]
pub enum ActivitySelectionError {
    #[error("activity selection Workstream context is invalid: {0}")]
    Context(#[from] WorkstreamContextError),
    #[error("activity selection scope is invalid: {0}")]
    Scope(&'static str),
    #[error("activity selection requires permission: {0}")]
    PermissionDenied(String),
    #[error("activity selection requires a non-empty idempotency key")]
    IdempotencyKeyRequired,
    #[error("activity selection projection revision is stale")]
    RevisionConflict,
    #[error("activity selection event cursor is stale")]
    CursorConflict,
    #[error("current workspace profile is unavailable: {0}")]
    ProfileUnavailable(String),
    #[error("selected activity mode is unavailable: {0}")]
    ActivityUnavailable(String),
    #[error("selected activity has no meaningful eligible contribution")]
    NoMeaningfulContribution,
    #[error(transparent)]
    Recomposition(#[from] RecompositionError),
}

/// The reusable Core operation behind `activity.select`. It changes only the
/// canonical activity selection and recomposes the projection. Workstream and
/// subordinate Attachment authority are copied from the validated command and
/// are never inferred or replaced by the selected activity.
#[derive(Clone, Copy, Debug, Default)]
pub struct ActivitySelectionService;

impl ActivitySelectionService {
    pub fn select(
        &self,
        command: ActivitySelectionCommand,
    ) -> Result<RecompositionResult, ActivitySelectionError> {
        validate_activity_selection_context(&command)?;

        if !has_activity_selection_permission(&command.permissions) {
            return Err(ActivitySelectionError::PermissionDenied(
                ACTIVITY_SELECT_PERMISSION.into(),
            ));
        }
        if command.idempotency_key.trim().is_empty() {
            return Err(ActivitySelectionError::IdempotencyKeyRequired);
        }
        if command.profile.profile_id.trim().is_empty()
            || !command.profile.installed
            || command.profile.profile_id != command.current_projection.workspace_profile_id
        {
            return Err(ActivitySelectionError::ProfileUnavailable(
                command.profile.profile_id,
            ));
        }
        if command.activity.activity_mode_id.trim().is_empty()
            || command.activity.display_name.trim().is_empty()
            || command.activity.viability_rule_revision.trim().is_empty()
        {
            return Err(ActivitySelectionError::ActivityUnavailable(
                command.activity.activity_mode_id,
            ));
        }

        let profile_contribution_ids = command
            .profile
            .candidate_contribution_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let activity_contribution_ids = command
            .activity
            .candidate_contribution_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let scoped_candidates = command
            .candidates
            .into_iter()
            .map(|candidate| {
                ScopedCandidateContribution::new(candidate, command.scope.clone())
                    .map_err(ActivitySelectionError::Scope)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let candidate_ids = scoped_candidates
            .iter()
            .map(|candidate| candidate.candidate.contribution_id.clone())
            .collect::<BTreeSet<_>>();
        let candidates = collect_candidates(
            scoped_candidates,
            &profile_contribution_ids,
            &activity_contribution_ids,
            &command.scope,
        );
        if candidates.is_empty()
            || !candidates
                .iter()
                .any(|candidate| candidate_ids.contains(&candidate.contribution_id))
        {
            return Err(ActivitySelectionError::NoMeaningfulContribution);
        }

        let current_eligible = command
            .current_projection
            .eligible_contributions
            .iter()
            .map(|contribution| contribution.contribution_id.clone())
            .collect::<BTreeSet<_>>();
        let meaningful_content = candidates
            .iter()
            .filter(|candidate| current_eligible.contains(&candidate.contribution_id))
            .map(|candidate| (candidate.contribution_id.clone(), true))
            .collect::<BTreeMap<_, _>>();
        let focused_semantic_target = if candidates.iter().any(|candidate| {
            candidate.semantic_binding_id == command.current_projection.focused_semantic_target
        }) {
            command.current_projection.focused_semantic_target.clone()
        } else {
            String::new()
        };
        let next_projection_revision = command
            .expected_projection_revision
            .checked_add(1)
            .ok_or(RecompositionError::RevisionOverflow)?;
        let input = ResolveProjectionInput {
            candidates,
            eligibility: EligibilityContext {
                scope: command.scope.clone(),
                profile_id: command.profile.profile_id.clone(),
                activity_mode_id: command.activity.activity_mode_id.clone(),
                projection_revision: next_projection_revision,
                capabilities: command.capabilities,
                permissions: command.permissions,
                available_operations: command.available_operations,
                meaningful_content,
                previously_eligible: current_eligible,
                observed_at: Utc::now().to_rfc3339(),
            },
            workspace_profile_revision: command.profile.revision,
            activity_mode_revision: command.activity.revision,
            focused_work_surface_id: command.current_projection.focused_work_surface_id.clone(),
            canonical_read_model_revision: command.current_projection.canonical_read_model_revision,
            viewport_width: 1440,
            viewport_height: 900,
            viewport_class: "standard".into(),
            focused_semantic_target,
            previous_projection_revision: command.expected_projection_revision,
            previous_layout_revision: command.current_projection.layout_revision,
            event_cursor: command.current_projection.durable_event_cursor.clone(),
            causation_id: Some(command.idempotency_key.clone()),
            idempotency_key: command.idempotency_key.clone(),
        };
        let previous_activity_mode_id = command.current_projection.activity_mode_id.clone();
        let mut result = resolve_projection(
            input,
            Some(command.current_projection.projection_digest.clone()),
        )?;
        result.evidence.trigger = "activity_mode_change".into();
        result.event.event_kind = "activity_mode_changed".into();
        if let Some(payload) = result.event.payload.as_object_mut() {
            payload.insert(
                "activity_selection".into(),
                json!({
                    "operation_id": ACTIVITY_SELECT_OPERATION,
                    "activity_mode_id": command.activity.activity_mode_id.clone(),
                    "previous_activity_mode_id": previous_activity_mode_id,
                    "workstream": command.scope.workstream.clone(),
                }),
            );
            if let Some(evidence) = payload.get_mut("evidence") {
                evidence["trigger"] = json!("activity_mode_change");
            }
        }
        Ok(result)
    }
}

fn validate_activity_selection_context(
    command: &ActivitySelectionCommand,
) -> Result<(), ActivitySelectionError> {
    command
        .scope
        .validate()
        .map_err(ActivitySelectionError::Scope)?;
    command
        .context
        .validate_for_workstream(&command.scope.workstream)?;
    if command.context.attachment != command.scope.attachment {
        return Err(ActivitySelectionError::Context(
            WorkstreamContextError::WorkstreamMismatch,
        ));
    }
    let expected_continuity = command.scope.continuity_id.clone().or_else(|| {
        command
            .scope
            .attachment
            .as_ref()
            .and_then(|attachment| attachment.continuity_id.clone())
    });
    if command.context.continuity_id != expected_continuity {
        return Err(ActivitySelectionError::Context(
            WorkstreamContextError::ContinuityMismatch,
        ));
    }
    let expected_binding = command.scope.workspace_binding_id.clone().or_else(|| {
        command
            .scope
            .attachment
            .as_ref()
            .map(|attachment| attachment.workspace_binding_id.clone())
    });
    if command.context.workspace_binding_id != expected_binding {
        return Err(ActivitySelectionError::Context(
            WorkstreamContextError::WorkspaceBindingMismatch,
        ));
    }
    command
        .current_projection
        .validate_scope(&command.scope)
        .map_err(ActivitySelectionError::Scope)?;
    if command.current_projection.projection_revision != command.expected_projection_revision {
        return Err(ActivitySelectionError::RevisionConflict);
    }
    if command
        .expected_event_cursor
        .as_deref()
        .is_some_and(|cursor| cursor != command.current_projection.durable_event_cursor)
    {
        return Err(ActivitySelectionError::CursorConflict);
    }
    Ok(())
}

fn has_activity_selection_permission(permissions: &BTreeSet<String>) -> bool {
    permissions.contains(ACTIVITY_SELECT_PERMISSION)
        || permissions.contains("mission_canvas:*")
        || permissions.contains("admin:*")
        || permissions.contains("*")
}

/// Core-owned command for selecting a canonical Workspace Profile.  The API
/// adapter supplies the exact Workstream context, canonical profile/activity
/// records, scoped candidate registry, and capability/permission projection;
/// this service owns validation, eligibility, layout recomposition, evidence,
/// and receipt construction.
#[derive(Clone, Debug)]
pub struct ProfileSelectionCommand {
    pub context: WorkstreamContext,
    pub scope: super::model::MissionCanvasScope,
    pub current_projection: ResolvedWorkspaceProjection,
    pub profile: WorkspaceProfileDefinition,
    pub activity: ActivityModeDefinition,
    pub candidates: Vec<CandidateContribution>,
    pub capabilities: BTreeSet<String>,
    pub permissions: BTreeSet<String>,
    pub available_operations: BTreeSet<String>,
    pub expected_projection_revision: u64,
    pub expected_event_cursor: Option<String>,
    pub idempotency_key: String,
}

#[derive(Debug, Error)]
pub enum ProfileSelectionError {
    #[error("profile selection Workstream context is invalid: {0}")]
    Context(#[from] WorkstreamContextError),
    #[error("profile selection scope is invalid: {0}")]
    Scope(&'static str),
    #[error("profile selection requires permission: {0}")]
    PermissionDenied(String),
    #[error("profile selection requires a non-empty idempotency key")]
    IdempotencyKeyRequired,
    #[error("profile selection projection revision is stale")]
    RevisionConflict,
    #[error("profile selection event cursor is stale")]
    CursorConflict,
    #[error("workspace profile is unavailable: {0}")]
    ProfileUnavailable(String),
    #[error("current activity mode is unavailable: {0}")]
    ActivityUnavailable(String),
    #[error("selected profile has no meaningful eligible contribution")]
    NoMeaningfulContribution,
    #[error(transparent)]
    Recomposition(#[from] RecompositionError),
}

/// The reusable Core operation behind `profile.select`.  It deliberately
/// returns the same recomposition result as `projection.resolve`, so the API
/// can persist one direct generated projection and its exact Workstream-scoped
/// Evidence/Receipt references without a route-local composition algorithm.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProfileSelectionService;

impl ProfileSelectionService {
    pub fn select(
        &self,
        command: ProfileSelectionCommand,
    ) -> Result<RecompositionResult, ProfileSelectionError> {
        validate_profile_selection_context(&command)?;

        if !has_profile_selection_permission(&command.permissions) {
            return Err(ProfileSelectionError::PermissionDenied(
                PROFILE_SELECT_PERMISSION.into(),
            ));
        }
        if command.idempotency_key.trim().is_empty() {
            return Err(ProfileSelectionError::IdempotencyKeyRequired);
        }
        if command.profile.profile_id.trim().is_empty() || !command.profile.installed {
            return Err(ProfileSelectionError::ProfileUnavailable(
                command.profile.profile_id,
            ));
        }
        if command.activity.activity_mode_id.trim().is_empty()
            || command.activity.activity_mode_id != command.current_projection.activity_mode_id
        {
            return Err(ProfileSelectionError::ActivityUnavailable(
                command.activity.activity_mode_id,
            ));
        }

        let profile_contribution_ids = command
            .profile
            .candidate_contribution_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let activity_contribution_ids = command
            .activity
            .candidate_contribution_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let scoped_candidates = command
            .candidates
            .into_iter()
            .map(|candidate| {
                ScopedCandidateContribution::new(candidate, command.scope.clone())
                    .map_err(ProfileSelectionError::Scope)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let candidate_ids = scoped_candidates
            .iter()
            .map(|candidate| candidate.candidate.contribution_id.clone())
            .collect::<BTreeSet<_>>();
        let candidates = collect_candidates(
            scoped_candidates,
            &profile_contribution_ids,
            &activity_contribution_ids,
            &command.scope,
        );
        if candidates.is_empty()
            || !candidates
                .iter()
                .any(|candidate| candidate_ids.contains(&candidate.contribution_id))
        {
            return Err(ProfileSelectionError::NoMeaningfulContribution);
        }

        let current_eligible = command
            .current_projection
            .eligible_contributions
            .iter()
            .map(|contribution| contribution.contribution_id.clone())
            .collect::<BTreeSet<_>>();
        let meaningful_content = candidates
            .iter()
            .filter(|candidate| current_eligible.contains(&candidate.contribution_id))
            .map(|candidate| (candidate.contribution_id.clone(), true))
            .collect::<BTreeMap<_, _>>();
        let focused_semantic_target = if candidates.iter().any(|candidate| {
            candidate.semantic_binding_id == command.current_projection.focused_semantic_target
        }) {
            command.current_projection.focused_semantic_target.clone()
        } else {
            String::new()
        };
        let next_projection_revision = command
            .expected_projection_revision
            .checked_add(1)
            .ok_or(RecompositionError::RevisionOverflow)?;
        let input = ResolveProjectionInput {
            candidates,
            eligibility: EligibilityContext {
                scope: command.scope.clone(),
                profile_id: command.profile.profile_id.clone(),
                activity_mode_id: command.activity.activity_mode_id.clone(),
                projection_revision: next_projection_revision,
                capabilities: command.capabilities,
                permissions: command.permissions,
                available_operations: command.available_operations,
                meaningful_content,
                previously_eligible: current_eligible,
                observed_at: Utc::now().to_rfc3339(),
            },
            workspace_profile_revision: command.profile.revision,
            activity_mode_revision: command.activity.revision,
            focused_work_surface_id: command.current_projection.focused_work_surface_id.clone(),
            canonical_read_model_revision: command.current_projection.canonical_read_model_revision,
            viewport_width: 1440,
            viewport_height: 900,
            viewport_class: "standard".into(),
            focused_semantic_target,
            previous_projection_revision: command.expected_projection_revision,
            previous_layout_revision: command.current_projection.layout_revision,
            event_cursor: command.current_projection.durable_event_cursor.clone(),
            causation_id: Some(command.idempotency_key.clone()),
            idempotency_key: command.idempotency_key.clone(),
        };
        let mut result = resolve_projection(
            input,
            Some(command.current_projection.projection_digest.clone()),
        )?;
        result.evidence.trigger = "profile_change".into();
        result.event.event_kind = "profile_changed".into();
        if let Some(payload) = result.event.payload.as_object_mut() {
            payload.insert(
                "profile_selection".into(),
                json!({
                    "operation_id": PROFILE_SELECT_OPERATION,
                    "profile_id": command.profile.profile_id.clone(),
                    "previous_profile_id": command.current_projection.workspace_profile_id.clone(),
                    "workstream": command.scope.workstream.clone(),
                }),
            );
            if let Some(evidence) = payload.get_mut("evidence") {
                evidence["trigger"] = json!("profile_change");
            }
        }
        Ok(result)
    }
}

fn validate_profile_selection_context(
    command: &ProfileSelectionCommand,
) -> Result<(), ProfileSelectionError> {
    command
        .scope
        .validate()
        .map_err(ProfileSelectionError::Scope)?;
    command
        .context
        .validate_for_workstream(&command.scope.workstream)?;
    if command.context.attachment != command.scope.attachment {
        return Err(ProfileSelectionError::Context(
            WorkstreamContextError::WorkstreamMismatch,
        ));
    }
    let expected_continuity = command.scope.continuity_id.clone().or_else(|| {
        command
            .scope
            .attachment
            .as_ref()
            .and_then(|attachment| attachment.continuity_id.clone())
    });
    if command.context.continuity_id != expected_continuity {
        return Err(ProfileSelectionError::Context(
            WorkstreamContextError::ContinuityMismatch,
        ));
    }
    let expected_binding = command.scope.workspace_binding_id.clone().or_else(|| {
        command
            .scope
            .attachment
            .as_ref()
            .map(|attachment| attachment.workspace_binding_id.clone())
    });
    if command.context.workspace_binding_id != expected_binding {
        return Err(ProfileSelectionError::Context(
            WorkstreamContextError::WorkspaceBindingMismatch,
        ));
    }
    command
        .current_projection
        .validate_scope(&command.scope)
        .map_err(ProfileSelectionError::Scope)?;
    if command.current_projection.projection_revision != command.expected_projection_revision {
        return Err(ProfileSelectionError::RevisionConflict);
    }
    if command
        .expected_event_cursor
        .as_deref()
        .is_some_and(|cursor| cursor != command.current_projection.durable_event_cursor)
    {
        return Err(ProfileSelectionError::CursorConflict);
    }
    Ok(())
}

fn has_profile_selection_permission(permissions: &BTreeSet<String>) -> bool {
    permissions.contains(PROFILE_SELECT_PERMISSION)
        || permissions.contains("mission_canvas:*")
        || permissions.contains("admin:*")
        || permissions.contains("*")
}

#[derive(Debug, Error)]
pub enum RecompositionError {
    #[error(transparent)]
    Layout(#[from] LayoutError),
    #[error("projection serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("invalid Workstream authority: {0}")]
    Identity(&'static str),
    #[error("projection scope mismatch: {0}")]
    ScopeMismatch(&'static str),
    #[error("invalid recomposition input: {0}")]
    InvalidInput(&'static str),
    #[error("projection revision overflow")]
    RevisionOverflow,
}

fn scope_validation_error(reason: &'static str) -> RecompositionError {
    match reason {
        "foreign_attachment_workstream"
        | "continuity_mismatch"
        | "workspace_binding_mismatch"
        | "invalid_attachment_workstream"
        | "workstream_mismatch"
        | "attachment_mismatch"
        | "runtime_object_mismatch"
        | "work_surface_mismatch"
        | "work_surface_authority_missing"
        | "candidate_workstream_mismatch"
        | "candidate_scope_mismatch"
        | "scope_mismatch" => RecompositionError::ScopeMismatch(reason),
        _ => RecompositionError::Identity(reason),
    }
}

impl ResolveProjectionInput {
    /// Validate all authority-bearing input before candidate eligibility or
    /// layout can run. The generated candidate DTO is scope-neutral; this
    /// request's already-validated Workstream scope is the only core binding
    /// allowed at this boundary.
    pub fn validate_scope(&self) -> Result<(), RecompositionError> {
        let scope = &self.eligibility.scope;
        scope.validate().map_err(scope_validation_error)?;
        if scope.work_surface_id.is_some() && scope.attachment.is_none() {
            return Err(RecompositionError::ScopeMismatch("attachment_missing"));
        }

        if let Some(focused_work_surface_id) = self.focused_work_surface_id.as_deref() {
            if focused_work_surface_id.trim().is_empty() {
                return Err(RecompositionError::ScopeMismatch("invalid_work_surface"));
            }
            if scope.attachment.is_none() || scope.work_surface_id.is_none() {
                return Err(RecompositionError::ScopeMismatch(
                    "work_surface_authority_missing",
                ));
            }
            if scope
                .work_surface_id
                .as_ref()
                .map(crate::workstream_identity::WorkSurfaceId::as_str)
                != Some(focused_work_surface_id)
            {
                return Err(RecompositionError::ScopeMismatch("work_surface_mismatch"));
            }
        }

        if self.event_cursor.trim().is_empty() {
            return Err(RecompositionError::InvalidInput("event_cursor_missing"));
        }
        if self.idempotency_key.trim().is_empty() {
            return Err(RecompositionError::InvalidInput("idempotency_key_missing"));
        }
        self.previous_projection_revision
            .checked_add(1)
            .ok_or(RecompositionError::RevisionOverflow)?;
        self.previous_layout_revision
            .checked_add(1)
            .ok_or(RecompositionError::RevisionOverflow)?;
        Ok(())
    }
}

pub fn resolve_projection(
    input: ResolveProjectionInput,
    previous_projection_digest: Option<String>,
) -> Result<RecompositionResult, RecompositionError> {
    // This is deliberately the first operation: foreign focus, orphan
    // WorkSurface authority, malformed Workstream identity, and invalid input
    // must not be turned into eligibility or layout diagnostics.
    input.validate_scope()?;
    let scope = input.eligibility.scope.clone();
    let now = Utc::now().to_rfc3339();
    let raw_candidates = input.candidates;
    let candidate_ids = raw_candidates
        .iter()
        .map(|candidate| candidate.contribution_id.clone())
        .collect::<Vec<_>>();
    let candidate_id_filter = candidate_ids.iter().cloned().collect::<BTreeSet<_>>();

    // CandidateContribution remains the generated, scope-neutral transport
    // DTO. Bind it only to the exact request Workstream in core, then use the
    // existing registry collector before eligibility/layout. No project path,
    // continuity id, current tab, or registry position is used as authority.
    let scoped_candidates = raw_candidates
        .into_iter()
        .map(|candidate| {
            ScopedCandidateContribution::new(candidate, scope.clone())
                .map_err(scope_validation_error)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let candidates = collect_candidates(
        scoped_candidates,
        &candidate_id_filter,
        &candidate_id_filter,
        &scope,
    );
    let candidate_ids = candidates
        .iter()
        .map(|candidate| candidate.contribution_id.clone())
        .collect::<Vec<_>>();
    let eligibility = resolve_eligibility(candidates, &input.eligibility);
    // Generated ContributionEligibilityContext does not carry a semantic
    // focus token. Core therefore selects the deterministic highest-priority
    // eligible contribution rather than accepting a client-invented target.
    let focused_semantic_target = if input.focused_semantic_target.trim().is_empty() {
        eligibility
            .eligible
            .first()
            .map(|candidate| candidate.semantic_binding_id.clone())
            .ok_or(LayoutError::NoEligibleContributions)?
    } else {
        input.focused_semantic_target.clone()
    };
    let layout = resolve_layout(
        &eligibility.eligible,
        &LayoutConstraints {
            viewport_width: input.viewport_width,
            viewport_height: input.viewport_height,
            minimum_primary_span: if input.viewport_width <= 1024 { 8 } else { 6 },
            inspector_side: InspectorSide::End,
            focused_contribution_id: focused_contribution_id(
                &eligibility.eligible,
                &focused_semantic_target,
            ),
        },
    )?;
    let eligible_ids = eligibility
        .eligible
        .iter()
        .map(|candidate| candidate.contribution_id.clone())
        .collect::<BTreeSet<_>>();
    validate_no_dead_chrome(&layout, &eligible_ids)?;
    let projection_revision = input.previous_projection_revision + 1;
    let layout_revision = input.previous_layout_revision + 1;
    let resolved_contributions = eligibility
        .eligible
        .iter()
        .map(|candidate| resolve_contribution(candidate, &input.eligibility.scope, &now))
        .collect::<Vec<_>>();
    let operation_bindings = resolved_contributions
        .iter()
        .flat_map(|contribution| {
            contribution.operation_ids.iter().map(|operation_id| {
                json!({
                    "operation_id": operation_id,
                    "target_contribution_id": contribution.contribution_id,
                    "enabled": true,
                    "authority_ref": format!("authority:{}", contribution.contribution_id),
                    "confirmation": "none",
                    "disabled_reason_ref": Value::Null,
                })
            })
        })
        .collect::<Vec<_>>();
    let mut projection = ResolvedWorkspaceProjection {
        schema: "focusa.resolved_workspace_projection.v1".into(),
        scope: input.eligibility.scope.clone(),
        workspace_profile_id: input.eligibility.profile_id.clone(),
        workspace_profile_revision: input.workspace_profile_revision,
        activity_mode_id: input.eligibility.activity_mode_id.clone(),
        activity_mode_revision: input.activity_mode_revision,
        focused_work_surface_id: input.focused_work_surface_id,
        canonical_read_model_revision: input.canonical_read_model_revision,
        candidate_contribution_ids: candidate_ids.clone(),
        eligible_contributions: resolved_contributions,
        omission_diagnostics: eligibility.omissions,
        layout_tree: serde_json::to_value(layout)?,
        operation_bindings,
        focused_semantic_target,
        projection_revision,
        layout_revision,
        durable_event_cursor: input.event_cursor.clone(),
        projection_digest: String::new(),
        resolved_at: Some(now.clone()),
        evidence_refs: vec![],
        receipt_refs: vec![],
    };
    projection.projection_digest = projection_digest(&projection)?;
    projection
        .validate_scope(&scope)
        .map_err(scope_validation_error)?;
    let scope_key = scope.workstream.storage_key();
    let evidence_id = format!("recomposition-evidence:{scope_key}:{projection_revision}");
    let receipt_id = format!("recomposition-receipt:{scope_key}:{projection_revision}");
    projection.evidence_refs.push(evidence_id.clone());
    projection.receipt_refs.push(receipt_id.clone());
    let evidence = RecompositionEvidence {
        evidence_id: evidence_id.clone(),
        scope: input.eligibility.scope.clone(),
        trigger: "explicit_resolve".into(),
        input_projection_digest: previous_projection_digest,
        output_projection_digest: projection.projection_digest.clone(),
        rule_revision: super::resolver::RESOLVER_RULE_REVISION.into(),
        candidate_contribution_ids: candidate_ids,
        eligibility_decisions: eligibility.decisions,
        observed_at: now.clone(),
    };
    let receipt = RecompositionReceipt {
        receipt_id: receipt_id.clone(),
        scope: input.eligibility.scope.clone(),
        accepted: true,
        projection_revision,
        layout_revision,
        projection_digest: projection.projection_digest.clone(),
        event_cursor: input.event_cursor.clone(),
        evidence_id: evidence_id.clone(),
        idempotency_key: input.idempotency_key,
        issued_at: now.clone(),
    };
    let event = CompositionEvent {
        event_id: format!("projection-event:{scope_key}:{projection_revision}"),
        event_kind: "projection_resolved".into(),
        scope: projection.scope.clone(),
        projection_revision,
        layout_revision,
        causation_id: input.causation_id,
        correlation_id: Some(format!("resolve:{scope_key}:{projection_revision}")),
        occurred_at: now,
        payload: json!({
            "projection_digest": projection.projection_digest,
            "viewport_class": input.viewport_class,
            "evidence": evidence,
            "receipt": receipt,
            "omission_diagnostics": projection.omission_diagnostics,
        }),
        evidence_refs: vec![evidence_id],
        receipt_refs: vec![receipt_id],
    };
    Ok(RecompositionResult {
        projection,
        evidence,
        receipt,
        event,
    })
}

/// Hash the canonical projection material, including the flattened
/// WorkstreamKey and the semantic, projection, layout, and event-cursor
/// revisions carried by `ResolvedWorkspaceProjection`.
///
/// Evidence/Receipt references and resolution time are produced around the
/// digest itself, so they are deliberately excluded. Object keys are sorted
/// recursively for a stable transport-independent digest; array order remains
/// meaningful because it is part of the resolved composition.
pub fn projection_digest(
    projection: &ResolvedWorkspaceProjection,
) -> Result<String, serde_json::Error> {
    let mut normalized = serde_json::to_value(projection)?;
    if let Some(object) = normalized.as_object_mut() {
        object.remove("projection_digest");
        object.remove("resolved_at");
        object.remove("evidence_refs");
        object.remove("receipt_refs");
    }
    let normalized = canonical_json(&normalized);
    let bytes = serde_json::to_vec(&normalized)?;
    Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
}

/// Normalize JSON object key order without changing array order or values.
/// Arrays encode ordered projection decisions/layout children and therefore
/// must not be treated as sets.
fn canonical_json(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries: Vec<(&String, &Value)> = object.iter().collect();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            let mut normalized = Map::new();
            for (key, child) in entries {
                normalized.insert(key.clone(), canonical_json(child));
            }
            Value::Object(normalized)
        }
        Value::Array(values) => Value::Array(values.iter().map(canonical_json).collect()),
        other => other.clone(),
    }
}

fn focused_contribution_id(
    candidates: &[CandidateContribution],
    focused_semantic_target: &str,
) -> Option<String> {
    candidates
        .iter()
        .find(|candidate| candidate.semantic_binding_id == focused_semantic_target)
        .map(|candidate| candidate.contribution_id.clone())
}

fn resolve_contribution(
    candidate: &CandidateContribution,
    scope: &super::model::MissionCanvasScope,
    observed_at: &str,
) -> ResolvedContribution {
    let data_ref = candidate
        .canonical_content_refs
        .first()
        .cloned()
        .unwrap_or_else(|| json!({"kind": "none", "ref": "none", "revision": 0}));
    ResolvedContribution {
        contribution_id: candidate.contribution_id.clone(),
        kind: candidate.kind.clone(),
        semantic_binding_id: candidate.semantic_binding_id.clone(),
        renderer_binding_id: candidate.renderer_binding_id.clone(),
        data_ref,
        operation_ids: candidate.required_operations.clone(),
        authority: json!({
            "canonical_owner": "Focusa Core",
            "mutation_owner": "Focusa Core",
            "workstream": scope.workstream,
            "continuity_id": scope.continuity_id,
            "attachment": scope.attachment,
            "workspace_binding_id": scope.workspace_binding_id,
            "runtime_object": scope.runtime_object,
            "work_surface_id": scope.work_surface_id,
            "read_only": false
        }),
        freshness: json!({"status": "current", "observed_at": observed_at}),
        resolved_geometry: candidate.geometry.clone(),
        accessibility: json!({"label": candidate.contribution_id, "landmark_role": "region", "focus_semantic_id": candidate.semantic_binding_id}),
        contribution_revision: 1,
        evidence_refs: vec![],
    }
}

pub fn candidate_partition_is_complete(projection: &ResolvedWorkspaceProjection) -> bool {
    let candidates = projection
        .candidate_contribution_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let eligible = projection
        .eligible_contributions
        .iter()
        .map(|contribution| contribution.contribution_id.clone())
        .collect::<BTreeSet<_>>();
    let omitted = projection
        .omission_diagnostics
        .iter()
        .map(|diagnostic| diagnostic.contribution_id.clone())
        .collect::<BTreeSet<_>>();
    eligible.is_disjoint(&omitted) && candidates == eligible.union(&omitted).cloned().collect()
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::*;
    use crate::mission_canvas::model::{ContributionKind, MissionCanvasScope, OmissionDiagnostic};
    use crate::scoped_state::ScopeRef as LegacyScopeRef;
    use crate::workstream_identity::{
        AttachmentId, AttachmentKey, ContinuityId, InstanceId, ScopeRef, SessionId, WorkSurfaceId,
        WorkspaceBindingId, WorkstreamId, WorkstreamKey,
    };
    use serde_json::json;

    fn workstream(id: &str) -> WorkstreamKey {
        let legacy = LegacyScopeRef::project(
            "project:focusa",
            "/workspace/focusa",
            "Focusa",
            "host-a:worktree-main",
        )
        .unwrap();
        WorkstreamKey::new(
            ScopeRef::project(legacy).unwrap(),
            WorkstreamId::parse(id).unwrap(),
        )
    }

    fn attachment(owner: WorkstreamKey, id: &str) -> AttachmentKey {
        AttachmentKey::new(
            owner,
            Some(ContinuityId::parse("continuity:mission-canvas").unwrap()),
            InstanceId::parse("instance:pi").unwrap(),
            SessionId::parse("session:pi").unwrap(),
            AttachmentId::parse(format!("attachment:{id}")).unwrap(),
            WorkspaceBindingId::parse("workspace:mission-canvas").unwrap(),
        )
    }

    fn focused_scope(workstream_id: &str, surface_id: &str) -> MissionCanvasScope {
        let owner = workstream(workstream_id);
        let mut scope =
            MissionCanvasScope::new(owner.clone(), Some(attachment(owner, surface_id))).unwrap();
        scope.work_surface_id =
            Some(WorkSurfaceId::parse(format!("surface:{surface_id}")).unwrap());
        scope
    }

    fn candidate(id: &str) -> CandidateContribution {
        CandidateContribution {
            contribution_id: format!("contribution:{id}"),
            kind: ContributionKind::FocusedWorkSurface,
            semantic_binding_id: format!("semantic:{id}"),
            renderer_binding_id: format!("renderer:{id}"),
            priority: 10,
            applicable_profile_ids: vec!["software".into()],
            applicable_activity_mode_ids: vec!["overview".into()],
            canonical_content_refs: vec![json!({
                "kind": "work_surface",
                "ref": format!("surface:{id}"),
                "revision": 1
            })],
            required_capabilities: vec![],
            required_permissions: vec![],
            required_operations: vec![],
            geometry: json!({"minimum_span": 1, "maximum_span": 12}),
        }
    }

    fn resolve_input(
        scope: MissionCanvasScope,
        candidates: Vec<CandidateContribution>,
    ) -> ResolveProjectionInput {
        let focused_semantic_target = candidates
            .first()
            .map(|candidate| candidate.semantic_binding_id.clone())
            .unwrap_or_else(|| "semantic:none".into());
        ResolveProjectionInput {
            focused_work_surface_id: scope
                .work_surface_id
                .as_ref()
                .map(|id| id.as_str().to_owned()),
            candidates,
            eligibility: EligibilityContext {
                scope,
                profile_id: "software".into(),
                activity_mode_id: "overview".into(),
                projection_revision: 1,
                capabilities: BTreeSet::new(),
                permissions: BTreeSet::new(),
                available_operations: BTreeSet::new(),
                meaningful_content: BTreeMap::new(),
                previously_eligible: BTreeSet::new(),
                observed_at: "2026-08-07T00:00:00Z".into(),
            },
            workspace_profile_revision: 2,
            activity_mode_revision: 3,
            canonical_read_model_revision: 41,
            viewport_width: 1440,
            viewport_height: 900,
            viewport_class: "standard".into(),
            focused_semantic_target,
            previous_projection_revision: 0,
            previous_layout_revision: 0,
            event_cursor: "event:40".into(),
            causation_id: Some("cause:resolve".into()),
            idempotency_key: "idempotency:resolve".into(),
        }
    }

    fn projection(workstream_id: &str) -> ResolvedWorkspaceProjection {
        ResolvedWorkspaceProjection {
            schema: "focusa.resolved_workspace_projection.v1".into(),
            scope: MissionCanvasScope::new(workstream(workstream_id), None).unwrap(),
            workspace_profile_id: "software".into(),
            workspace_profile_revision: 2,
            activity_mode_id: "overview".into(),
            activity_mode_revision: 1,
            focused_work_surface_id: None,
            canonical_read_model_revision: 41,
            candidate_contribution_ids: vec!["contribution:primary".into()],
            eligible_contributions: vec![],
            omission_diagnostics: vec![],
            layout_tree: json!({
                "kind": "split",
                "node_id": "layout:root",
                "orientation": "horizontal",
                "ratio": 0.5,
                "children": [
                    {"kind": "single", "node_id": "layout:primary", "contribution_id": "contribution:primary"},
                    {"kind": "single", "node_id": "layout:inspector", "contribution_id": "contribution:inspector"}
                ]
            }),
            operation_bindings: vec![],
            focused_semantic_target: "semantic:primary".into(),
            projection_revision: 7,
            layout_revision: 5,
            durable_event_cursor: "event:41".into(),
            projection_digest: "sha256:placeholder".into(),
            resolved_at: None,
            evidence_refs: vec![],
            receipt_refs: vec![],
        }
    }

    #[test]
    fn resolve_projection_workstream_returns_one_exact_scoped_result() {
        let scope = focused_scope("local", "pi");
        let input = resolve_input(scope.clone(), vec![candidate("pi")]);
        let result = resolve_projection(input, None).expect("exact Workstream input resolves");

        assert_eq!(result.projection.scope, scope);
        assert_eq!(result.evidence.scope, result.projection.scope);
        assert_eq!(result.receipt.scope, result.projection.scope);
        assert_eq!(result.event.scope, result.projection.scope);
        assert_eq!(result.projection.projection_revision, 1);
        assert_eq!(result.projection.layout_revision, 1);
        assert!(candidate_partition_is_complete(&result.projection));
        assert_eq!(
            result.projection.validate_scope(&scope),
            Ok(()),
            "the reducer must not emit a projection that cannot be read back for its scope"
        );
    }

    #[test]
    fn resolve_projection_workstream_rejects_foreign_focus_before_eligibility_or_layout() {
        let scope = focused_scope("local", "pi");
        let mut input = resolve_input(scope, vec![candidate("pi")]);
        input.focused_work_surface_id = Some("surface:foreign".into());
        input.candidates[0]
            .required_capabilities
            .push("capability:missing".into());

        let error = resolve_projection(input, None).expect_err(
            "a foreign focused Work Surface must fail before an unavailable candidate reaches layout",
        );
        assert!(matches!(
            error,
            RecompositionError::ScopeMismatch("work_surface_mismatch")
        ));
    }

    #[test]
    fn resolve_projection_workstream_rejects_orphan_surface_authority() {
        let mut scope = focused_scope("local", "pi");
        scope.attachment = None;
        let input = resolve_input(scope, vec![candidate("pi")]);

        let error = resolve_projection(input, None).expect_err(
            "project/continuity data without Attachment authority cannot focus a Work Surface",
        );
        assert!(matches!(
            error,
            RecompositionError::ScopeMismatch("attachment_missing")
        ));
    }

    #[test]
    fn resolve_projection_workstream_rejects_foreign_attachment_authority() {
        let local = workstream("local");
        let foreign = workstream("foreign");
        let foreign_attachment = attachment(foreign, "foreign");
        let scope = MissionCanvasScope {
            workstream: local,
            continuity_id: foreign_attachment.continuity_id.clone(),
            attachment: Some(foreign_attachment.clone()),
            workspace_binding_id: Some(foreign_attachment.workspace_binding_id.clone()),
            runtime_object: None,
            work_surface_id: None,
        };
        let input = resolve_input(scope, vec![candidate("pi")]);

        let error = resolve_projection(input, None)
            .expect_err("a subordinate Attachment owned by another Workstream must fail closed");
        assert!(matches!(
            error,
            RecompositionError::ScopeMismatch("foreign_attachment_workstream")
        ));
    }

    #[test]
    fn resolve_projection_workstream_omits_unavailable_contribution_without_dead_chrome() {
        let scope = MissionCanvasScope::new(workstream("local"), None).unwrap();
        let mut unavailable = candidate("browser");
        unavailable.required_capabilities.push("browser".into());
        let available = candidate("pi");
        let result = resolve_projection(resolve_input(scope, vec![unavailable, available]), None)
            .expect("one eligible contribution still composes a complete layout");

        assert_eq!(result.projection.eligible_contributions.len(), 1);
        assert_eq!(
            result.projection.omission_diagnostics[0].reason,
            "capability_not_present"
        );
        assert!(!result.projection.layout_tree.is_null());
        assert!(candidate_partition_is_complete(&result.projection));
        assert!(result
            .evidence
            .eligibility_decisions
            .iter()
            .any(
                |decision| decision.contribution_id == "contribution:browser"
                    && decision.outcome == super::super::resolver::EligibilityOutcome::Omitted
            ));
    }

    #[test]
    fn resolve_projection_workstream_rejects_empty_cursor_and_revision_overflow() {
        let scope = MissionCanvasScope::new(workstream("local"), None).unwrap();
        let mut empty_cursor = resolve_input(scope.clone(), vec![candidate("pi")]);
        empty_cursor.event_cursor = "  ".into();
        assert!(matches!(
            resolve_projection(empty_cursor, None),
            Err(RecompositionError::InvalidInput("event_cursor_missing"))
        ));

        let mut overflow = resolve_input(scope, vec![candidate("pi")]);
        overflow.previous_projection_revision = u64::MAX;
        assert!(matches!(
            resolve_projection(overflow, None),
            Err(RecompositionError::RevisionOverflow)
        ));
    }

    #[test]
    fn mission_canvas_projection_digest_distinguishes_equal_layouts_under_different_workstreams() {
        let local = projection("ws:local");
        let foreign = projection("ws:foreign");

        assert_eq!(local.layout_tree, foreign.layout_tree);
        assert_ne!(local.scope.workstream, foreign.scope.workstream);
        assert_eq!(
            serde_json::to_value(&local).unwrap()["workstream"]["workstream_id"],
            json!("ws:local")
        );
        assert_ne!(
            projection_digest(&local).unwrap(),
            projection_digest(&foreign).unwrap(),
            "a digest must not collapse equal layouts across WorkstreamKey boundaries"
        );
    }

    #[test]
    fn mission_canvas_projection_digest_includes_semantic_and_layout_revisions() {
        let base = projection("ws:revisions");
        let base_digest = projection_digest(&base).unwrap();

        let mut profile_revision = base.clone();
        profile_revision.workspace_profile_revision += 1;
        assert_ne!(projection_digest(&profile_revision).unwrap(), base_digest);

        let mut activity_revision = base.clone();
        activity_revision.activity_mode_revision += 1;
        assert_ne!(projection_digest(&activity_revision).unwrap(), base_digest);

        let mut semantic_read_model_revision = base.clone();
        semantic_read_model_revision.canonical_read_model_revision += 1;
        assert_ne!(
            projection_digest(&semantic_read_model_revision).unwrap(),
            base_digest
        );

        let mut projection_revision = base.clone();
        projection_revision.projection_revision += 1;
        assert_ne!(
            projection_digest(&projection_revision).unwrap(),
            base_digest
        );

        let mut layout_revision = base.clone();
        layout_revision.layout_revision += 1;
        assert_ne!(projection_digest(&layout_revision).unwrap(), base_digest);

        let mut cursor = base.clone();
        cursor.durable_event_cursor = "event:42".into();
        assert_ne!(
            projection_digest(&cursor).unwrap(),
            base_digest,
            "a stale/replayed cursor must not share a digest with current projection state"
        );
    }

    #[test]
    fn mission_canvas_projection_digest_ignores_only_volatile_proof_metadata() {
        let base = projection("ws:metadata");
        let base_digest = projection_digest(&base).unwrap();
        let mut metadata = base.clone();
        metadata.projection_digest = "sha256:another-value".into();
        metadata.resolved_at = Some("2026-08-07T00:00:00Z".into());
        metadata.evidence_refs = vec!["evidence:recomposition".into()];
        metadata.receipt_refs = vec!["receipt:recomposition".into()];

        assert_eq!(
            projection_digest(&metadata).unwrap(),
            base_digest,
            "proof links and resolution time must not create a recursive digest"
        );
    }

    #[test]
    fn mission_canvas_projection_digest_is_stable_for_object_key_order() {
        let mut first = projection("ws:canonical-json");
        first.layout_tree =
            serde_json::from_str(r#"{"z":{"b":2,"a":1},"a":[{"d":4,"c":3}],"kind":"single"}"#)
                .unwrap();
        let mut second = first.clone();
        second.layout_tree =
            serde_json::from_str(r#"{"kind":"single","a":[{"c":3,"d":4}],"z":{"a":1,"b":2}}"#)
                .unwrap();

        assert_eq!(
            projection_digest(&first).unwrap(),
            projection_digest(&second).unwrap(),
            "JSON object ordering is not semantic projection state"
        );
    }

    #[test]
    fn mission_canvas_projection_digest_preserves_fail_closed_omissions() {
        let base = projection("ws:omissions");
        let base_digest = projection_digest(&base).unwrap();
        let mut omitted = base;
        omitted.candidate_contribution_ids = vec!["contribution:empty".into()];
        omitted.omission_diagnostics = vec![OmissionDiagnostic {
            contribution_id: "contribution:empty".into(),
            reason: "capability_not_present".into(),
            rule_revision: "adaptive-composition:v1".into(),
            projection_revision: omitted.projection_revision,
            canonical_input_refs: vec![],
            details_ref: Some("diagnostic:capability_not_present".into()),
            observed_at: "2026-08-07T00:00:00Z".into(),
        }];

        assert!(omitted.eligible_contributions.is_empty());
        assert_ne!(
            projection_digest(&omitted).unwrap(),
            base_digest,
            "unavailable contributions remain omitted and observable in canonical digest material"
        );
    }

    #[test]
    fn mission_canvas_projection_digest_never_repairs_missing_or_foreign_authority() {
        let local = projection("ws:local");
        let foreign = projection("ws:foreign");
        assert_ne!(
            projection_digest(&local).unwrap(),
            projection_digest(&foreign).unwrap()
        );

        let mut legacy = serde_json::to_value(&local).unwrap();
        legacy.as_object_mut().unwrap().remove("workstream");
        assert!(
            serde_json::from_value::<ResolvedWorkspaceProjection>(legacy).is_err(),
            "a legacy project/continuity row cannot be repaired into a canonical projection"
        );
    }
}
