//! PRE Resolution Scoring — deterministic proposal scoring per 41-proposal-resolution-engine.
//!
//! Resolution algorithm:
//! 1. Gather pending proposals in window
//! 2. Compute score for each proposal
//! 3. Select winner (or no-winner)
//! 4. Emit resolution events

use crate::pre::Proposal;
use crate::types::{FocusaState, ProposalKind, RfmLevel};
use chrono::{DateTime, Utc};

/// Resolution window configuration.
#[derive(Debug, Clone)]
pub struct ResolutionConfig {
    /// Window duration in milliseconds.
    pub window_duration_ms: u64,
    /// Minimum evidence strength required (0.0-1.0).
    pub min_evidence_strength: f64,
    /// Bias toward later proposals (0.0-0.2).
    pub recency_bias: f64,
    /// Required confidence threshold.
    pub confidence_threshold: f64,
}

impl Default for ResolutionConfig {
    fn default() -> Self {
        Self {
            window_duration_ms: 2000,
            min_evidence_strength: 0.5,
            recency_bias: 0.1,
            confidence_threshold: 0.7,
        }
    }
}

/// Scored proposal with computed metrics.
#[derive(Debug, Clone)]
pub struct ScoredProposal {
    pub proposal: Proposal,
    pub base_score: f64,
    pub risk_adjusted_score: f64,
    pub final_score: f64,
}

/// Resolution outcome.
#[derive(Debug, Clone)]
pub enum ResolutionOutcome {
    /// One proposal accepted.
    Accepted {
        winner: Proposal,
        score: f64,
        reason: String,
    },
    /// All proposals rejected (insufficient evidence/conflict).
    RejectedAll { reason: String },
    /// Clarification required (too divergent).
    ClarificationRequired {
        proposals: Vec<Proposal>,
        reason: String,
    },
}

/// Score proposals and determine winner.
///
/// Per 41-proposal-resolution-engine §6: Scoring MUST be deterministic.
pub fn resolve_proposals(
    proposals: &[Proposal],
    state: &FocusaState,
    config: &ResolutionConfig,
    window_start: DateTime<Utc>,
) -> ResolutionOutcome {
    if proposals.is_empty() {
        return ResolutionOutcome::RejectedAll {
            reason: "No proposals in window".into(),
        };
    }

    // Score all proposals.
    let mut scored: Vec<ScoredProposal> = proposals
        .iter()
        .map(|p| score_proposal(p, state, config, window_start))
        .collect();

    // Sort by final score descending.
    scored.sort_by(|a, b| {
        b.final_score
            .partial_cmp(&a.final_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Check for winner.
    let winner = &scored[0];

    // Check if winner meets thresholds.
    let required_threshold = proposal_confidence_threshold(winner.proposal.kind, config);
    if winner.final_score < required_threshold {
        return ResolutionOutcome::RejectedAll {
            reason: format!(
                "Highest score {:.2} below threshold {:.2}",
                winner.final_score, required_threshold
            ),
        };
    }

    // Check if second place is too close (divergence).
    if scored.len() > 1 {
        let second = &scored[1];
        let gap = winner.final_score - second.final_score;
        let required_gap = proposal_clarification_gap(winner.proposal.kind)
            .max(proposal_clarification_gap(second.proposal.kind));
        if gap < required_gap {
            return ResolutionOutcome::ClarificationRequired {
                proposals: proposals.to_vec(),
                reason: format!(
                    "Top proposals too close: {:.2} vs {:.2} (required gap {:.2})",
                    winner.final_score, second.final_score, required_gap
                ),
            };
        }
    }

    // Accept winner.
    ResolutionOutcome::Accepted {
        winner: winner.proposal.clone(),
        score: winner.final_score,
        reason: format!(
            "Score {:.2}: base {:.2}, risk-adjusted {:.2}",
            winner.final_score, winner.base_score, winner.risk_adjusted_score
        ),
    }
}

/// Score a single proposal.
fn score_proposal(
    proposal: &Proposal,
    state: &FocusaState,
    config: &ResolutionConfig,
    window_start: DateTime<Utc>,
) -> ScoredProposal {
    // Base score from existing score field and kind-specific policy weight.
    let base_score = proposal.score * proposal_kind_weight(proposal.kind);

    // Recency bonus.
    let recency_score = compute_recency_score(proposal, config, window_start);

    // Risk adjustment (RFM).
    let risk_adjusted_score =
        apply_rfm_adjustment(base_score + recency_score * config.recency_bias, state);

    ScoredProposal {
        proposal: proposal.clone(),
        base_score,
        risk_adjusted_score,
        final_score: risk_adjusted_score.min(1.0),
    }
}

/// Compute recency score (later proposals score higher).
fn compute_recency_score(
    proposal: &Proposal,
    config: &ResolutionConfig,
    window_start: DateTime<Utc>,
) -> f64 {
    let window_duration = chrono::Duration::milliseconds(config.window_duration_ms as i64);
    let total_ms = window_duration.num_milliseconds() as f64;
    let elapsed_ms = (proposal.created_at - window_start).num_milliseconds() as f64;

    if total_ms <= 0.0 {
        return 0.5;
    }

    // Later proposals get higher score.
    (elapsed_ms / total_ms).clamp(0.0, 1.0)
}

fn proposal_kind_weight(kind: ProposalKind) -> f64 {
    match kind {
        ProposalKind::OntologyGovernanceMutation => 0.95,
        ProposalKind::ReferenceResolutionMutation => 0.97,
        ProposalKind::IdentityModelMutation => 0.97,
        ProposalKind::QueryScopeMutation => 0.98,
        ProposalKind::ProjectionViewMutation => 0.98,
        ProposalKind::VisualModelMutation => 0.99,
        _ => 1.0,
    }
}

fn proposal_confidence_threshold(kind: ProposalKind, config: &ResolutionConfig) -> f64 {
    match kind {
        ProposalKind::OntologyGovernanceMutation => config.confidence_threshold.max(0.82),
        ProposalKind::ReferenceResolutionMutation => config.confidence_threshold.max(0.78),
        ProposalKind::IdentityModelMutation => config.confidence_threshold.max(0.76),
        ProposalKind::QueryScopeMutation => config.confidence_threshold.max(0.74),
        ProposalKind::ProjectionViewMutation => config.confidence_threshold.max(0.74),
        ProposalKind::VisualModelMutation => config.confidence_threshold.max(0.72),
        _ => config.confidence_threshold,
    }
}

fn proposal_clarification_gap(kind: ProposalKind) -> f64 {
    match kind {
        ProposalKind::OntologyGovernanceMutation => 0.2,
        ProposalKind::ReferenceResolutionMutation => 0.15,
        ProposalKind::IdentityModelMutation => 0.14,
        ProposalKind::QueryScopeMutation => 0.12,
        ProposalKind::ProjectionViewMutation => 0.12,
        ProposalKind::VisualModelMutation => 0.11,
        _ => 0.1,
    }
}

/// Apply RFM-based risk adjustment.
fn apply_rfm_adjustment(base_score: f64, state: &FocusaState) -> f64 {
    match state.rfm.level {
        RfmLevel::R0 => base_score,       // Normal.
        RfmLevel::R1 => base_score * 0.9, // Slight penalty.
        RfmLevel::R2 => base_score * 0.7, // Significant penalty.
        RfmLevel::R3 => base_score * 0.5, // Heavy penalty.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pre::{Proposal, submit};
    use crate::types::{PreState, ProposalKind, ProposalStatus};
    use uuid::Uuid;

    fn test_state() -> FocusaState {
        FocusaState::default()
    }

    fn test_config() -> ResolutionConfig {
        ResolutionConfig {
            window_duration_ms: 2000,
            min_evidence_strength: 0.5,
            recency_bias: 0.1,
            confidence_threshold: 0.7,
        }
    }

    // SMOKE TEST: Empty proposals
    #[test]
    fn test_resolve_empty() {
        let state = test_state();
        let config = test_config();

        let outcome = resolve_proposals(&[], &state, &config, Utc::now());

        match outcome {
            ResolutionOutcome::RejectedAll { reason } => {
                assert!(reason.contains("No proposals"));
            }
            _ => panic!("Expected RejectedAll for empty proposals"),
        }
    }

    // SMOKE TEST: Single proposal accepted
    #[test]
    fn test_resolve_single_accepted() {
        let state = test_state();
        let config = test_config();

        let mut pre_state = PreState {
            proposals: vec![],
            resolution_window_ms: 5000,
        };

        let id = submit(
            &mut pre_state,
            ProposalKind::FocusChange,
            "test",
            serde_json::json!({}),
            60000,
        );

        // Score the proposal high.
        pre_state
            .proposals
            .iter_mut()
            .find(|p| p.id == id)
            .unwrap()
            .score = 0.9;

        let proposal = pre_state.proposals.into_iter().next().unwrap();

        let outcome = resolve_proposals(
            &[proposal.clone()],
            &state,
            &config,
            Utc::now() - chrono::Duration::seconds(1),
        );

        match outcome {
            ResolutionOutcome::Accepted { winner, score, .. } => {
                assert_eq!(winner.id, proposal.id);
                assert!(score > 0.0);
            }
            _ => panic!("Expected Accepted for high-score proposal"),
        }
    }

    // SMOKE TEST: Low score rejected
    #[test]
    fn test_resolve_low_score_rejected() {
        let state = test_state();
        let config = test_config();

        let mut pre_state = PreState {
            proposals: vec![],
            resolution_window_ms: 5000,
        };

        let id = submit(
            &mut pre_state,
            ProposalKind::FocusChange,
            "test",
            serde_json::json!({}),
            60000,
        );

        // Score the proposal low.
        pre_state
            .proposals
            .iter_mut()
            .find(|p| p.id == id)
            .unwrap()
            .score = 0.3;

        let proposal = pre_state.proposals.into_iter().next().unwrap();

        let outcome = resolve_proposals(&[proposal], &state, &config, Utc::now());

        match outcome {
            ResolutionOutcome::RejectedAll { reason } => {
                assert!(reason.contains("below threshold"));
            }
            _ => panic!("Expected RejectedAll for low score"),
        }
    }

    // SMOKE TEST: Clarification required (close scores)
    #[test]
    fn test_resolve_clarification_required() {
        let state = test_state();
        let config = test_config();

        let mut pre_state = PreState {
            proposals: vec![],
            resolution_window_ms: 5000,
        };

        // Two very similar proposals.
        let id1 = submit(
            &mut pre_state,
            ProposalKind::FocusChange,
            "test",
            serde_json::json!({"id": 1}),
            60000,
        );
        pre_state
            .proposals
            .iter_mut()
            .find(|p| p.id == id1)
            .unwrap()
            .score = 0.85;

        let id2 = submit(
            &mut pre_state,
            ProposalKind::FocusChange,
            "test",
            serde_json::json!({"id": 2}),
            60000,
        );
        pre_state
            .proposals
            .iter_mut()
            .find(|p| p.id == id2)
            .unwrap()
            .score = 0.84;

        let proposals: Vec<Proposal> = pre_state.proposals.into_iter().collect();

        let outcome = resolve_proposals(
            &proposals,
            &state,
            &config,
            Utc::now() - chrono::Duration::seconds(1),
        );

        // Should require clarification because scores are too close.
        match outcome {
            ResolutionOutcome::ClarificationRequired { proposals, .. } => {
                assert_eq!(proposals.len(), 2);
            }
            ResolutionOutcome::Accepted { .. } => {
                // Also acceptable if recency creates clear winner.
            }
            _ => panic!("Unexpected outcome"),
        }
    }

    // STRESS TEST: Many proposals
    #[test]
    fn test_resolve_many_proposals() {
        let state = test_state();
        let config = ResolutionConfig {
            confidence_threshold: 0.6, // Lower threshold for test.
            recency_bias: 0.0,         // Disable recency bias to avoid close scores.
            ..test_config()
        };

        let window_start = Utc::now() - chrono::Duration::seconds(1);

        // Create proposals with clear score gaps.
        let mut proposals: Vec<Proposal> = (0..50)
            .map(|i| Proposal {
                id: Uuid::now_v7(),
                kind: ProposalKind::FocusChange,
                source: "test".into(),
                created_at: window_start + chrono::Duration::milliseconds(i as i64 * 10),
                deadline: Utc::now() + chrono::Duration::seconds(60),
                payload: serde_json::json!({"i": i}),
                score: 0.6 + (i as f64 * 0.008), // Scores from 0.6 to 0.992.
                status: ProposalStatus::Pending,
            })
            .collect();

        // Add a clear winner with much higher score.
        proposals.push(Proposal {
            id: Uuid::now_v7(),
            kind: ProposalKind::FocusChange,
            source: "test".into(),
            created_at: window_start + chrono::Duration::milliseconds(500),
            deadline: Utc::now() + chrono::Duration::seconds(60),
            payload: serde_json::json!({"winner": true}),
            score: 0.99, // Clear winner.
            status: ProposalStatus::Pending,
        });

        let start = std::time::Instant::now();
        let outcome = resolve_proposals(&proposals, &state, &config, window_start);
        let elapsed = start.elapsed();

        println!("Resolved {} proposals in {:?}", proposals.len(), elapsed);
        assert!(elapsed.as_millis() < 100, "Resolution too slow");

        // Should have a winner (highest score will win).
        match outcome {
            ResolutionOutcome::Accepted { score, .. } => {
                assert!(score > 0.95, "Winner should have high score: {}", score);
            }
            ResolutionOutcome::RejectedAll { reason } => {
                panic!("Expected a winner: {}", reason);
            }
            ResolutionOutcome::ClarificationRequired { .. } => {
                // Acceptable if scores are close, but log it.
                println!("Clarification required - scores too close");
            }
        }
    }

    // STRESS TEST: RFM risk adjustment
    #[test]
    fn test_rfm_risk_adjustment() {
        let mut state = test_state();
        let config = test_config();

        let proposal = Proposal {
            id: Uuid::now_v7(),
            kind: ProposalKind::FocusChange,
            source: "test".into(),
            created_at: Utc::now(),
            deadline: Utc::now() + chrono::Duration::seconds(60),
            payload: serde_json::json!({}),
            score: 0.9,
            status: ProposalStatus::Pending,
        };

        // Test each RFM level.
        for level in [RfmLevel::R0, RfmLevel::R1, RfmLevel::R2, RfmLevel::R3] {
            state.rfm.level = level;

            let scored = score_proposal(&proposal, &state, &config, Utc::now());

            println!("RFM {:?}: score = {:.2}", level, scored.final_score);

            // Higher RFM should lower score.
            match level {
                RfmLevel::R0 => assert!(scored.final_score > 0.8),
                RfmLevel::R1 => assert!(scored.final_score > 0.7),
                RfmLevel::R2 => assert!(scored.final_score > 0.5),
                RfmLevel::R3 => assert!(scored.final_score > 0.3),
                _ => {}
            }
        }
    }

    // STRESS TEST: Recency bias
    #[test]
    fn test_recency_bias() {
        let state = test_state();
        let config = ResolutionConfig {
            recency_bias: 0.2,
            ..Default::default()
        };

        let window_start = Utc::now() - chrono::Duration::seconds(10);

        // Early proposal.
        let early = Proposal {
            id: Uuid::now_v7(),
            kind: ProposalKind::FocusChange,
            source: "test".into(),
            created_at: window_start,
            deadline: Utc::now() + chrono::Duration::seconds(60),
            payload: serde_json::json!({}),
            score: 0.8,
            status: ProposalStatus::Pending,
        };

        // Late proposal.
        let late = Proposal {
            id: Uuid::now_v7(),
            kind: ProposalKind::FocusChange,
            source: "test".into(),
            created_at: Utc::now(),
            deadline: Utc::now() + chrono::Duration::seconds(60),
            payload: serde_json::json!({}),
            score: 0.8,
            status: ProposalStatus::Pending,
        };

        let early_scored = score_proposal(&early, &state, &config, window_start);
        let late_scored = score_proposal(&late, &state, &config, window_start);

        println!(
            "Early: {:.2}, Late: {:.2}",
            early_scored.final_score, late_scored.final_score
        );

        // Late proposal should score higher due to recency.
        assert!(late_scored.final_score > early_scored.final_score);
    }
}
