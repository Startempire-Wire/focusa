// In-memory publication-boundary cases only; no live scope or malformed record is created.
fn promotion_shape_record() -> WorkpointRecord {
    WorkpointRecord {
        canonical: true,
        status: WorkpointStatus::Proposed,
        mission: Some("private-promotion-shape-sentinel".into()),
        action_intent: Some(WorkpointActionIntentRecord {
            action_type: "inspect_existing_artifact".into(),
            lifecycle_stage: WorkpointLifecycleStage::Implement,
            ..Default::default()
        }),
        next_slice: Some(
            "Inspect the existing artifact and capture its verification result".into(),
        ),
        ..Default::default()
    }
}

fn promotion_shape_state(
    record: WorkpointRecord,
) -> (FocusaEvent, focusa_core::types::FocusaState) {
    let event = FocusaEvent::WorkpointCheckpointPromoted {
        workpoint_id: record.workpoint_id,
        confidence: WorkpointConfidence::Verified,
        reason: "publication shape test".into(),
    };
    let mut current = focusa_core::types::FocusaState::default();
    current.workpoint.records.push(record);
    (event, current)
}

#[test]
fn live_promotion_shape_accepts_complete_context_without_mutating_it() {
    let (event, current) = promotion_shape_state(promotion_shape_record());
    let before = serde_json::to_value(&current).unwrap();
    assert!(validate_live_workpoint_promotion(&event, &current).is_ok());
    assert_eq!(serde_json::to_value(&current).unwrap(), before);
}

#[test]
fn live_promotion_shape_reports_missing_fields_without_publishing_or_echoing_context() {
    let mut record = promotion_shape_record();
    record.action_intent = None;
    record.next_slice = None;
    let (event, current) = promotion_shape_state(record);
    let before = serde_json::to_value(&current).unwrap();
    let (status, Json(body)) = validate_live_workpoint_promotion(&event, &current).unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["canonical"], false);
    assert_eq!(body["code"], "WORKPOINT_PROMOTION_INCOMPLETE");
    assert_eq!(body["failed_operation"], "workpoint_checkpoint_promote");
    assert_eq!(body["failure_class"], "validation_rejected");
    assert_eq!(body["retry_posture"], "do_not_retry_unchanged");
    assert_eq!(
        body["missing_fields"],
        json!([
            "action_intent.action_type",
            "action_intent.lifecycle_stage",
            "next_slice"
        ])
    );
    assert!(
        !body
            .to_string()
            .contains("private-promotion-shape-sentinel")
    );
    assert_eq!(serde_json::to_value(&current).unwrap(), before);
}

#[test]
fn live_promotion_shape_rejects_blank_action_type() {
    for action_type in ["", " \t\n"] {
        let mut record = promotion_shape_record();
        record.action_intent.as_mut().unwrap().action_type = action_type.into();
        let (event, current) = promotion_shape_state(record);
        let (_, Json(body)) = validate_live_workpoint_promotion(&event, &current).unwrap_err();
        assert_eq!(body["missing_fields"], json!(["action_intent.action_type"]));
    }
}

#[test]
fn live_promotion_shape_rejects_unknown_lifecycle_stage() {
    let mut record = promotion_shape_record();
    record.action_intent.as_mut().unwrap().lifecycle_stage = WorkpointLifecycleStage::Unknown;
    let (event, current) = promotion_shape_state(record);
    let (_, Json(body)) = validate_live_workpoint_promotion(&event, &current).unwrap_err();
    assert_eq!(
        body["missing_fields"],
        json!(["action_intent.lifecycle_stage"])
    );
}

#[test]
fn live_promotion_shape_rejects_absent_or_blank_next_slice() {
    for next_slice in [None, Some(" \n\t".to_string())] {
        let mut record = promotion_shape_record();
        record.next_slice = next_slice;
        let (event, current) = promotion_shape_state(record);
        let (_, Json(body)) = validate_live_workpoint_promotion(&event, &current).unwrap_err();
        assert_eq!(body["missing_fields"], json!(["next_slice"]));
    }
}

#[test]
fn live_promotion_shape_leaves_partial_draft_proposals_available() {
    let mut record = promotion_shape_record();
    record.action_intent = None;
    record.next_slice = None;
    let event = FocusaEvent::WorkpointCheckpointProposed { workpoint: record };
    let current = focusa_core::types::FocusaState::default();
    assert!(validate_live_workpoint_promotion(&event, &current).is_ok());
    let proposed = reducer::reduce_with_meta(current, event, None, None, false).unwrap();
    assert!(proposed.new_state.workpoint.active_workpoint_id.is_none());
}

#[test]
fn live_promotion_shape_preserves_reducer_not_found_and_rejected_refusals() {
    let (event, _) = promotion_shape_state(promotion_shape_record());
    let empty = focusa_core::types::FocusaState::default();
    let (status, Json(body)) = validate_live_workpoint_promotion(&event, &empty).unwrap_err();
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["failure_class"], "not_found");
    assert!(reducer::reduce_with_meta(empty, event, None, None, false).is_err());
    for (canonical, status) in [
        (false, WorkpointStatus::Proposed),
        (true, WorkpointStatus::Rejected),
        (true, WorkpointStatus::DegradedFallback),
    ] {
        let mut record = promotion_shape_record();
        record.canonical = canonical;
        record.status = status;
        let (event, current) = promotion_shape_state(record);
        assert!(validate_live_workpoint_promotion(&event, &current).is_ok());
        assert!(reducer::reduce_with_meta(current, event, None, None, false).is_err());
    }
}

#[test]
fn historical_reducer_retains_legacy_incomplete_record_compatibility() {
    let mut record = promotion_shape_record();
    record.action_intent = None;
    record.next_slice = None;
    let (promotion, current) = promotion_shape_state(record);
    // Live publication refuses the incomplete candidate, while historical reducer
    // semantics remain untouched; this is not a live malformed-record write.
    assert!(validate_live_workpoint_promotion(&promotion, &current).is_err());
    let restored = reducer::reduce_with_meta(current, promotion, None, None, false).unwrap();
    assert!(restored.new_state.workpoint.active_workpoint_id.is_some());
}
