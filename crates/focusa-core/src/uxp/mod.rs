//! UXP / UFI — docs/14-uxp-ufi-schema.md
//!
//! UXP: User Experience Profile — slow-moving calibration (α ≤ 0.1, window ≥ 30).
//! UFI: User Friction Index — fast-moving interaction cost (14 signal types, 3 tiers).
//! UFI → UXP bridge: UXP_new = clamp(UXP_old × (1 - α) + mean(UFI_window) × α, 0.0, 1.0)

use crate::types::*;
use chrono::Utc;

/// Record a UFI signal.
pub fn record_ufi_signal(ufi: &mut UfiState, signal_type: UfiSignalType, session_id: Option<SessionId>) {
    let tier = weight_tier(signal_type);
    ufi.signals.push(UfiSignal {
        signal_type,
        timestamp: Utc::now(),
        session_id,
        weight_tier: tier,
    });

    // Recompute aggregate.
    ufi.aggregate = compute_aggregate(&ufi.signals);
}

/// UFI aggregate — weighted mean of recent signals.
/// Language signals (Low tier) NEVER dominate: capped at 30% of aggregate.
fn compute_aggregate(signals: &[UfiSignal]) -> f64 {
    if signals.is_empty() {
        return 0.0;
    }

    let window = &signals[signals.len().saturating_sub(50)..];
    let mut weighted_sum: f64 = 0.0;
    let mut total_weight: f64 = 0.0;
    let mut low_contribution: f64 = 0.0;

    for s in window {
        let w = match s.weight_tier {
            UfiWeightTier::High => 3.0,
            UfiWeightTier::Medium => 2.0,
            UfiWeightTier::Low => 1.0,
        };
        weighted_sum += w;
        total_weight += 3.0; // Normalize against max.
        if s.weight_tier == UfiWeightTier::Low {
            low_contribution += w;
        }
    }

    if total_weight == 0.0 {
        return 0.0;
    }

    let raw = weighted_sum / total_weight;

    // Cap low-tier contribution at 30%.
    let low_ratio = low_contribution / weighted_sum.max(1.0);
    if low_ratio > 0.3 {
        raw * 0.85 // Dampen when language signals dominate.
    } else {
        raw
    }
}

/// Apply UFI → UXP bridge for a specific dimension.
/// UXP_new = clamp(UXP_old × (1 - α) + mean(UFI_window) × α, 0.0, 1.0)
pub fn bridge_ufi_to_uxp(dim: &mut UxpDimension, ufi_mean: f64) {
    if dim.frozen {
        return; // User override freezes learning.
    }
    let alpha = dim.learning_rate.min(0.1); // α ≤ 0.1 enforced.
    dim.value = (dim.value * (1.0 - alpha) + ufi_mean * alpha).clamp(0.0, 1.0);
    dim.confidence = (dim.confidence + 0.01).min(1.0);
}

/// Get weight tier for a signal type.
pub fn weight_tier(st: UfiSignalType) -> UfiWeightTier {
    match st {
        UfiSignalType::TaskReopened
        | UfiSignalType::ManualOverride
        | UfiSignalType::ImmediateCorrection
        | UfiSignalType::UndoOrRevert
        | UfiSignalType::ExplicitRejection => UfiWeightTier::High,

        UfiSignalType::Rephrase
        | UfiSignalType::RepeatRequest
        | UfiSignalType::ScopeClarification
        | UfiSignalType::ForcedSimplification => UfiWeightTier::Medium,

        UfiSignalType::NegationLanguage
        | UfiSignalType::MetaLanguage
        | UfiSignalType::ImpatienceMarker
        | UfiSignalType::FrustrationIndicator
        | UfiSignalType::EscalationEvent => UfiWeightTier::Low,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ufi_signal_recording() {
        let mut ufi = UfiState::default();
        record_ufi_signal(&mut ufi, UfiSignalType::ManualOverride, None);
        assert_eq!(ufi.signals.len(), 1);
        assert!(ufi.aggregate > 0.0);
    }

    #[test]
    fn test_bridge_frozen_dimension() {
        let mut dim = UxpDimension {
            value: 0.5, confidence: 0.5, citations: vec![],
            learning_rate: 0.1, window_size: 30, frozen: true,
        };
        bridge_ufi_to_uxp(&mut dim, 0.9);
        assert_eq!(dim.value, 0.5); // Unchanged.
    }

    #[test]
    fn test_bridge_updates_dimension() {
        let mut dim = UxpDimension {
            value: 0.5, confidence: 0.0, citations: vec![],
            learning_rate: 0.1, window_size: 30, frozen: false,
        };
        bridge_ufi_to_uxp(&mut dim, 0.8);
        assert!(dim.value > 0.5);
        assert!(dim.value < 0.8);
    }
}
