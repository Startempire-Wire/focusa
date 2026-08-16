//! Session fan-out — the fast-forward primitive. A multiplier (2x, 4x,
//! 6x, 8x…) maps to N parallel workloop-bound silent sessions with
//! deterministic task division and budget allocation. Pure compiler — the
//! execution leg reuses the existing silent-session create/start routes
//! and the bg receipt/completion delivery (docs/168).

use serde::{Deserialize, Serialize};

pub const FANOUT_SCHEMA: &str = "focusa.session_fanout.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FanoutInput {
    pub work_items: Vec<String>,
    pub multiplier: u32,
    pub policy_max_turns_per_session: u32,
    pub policy_max_wall_clock_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionAllocation {
    pub session_index: u32,
    pub work_items: Vec<String>,
    pub max_turns: u32,
    pub max_wall_clock_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FanoutPlan {
    pub schema: String,
    pub multiplier: u32,
    pub session_count: u32,
    pub sessions: Vec<SessionAllocation>,
    pub total_budget_turns: u32,
    pub total_budget_wall_clock_ms: u64,
    pub join_spec: JoinSpec,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinSpec {
    /// Wait for every session to settle (all) or the first N (any_n).
    pub policy: String,
    pub settle_n: u32,
    pub settlement_route: String,
}

/// Deterministic fan-out: `multiplier` sessions divide the work items
/// round-robin; each session's budget derives from the policy budget
/// (per-session turns = policy turns; wall clock = policy wall clock —
/// the multiplier parallelizes, never stretches policy bounds).
pub fn compile_fanout(input: &FanoutInput) -> Result<FanoutPlan, String> {
    if input.multiplier == 0 {
        return Err("multiplier must be >= 1".to_string());
    }
    if input.work_items.is_empty() {
        return Err("at least one work item required".to_string());
    }
    let session_count = input.multiplier.min(input.work_items.len() as u32);
    let mut sessions: Vec<SessionAllocation> = (0..session_count)
        .map(|index| SessionAllocation {
            session_index: index,
            work_items: vec![],
            max_turns: input.policy_max_turns_per_session,
            max_wall_clock_ms: input.policy_max_wall_clock_ms,
        })
        .collect();
    // Round-robin division — deterministic for the same input.
    for (position, item) in input.work_items.iter().enumerate() {
        sessions[position % session_count as usize]
            .work_items
            .push(item.clone());
    }
    let total_budget_turns = sessions.iter().map(|s| s.max_turns).sum::<u32>();
    let total_budget_wall_clock_ms = sessions.iter().map(|s| s.max_wall_clock_ms).sum::<u64>();
    Ok(FanoutPlan {
        schema: FANOUT_SCHEMA.to_string(),
        multiplier: input.multiplier,
        session_count,
        sessions,
        total_budget_turns,
        total_budget_wall_clock_ms,
        join_spec: JoinSpec {
            policy: "all".to_string(),
            settle_n: session_count,
            settlement_route: "/v1/silent-sessions/wait".to_string(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(items: &[&str], multiplier: u32) -> FanoutInput {
        FanoutInput {
            work_items: items.iter().map(|s| s.to_string()).collect(),
            multiplier,
            policy_max_turns_per_session: 12,
            policy_max_wall_clock_ms: 1_800_000,
        }
    }

    #[test]
    fn multiplier_maps_to_session_count_and_deterministic_division() {
        let plan = compile_fanout(&input(&["a", "b", "c", "d", "e", "f", "g", "h"], 4)).unwrap();
        assert_eq!(plan.session_count, 4);
        assert_eq!(plan.sessions.len(), 4);
        // Round-robin: session 0 gets a, e; session 1 gets b, f; …
        assert_eq!(plan.sessions[0].work_items, vec!["a", "e"]);
        assert_eq!(plan.sessions[1].work_items, vec!["b", "f"]);
        let again = compile_fanout(&input(&["a", "b", "c", "d", "e", "f", "g", "h"], 4)).unwrap();
        assert_eq!(plan, again, "fan-out must be deterministic");
    }

    #[test]
    fn multiplier_never_exceeds_work_item_count() {
        let plan = compile_fanout(&input(&["a", "b"], 8)).unwrap();
        assert_eq!(plan.session_count, 2);
    }

    #[test]
    fn budgets_scale_with_sessions_without_stretching_policy() {
        let plan = compile_fanout(&input(&["a", "b", "c", "d"], 2)).unwrap();
        assert_eq!(plan.total_budget_turns, 24); // 2 sessions × 12
        assert_eq!(plan.total_budget_wall_clock_ms, 3_600_000);
        assert_eq!(plan.join_spec.policy, "all");
        assert_eq!(plan.join_spec.settle_n, 2);
    }

    #[test]
    fn zero_multiplier_and_empty_items_are_rejected() {
        assert!(compile_fanout(&input(&[], 2)).is_err());
        assert!(compile_fanout(&input(&["a"], 0)).is_err());
    }
}
