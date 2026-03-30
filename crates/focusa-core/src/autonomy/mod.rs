//! Autonomy Calibration — docs/37-autonomy-calibration-spec.md
//!
//! Evidence-based trust scoring.
//! ARI: 0–100, 6 dimensions: Correctness, Stability, Efficiency, Trust, Grounding, Recovery.
//! ARI weights: Outcome 50%, Efficiency 20%, Discipline 15%, Safety 15%.
//! 6 levels: AL0 (advisory) → AL5 (long-horizon).
//! Never self-escalates. Human grant required.

use crate::types::*;
use chrono::Utc;

/// Compute ARI score from dimensions.
/// Outcome 50%, Efficiency 20%, Discipline 15%, Safety 15%.
pub fn compute_ari(dims: &AutonomyDimensions) -> f64 {
    let outcome = (dims.correctness + dims.trust) / 2.0;
    let efficiency = dims.efficiency;
    let discipline = dims.grounding;
    let safety = (dims.stability + dims.recovery) / 2.0;

    let ari = outcome * 0.50 + efficiency * 0.20 + discipline * 0.15 + safety * 0.15;
    (ari * 100.0).clamp(0.0, 100.0)
}

/// Update a dimension score (exponential moving average).
pub fn update_dimension(current: &mut f64, observation: f64, alpha: f64) {
    *current = *current * (1.0 - alpha) + observation * alpha;
    *current = current.clamp(0.0, 1.0);
}

/// Check if promotion is recommended based on ARI and sample count.
pub fn should_recommend_promotion(state: &AutonomyState) -> Option<AutonomyLevel> {
    let ari = state.ari_score;
    let samples = state.sample_count;

    // Check TTL expiry.
    if let Some(ttl) = state.granted_ttl
        && Utc::now() > ttl
    {
        return None; // Grant expired.
    }

    // Never self-escalates — return recommendation only.
    match state.level {
        AutonomyLevel::AL0 if ari >= 40.0 && samples >= 10 => Some(AutonomyLevel::AL1),
        AutonomyLevel::AL1 if ari >= 55.0 && samples >= 25 => Some(AutonomyLevel::AL2),
        AutonomyLevel::AL2 if ari >= 70.0 && samples >= 50 => Some(AutonomyLevel::AL3),
        AutonomyLevel::AL3 if ari >= 80.0 && samples >= 100 => Some(AutonomyLevel::AL4),
        AutonomyLevel::AL4 if ari >= 90.0 && samples >= 200 => Some(AutonomyLevel::AL5),
        _ => None,
    }
}

/// Grant autonomy level (human-initiated).
pub fn grant_level(
    state: &mut AutonomyState,
    level: AutonomyLevel,
    scope: Option<String>,
    ttl: Option<chrono::DateTime<Utc>>,
    reason: &str,
) {
    let event = AutonomyEvent {
        timestamp: Utc::now(),
        event_type: "grant".into(),
        from_level: state.level,
        to_level: level,
        reason: reason.into(),
        evidence: vec![format!(
            "ARI: {:.1}, samples: {}",
            state.ari_score, state.sample_count
        )],
    };
    state.history.push(event);
    state.level = level;
    state.granted_scope = scope;
    state.granted_ttl = ttl;
}

/// Record an observation for autonomy scoring.
pub fn record_observation(
    state: &mut AutonomyState,
    correctness: f64,
    stability: f64,
    efficiency: f64,
    trust: f64,
    grounding: f64,
    recovery: f64,
) {
    let alpha = 0.05;
    update_dimension(&mut state.dimensions.correctness, correctness, alpha);
    update_dimension(&mut state.dimensions.stability, stability, alpha);
    update_dimension(&mut state.dimensions.efficiency, efficiency, alpha);
    update_dimension(&mut state.dimensions.trust, trust, alpha);
    update_dimension(&mut state.dimensions.grounding, grounding, alpha);
    update_dimension(&mut state.dimensions.recovery, recovery, alpha);
    state.ari_score = compute_ari(&state.dimensions);
    state.sample_count += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_ari() {
        let dims = AutonomyDimensions {
            correctness: 0.8,
            stability: 0.7,
            efficiency: 0.9,
            trust: 0.8,
            grounding: 0.6,
            recovery: 0.7,
        };
        let ari = compute_ari(&dims);
        assert!(ari > 50.0 && ari < 100.0);
    }

    #[test]
    fn test_al0_no_promotion_without_samples() {
        let state = AutonomyState::default();
        assert!(should_recommend_promotion(&state).is_none());
    }

    #[test]
    fn test_grant_records_history() {
        let mut state = AutonomyState::default();
        grant_level(
            &mut state,
            AutonomyLevel::AL2,
            Some("./repo".into()),
            None,
            "test",
        );
        assert_eq!(state.level, AutonomyLevel::AL2);
        assert_eq!(state.history.len(), 1);
    }
}
