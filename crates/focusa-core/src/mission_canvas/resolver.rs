use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::model::{
    CandidateContribution, MissionCanvasScope, OmissionDiagnostic, ScopedCandidateContribution,
};

pub const RESOLVER_RULE_REVISION: &str = "adaptive-composition:v1";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EligibilityContext {
    #[serde(flatten)]
    pub scope: MissionCanvasScope,
    pub profile_id: String,
    pub activity_mode_id: String,
    pub projection_revision: u64,
    pub capabilities: BTreeSet<String>,
    pub permissions: BTreeSet<String>,
    pub available_operations: BTreeSet<String>,
    #[serde(default)]
    pub meaningful_content: BTreeMap<String, bool>,
    #[serde(default)]
    pub previously_eligible: BTreeSet<String>,
    pub observed_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EligibilityOutcome {
    Eligible,
    Omitted,
    Suspended,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EligibilityDecision {
    pub contribution_id: String,
    pub outcome: EligibilityOutcome,
    pub reason: Option<String>,
    pub rule_revision: String,
    pub projection_revision: u64,
}

#[derive(Clone, Debug)]
pub struct EligibilityResolution {
    pub eligible: Vec<CandidateContribution>,
    pub decisions: Vec<EligibilityDecision>,
    pub omissions: Vec<OmissionDiagnostic>,
}

/// A candidate may be supplied as a generated, scope-neutral registry DTO or
/// as an explicit core binding carrying the Workstream that produced it.
///
/// The latter is required for registry projections that contain candidates
/// from more than one Workstream.  It makes ownership data explicit instead
/// of trying to recover it from a project path, continuity id, selected tab,
/// or a "nearest" registry row.
pub trait CandidateScopeInput {
    fn into_candidate_scope(self) -> (CandidateContribution, Option<MissionCanvasScope>);
}

impl CandidateScopeInput for CandidateContribution {
    fn into_candidate_scope(self) -> (CandidateContribution, Option<MissionCanvasScope>) {
        (self, None)
    }
}

impl CandidateScopeInput for ScopedCandidateContribution {
    fn into_candidate_scope(self) -> (CandidateContribution, Option<MissionCanvasScope>) {
        (self.candidate, Some(self.scope))
    }
}

impl CandidateScopeInput for (CandidateContribution, MissionCanvasScope) {
    fn into_candidate_scope(self) -> (CandidateContribution, Option<MissionCanvasScope>) {
        (self.0, Some(self.1))
    }
}

impl CandidateScopeInput for (MissionCanvasScope, CandidateContribution) {
    fn into_candidate_scope(self) -> (CandidateContribution, Option<MissionCanvasScope>) {
        (self.1, Some(self.0))
    }
}

fn scope_failure(
    candidate: &CandidateContribution,
    candidate_scope: Option<&MissionCanvasScope>,
    expected_scope: &MissionCanvasScope,
) -> Option<&'static str> {
    if let Err(error) = candidate.validate_scope(expected_scope) {
        return Some(match error {
            "foreign_attachment_workstream"
            | "continuity_mismatch"
            | "workspace_binding_mismatch"
            | "invalid_attachment_workstream" => "scope_mismatch",
            _ => "not_authorized",
        });
    }

    if let Some(candidate_scope) = candidate_scope {
        if let Err(error) = candidate.validate_scope(candidate_scope) {
            return Some(match error {
                "foreign_attachment_workstream"
                | "continuity_mismatch"
                | "workspace_binding_mismatch"
                | "invalid_attachment_workstream" => "scope_mismatch",
                _ => "not_authorized",
            });
        }
        if candidate_scope != expected_scope {
            return Some("scope_mismatch");
        }
    }

    None
}

/// Collect only the profile/activity candidates owned by one exact
/// Workstream.  A scope-neutral generated registry definition is accepted
/// only in the presence of a valid expected scope; an explicitly foreign
/// binding is omitted before any layout or eligibility work occurs.
pub fn collect_candidates<I, C>(
    registry_candidates: I,
    profile_contribution_ids: &BTreeSet<String>,
    activity_contribution_ids: &BTreeSet<String>,
    expected_scope: &MissionCanvasScope,
) -> Vec<CandidateContribution>
where
    I: IntoIterator<Item = C>,
    C: CandidateScopeInput,
{
    let mut candidates: BTreeMap<String, CandidateContribution> = BTreeMap::new();
    for input in registry_candidates {
        let (candidate, candidate_scope) = input.into_candidate_scope();
        if scope_failure(&candidate, candidate_scope.as_ref(), expected_scope).is_some() {
            continue;
        }
        if profile_contribution_ids.contains(&candidate.contribution_id)
            && activity_contribution_ids.contains(&candidate.contribution_id)
        {
            candidates.insert(candidate.contribution_id.clone(), candidate);
        }
    }
    candidates.into_values().collect()
}

pub fn resolve_eligibility<I, C>(
    candidates: I,
    context: &EligibilityContext,
) -> EligibilityResolution
where
    I: IntoIterator<Item = C>,
    C: CandidateScopeInput,
{
    let mut candidates = candidates
        .into_iter()
        .map(|candidate| candidate.into_candidate_scope())
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .0
            .priority
            .cmp(&left.0.priority)
            .then_with(|| left.0.contribution_id.cmp(&right.0.contribution_id))
    });
    let mut eligible = Vec::new();
    let mut decisions = Vec::new();
    let mut omissions = Vec::new();

    for (candidate, candidate_scope) in candidates {
        let scope_reason = scope_failure(&candidate, candidate_scope.as_ref(), &context.scope);
        let scope_violation = scope_reason.is_some();
        let reason = if let Some(reason) = scope_reason {
            Some(reason)
        } else {
            omission_reason(&candidate, context)
        };
        match reason {
            None => {
                decisions.push(decision(
                    &candidate,
                    context,
                    EligibilityOutcome::Eligible,
                    None,
                ));
                eligible.push(candidate);
            }
            Some(reason) => {
                // Scope violations are never suspended.  A stale or foreign
                // authority packet must be omitted, not retained as a
                // resumable presentation state.
                let suspended = !scope_violation
                    && context
                        .previously_eligible
                        .contains(&candidate.contribution_id)
                    && matches!(reason, "capability_not_present" | "not_authorized");
                let outcome = if suspended {
                    EligibilityOutcome::Suspended
                } else {
                    EligibilityOutcome::Omitted
                };
                let diagnostic_reason = if suspended { "suspended" } else { reason };
                decisions.push(decision(
                    &candidate,
                    context,
                    outcome,
                    Some(diagnostic_reason.to_owned()),
                ));
                omissions.push(OmissionDiagnostic {
                    contribution_id: candidate.contribution_id,
                    reason: diagnostic_reason.to_owned(),
                    rule_revision: RESOLVER_RULE_REVISION.to_owned(),
                    projection_revision: context.projection_revision,
                    canonical_input_refs: candidate.canonical_content_refs,
                    details_ref: Some(format!("diagnostic:eligibility:{diagnostic_reason}")),
                    observed_at: context.observed_at.clone(),
                });
            }
        }
    }

    EligibilityResolution {
        eligible,
        decisions,
        omissions,
    }
}

fn decision(
    candidate: &CandidateContribution,
    context: &EligibilityContext,
    outcome: EligibilityOutcome,
    reason: Option<String>,
) -> EligibilityDecision {
    EligibilityDecision {
        contribution_id: candidate.contribution_id.clone(),
        outcome,
        reason,
        rule_revision: RESOLVER_RULE_REVISION.to_owned(),
        projection_revision: context.projection_revision,
    }
}

fn omission_reason<'a>(
    candidate: &CandidateContribution,
    context: &'a EligibilityContext,
) -> Option<&'a str> {
    if !candidate.applicable_profile_ids.is_empty()
        && !candidate
            .applicable_profile_ids
            .iter()
            .any(|profile| profile == &context.profile_id)
    {
        return Some("not_applicable");
    }
    if !candidate.applicable_activity_mode_ids.is_empty()
        && !candidate
            .applicable_activity_mode_ids
            .iter()
            .any(|activity| activity == &context.activity_mode_id)
    {
        return Some("not_applicable");
    }
    if candidate
        .required_capabilities
        .iter()
        .any(|required| !context.capabilities.contains(required))
    {
        return Some("capability_not_present");
    }
    if candidate
        .required_permissions
        .iter()
        .any(|required| !context.permissions.contains(required))
    {
        return Some("not_authorized");
    }
    if candidate
        .required_operations
        .iter()
        .any(|required| !context.available_operations.contains(required))
    {
        return Some("capability_not_present");
    }
    let meaningful = context
        .meaningful_content
        .get(&candidate.contribution_id)
        .copied()
        .unwrap_or(!candidate.canonical_content_refs.is_empty());
    if !meaningful {
        return Some("no_relevant_content");
    }
    None
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::mission_canvas::model::ContributionKind;
    use crate::scoped_state::ScopeRef as LegacyScopeRef;
    use crate::workstream_identity::{
        AttachmentId, ContinuityId, InstanceId, ScopeRef, SessionId, WorkspaceBindingId,
        WorkstreamId, WorkstreamKey,
    };

    fn candidate(id: &str, priority: i64) -> CandidateContribution {
        CandidateContribution {
            contribution_id: id.into(),
            kind: ContributionKind::Inspector,
            semantic_binding_id: format!("semantic:{id}"),
            renderer_binding_id: format!("renderer:{id}"),
            priority,
            applicable_profile_ids: vec!["software".into()],
            applicable_activity_mode_ids: vec!["overview".into()],
            canonical_content_refs: vec![json!({"ref": id})],
            required_capabilities: vec![],
            required_permissions: vec![],
            required_operations: vec![],
            geometry: json!({"preferred_regions": ["inspector"]}),
        }
    }

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

    fn attachment(owner: WorkstreamKey, id: &str) -> crate::workstream_identity::AttachmentKey {
        crate::workstream_identity::AttachmentKey::new(
            owner,
            Some(ContinuityId::parse("continuity:mission-canvas").unwrap()),
            InstanceId::parse("instance:pi").unwrap(),
            SessionId::parse("session:1").unwrap(),
            AttachmentId::parse(id).unwrap(),
            WorkspaceBindingId::parse("workspace:mission-canvas").unwrap(),
        )
    }

    fn scope() -> MissionCanvasScope {
        let owner = workstream("ws:mission-canvas");
        MissionCanvasScope::new(owner.clone(), Some(attachment(owner, "attachment:1"))).unwrap()
    }

    fn context() -> EligibilityContext {
        EligibilityContext {
            scope: scope(),
            profile_id: "software".into(),
            activity_mode_id: "overview".into(),
            projection_revision: 1,
            capabilities: BTreeSet::new(),
            permissions: BTreeSet::new(),
            available_operations: BTreeSet::new(),
            meaningful_content: BTreeMap::new(),
            previously_eligible: BTreeSet::new(),
            observed_at: "2026-07-30T12:00:00Z".into(),
        }
    }

    fn foreign_scope() -> MissionCanvasScope {
        let owner = workstream("ws:foreign");
        MissionCanvasScope::new(owner.clone(), Some(attachment(owner, "attachment:foreign")))
            .unwrap()
    }

    #[test]
    fn mission_canvas_candidate_scope_collects_only_exact_workstream() {
        let local = candidate("contribution:shared", 20);
        let foreign = candidate("contribution:shared", 99);
        let local_scope = scope();
        let profile_ids = BTreeSet::from(["contribution:shared".to_owned()]);
        let activity_ids = profile_ids.clone();
        let candidates = collect_candidates(
            vec![
                ScopedCandidateContribution::new(local, local_scope.clone()).unwrap(),
                ScopedCandidateContribution::new(foreign, foreign_scope()).unwrap(),
            ],
            &profile_ids,
            &activity_ids,
            &local_scope,
        );

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].priority, 20);
    }

    #[test]
    fn mission_canvas_candidate_scope_reports_foreign_diagnostic_before_capability_checks() {
        let foreign = ScopedCandidateContribution::new(
            candidate("contribution:foreign", 20),
            foreign_scope(),
        )
        .unwrap();
        let resolution = resolve_eligibility(vec![foreign], &context());

        assert!(resolution.eligible.is_empty());
        assert_eq!(resolution.decisions.len(), 1);
        assert_eq!(resolution.decisions[0].outcome, EligibilityOutcome::Omitted);
        assert_eq!(
            resolution.decisions[0].reason.as_deref(),
            Some("scope_mismatch")
        );
        assert_eq!(resolution.omissions[0].reason, "scope_mismatch");
        assert_eq!(resolution.omissions[0].projection_revision, 1);
    }

    #[test]
    fn mission_canvas_candidate_scope_rejects_invalid_expected_authority() {
        let mut invalid_scope = scope();
        invalid_scope.continuity_id = Some(ContinuityId::parse("continuity:wrong").unwrap());
        let candidate = candidate("contribution:invalid-scope", 20);

        assert_eq!(
            candidate.validate_scope(&invalid_scope),
            Err("continuity_mismatch")
        );
        let resolution = resolve_eligibility(
            vec![candidate],
            &EligibilityContext {
                scope: invalid_scope,
                ..context()
            },
        );
        assert!(resolution.eligible.is_empty());
        assert_eq!(resolution.omissions[0].reason, "scope_mismatch");
    }

    #[test]
    fn mission_canvas_candidate_scope_keeps_scope_neutral_generated_candidate_with_exact_context() {
        let candidate = candidate("contribution:generated", 20);
        let profile_ids = BTreeSet::from(["contribution:generated".to_owned()]);
        let candidates = collect_candidates(vec![candidate], &profile_ids, &profile_ids, &scope());

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].contribution_id, "contribution:generated");
    }

    #[test]
    fn filters_by_content_capability_permission_and_operation() {
        let mut candidates = vec![
            candidate("contribution:eligible", 10),
            candidate("contribution:empty", 20),
        ];
        candidates.push({
            let mut value = candidate("contribution:capability", 30);
            value.required_capabilities.push("browser".into());
            value
        });
        candidates.push({
            let mut value = candidate("contribution:permission", 40);
            value.required_permissions.push("evidence:read".into());
            value
        });
        candidates.push({
            let mut value = candidate("contribution:operation", 50);
            value
                .required_operations
                .push("focusa.browser.click".into());
            value
        });
        let mut context = context();
        context
            .meaningful_content
            .insert("contribution:empty".into(), false);
        let resolution = resolve_eligibility(candidates, &context);
        assert_eq!(
            resolution
                .eligible
                .iter()
                .map(|item| item.contribution_id.as_str())
                .collect::<Vec<_>>(),
            vec!["contribution:eligible"]
        );
        assert_eq!(resolution.omissions.len(), 4);
    }

    #[test]
    fn active_capability_loss_is_suspended_and_ranking_is_stable() {
        let mut browser = candidate("contribution:browser", 100);
        browser.required_capabilities.push("browser".into());
        let mut context = context();
        context
            .previously_eligible
            .insert(browser.contribution_id.clone());
        let resolution = resolve_eligibility(
            vec![
                candidate("contribution:z", 10),
                candidate("contribution:a", 10),
                browser,
            ],
            &context,
        );
        assert_eq!(
            resolution
                .eligible
                .iter()
                .map(|item| item.contribution_id.as_str())
                .collect::<Vec<_>>(),
            vec!["contribution:a", "contribution:z"]
        );
        assert_eq!(resolution.omissions[0].reason, "suspended");
    }

    #[test]
    fn resolver_handles_ten_thousand_candidates_within_budget() {
        let candidates = (0..10_000)
            .map(|index| candidate(&format!("contribution:item-{index:05}"), index % 100))
            .collect::<Vec<_>>();
        let started = std::time::Instant::now();
        let resolution = resolve_eligibility(candidates, &context());
        assert_eq!(resolution.eligible.len(), 10_000);
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
    }
}
