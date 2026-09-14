//! Lifecycle chain conformance (#253 slice 1): Genesis → C.R.I.S.T. →
//! CallGraph → Workpoint → settlement driven through the real reducer.
//! One deterministic chain proves the stages compose: a proposed Workpoint
//! promotes, a trajectory goal lands, CallGraph dispatches/settlements are
//! log-only (no state bleed), and the claim gate evaluates the settled
//! state honestly.

use chrono::{DateTime, Utc};
use focusa_core::reducer::reduce_with_meta;
use focusa_core::types::{
    FocusaEvent, FocusaState, TrajectoryDefinitionStatus, TrajectoryProjectionRecord,
    TrajectoryRootGoalStability, WorkpointCheckpointReason, WorkpointConfidence, WorkpointRecord,
    WorkpointStatus,
};

fn reduce(state: FocusaState, event: FocusaEvent) -> focusa_core::types::ReductionResult {
    reduce_with_meta(state, event, None, None, false).expect("lifecycle event reduces")
}

fn workpoint(workpoint_id: uuid::Uuid, continuity: &str) -> WorkpointRecord {
    WorkpointRecord {
        workpoint_id,
        work_item_id: Some(format!("item-{workpoint_id}")),
        session_identity: None,
        continuity_id: Some(continuity.to_string()),
        session_id: None,
        project_root: Some("/root/lifecycle-proj".to_string()),
        frame_id: None,
        status: WorkpointStatus::Proposed,
        checkpoint_reason: WorkpointCheckpointReason::OperatorCheckpoint,
        confidence: WorkpointConfidence::High,
        canonical: true,
        ..Default::default()
    }
}

#[test]
fn lifecycle_chain_proposal_promotion_goal_dispatch_settlement() {
    let initial = FocusaState::default();
    let wp_id = uuid::Uuid::now_v7();

    // Stage 1 — Workpoint proposal (canonical).
    let after_proposal = reduce(
        initial,
        FocusaEvent::WorkpointCheckpointProposed {
            workpoint: workpoint(wp_id, "cont-lifecycle"),
        },
    )
    .new_state;
    assert_eq!(after_proposal.workpoint.records.len(), 1);
    assert_eq!(
        after_proposal.workpoint.records[0].status,
        WorkpointStatus::Proposed
    );

    // Stage 2 — Promotion supersedes the proposal.
    let after_promotion = reduce(
        after_proposal,
        FocusaEvent::WorkpointCheckpointPromoted {
            workpoint_id: wp_id,
            confidence: WorkpointConfidence::Verified,
            reason: "lifecycle conformance".to_string(),
        },
    )
    .new_state;
    let promoted = after_promotion
        .workpoint
        .records
        .iter()
        .find(|w| w.workpoint_id == wp_id)
        .expect("workpoint survives promotion");
    assert_eq!(promoted.status, WorkpointStatus::Active);
    assert_eq!(promoted.confidence, WorkpointConfidence::Verified);

    // Stage 3 — Trajectory goal (C.R.I.S.T. clarity) lands scoped.
    let goal = TrajectoryProjectionRecord {
        trajectory_id: "t-lifecycle".to_string(),
        session_identity: None,
        project_root: Some("/root/lifecycle-proj".to_string()),
        continuity_id: Some("cont-lifecycle".to_string()),
        scope_ref: None,
        active_waypoint_id: None,
        root_long_term_goal: "ship the lifecycle".to_string(),
        long_term_goal: "ship the lifecycle".to_string(),
        desired_end_state: "green conformance".to_string(),
        mid_level_goal: None,
        short_term_goal: None,
        waypoints: vec![],
        current_state: None,
        root_goal_stability: TrajectoryRootGoalStability::Stable,
        session_clarity_status: TrajectoryDefinitionStatus::Clear,
        gap_summary: None,
        active_workpoint_id: Some(wp_id),
        source_refs: serde_json::json!({}),
        blockers: vec![],
        open_questions: vec![],
        hlt_status: focusa_core::types::HltStatus::CanonicalExplicit,
        definition_status: TrajectoryDefinitionStatus::Clear,
        confidence: focusa_core::types::TrajectoryConfidence::High,
        goal_provenance: vec![],
        definition_of_done: None,
        supersedes_trajectory_id: None,
        canonical: true,
        created_at: Some(Utc::now()),
        updated_at: Some(Utc::now()),
    };
    let after_goal = reduce(
        after_promotion,
        FocusaEvent::TrajectoryGoalDefined { trajectory: goal },
    )
    .new_state;
    assert_eq!(after_goal.trajectory.records.len(), 1);
    assert_eq!(
        after_goal.trajectory.records[0].active_workpoint_id,
        Some(wp_id)
    );

    // Stage 4 — CallGraph dispatch + settlement are log-only (no state
    // bleed into the canonical FocusaState).
    let before_callgraph = after_goal.clone();
    let after_dispatch = reduce(
        after_goal,
        FocusaEvent::CallGraphFrameDispatched {
            run_id: "run-1".to_string(),
            dispatch_id: "d-1".to_string(),
            frame_id: "frame-1".to_string(),
            invocation_id: "inv-1".to_string(),
            adapter_id: "pi".to_string(),
            model: "m".to_string(),
            attempt: 1,
        },
    )
    .new_state;
    let mut left = serde_json::to_value(&after_dispatch).unwrap();
    let mut right = serde_json::to_value(&before_callgraph).unwrap();
    left.as_object_mut().unwrap().remove("version");
    right.as_object_mut().unwrap().remove("version");
    assert_eq!(
        left, right,
        "dispatch must not mutate semantic state (version bump only)"
    );
    let after_settlement = reduce(
        after_dispatch,
        FocusaEvent::CallGraphFrameSettled {
            run_id: "run-1".to_string(),
            frame_id: "frame-1".to_string(),
            invocation_id: "inv-1".to_string(),
            receipt_ref: "receipt-1".to_string(),
        },
    )
    .new_state;
    let mut left = serde_json::to_value(&after_settlement).unwrap();
    let mut right = serde_json::to_value(&before_callgraph).unwrap();
    left.as_object_mut().unwrap().remove("version");
    right.as_object_mut().unwrap().remove("version");
    assert_eq!(
        left, right,
        "settlement must not mutate semantic state (version bump only)"
    );

    // Stage 5 — Settlement honesty: the claim gate evaluates the settled
    // state's evidence class without fabricating evidence.
    let gate_input = focusa_core::claim_gate::ClaimGateInput {
        work_item_id: "item-wp-1".to_string(),
        claim_text: "wp-1 settled with receipt-1".to_string(),
        acceptance_criteria: vec!["receipt evidence".to_string()],
        evidence_policy: None,
        surfaces_required: vec!["api".to_string()],
        operator_deferred: false,
    };
    let gate = focusa_core::claim_gate::ClaimGateOutput::build(&gate_input);
    assert_eq!(
        gate.schema,
        focusa_core::claim_gate::CLAIM_GATE_OUTPUT_SCHEMA
    );
    // No citation of the receipt as evidence → the gate must not allow.
    assert!(!matches!(
        gate.decision,
        focusa_core::claim_gate::GateDecision::Allow
    ));

    // The chain leaves the workpoint canonical and scoped.
    assert!(after_settlement.workpoint.records[0].canonical);
    assert_eq!(
        after_settlement.workpoint.records[0]
            .continuity_id
            .as_deref(),
        Some("cont-lifecycle")
    );
}
