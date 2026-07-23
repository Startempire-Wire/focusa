use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const GRILL_WITH_DOCS_STRATEGY_ID: &str = "focusa.interview.strategy.grill-with-docs.v1";
pub const GRILL_WITH_DOCS_STRATEGY_VERSION: u64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrillTranche {
    Discovery,
    Boundary,
    Failure,
    Evidence,
    Architecture,
    SpecReadiness,
}

impl GrillTranche {
    pub const ALL: [Self; 6] = [
        Self::Discovery,
        Self::Boundary,
        Self::Failure,
        Self::Evidence,
        Self::Architecture,
        Self::SpecReadiness,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterviewGapPriority {
    Blocker,
    High,
    Normal,
    Optional,
}

impl InterviewGapPriority {
    fn rank(self) -> u8 {
        match self {
            Self::Blocker => 0,
            Self::High => 1,
            Self::Normal => 2,
            Self::Optional => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterviewGapCandidate {
    pub gap_id: String,
    pub tranche: GrillTranche,
    pub decision_branch_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[serde(default)]
    pub domain_term_candidates: Vec<String>,
    #[serde(default)]
    pub architecture_decision_candidates: Vec<String>,
    pub decision_required: bool,
    pub priority: InterviewGapPriority,
    pub answer_type: String,
    pub readiness_effect: String,
    pub stop_condition: String,
    pub downstream_dependency_count: u64,
    #[serde(default)]
    pub resolved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrillInterviewContext {
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
    pub session_id: String,
    pub approved_role_profile_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_branch_id: Option<String>,
    #[serde(default)]
    pub completed_tranches: Vec<GrillTranche>,
    #[serde(default)]
    pub gaps: Vec<InterviewGapCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterviewNextQuestionProposal {
    pub schema: String,
    pub strategy_id: String,
    pub strategy_version: u64,
    pub session_id: String,
    pub tranche: GrillTranche,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_question_id: Option<String>,
    pub decision_branch_id: String,
    pub question: String,
    pub reason_for_asking: String,
    pub triggering_gap: String,
    pub recommendation: String,
    pub recommendation_basis_refs: Vec<String>,
    pub environment_facts_checked: Vec<String>,
    pub contradiction_refs: Vec<String>,
    pub linked_context_refs: Vec<String>,
    pub linked_spec_sections: Vec<String>,
    pub domain_term_candidates: Vec<String>,
    pub architecture_decision_candidates: Vec<String>,
    pub decision_required: bool,
    pub priority: InterviewGapPriority,
    pub answer_type: String,
    pub readiness_effect: String,
    pub stop_condition: String,
    pub branch_progress: String,
    pub operator_answer_is_authoritative: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrillInterviewStrategyResult {
    pub schema: String,
    pub strategy_id: String,
    pub strategy_version: u64,
    pub retrieval_performed_before_question: bool,
    pub one_question_only: bool,
    pub all_core_tranches_accounted_for: bool,
    pub ready_for_spec: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal: Option<InterviewNextQuestionProposal>,
}

fn nonblank(value: &str) -> bool {
    !value.trim().is_empty()
}

pub fn generate_next_question(
    context: &GrillInterviewContext,
) -> Result<GrillInterviewStrategyResult, String> {
    if !nonblank(&context.session_id) || !nonblank(&context.approved_role_profile_ref) {
        return Err("session_id and approved_role_profile_ref are required".into());
    }
    let represented: BTreeSet<GrillTranche> = context
        .completed_tranches
        .iter()
        .copied()
        .chain(context.gaps.iter().map(|gap| gap.tranche))
        .collect();
    let all_core_tranches_accounted_for = GrillTranche::ALL
        .iter()
        .all(|tranche| represented.contains(tranche));
    if !all_core_tranches_accounted_for {
        return Err("all six core Grill tranches must be completed or represented by gaps".into());
    }
    for gap in context.gaps.iter().filter(|gap| !gap.resolved) {
        if !nonblank(&gap.question)
            || !nonblank(&gap.reason_for_asking)
            || !nonblank(&gap.triggering_gap)
            || !nonblank(&gap.stop_condition)
            || !nonblank(&gap.decision_branch_id)
        {
            return Err(format!(
                "gap {} is missing required question metadata",
                gap.gap_id
            ));
        }
        if gap.decision_required
            && (!nonblank(&gap.recommendation) || gap.recommendation_basis_refs.is_empty())
        {
            return Err(format!(
                "decision gap {} requires one recommendation with cited basis",
                gap.gap_id
            ));
        }
        if gap.environment_facts_checked.is_empty() || gap.linked_context_refs.is_empty() {
            return Err(format!(
                "gap {} must prove fact retrieval and linked Context before asking",
                gap.gap_id
            ));
        }
    }

    let unresolved: Vec<&InterviewGapCandidate> =
        context.gaps.iter().filter(|gap| !gap.resolved).collect();
    if unresolved.is_empty() {
        return Ok(GrillInterviewStrategyResult {
            schema: "focusa.grill_interview_strategy_result.v1".into(),
            strategy_id: GRILL_WITH_DOCS_STRATEGY_ID.into(),
            strategy_version: GRILL_WITH_DOCS_STRATEGY_VERSION,
            retrieval_performed_before_question: true,
            one_question_only: true,
            all_core_tranches_accounted_for,
            ready_for_spec: true,
            proposal: None,
        });
    }

    let active_branch_has_gap = context.active_branch_id.as_deref().is_some_and(|branch| {
        unresolved
            .iter()
            .any(|gap| gap.decision_branch_id == branch)
    });
    let mut eligible: Vec<&InterviewGapCandidate> = unresolved
        .iter()
        .copied()
        .filter(|gap| {
            !active_branch_has_gap
                || context.active_branch_id.as_deref() == Some(gap.decision_branch_id.as_str())
        })
        .collect();
    eligible.sort_by(|left, right| {
        left.priority
            .rank()
            .cmp(&right.priority.rank())
            .then(
                right
                    .downstream_dependency_count
                    .cmp(&left.downstream_dependency_count),
            )
            .then(left.gap_id.cmp(&right.gap_id))
    });
    let selected = eligible[0];
    let branch_remaining = unresolved
        .iter()
        .filter(|gap| gap.decision_branch_id == selected.decision_branch_id)
        .count();
    let proposal = InterviewNextQuestionProposal {
        schema: "focusa.interview_next_question_proposal.v1".into(),
        strategy_id: GRILL_WITH_DOCS_STRATEGY_ID.into(),
        strategy_version: GRILL_WITH_DOCS_STRATEGY_VERSION,
        session_id: context.session_id.clone(),
        tranche: selected.tranche,
        parent_question_id: selected.parent_question_id.clone(),
        decision_branch_id: selected.decision_branch_id.clone(),
        question: selected.question.clone(),
        reason_for_asking: selected.reason_for_asking.clone(),
        triggering_gap: selected.triggering_gap.clone(),
        recommendation: selected.recommendation.clone(),
        recommendation_basis_refs: selected.recommendation_basis_refs.clone(),
        environment_facts_checked: selected.environment_facts_checked.clone(),
        contradiction_refs: selected.contradiction_refs.clone(),
        linked_context_refs: selected.linked_context_refs.clone(),
        linked_spec_sections: selected.linked_spec_sections.clone(),
        domain_term_candidates: selected.domain_term_candidates.clone(),
        architecture_decision_candidates: selected.architecture_decision_candidates.clone(),
        decision_required: selected.decision_required,
        priority: selected.priority,
        answer_type: selected.answer_type.clone(),
        readiness_effect: selected.readiness_effect.clone(),
        stop_condition: selected.stop_condition.clone(),
        branch_progress: format!("{branch_remaining} unresolved question(s) remain in branch"),
        operator_answer_is_authoritative: true,
    };
    Ok(GrillInterviewStrategyResult {
        schema: "focusa.grill_interview_strategy_result.v1".into(),
        strategy_id: GRILL_WITH_DOCS_STRATEGY_ID.into(),
        strategy_version: GRILL_WITH_DOCS_STRATEGY_VERSION,
        retrieval_performed_before_question: true,
        one_question_only: true,
        all_core_tranches_accounted_for,
        ready_for_spec: false,
        proposal: Some(proposal),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gap(
        id: &str,
        tranche: GrillTranche,
        branch: &str,
        priority: InterviewGapPriority,
        deps: u64,
    ) -> InterviewGapCandidate {
        InterviewGapCandidate {
            gap_id: id.into(),
            tranche,
            decision_branch_id: branch.into(),
            parent_question_id: None,
            question: format!("Decide {id}?"),
            reason_for_asking: "Operator-owned tradeoff".into(),
            triggering_gap: id.into(),
            recommendation: "Choose the bounded option".into(),
            recommendation_basis_refs: vec!["context-source:1".into()],
            environment_facts_checked: vec!["context-source:1".into()],
            contradiction_refs: vec![],
            linked_context_refs: vec!["context-source:1".into()],
            linked_spec_sections: vec!["135H §4".into()],
            domain_term_candidates: vec![],
            architecture_decision_candidates: vec![],
            decision_required: true,
            priority,
            answer_type: "select".into(),
            readiness_effect: "closes blocker".into(),
            stop_condition: "operator selects one option".into(),
            downstream_dependency_count: deps,
            resolved: false,
        }
    }

    #[test]
    fn active_branch_precedes_global_priority_and_returns_one_question() {
        let mut gaps = vec![
            gap(
                "discovery",
                GrillTranche::Discovery,
                "branch-a",
                InterviewGapPriority::Blocker,
                9,
            ),
            gap(
                "boundary",
                GrillTranche::Boundary,
                "branch-b",
                InterviewGapPriority::Normal,
                2,
            ),
            gap(
                "failure",
                GrillTranche::Failure,
                "branch-c",
                InterviewGapPriority::Normal,
                1,
            ),
            gap(
                "evidence",
                GrillTranche::Evidence,
                "branch-d",
                InterviewGapPriority::Normal,
                1,
            ),
            gap(
                "architecture",
                GrillTranche::Architecture,
                "branch-e",
                InterviewGapPriority::Normal,
                1,
            ),
            gap(
                "readiness",
                GrillTranche::SpecReadiness,
                "branch-f",
                InterviewGapPriority::Normal,
                1,
            ),
        ];
        gaps.push(gap(
            "boundary-2",
            GrillTranche::Boundary,
            "branch-b",
            InterviewGapPriority::High,
            5,
        ));
        let result = generate_next_question(&GrillInterviewContext {
            project_root: "/project".into(),
            continuity_id: "cont".into(),
            attachment_id: "attachment".into(),
            session_id: "session".into(),
            approved_role_profile_ref: "role:1".into(),
            active_branch_id: Some("branch-b".into()),
            completed_tranches: vec![],
            gaps,
        })
        .unwrap();
        assert!(result.one_question_only && result.retrieval_performed_before_question);
        assert_eq!(result.proposal.unwrap().question, "Decide boundary-2?");
    }
}
