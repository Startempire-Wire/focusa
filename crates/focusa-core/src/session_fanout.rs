//! Session fan-out — the fast-forward primitive. A multiplier (2x, 4x,
//! 6x, 8x…) maps to N parallel workloop-bound silent sessions with
//! deterministic task division and budget allocation. Pure compiler — the
//! execution leg reuses the existing silent-session create/start routes
//! and the bg receipt/completion delivery (docs/168).

use serde::{Deserialize, Serialize};

use crate::callgraph::FrameKind;

pub const FANOUT_SCHEMA: &str = "focusa.session_fanout.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaneRole {
    /// Strong frontier model: plans, divides, adjudicates.
    Orchestrator,
    /// Weaker model: executes the assigned work items.
    Worker,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FanoutInput {
    pub work_items: Vec<String>,
    pub multiplier: u32,
    pub policy_max_turns_per_session: u32,
    pub policy_max_wall_clock_ms: u64,
    /// Capability refs required of the orchestrator lane (strong/frontier
    /// tier — the CallGraph routes the orchestrator frames against these).
    #[serde(default)]
    pub orchestrator_capability_refs: Vec<String>,
    /// Capability refs required of worker lanes (weaker/implementation
    /// tier — the CallGraph routes worker frames against these).
    #[serde(default)]
    pub worker_capability_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionAllocation {
    pub session_index: u32,
    pub role: LaneRole,
    /// The CallGraph frame kind this lane binds to (docs/155 §9) — the
    /// future CallGraph runtime dispatches each lane as exactly one
    /// FocusaCallFrame of this kind with the capability refs below.
    pub frame_kind: FrameKind,
    pub work_items: Vec<String>,
    pub max_turns: u32,
    pub max_wall_clock_ms: u64,
    pub capability_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FanoutPlan {
    pub schema: String,
    pub multiplier: u32,
    /// Total lanes = worker lanes + 1 dedicated orchestrator lane.
    pub session_count: u32,
    pub worker_lane_count: u32,
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
    let worker_lane_count = input.multiplier.min(input.work_items.len() as u32);
    let mut sessions: Vec<SessionAllocation> = Vec::new();
    // One dedicated ORCHESTRATOR lane: strong frontier model — plans,
    // divides, adjudicates; it holds no implementation work items.
    sessions.push(SessionAllocation {
        session_index: 0,
        role: LaneRole::Orchestrator,
        frame_kind: FrameKind::Agent,
        work_items: vec![],
        max_turns: input.policy_max_turns_per_session,
        max_wall_clock_ms: input.policy_max_wall_clock_ms,
        capability_refs: input.orchestrator_capability_refs.clone(),
    });
    // Worker lanes: weaker implementation models, round-robin division.
    for index in 0..worker_lane_count {
        sessions.push(SessionAllocation {
            session_index: index + 1,
            role: LaneRole::Worker,
            frame_kind: FrameKind::Tool,
            work_items: vec![],
            max_turns: input.policy_max_turns_per_session,
            max_wall_clock_ms: input.policy_max_wall_clock_ms,
            capability_refs: input.worker_capability_refs.clone(),
        });
    }
    for (position, item) in input.work_items.iter().enumerate() {
        sessions[1 + (position % worker_lane_count as usize)]
            .work_items
            .push(item.clone());
    }
    let total_budget_turns = sessions.iter().map(|s| s.max_turns).sum::<u32>();
    let total_budget_wall_clock_ms = sessions.iter().map(|s| s.max_wall_clock_ms).sum::<u64>();
    let session_count = sessions.len() as u32;
    Ok(FanoutPlan {
        schema: FANOUT_SCHEMA.to_string(),
        multiplier: input.multiplier,
        session_count,
        worker_lane_count,
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
            orchestrator_capability_refs: vec!["orchestration".to_string()],
            worker_capability_refs: vec!["implementation".to_string()],
        }
    }

    #[test]
    fn multiplier_maps_to_worker_lanes_plus_orchestrator() {
        let plan = compile_fanout(&input(&["a", "b", "c", "d", "e", "f", "g", "h"], 4)).unwrap();
        assert_eq!(plan.worker_lane_count, 4);
        assert_eq!(plan.session_count, 5); // 1 orchestrator + 4 workers
        assert_eq!(plan.sessions[0].role, LaneRole::Orchestrator);
        assert_eq!(plan.sessions[0].frame_kind, FrameKind::Agent);
        assert!(
            plan.sessions[0].work_items.is_empty(),
            "orchestrator holds no implementation items"
        );
        // Round-robin workers: lane 1 gets a, e; lane 2 gets b, f; …
        assert_eq!(plan.sessions[1].work_items, vec!["a", "e"]);
        assert_eq!(plan.sessions[2].work_items, vec!["b", "f"]);
        assert_eq!(plan.sessions[1].frame_kind, FrameKind::Tool);
        let again = compile_fanout(&input(&["a", "b", "c", "d", "e", "f", "g", "h"], 4)).unwrap();
        assert_eq!(plan, again, "fan-out must be deterministic");
    }

    #[test]
    fn multiplier_never_exceeds_work_item_count() {
        let plan = compile_fanout(&input(&["a", "b"], 8)).unwrap();
        assert_eq!(plan.worker_lane_count, 2);
        assert_eq!(plan.session_count, 3);
    }

    #[test]
    fn budgets_scale_with_sessions_without_stretching_policy() {
        let plan = compile_fanout(&input(&["a", "b", "c", "d"], 2)).unwrap();
        assert_eq!(plan.total_budget_turns, 36); // (1 orchestrator + 2 workers) × 12
        assert_eq!(plan.total_budget_wall_clock_ms, 5_400_000);
        assert_eq!(plan.join_spec.policy, "all");
        assert_eq!(plan.join_spec.settle_n, 3);
    }

    #[test]
    fn zero_multiplier_and_empty_items_are_rejected() {
        assert!(compile_fanout(&input(&[], 2)).is_err());
        assert!(compile_fanout(&input(&["a"], 0)).is_err());
    }
}
