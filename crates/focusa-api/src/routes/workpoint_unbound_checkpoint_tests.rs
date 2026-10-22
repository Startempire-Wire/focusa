// Actual checkpoint consumer regression using existing offline API fixtures.

#[tokio::test]
async fn unbound_saved_predecessor_does_not_block_fresh_prepare_checkpoint() {
    for scenario in [
        "first",
        "saved",
        "nonprepare",
        "old_id",
        "missing_intent",
        "target_outside",
        "old_proof",
        "goal_unknown",
        "scope_foreign",
        "marker_tampered",
        "writer_live",
        "writer_expired",
        "writer_foreign",
        "bound_unknown",
        "missing_mission",
        "missing_slice",
        "old_frame",
        "writer_race",
        "context_race",
    ] {
        let with_saved = scenario != "first";
        let should_succeed = matches!(
            scenario,
            "first" | "saved" | "writer_expired" | "writer_foreign"
        );
        let root = tempfile::Builder::new()
            .prefix("focusa-checkpoint-fixture-")
            .tempdir()
            .unwrap();
        let project_root = root
            .path()
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let continuity = "fixture-checkpoint-continuity";
        let (state, persistence) = crate::routes::turn::tests::setup_state(None).await;
        let (mut trajectory, mut events) = crate::routes::project::tests::workstream_marker_fixture(
            "trajectory:checkpoint-fixture",
            continuity,
        );
        trajectory.project_root = Some(project_root.clone());
        trajectory.canonical = true;
        for event in &mut events {
            event.project_root = project_root.clone();
        }
        let goal = trajectory.long_term_goal.clone();
        state
            .focusa
            .write()
            .await
            .trajectory
            .records
            .push(trajectory);
        persistence
            .append_trajectory_ladder_events(&events)
            .unwrap();
        std::fs::write(root.path().join(".focusa-project.json"), serde_json::to_vec(&json!({
            "schema":"focusa.project.v1", "project_id":"checkpoint-fixture", "project_root":project_root
        })).unwrap()).unwrap();
        let scope = ScopeContext {
            project_root: Some(project_root.clone()),
            continuity_id: Some(continuity.into()),
            ..Default::default()
        };
        let (status, Json(reply)) = crate::routes::project::trajectory_guard(
            scope.clone(),
            State(state.clone()),
            Json(crate::routes::project::TrajectoryGuardRequest {
                action: Some("migrate".into()),
                project_root: Some(project_root.clone()),
                continuity_id: Some(continuity.into()),
                confirm: Some(true),
                ..Default::default()
            }),
        )
        .await;
        assert!(
            status.is_success(),
            "synthetic goal fixture migration failed: {reply}"
        );
        assert_eq!(
            reply
                .pointer("/trajectory_integrity_guard/status")
                .and_then(Value::as_str),
            Some("READY")
        );
        let old = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            canonical: true,
            status: WorkpointStatus::Active,
            project_root: Some(project_root.clone()),
            continuity_id: Some(continuity.into()),
            work_item_id: Some("fixture-old-task".into()),
            mission: Some("Historical scoped work retained without adoption".into()),
            verification_records: vec![WorkpointVerificationRecord {
                target_ref: project_root.clone(),
                result: "synthetic historical evidence".into(),
                evidence_ref: Some("fixture:historical-proof".into()),
                ..Default::default()
            }],
            action_intent: Some(WorkpointActionIntentRecord {
                action_type: "historical_source_action".into(),
                target_ref: Some(project_root.clone()),
                lifecycle_stage: WorkpointLifecycleStage::Prepare,
                ..Default::default()
            }),
            ..Default::default()
        };
        if with_saved {
            let mut focusa = state.focusa.write().await;
            focusa.workpoint.active_workpoint_id = Some(old.workpoint_id);
            focusa.workpoint.records.push(old.clone());
        }
        let mut request = WorkpointCheckpointRequest {
            project_root: Some(project_root.clone()),
            continuity_id: Some(continuity.into()),
            session_id: Some(Uuid::now_v7().to_string()),
            work_item_id: Some("fixture-current-task".into()),
            checkpoint_reason: Some("operator_checkpoint".into()),
            mission: Some(format!("Prepare current source recovery in {project_root}")),
            next_slice: Some(format!("Inspect the current source in {project_root}")),
            source_turn_id: Some("fixture:explicit-current-intent".into()),
            action_intent: Some(WorkpointActionIntentRecord {
                action_type: "prepare_current_source".into(),
                target_ref: Some(project_root.clone()),
                lifecycle_stage: WorkpointLifecycleStage::Prepare,
                ..Default::default()
            }),
            canonical: Some(true),
            promote: Some(true),
            idempotency_key: Some(format!("fixture-current-{scenario}")),
            ..Default::default()
        };
        let mut foreign = old.clone();
        foreign.workpoint_id = Uuid::now_v7();
        foreign.continuity_id = Some("fixture-foreign-continuity".into());
        {
            let mut focusa = state.focusa.write().await;
            focusa.workpoint.records.push(foreign.clone());
            focusa.workpoint.active_workpoint_id = Some(foreign.workpoint_id);
            let mut foreign_goal = focusa.trajectory.records[0].clone();
            foreign_goal.trajectory_id = "trajectory:foreign".into();
            foreign_goal.continuity_id = foreign.continuity_id.clone();
            foreign_goal.active_workpoint_id = Some(foreign.workpoint_id);
            focusa.trajectory.active_trajectory_id = Some(foreign_goal.trajectory_id.clone());
            focusa.trajectory.records.push(foreign_goal);
            if scenario == "goal_unknown" {
                focusa.trajectory.records[0].canonical = false;
            }
            if scenario == "bound_unknown" {
                focusa.trajectory.records[0].active_workpoint_id = Some(old.workpoint_id);
                focusa
                    .workpoint
                    .records
                    .iter_mut()
                    .find(|w| w.workpoint_id == old.workpoint_id)
                    .unwrap()
                    .action_intent
                    .as_mut()
                    .unwrap()
                    .lifecycle_stage = WorkpointLifecycleStage::Unknown;
            }
        }
        match scenario {
            "nonprepare" => {
                request.action_intent.as_mut().unwrap().lifecycle_stage =
                    WorkpointLifecycleStage::Implement
            }
            "old_id" => request.workpoint_id = Some(old.workpoint_id),
            "missing_intent" => request.source_turn_id = None,
            "missing_mission" => request.mission = None,
            "missing_slice" => request.next_slice = None,
            "old_frame" => request.frame_id = Some(old.workpoint_id),
            "target_outside" => {
                request.action_intent.as_mut().unwrap().target_ref =
                    Some("/tmp/foreign-source".into())
            }
            "old_proof" => request.verification_records = Some(old.verification_records.clone()),
            "scope_foreign" => request.continuity_id = Some("fixture-foreign-continuity".into()),
            "marker_tampered" => {
                let path = root.path().join(".focusa-project.json");
                let mut marker: Value =
                    serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                marker["workstream_trajectory_bindings"]["entries"][continuity]["trajectory_binding"]
                    ["active_trajectory_id"] = json!("trajectory:tampered");
                std::fs::write(path, serde_json::to_vec(&marker).unwrap()).unwrap();
            }
            _ => {}
        }
        if matches!(
            scenario,
            "writer_live" | "writer_expired" | "writer_foreign"
        ) {
            let key = crate::routes::work_loop::writer_claim_key_for_partition(
                &project_root,
                if scenario == "writer_foreign" {
                    "fixture-foreign-continuity"
                } else {
                    continuity
                },
                "fixture-writer-task",
            );
            let now = Utc::now();
            state.writer_claims.write().await.insert(
                key,
                crate::server::WriterLease {
                    writer_id: "fixture-other-writer".into(),
                    fencing_token: 7,
                    acquired_at: now,
                    renewed_at: now,
                    expires_at: now
                        + chrono::Duration::seconds(if scenario == "writer_expired" {
                            -1
                        } else {
                            300
                        }),
                },
            );
        }
        tokio::task::yield_now().await;
        let mut before = serde_json::to_value(&*state.focusa.read().await).unwrap();
        let mut claims_before = serde_json::to_value(&*state.writer_claims.read().await).unwrap();
        let fence_before = state
            .next_writer_fencing_token
            .load(std::sync::atomic::Ordering::SeqCst);
        let events_before = persistence.event_count().unwrap();
        let result = if matches!(scenario, "writer_race" | "context_race") {
            let serial = state.write_serial_lock.lock().await;
            let mut operation = Box::pin(checkpoint(
                scope,
                State(state.clone()),
                HeaderMap::new(),
                Json(request),
            ));
            std::future::poll_fn(
                |cx| match std::future::Future::poll(operation.as_mut(), cx) {
                    std::task::Poll::Pending => std::task::Poll::Ready(()),
                    std::task::Poll::Ready(value) => {
                        panic!("checkpoint escaped transaction fence: {value:?}")
                    }
                },
            )
            .await;
            if scenario == "writer_race" {
                let key = crate::routes::work_loop::writer_claim_key_for_partition(
                    &project_root,
                    continuity,
                    "fixture-racing-writer",
                );
                let now = Utc::now();
                state.writer_claims.write().await.insert(
                    key,
                    crate::server::WriterLease {
                        writer_id: "fixture-racing-writer".into(),
                        fencing_token: 9,
                        acquired_at: now,
                        renewed_at: now,
                        expires_at: now + chrono::Duration::minutes(5),
                    },
                );
                claims_before = serde_json::to_value(&*state.writer_claims.read().await).unwrap();
            } else {
                state.focusa.write().await.trajectory.records[0].active_workpoint_id =
                    Some(old.workpoint_id);
                before = serde_json::to_value(&*state.focusa.read().await).unwrap();
            }
            drop(serial);
            operation.await
        } else {
            checkpoint(scope, State(state.clone()), HeaderMap::new(), Json(request)).await
        };
        assert_eq!(
            serde_json::to_value(&*state.writer_claims.read().await).unwrap(),
            claims_before
        );
        assert_eq!(
            state
                .next_writer_fencing_token
                .load(std::sync::atomic::Ordering::SeqCst),
            fence_before
        );
        if !should_succeed {
            assert!(
                result.is_err(),
                "inadmissible scenario accepted: {scenario}"
            );
            if matches!(scenario, "writer_live" | "writer_race") {
                assert_eq!(
                    result.unwrap_err().1.0["code"],
                    "WORKPOINT_PREPARE_WRITER_ACTIVE"
                );
            }
            assert_eq!(
                serde_json::to_value(&*state.focusa.read().await).unwrap(),
                before,
                "state changed: {scenario}"
            );
            assert_eq!(
                persistence.event_count().unwrap(),
                events_before,
                "events changed: {scenario}"
            );
            println!("checkpoint_safety_case={scenario}: refused_without_mutation");
            continue;
        }
        assert!(
            result.is_ok(),
            "fresh_prepare_checkpoint_unbound_saved_predecessor_denied scenario={scenario}: {result:?}"
        );
        let focusa = state.focusa.read().await;
        let current = focusa
            .trajectory
            .records
            .iter()
            .find(|record| record.project_root.as_deref() == Some(project_root.as_str()))
            .unwrap();
        assert_eq!(current.long_term_goal, goal);
        let new_id = current
            .active_workpoint_id
            .expect("actual checkpoint must bind the verified trajectory");
        if with_saved {
            assert_ne!(new_id, old.workpoint_id);
            let preserved = focusa
                .workpoint
                .records
                .iter()
                .find(|record| record.workpoint_id == old.workpoint_id)
                .unwrap();
            assert_eq!(preserved.mission, old.mission);
            assert_eq!(
                serde_json::to_value(&preserved.verification_records).unwrap(),
                serde_json::to_value(&old.verification_records).unwrap()
            );
            assert_eq!(
                serde_json::to_value(&preserved.action_intent).unwrap(),
                serde_json::to_value(&old.action_intent).unwrap()
            );
            assert_eq!(preserved.work_item_id, old.work_item_id);
        }
        let fresh = focusa
            .workpoint
            .records
            .iter()
            .find(|w| w.workpoint_id == new_id)
            .unwrap();
        assert_eq!(fresh.work_item_id.as_deref(), Some("fixture-current-task"));
        assert_ne!(fresh.mission, old.mission);
        assert!(
            fresh.verification_records.is_empty(),
            "historical proofs were adopted"
        );
        assert_eq!(
            serde_json::to_value(
                focusa
                    .workpoint
                    .records
                    .iter()
                    .find(|w| w.workpoint_id == foreign.workpoint_id)
                    .unwrap()
            )
            .unwrap(),
            serde_json::to_value(&foreign).unwrap()
        );
        let foreign_after = focusa
            .trajectory
            .records
            .iter()
            .find(|g| g.trajectory_id == "trajectory:foreign")
            .unwrap();
        assert_eq!(
            foreign_after.active_workpoint_id,
            Some(foreign.workpoint_id)
        );
        let saved = persistence
            .load_state()
            .unwrap()
            .expect("checkpoint must be durable");
        assert_eq!(
            saved.trajectory.records[0].active_workpoint_id,
            Some(new_id)
        );
        assert!(persistence.event_count().unwrap() > events_before);
        assert!(
            saved
                .workpoint
                .records
                .iter()
                .any(|w| w.workpoint_id == new_id && w.canonical)
        );
        println!("checkpoint_safety_case={scenario}: fresh_prepare_persisted_without_adoption");
    }
}
