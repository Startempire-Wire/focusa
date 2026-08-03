use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::model::{CandidateContribution, MissionCanvasScope, OmissionDiagnostic};

pub const RESOLVER_RULE_REVISION: &str = "adaptive-composition:v1";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EligibilityContext {
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

pub fn collect_candidates(
    registry_candidates: impl IntoIterator<Item = CandidateContribution>,
    profile_contribution_ids: &BTreeSet<String>,
    activity_contribution_ids: &BTreeSet<String>,
) -> Vec<CandidateContribution> {
    let mut candidates: BTreeMap<String, CandidateContribution> = BTreeMap::new();
    for candidate in registry_candidates {
        if profile_contribution_ids.contains(&candidate.contribution_id)
            && activity_contribution_ids.contains(&candidate.contribution_id)
        {
            candidates.insert(candidate.contribution_id.clone(), candidate);
        }
    }
    candidates.into_values().collect()
}

pub fn resolve_eligibility(
    mut candidates: Vec<CandidateContribution>,
    context: &EligibilityContext,
) -> EligibilityResolution {
    candidates.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.contribution_id.cmp(&right.contribution_id))
    });
    let mut eligible = Vec::new();
    let mut decisions = Vec::new();
    let mut omissions = Vec::new();

    for candidate in candidates {
        let reason = omission_reason(&candidate, context);
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
                let suspended = context
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

    fn context() -> EligibilityContext {
        EligibilityContext {
            scope: MissionCanvasScope {
                project_root: "/tmp/focusa".into(),
                continuity_id: "mission-canvas".into(),
                instance_id: None,
                session_id: "session:1".into(),
                attachment_id: "attachment:1".into(),
                working_subpath_id: None,
            },
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
