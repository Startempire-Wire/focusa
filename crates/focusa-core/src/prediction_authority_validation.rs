//! Fail-closed semantic validation for every Spec 138 authority event variant.

use crate::prediction_authority::*;

fn text(value: &str) -> bool {
    !value.trim().is_empty()
}

fn refs(values: &[String]) -> bool {
    !values.is_empty() && values.iter().all(|value| text(value))
}

fn probability(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

fn require(condition: bool, reason: &str) -> Result<(), String> {
    condition.then_some(()).ok_or_else(|| reason.to_string())
}

pub fn validate_scoped_authority_event(event: &ScopedAuthorityEvent) -> Result<(), String> {
    event.scope.validate().map_err(|error| error.to_string())?;
    require(text(&event.event_id), "event_id is required")?;
    require(event.sequence > 0, "sequence must be positive")?;
    require(
        refs(&event.evidence_refs),
        "event evidence_refs are required",
    )?;
    require(text(&event.receipt_ref), "event receipt_ref is required")?;

    match &event.event {
        PredictionAuthorityEvent::EpistemicPrimitive(value) => {
            require(value.scope == event.scope, "primitive scope mismatch")?;
            crate::epistemic_primitives::validate_epistemic_primitive(value)
                .map_err(|error| format!("invalid epistemic primitive: {error:?}"))
        }
        PredictionAuthorityEvent::ReflectionClaim(value) => {
            crate::metacognitive_learning::validate_reflection_claim(value)
                .map_err(|error| format!("invalid reflection claim: {error:?}"))
        }
        PredictionAuthorityEvent::PromotionAssessment(value) => require(
            text(&value.assessment_id)
                && text(&value.candidate_id)
                && text(&value.policy_ref)
                && refs(&value.evidence_refs)
                && text(&value.receipt_ref),
            "promotion assessment identity, reasons, policy, evidence, and receipt are required",
        ),
        PredictionAuthorityEvent::LearningSettlement(value) => {
            let rollback_valid = !matches!(
                value.disposition,
                crate::metacognitive_learning::LearningDisposition::Rollback
            ) || value.rollback_ref.as_deref().is_some_and(text);
            require(
                text(&value.settlement_id)
                    && text(&value.application_id)
                    && !value.reason_codes.is_empty()
                    && rollback_valid
                    && refs(&value.evidence_refs)
                    && text(&value.receipt_ref),
                "learning settlement identity, reasons, rollback lineage, evidence, and receipt are required",
            )
        }
        PredictionAuthorityEvent::OutcomeAuthority(value) => {
            crate::outcome_resolution::validate_outcome_authority_event(value)
                .map_err(|error| format!("invalid outcome authority event: {error:?}"))?;
            use crate::outcome_resolution::OutcomeAuthorityAction;
            let action_valid = match &value.action {
                OutcomeAuthorityAction::Claim { claimed_outcome }
                | OutcomeAuthorityAction::Resolve {
                    resolved_outcome: claimed_outcome,
                } => text(claimed_outcome),
                OutcomeAuthorityAction::Dispute { reason }
                | OutcomeAuthorityAction::Void { reason }
                | OutcomeAuthorityAction::Censor { reason } => text(reason),
                OutcomeAuthorityAction::Correct {
                    resolved_outcome,
                    supersedes_event_ref,
                } => text(resolved_outcome) && text(supersedes_event_ref),
                OutcomeAuthorityAction::Escalate => true,
            };
            require(action_valid, "outcome authority action payload is required")
        }
        PredictionAuthorityEvent::FusionResult(value) => require(
            text(&value.fusion_id)
                && text(&value.policy_ref)
                && value.fused_value.is_finite()
                && value.independent_cluster_count > 0
                && !value.contributions.is_empty()
                && value.contributions.iter().all(|item| {
                    text(&item.signal_id)
                        && text(&item.source_ref)
                        && text(&item.correlation_cluster_ref)
                        && item.normalized_weight.is_finite()
                        && item.effective_weight.is_finite()
                        && item.value.is_finite()
                        && item.weighted_value.is_finite()
                })
                && refs(&value.evidence_refs)
                && text(&value.receipt_ref),
            "fusion result identity, finite contributions, evidence, and receipt are required",
        ),
        PredictionAuthorityEvent::ScenarioProjection(value) => require(
            text(&value.scenario_id)
                && value.expected_value.is_finite()
                && !value.branch_contributions.is_empty()
                && value
                    .branch_contributions
                    .iter()
                    .all(|(id, contribution)| text(id) && contribution.is_finite())
                && refs(&value.evidence_refs)
                && text(&value.receipt_ref),
            "scenario projection identity, finite branches, evidence, and receipt are required",
        ),
        PredictionAuthorityEvent::TransferEvaluation(value) => require(
            text(&value.evaluation_id)
                && text(&value.transfer_id)
                && value.baseline_metric.is_finite()
                && value.observed_metric.is_finite()
                && value.observed_effect.is_finite()
                && probability(value.confidence)
                && refs(&value.evidence_refs)
                && text(&value.receipt_ref),
            "transfer evaluation identity, finite metrics, confidence, evidence, and receipt are required",
        ),
        PredictionAuthorityEvent::SelfModelEstimate(value) => {
            crate::prediction_advanced::validate_self_model(value, event.recorded_at)
                .map_err(|error| format!("invalid self-model estimate: {error:?}"))
        }
        PredictionAuthorityEvent::MemoryLifecycle(value) => require(
            text(&value.event_id)
                && text(&value.memory_id)
                && text(&value.authority_ref)
                && refs(&value.evidence_refs)
                && text(&value.receipt_ref),
            "memory lifecycle identity, authority, evidence, and receipt are required",
        ),
        PredictionAuthorityEvent::SourceSecurityDecision(value) => require(
            text(&value.decision_id)
                && text(&value.request_id)
                && !value.reason_codes.is_empty()
                && refs(&value.least_privilege_scope_refs)
                && text(&value.retention_policy_ref)
                && refs(&value.evidence_refs)
                && text(&value.receipt_ref),
            "source security identity, reasons, scope, retention, evidence, and receipt are required",
        ),
        PredictionAuthorityEvent::LegacyMigration(value) => require(
            text(&value.migration_id)
                && text(&value.source_record_ref)
                && value.source_sha256.len() == 64
                && value
                    .source_sha256
                    .chars()
                    .all(|character| character.is_ascii_hexdigit())
                && !value.mapped_primitive_refs.is_empty()
                && refs(&value.lineage_refs)
                && text(&value.rollback_ref)
                && refs(&value.evidence_refs)
                && text(&value.receipt_ref),
            "legacy migration identity, digest, mappings, lineage, rollback, evidence, and receipt are required",
        ),
        PredictionAuthorityEvent::Question(value) => require(
            text(&value.question_id)
                && text(&value.subject_ref)
                && value.outcome_space.len() >= 2
                && refs(&value.outcome_space)
                && text(&value.horizon_claim_ref)
                && refs(&value.evidence_refs),
            "prediction question identity, outcome space, horizon, and evidence are required",
        ),
        PredictionAuthorityEvent::Commitment(value) => validate_commitment(value),
        PredictionAuthorityEvent::ActionCommitment(value) => {
            crate::prediction_authority::validate_action_prediction_commitment(value)
                .map_err(|error| format!("invalid action commitment: {error:?}"))
        }
        PredictionAuthorityEvent::ActionOutcome(value) => require(
            text(&value.observation_id)
                && text(&value.action_id)
                && text(&value.commitment_id)
                && text(&value.predicted_outcome)
                && text(&value.actual_outcome)
                && probability(value.outcome_match_score)
                && value.actual_duration_ns == value.timing_trace.total_elapsed_ns
                && value.duration_delta_ns
                    == value.actual_duration_ns as i128 - value.expected_duration_ns as i128
                && value.outcome_claim.commitment_id == value.commitment_id
                && refs(&value.evidence_refs),
            "action outcome identity, score, timing, claim linkage, and evidence are required",
        ),
        PredictionAuthorityEvent::ActionPattern(value) => {
            crate::metacognitive_learning::validate_action_delta_pattern(value)
                .map_err(|error| format!("invalid action pattern: {error:?}"))
        }
        PredictionAuthorityEvent::OutcomeClaim(value) => require(
            text(&value.claim_id)
                && text(&value.commitment_id)
                && text(&value.claimed_outcome)
                && text(&value.source_ref)
                && refs(&value.evidence_refs),
            "outcome claim identity, outcome, source, and evidence are required",
        ),
        PredictionAuthorityEvent::OutcomeResolution(value) => require(
            text(&value.resolution_id)
                && text(&value.claim_id)
                && text(&value.resolved_outcome)
                && text(&value.resolver_policy_ref)
                && probability(value.resolution_confidence)
                && refs(&value.evidence_refs)
                && text(&value.receipt_ref),
            "outcome resolution identity, policy, confidence, evidence, and receipt are required",
        ),
        PredictionAuthorityEvent::ScoringPolicy(value) => require(
            text(&value.policy_id)
                && value.version > 0
                && value.range_min.is_finite()
                && value.range_max.is_finite()
                && value.range_min < value.range_max
                && !value.assumptions.is_empty()
                && refs(&value.evidence_refs),
            "scoring policy identity, version, range, assumptions, and evidence are required",
        ),
        PredictionAuthorityEvent::Evaluation(value) => require(
            text(&value.evaluation_id)
                && text(&value.commitment_id)
                && text(&value.resolution_id)
                && text(&value.scoring_policy_ref)
                && value.canonical_score.is_finite()
                && refs(&value.evidence_refs)
                && text(&value.receipt_ref),
            "prediction evaluation identity, policy, finite score, evidence, and receipt are required",
        ),
        PredictionAuthorityEvent::LearningCandidate(value) => require(
            text(&value.candidate_id)
                && text(&value.evaluation_id)
                && text(&value.reason_code)
                && text(&value.hypothesis)
                && refs(&value.applicability.includes)
                && value.review_at > value.created_at
                && value.expires_at > value.review_at
                && refs(&value.evidence_refs),
            "learning candidate identity, applicability, review window, and evidence are required",
        ),
        PredictionAuthorityEvent::PromotionDecision(value) => {
            let reasons_valid = matches!(value.decision, PromotionDecisionKind::Promoted)
                || !value.reason_codes.is_empty();
            require(
                text(&value.decision_id)
                    && text(&value.candidate_id)
                    && reasons_valid
                    && text(&value.receipt_ref),
                "promotion decision identity, rejection reasons, and receipt are required",
            )
        }
        PredictionAuthorityEvent::LearningRecord(value) => {
            let rollback_valid = !matches!(value.status, LearningStatus::RolledBack)
                || value.rollback_ref.as_deref().is_some_and(text);
            require(
                text(&value.learning_id)
                    && text(&value.candidate_id)
                    && text(&value.decision_id)
                    && text(&value.content)
                    && refs(&value.applicability.includes)
                    && text(&value.review_at_claim_ref)
                    && text(&value.expiry_claim_ref)
                    && rollback_valid
                    && refs(&value.evidence_refs)
                    && text(&value.receipt_ref),
                "learning record identity, applicability, lifecycle refs, evidence, and receipt are required",
            )
        }
        PredictionAuthorityEvent::TransferPrediction(value) => require(
            text(&value.transfer_id)
                && text(&value.learning_id)
                && text(&value.source_context_ref)
                && text(&value.target_context_ref)
                && value.source_context_ref != value.target_context_ref
                && value.expected_metric_delta.is_finite()
                && text(&value.window_claim_ref)
                && refs(&value.evidence_refs),
            "transfer prediction identity, distinct contexts, finite delta, window, and evidence are required",
        ),
        PredictionAuthorityEvent::TransferOutcome(value) => require(
            text(&value.outcome_id)
                && text(&value.transfer_id)
                && value.observed_metric_delta.is_finite()
                && refs(&value.evidence_refs)
                && text(&value.receipt_ref),
            "transfer outcome identity, finite delta, evidence, and receipt are required",
        ),
    }
}

fn validate_commitment(value: &PredictionCommitment) -> Result<(), String> {
    value.confidence.validate()?;
    require(
        text(&value.commitment_id)
            && text(&value.question_id)
            && text(&value.predicted_outcome)
            && text(&value.information_set.information_set_id)
            && value.information_set.version > 0
            && text(&value.information_set.as_of_claim_ref)
            && refs(&value.information_set.evidence_refs)
            && text(&value.resolver_policy_ref)
            && text(&value.scoring_policy_ref)
            && refs(&value.evidence_refs)
            && text(&value.receipt_ref),
        "prediction commitment identity, information set, policies, evidence, and receipt are required",
    )
}
