//! Direction Workbench operations — #291 slice 1: typed steering,
//! adjudication, and decision review bound to evidence refs and receipts.
//! The Workbench is a projection of the event ledger — never a second
//! store.

use serde::{Deserialize, Serialize};

pub const DIRECTION_OPERATION_SCHEMA: &str = "focusa.direction_operation.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum DirectionOperation {
    Steer {
        target_ref: String,
        direction: String,
        rationale: String,
        scope: String,
        evidence_ref: Option<String>,
    },
    Adjudicate {
        claim_ref: String,
        verdict: String,
        adjudicator_ref: String,
        overridden_atom: Option<String>,
        override_reason: Option<String>,
    },
    ReviewDecision {
        decision_ref: String,
        outcome: String,
        feedback: String,
    },
}

/// Verify an operation is typed and evidence-bound where required.
pub fn verify_operation(operation: &DirectionOperation) -> Result<(), String> {
    match operation {
        DirectionOperation::Steer {
            target_ref,
            direction,
            evidence_ref,
            ..
        } => {
            if target_ref.trim().is_empty() || direction.trim().is_empty() {
                return Err("steer requires target_ref + direction".to_string());
            }
            if evidence_ref.as_deref().unwrap_or("").trim().is_empty() {
                return Err(
                    "steer requires an evidence ref (no free-text-only steering)".to_string(),
                );
            }
        }
        DirectionOperation::Adjudicate {
            claim_ref,
            verdict,
            overridden_atom,
            override_reason,
            ..
        } => {
            if claim_ref.trim().is_empty() || verdict.trim().is_empty() {
                return Err("adjudicate requires claim_ref + verdict".to_string());
            }
            if overridden_atom.is_some() && override_reason.is_none() {
                return Err("adjudication overrides must name the reason".to_string());
            }
        }
        DirectionOperation::ReviewDecision {
            decision_ref,
            outcome,
            ..
        } => {
            if decision_ref.trim().is_empty() || outcome.trim().is_empty() {
                return Err("review requires decision_ref + outcome".to_string());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steer_requires_evidence() {
        let op = DirectionOperation::Steer {
            target_ref: "wp-1".to_string(),
            direction: "prioritize compaction".to_string(),
            rationale: "quota pressure".to_string(),
            scope: "workpoint".to_string(),
            evidence_ref: None,
        };
        assert!(verify_operation(&op).is_err());
        let op = DirectionOperation::Steer {
            target_ref: "wp-1".to_string(),
            direction: "prioritize compaction".to_string(),
            rationale: "quota pressure".to_string(),
            scope: "workpoint".to_string(),
            evidence_ref: Some("docs/evidence/compaction.md".to_string()),
        };
        assert_eq!(verify_operation(&op), Ok(()));
    }

    #[test]
    fn adjudication_override_needs_reason() {
        let op = DirectionOperation::Adjudicate {
            claim_ref: "c1".to_string(),
            verdict: "allow".to_string(),
            adjudicator_ref: "op-1".to_string(),
            overridden_atom: Some("plan-doc".to_string()),
            override_reason: None,
        };
        assert!(verify_operation(&op).is_err());
    }

    #[test]
    fn operations_roundtrip_with_tags() {
        let op = DirectionOperation::Steer {
            target_ref: "wp-1".to_string(),
            direction: "go".to_string(),
            rationale: "r".to_string(),
            scope: "workpoint".to_string(),
            evidence_ref: Some("e1".to_string()),
        };
        let json = serde_json::to_string(&op).unwrap();
        let parsed: DirectionOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, op);
        let value = serde_json::to_value(&parsed).unwrap();
        assert_eq!(value["operation"], "steer");
    }
}

/// Gap-D flywheel seam: a Steer operation re-ranks its target so the
/// operator's direction reshapes next-ready selection. Deterministic:
/// bump = steers targeting the item × 10_000 (stable, no ties with
/// normal priorities).
pub const STEER_PRIORITY_BUMP: i32 = 10_000;

/// Does this steer target the given provider item id? Exact match or
/// the id appears as a trailing path/ref suffix.
pub fn steer_targets_item(target_ref: &str, provider_item_id: &str) -> bool {
    let target = target_ref.trim().trim_matches('"');
    let item = provider_item_id.trim();
    if target.is_empty() || item.is_empty() {
        return false;
    }
    target == item
        || target.trim_end_matches('/').ends_with(&format!("/{item}"))
        || item.ends_with(target)
        || item
            .trim_end_matches('/')
            .starts_with(&format!("{target}/"))
        || target.starts_with(&format!("{item}/"))
}

/// Total re-rank bump for an item given the recorded steer operations.
pub fn steer_priority_bump(provider_item_id: &str, steers: &[DirectionOperation]) -> i32 {
    steers
        .iter()
        .filter(|op| {
            matches!(op, DirectionOperation::Steer { target_ref, .. } if steer_targets_item(target_ref, provider_item_id))
        })
        .count() as i32
        * STEER_PRIORITY_BUMP
}

#[cfg(test)]
mod steer_re_rank_tests {
    use super::*;

    fn steer(target: &str) -> DirectionOperation {
        DirectionOperation::Steer {
            target_ref: target.to_string(),
            direction: "raise".into(),
            rationale: "operator".into(),
            scope: "project".into(),
            evidence_ref: None,
        }
    }

    #[test]
    fn exact_and_suffix_targets_match() {
        assert!(steer_targets_item("focusa-252", "focusa-252"));
        assert!(steer_targets_item("beads/focusa-252", "focusa-252"));
        assert!(steer_targets_item("focusa-252", "focusa-252/sub-task"));
        assert!(!steer_targets_item("focusa-253", "focusa-252"));
    }

    #[test]
    fn bump_is_deterministic_and_scales_with_steer_count() {
        let steers = vec![
            steer("focusa-252"),
            steer("beads/focusa-252"),
            steer("other"),
        ];
        assert_eq!(
            steer_priority_bump("focusa-252", &steers),
            2 * STEER_PRIORITY_BUMP
        );
        assert_eq!(steer_priority_bump("focusa-999", &steers), 0);
        assert_eq!(steer_priority_bump("focusa-252", &[]), 0);
    }
}
