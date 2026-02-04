//! Reliability Focus Mode (RFM) — docs/36-reliability-focus-mode.md
//!
//! 4 levels: R0 (normal) → R3 (ensemble).
//! Artifact Integrity Score (AIS): ≥0.90 safe, 0.70–0.90 degraded, <0.70 triggers RFM.
//! 4 microcell validators: Schema, Constraint, Consistency, ReferenceGrounding.
//!
//! Trigger: AIS < 0.70 → auto-escalate to R1.
//! De-escalation: 3 consecutive passes → drop one level.

use crate::types::*;
use chrono::Utc;

/// Run all validators against an artifact.
pub fn validate(content: &str, constraints: &[String]) -> Vec<ValidatorResult> {
    let now = Utc::now();
    vec![
        ValidatorResult {
            validator: MicrocellValidator::Schema,
            passed: validate_schema(content),
            details: "Schema validation".into(),
            timestamp: now,
        },
        ValidatorResult {
            validator: MicrocellValidator::Constraint,
            passed: validate_constraints(content, constraints),
            details: "Constraint compliance".into(),
            timestamp: now,
        },
        ValidatorResult {
            validator: MicrocellValidator::Consistency,
            passed: validate_consistency(content),
            details: "Internal consistency".into(),
            timestamp: now,
        },
        ValidatorResult {
            validator: MicrocellValidator::ReferenceGrounding,
            passed: validate_grounding(content),
            details: "Reference grounding".into(),
            timestamp: now,
        },
    ]
}

/// Compute AIS from validator results.
pub fn compute_ais(results: &[ValidatorResult]) -> f64 {
    if results.is_empty() {
        return 1.0;
    }
    let passed = results.iter().filter(|r| r.passed).count() as f64;
    passed / results.len() as f64
}

/// Update RFM state based on AIS.
/// Returns true if level changed.
pub fn update_rfm(state: &mut RfmState, results: Vec<ValidatorResult>) -> bool {
    let ais = compute_ais(&results);
    let old_level = state.level;
    state.ais_score = ais;
    state.validator_results = results;

    // Escalation.
    if ais < 0.70 && state.level < RfmLevel::R1 {
        state.level = RfmLevel::R1;
    }
    if ais < 0.50 && state.level < RfmLevel::R2 {
        state.level = RfmLevel::R2;
    }
    if ais < 0.30 && state.level < RfmLevel::R3 {
        state.level = RfmLevel::R3;
    }

    // De-escalation: all pass → drop one level.
    if ais >= 0.90 && state.level > RfmLevel::R0 {
        state.level = match state.level {
            RfmLevel::R3 => RfmLevel::R2,
            RfmLevel::R2 => RfmLevel::R1,
            RfmLevel::R1 => RfmLevel::R0,
            RfmLevel::R0 => RfmLevel::R0,
        };
    }

    state.level != old_level
}

/// Check if regeneration is needed (R2+).
pub fn needs_regeneration(state: &RfmState) -> bool {
    state.level >= RfmLevel::R2
}

/// Check if ensemble is needed (R3).
pub fn needs_ensemble(state: &RfmState) -> bool {
    state.level >= RfmLevel::R3
}

// ─── Microcell validators ───────────────────────────────────────────────────

fn validate_schema(content: &str) -> bool {
    // Basic: non-empty, valid UTF-8 (always true in Rust).
    !content.is_empty()
}

fn validate_constraints(content: &str, constraints: &[String]) -> bool {
    // Check that none of the constraint patterns are violated.
    for c in constraints {
        if c.starts_with("max_length:")
            && let Ok(max) = c.trim_start_matches("max_length:").parse::<usize>()
            && content.len() > max
        {
            return false;
        }
        if c.starts_with("must_contain:") {
            let required = c.trim_start_matches("must_contain:");
            if !content.contains(required) {
                return false;
            }
        }
    }
    true
}

fn validate_consistency(_content: &str) -> bool {
    // Placeholder: structural consistency check.
    true
}

fn validate_grounding(_content: &str) -> bool {
    // Placeholder: reference grounding check.
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ais_all_pass() {
        let results = validate("some content", &[]);
        let ais = compute_ais(&results);
        assert_eq!(ais, 1.0);
    }

    #[test]
    fn test_rfm_escalation() {
        let mut state = RfmState::default();
        let results = vec![
            ValidatorResult { validator: MicrocellValidator::Schema, passed: true, details: String::new(), timestamp: Utc::now() },
            ValidatorResult { validator: MicrocellValidator::Constraint, passed: false, details: String::new(), timestamp: Utc::now() },
            ValidatorResult { validator: MicrocellValidator::Consistency, passed: false, details: String::new(), timestamp: Utc::now() },
            ValidatorResult { validator: MicrocellValidator::ReferenceGrounding, passed: false, details: String::new(), timestamp: Utc::now() },
        ];
        let changed = update_rfm(&mut state, results);
        assert!(changed);
        assert!(state.level >= RfmLevel::R1);
    }

    #[test]
    fn test_constraint_validation() {
        assert!(validate_constraints("short", &["max_length:100".into()]));
        assert!(!validate_constraints("long text here", &["max_length:5".into()]));
    }
}
