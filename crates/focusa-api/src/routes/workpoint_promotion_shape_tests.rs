// Offline publication-boundary cases; transaction checks reuse the temporary SQLite fixture.
// No live scope or malformed live record is created.
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
    assert_eq!(
        body["details"]["tool_result_v1"]["retry"]["posture"],
        "do_not_retry_unchanged"
    );
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

fn promotion_shape_batch(record: WorkpointRecord) -> Vec<FocusaEvent> {
    let (promotion, _) = promotion_shape_state(record.clone());
    vec![
        FocusaEvent::WorkpointCheckpointProposed { workpoint: record },
        promotion,
    ]
}

async fn seed_complete_promotion_shape_record(state: &Arc<AppState>) -> Uuid {
    let mut prior = promotion_shape_record();
    prior.workpoint_id = Uuid::now_v7();
    let prior_id = prior.workpoint_id;
    let saved = materialize_workpoint_events(
        ScopeContext::default(),
        state,
        promotion_shape_batch(prior),
        "promotion-shape-offline-seed",
    )
    .await
    .unwrap();
    assert_eq!(saved.workpoint.active_workpoint_id, Some(prior_id));
    prior_id
}

#[tokio::test]
async fn incomplete_promotion_batch_preserves_prior_memory_durability_and_notifications() {
    // Only the existing offline fixture is used. No daemon, live project or
    // customer record participates in these transaction-boundary cases.
    for case in [
        "absent_action",
        "blank_action",
        "absent_stage",
        "unknown_stage",
        "absent_next_slice",
        "blank_next_slice",
    ] {
        let (state, persistence) = crate::routes::turn::tests::setup_state(None).await;
        let prior_id = seed_complete_promotion_shape_record(&state).await;
        let before = serde_json::to_value(state.focusa.read().await.clone()).unwrap();
        let saved_before =
            serde_json::to_value(persistence.load_state().unwrap().unwrap()).unwrap();
        let events_before = persistence.event_count().unwrap();
        let epoch_before = state
            .external_mutation_epoch
            .load(std::sync::atomic::Ordering::SeqCst);
        let fence_before = state
            .next_writer_fencing_token
            .load(std::sync::atomic::Ordering::SeqCst);
        let mut notifications = state.events_tx.subscribe();
        let mut candidate = promotion_shape_record();
        candidate.workpoint_id = Uuid::now_v7();
        let candidate_id = candidate.workpoint_id;
        let expected_field = match case {
            "absent_action" => {
                candidate.action_intent = None;
                "action_intent.action_type"
            }
            "blank_action" => {
                candidate.action_intent.as_mut().unwrap().action_type = " \t\n".into();
                "action_intent.action_type"
            }
            "absent_stage" => {
                candidate.action_intent = Some(
                    serde_json::from_value(json!({
                        "action_type": "inspect_existing_artifact"
                    }))
                    .unwrap(),
                );
                "action_intent.lifecycle_stage"
            }
            "unknown_stage" => {
                candidate.action_intent.as_mut().unwrap().lifecycle_stage =
                    WorkpointLifecycleStage::Unknown;
                "action_intent.lifecycle_stage"
            }
            "absent_next_slice" => {
                candidate.next_slice = None;
                "next_slice"
            }
            "blank_next_slice" => {
                candidate.next_slice = Some(" \n\t".into());
                "next_slice"
            }
            _ => unreachable!(),
        };
        let (status, Json(body)) = materialize_workpoint_events(
            ScopeContext::default(),
            &state,
            promotion_shape_batch(candidate),
            "promotion-shape-offline-rejected",
        )
        .await
        .unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST, "{case}: {body}");
        assert_eq!(body["code"], "WORKPOINT_PROMOTION_INCOMPLETE", "{case}");
        assert_eq!(body["failure_class"], "validation_rejected", "{case}");
        assert_eq!(
            body["details"]["tool_result_v1"]["retry"]["posture"], "do_not_retry_unchanged",
            "{case}"
        );
        assert!(
            body["missing_fields"]
                .as_array()
                .unwrap()
                .iter()
                .any(|field| field == expected_field),
            "{case}: {body}"
        );
        let memory_after = state.focusa.read().await.clone();
        let saved_after = persistence.load_state().unwrap().unwrap();
        assert_eq!(
            serde_json::to_value(&memory_after).unwrap(),
            before,
            "{case}: memory changed"
        );
        assert_eq!(
            serde_json::to_value(&saved_after).unwrap(),
            saved_before,
            "{case}: saved checkpoint changed"
        );
        assert_eq!(
            memory_after.workpoint.active_workpoint_id,
            Some(prior_id),
            "{case}"
        );
        assert_eq!(
            saved_after.workpoint.active_workpoint_id,
            Some(prior_id),
            "{case}"
        );
        assert!(
            !saved_after
                .workpoint
                .records
                .iter()
                .any(|record| record.workpoint_id == candidate_id),
            "{case}: rejected proposal persisted"
        );
        assert_eq!(
            persistence.event_count().unwrap(),
            events_before,
            "{case}: events changed"
        );
        assert_eq!(
            state
                .external_mutation_epoch
                .load(std::sync::atomic::Ordering::SeqCst),
            epoch_before,
            "{case}: mutation epoch changed"
        );
        assert_eq!(
            state
                .next_writer_fencing_token
                .load(std::sync::atomic::Ordering::SeqCst),
            fence_before,
            "{case}: fencing counter changed"
        );
        assert!(
            state.writer_claims.read().await.is_empty(),
            "{case}: writer claim created"
        );
        assert!(
            matches!(
                notifications.try_recv(),
                Err(tokio::sync::broadcast::error::TryRecvError::Empty)
            ),
            "{case}: notification published or receiver failed"
        );
        assert!(
            state.write_serial_lock.try_lock().is_ok(),
            "{case}: serial lock retained"
        );
    }
}

#[test]
fn blank_lifecycle_stage_is_rejected_by_the_existing_request_decoder() {
    let request = serde_json::from_value::<WorkpointCheckpointRequest>(json!({
        "mission": "Inspect the existing artifact",
        "next_slice": "Capture its verification result",
        "action_intent": {"action_type": "inspect_existing_artifact", "lifecycle_stage": ""}
    }));
    assert!(
        request.is_err(),
        "blank enum stage must not reach publication"
    );
}

#[tokio::test]
async fn complete_promotion_batch_persists_and_broadcasts_the_same_successor() {
    let (state, persistence) = crate::routes::turn::tests::setup_state(None).await;
    let prior_id = seed_complete_promotion_shape_record(&state).await;
    let events_before = persistence.event_count().unwrap();
    let epoch_before = state
        .external_mutation_epoch
        .load(std::sync::atomic::Ordering::SeqCst);
    let mut notifications = state.events_tx.subscribe();
    let mut candidate = promotion_shape_record();
    candidate.workpoint_id = Uuid::now_v7();
    let candidate_id = candidate.workpoint_id;
    let returned = materialize_workpoint_events(
        ScopeContext::default(),
        &state,
        promotion_shape_batch(candidate),
        "promotion-shape-offline-complete",
    )
    .await
    .unwrap();
    let memory = state.focusa.read().await.clone();
    let durable = persistence.load_state().unwrap().unwrap();
    assert_eq!(
        serde_json::to_value(&returned).unwrap(),
        serde_json::to_value(&memory).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&durable).unwrap(),
        serde_json::to_value(&memory).unwrap()
    );
    assert_eq!(durable.workpoint.active_workpoint_id, Some(candidate_id));
    assert_eq!(
        durable
            .workpoint
            .records
            .iter()
            .find(|record| record.workpoint_id == candidate_id)
            .unwrap()
            .status,
        WorkpointStatus::Active
    );
    assert_eq!(
        durable
            .workpoint
            .records
            .iter()
            .find(|record| record.workpoint_id == prior_id)
            .unwrap()
            .status,
        WorkpointStatus::Superseded
    );
    let events_after = persistence.event_count().unwrap();
    assert!(events_after >= events_before + 2);
    assert_eq!(
        state
            .external_mutation_epoch
            .load(std::sync::atomic::Ordering::SeqCst),
        epoch_before + 1
    );
    let mut published = Vec::new();
    loop {
        match notifications.try_recv() {
            Ok(message) => published.push(serde_json::from_str::<EventLogEntry>(&message).unwrap()),
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => break,
            Err(error) => panic!("publication notification read failed: {error}"),
        }
    }
    assert_eq!(published.len(), (events_after - events_before) as usize);
    assert!(published.iter().any(|entry| matches!(&entry.event, FocusaEvent::WorkpointCheckpointPromoted { workpoint_id, .. } if *workpoint_id == candidate_id)));
    assert!(state.write_serial_lock.try_lock().is_ok());
}
