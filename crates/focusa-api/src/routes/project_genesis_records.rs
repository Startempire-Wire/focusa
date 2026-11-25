//! Shared canonical record compiler for local and binding-qualified Genesis.

use super::{Path, ProjectGenesisRequest, ScopeRef, Utc, Value, json, stable_id, stable_uuid};
use focusa_core::types::{
    FocusaEvent, HltStatus, TrajectoryConfidence, TrajectoryDefinitionStatus,
    TrajectoryGoalProvenanceRecord, TrajectoryProjectionRecord, TrajectoryRootGoalStability,
    TrajectoryWaypointRecord, TrajectoryWaypointStatus, WorkpointActionIntentRecord,
    WorkpointCheckpointReason, WorkpointConfidence, WorkpointRecord, WorkpointStatus,
};

/// Compile the same canonical record types for local and remote Genesis.
pub(super) fn compile_genesis_records(
    projection_root: &Path,
    req: &ProjectGenesisRequest,
    packet: &Value,
    scope_ref: ScopeRef,
) -> (TrajectoryProjectionRecord, WorkpointRecord) {
    let trajectory_id = stable_id("trajectory", projection_root, &req.idempotency_key);
    let workpoint_id = stable_uuid(projection_root, &req.idempotency_key);
    let hlt = packet["hlt"].as_str().unwrap_or_default().to_string();
    let mlg = packet["mlg"].as_str().unwrap_or_default().to_string();
    let stg = packet["stg"].as_str().unwrap_or_default().to_string();
    let waypoint_titles = packet["waypoints"].as_array().cloned().unwrap_or_default();
    let first_task = packet["first_workpoint_candidate"].clone();
    let now = Utc::now();
    let trajectory = TrajectoryProjectionRecord {
        trajectory_id: trajectory_id.clone(),
        scope_ref: Some(scope_ref),
        project_root: Some(projection_root.to_string_lossy().into_owned()),
        continuity_id: Some(req.continuity_id.clone()),
        root_long_term_goal: hlt.clone(),
        long_term_goal: hlt.clone(),
        desired_end_state: req.desired_end_state.clone().unwrap_or_default(),
        mid_level_goal: Some(mlg.clone()),
        short_term_goal: Some(stg.clone()),
        waypoints: waypoint_titles
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                value.as_str().map(|title| TrajectoryWaypointRecord {
                    waypoint_id: format!("{trajectory_id}:waypoint:{}", index + 1),
                    title: title.to_string(),
                    desired_state_delta: title.to_string(),
                    status: if index == 0 {
                        TrajectoryWaypointStatus::Active
                    } else {
                        TrajectoryWaypointStatus::NotStarted
                    },
                    next_workpoint_candidate: if index == 0 {
                        first_task.clone()
                    } else {
                        Value::Null
                    },
                    ..TrajectoryWaypointRecord::default()
                })
            })
            .collect(),
        current_state: req.current_state.clone(),
        root_goal_stability: TrajectoryRootGoalStability::Stable,
        session_clarity_status: TrajectoryDefinitionStatus::Clear,
        gap_summary: Some(format!("{mlg} → {stg}")),
        active_waypoint_id: Some(format!("{trajectory_id}:waypoint:1")),
        active_workpoint_id: Some(workpoint_id),
        source_refs: json!({"specification_ref": req.specification_ref, "genesis_receipt": packet["bootstrap_receipt"]["receipt_id"]}),
        blockers: Vec::new(),
        open_questions: Vec::new(),
        definition_status: TrajectoryDefinitionStatus::Clear,
        hlt_status: HltStatus::CanonicalExplicit,
        confidence: TrajectoryConfidence::High,
        goal_provenance: vec![
            TrajectoryGoalProvenanceRecord {
                field: "hlt".into(),
                source: "project_genesis".into(),
                source_ref: req.specification_ref.clone(),
                inferred: false,
                confidence: TrajectoryConfidence::High,
            },
            TrajectoryGoalProvenanceRecord {
                field: "mlg_stg_waypoints".into(),
                source: "spec143_deliberate_inference".into(),
                source_ref: req.specification_ref.clone(),
                inferred: req.mid_level_goal.is_none(),
                confidence: TrajectoryConfidence::Medium,
            },
        ],
        canonical: true,
        created_at: Some(now),
        updated_at: Some(now),
        ..TrajectoryProjectionRecord::default()
    };
    let work_item_id = first_task
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string);
    let mission = first_task
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or(&stg)
        .to_string();
    let workpoint = WorkpointRecord {
        workpoint_id,
        work_item_id,
        continuity_id: Some(req.continuity_id.clone()),
        project_root: Some(projection_root.to_string_lossy().into_owned()),
        status: WorkpointStatus::Proposed,
        checkpoint_reason: WorkpointCheckpointReason::SessionStart,
        confidence: WorkpointConfidence::High,
        canonical: true,
        mission: Some(mission.clone()),
        action_intent: Some(WorkpointActionIntentRecord {
            action_type: "project_genesis_first_workpoint".into(),
            target_ref: req.specification_ref.clone(),
            verification_hooks: req.acceptance_criteria.clone(),
            status: Some("ready".into()),
            lifecycle_stage: focusa_core::types::WorkpointLifecycleStage::Prepare,
            ..WorkpointActionIntentRecord::default()
        }),
        next_slice: Some(mission),
        idempotency_key: Some(req.idempotency_key.clone()),
        ..WorkpointRecord::default()
    };
    (trajectory, workpoint)
}

pub(super) fn genesis_events(
    trajectory: TrajectoryProjectionRecord,
    workpoint: WorkpointRecord,
) -> Vec<FocusaEvent> {
    let workpoint_id = workpoint.workpoint_id;
    vec![
        FocusaEvent::TrajectoryGoalDefined { trajectory },
        FocusaEvent::WorkpointCheckpointProposed { workpoint },
        FocusaEvent::WorkpointCheckpointPromoted {
            workpoint_id,
            confidence: WorkpointConfidence::High,
            reason: "Project Genesis atomic first Workpoint".into(),
        },
    ]
}
