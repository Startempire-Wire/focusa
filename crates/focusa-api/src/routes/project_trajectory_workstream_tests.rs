// Focused two-workstream marker regression; synthetic records only.

fn workstream_marker_fixture(
    trajectory_id: &str,
    continuity_id: &str,
) -> (TrajectoryProjectionRecord, Vec<TrajectoryLadderEvent>) {
    let waypoint = focusa_core::types::TrajectoryWaypointRecord {
        waypoint_id: format!("{continuity_id}:waypoint"),
        title: "Verify independent mission".to_string(),
        desired_state_delta: "Independent scoped progress".to_string(),
        ..focusa_core::types::TrajectoryWaypointRecord::default()
    };
    let trajectory = TrajectoryProjectionRecord {
        trajectory_id: trajectory_id.to_string(),
        project_root: Some("/tmp/focusa-test".to_string()),
        continuity_id: Some(continuity_id.to_string()),
        long_term_goal: format!("Deliver independent mission for {continuity_id}"),
        mid_level_goal: Some(format!("Preserve isolated direction for {continuity_id}")),
        short_term_goal: Some(format!("Verify independent progress for {continuity_id}")),
        waypoints: vec![waypoint.clone()],
        ..TrajectoryProjectionRecord::default()
    };
    let ids: Vec<String> = ["hlt", "mlg", "stg", "waypoint"]
        .iter()
        .map(|level| format!("{continuity_id}:{level}:event"))
        .collect();
    let mut events = vec![
        guard_test_event(
            TrajectoryLadderLevel::Hlt,
            &ids[0],
            None,
            None,
            json!(trajectory.long_term_goal),
            1,
        ),
        guard_test_event(
            TrajectoryLadderLevel::Mlg,
            &ids[1],
            Some(&ids[0]),
            None,
            json!(trajectory.mid_level_goal),
            2,
        ),
        guard_test_event(
            TrajectoryLadderLevel::Stg,
            &ids[2],
            Some(&ids[1]),
            None,
            json!(trajectory.short_term_goal),
            3,
        ),
        guard_test_event(
            TrajectoryLadderLevel::Waypoint,
            &ids[3],
            Some(&ids[2]),
            Some(&waypoint.waypoint_id),
            json!(waypoint),
            4,
        ),
    ];
    for event in &mut events {
        event.trajectory_id = trajectory_id.to_string();
        event.continuity_id = Some(continuity_id.to_string());
    }
    (trajectory, events)
}

#[test]
fn workstream_scoped_marker_bindings_isolate_foreign_singleton() {
    let root = "/tmp/focusa-test";
    let (a, a_events) = workstream_marker_fixture("trajectory:A", "continuity:A");
    let (b, b_events) = workstream_marker_fixture("trajectory:B", "continuity:B");
    let (a_binding, mut a_guard) = build_trajectory_guard(root, Some(&a), &a_events, &json!({}));
    let (b_binding, mut b_guard) = build_trajectory_guard(root, Some(&b), &b_events, &json!({}));
    for guard in [&a_guard, &b_guard] {
        assert!(guard.minimum_ladder_complete && guard.causal_chain_valid);
        assert!(guard.projection_matches_ledger && guard.no_placeholder_values);
        assert!(guard.ladder_links_complete && guard.unresolved_conflicts.is_empty());
    }
    a_guard.status = TrajectoryIntegrityStatus::Ready;
    b_guard.status = TrajectoryIntegrityStatus::Ready;
    let marker = json!({
        "schema": "focusa.project.v2",
        "project_id": "synthetic-project",
        "project_root": root,
        "genesis_binding": {"continuity_id": "continuity:B"},
        "trajectory_binding": b_binding,
        "trajectory_integrity_guard": b_guard,
        "retained_extension": {"unchanged": true},
        "workstream_trajectory_bindings": {
            "schema": "focusa.project_workstream_bindings.v1",
            "entries": {
                "continuity:A": {
                    "project_root": root,
                    "continuity_id": "continuity:A",
                    "trajectory_binding": a_binding,
                    "trajectory_integrity_guard": a_guard
                }
            }
        }
    });
    let unchanged = marker.clone();
    for _ in 0..3 {
        let (actual_a, verified_a) = build_trajectory_guard(root, Some(&a), &a_events, &marker);
        assert_eq!(
            verified_a.status,
            TrajectoryIntegrityStatus::Ready,
            "a verified workstream binding must not borrow the foreign project singleton"
        );
        assert_eq!(actual_a.unwrap().active_trajectory_id, a.trajectory_id);
        let (actual_b, verified_b) = build_trajectory_guard(root, Some(&b), &b_events, &marker);
        assert_eq!(verified_b.status, TrajectoryIntegrityStatus::Ready);
        assert_eq!(actual_b.unwrap().active_trajectory_id, b.trajectory_id);
    }
    assert_eq!(
        marker, unchanged,
        "verification must not rewrite either workstream"
    );
}

fn workstream_ready_marker_parts(
    trajectory: &TrajectoryProjectionRecord,
    events: &[TrajectoryLadderEvent],
) -> (
    ProjectTrajectoryBindingRecord,
    TrajectoryIntegrityGuardRecord,
) {
    let (binding, mut guard) =
        build_trajectory_guard("/tmp/focusa-test", Some(trajectory), events, &json!({}));
    assert!(
        guard.minimum_ladder_complete
            && guard.causal_chain_valid
            && guard.projection_matches_ledger
    );
    guard.status = TrajectoryIntegrityStatus::Ready;
    (binding.unwrap(), guard)
}

#[test]
fn workstream_marker_migration_preserves_foreign_legacy_and_extensions() {
    let root = "/tmp/focusa-test";
    let (a, a_events) = workstream_marker_fixture("trajectory:A", "continuity:A");
    let (b, b_events) = workstream_marker_fixture("trajectory:B", "continuity:B");
    let (a_binding, a_guard) = workstream_ready_marker_parts(&a, &a_events);
    let (b_binding, b_guard) = workstream_ready_marker_parts(&b, &b_events);
    let mut marker = json!({
        "schema": "focusa.project.v2", "project_id": "retained-project",
        "genesis_binding": {"continuity_id": "continuity:B"},
        "trajectory_binding": b_binding, "trajectory_integrity_guard": b_guard,
        "retained_extension": {"history": [1, 2, 3]}
    });
    let original = marker.clone();
    project_trajectory_marker::write_projection(
        &mut marker,
        root,
        "continuity:A",
        &a_binding,
        &a_guard,
    )
    .unwrap();
    for key in [
        "project_id",
        "genesis_binding",
        "trajectory_binding",
        "trajectory_integrity_guard",
        "retained_extension",
    ] {
        assert_eq!(
            marker[key], original[key],
            "foreign/shared metadata changed: {key}"
        );
    }
    project_trajectory_marker::write_projection(
        &mut marker,
        root,
        "continuity:B",
        &b_binding,
        &b_guard,
    )
    .unwrap();
    marker["workstream_trajectory_bindings"]["entries"]["continuity:A"]["retained_extension"] =
        json!({"keep": true});
    let migrated = marker.clone();
    for _ in 0..3 {
        project_trajectory_marker::write_projection(
            &mut marker,
            root,
            "continuity:A",
            &a_binding,
            &a_guard,
        )
        .unwrap();
        project_trajectory_marker::write_projection(
            &mut marker,
            root,
            "continuity:B",
            &b_binding,
            &b_guard,
        )
        .unwrap();
        for (trajectory, events, binding) in
            [(&a, &a_events, &a_binding), (&b, &b_events, &b_binding)]
        {
            let (actual, guard) = build_trajectory_guard(root, Some(trajectory), events, &marker);
            assert_eq!(guard.status, TrajectoryIntegrityStatus::Ready);
            assert_eq!(actual.unwrap().ledger_digest, binding.ledger_digest);
        }
        assert_eq!(
            marker, migrated,
            "repeated A/B projection writes must not churn metadata"
        );
    }
}

#[test]
fn workstream_marker_rejects_foreign_scope_and_unknown_schema_without_mutation() {
    let root = "/tmp/focusa-test";
    let (a, events) = workstream_marker_fixture("trajectory:A", "continuity:A");
    let (binding, guard) = workstream_ready_marker_parts(&a, &events);
    let mut valid = json!({"schema":"focusa.project.v1","project_id":"retained-project"});
    project_trajectory_marker::write_projection(&mut valid, root, "continuity:A", &binding, &guard)
        .unwrap();
    assert_eq!(
        valid["trajectory_binding"]["active_trajectory_id"],
        "trajectory:A"
    );
    for field in ["project_root", "continuity_id", "schema"] {
        let mut marker = valid.clone();
        if field == "schema" {
            marker["workstream_trajectory_bindings"]["schema"] = json!("unsupported.v999");
        } else {
            marker["workstream_trajectory_bindings"]["entries"]["continuity:A"][field] =
                json!("foreign");
        }
        let before = marker.clone();
        let (_, rejected) = build_trajectory_guard(root, Some(&a), &events, &marker);
        assert_eq!(
            rejected.status,
            TrajectoryIntegrityStatus::IntegrityRepairRequired
        );
        assert!(!rejected.schema_supported);
        assert!(
            project_trajectory_marker::write_projection(
                &mut marker,
                root,
                "continuity:A",
                &binding,
                &guard
            )
            .is_err()
        );
        assert_eq!(marker, before);
    }
    let mut unchanged = valid.clone();
    assert!(
        project_trajectory_marker::write_projection(&mut unchanged, root, "", &binding, &guard)
            .is_err()
    );
    assert_eq!(unchanged, valid);
}

#[test]
fn workstream_marker_tamper_cannot_fall_back_to_matching_legacy_binding() {
    let root = "/tmp/focusa-test";
    let (a, events) = workstream_marker_fixture("trajectory:A", "continuity:A");
    let (binding, guard) = workstream_ready_marker_parts(&a, &events);
    let mut marker = json!({"schema":"focusa.project.v1"});
    project_trajectory_marker::write_projection(
        &mut marker,
        root,
        "continuity:A",
        &binding,
        &guard,
    )
    .unwrap();
    marker["workstream_trajectory_bindings"]["entries"]["continuity:A"]["trajectory_binding"]["ledger_digest"] =
        json!("tampered");
    let (actual, rejected) = build_trajectory_guard(root, Some(&a), &events, &marker);
    assert_eq!(
        rejected.status,
        TrajectoryIntegrityStatus::MigrationRequired
    );
    assert_eq!(actual.unwrap().ledger_digest, binding.ledger_digest);
    let (missing, missing_guard) = build_trajectory_guard(root, None, &[], &marker);
    assert!(missing.is_none());
    assert_eq!(missing_guard.status, TrajectoryIntegrityStatus::HltImpasse);
}

#[test]
fn workstream_marker_preserves_legacy_on_shared_ids_and_partial_slots() {
    let root = "/tmp/focusa-test";
    let (a, a_events) = workstream_marker_fixture("trajectory:shared", "continuity:A");
    let (b, b_events) = workstream_marker_fixture("trajectory:shared", "continuity:B");
    let (a_binding, a_guard) = workstream_ready_marker_parts(&a, &a_events);
    let (b_binding, b_guard) = workstream_ready_marker_parts(&b, &b_events);
    for slot in ["both", "binding_only", "guard_only"] {
        let mut marker = json!({
            "schema":"focusa.project.v2", "project_id":"retained-project",
            "genesis_binding":{"continuity_id":"continuity:B"},
            "trajectory_binding":b_binding, "trajectory_integrity_guard":b_guard
        });
        if slot == "binding_only" {
            marker
                .as_object_mut()
                .unwrap()
                .remove("trajectory_integrity_guard");
        } else if slot == "guard_only" {
            marker.as_object_mut().unwrap().remove("trajectory_binding");
        }
        let before = marker.clone();
        project_trajectory_marker::write_projection(
            &mut marker,
            root,
            "continuity:A",
            &a_binding,
            &a_guard,
        )
        .unwrap();
        for field in [
            "project_id",
            "genesis_binding",
            "trajectory_binding",
            "trajectory_integrity_guard",
        ] {
            assert_eq!(
                marker.get(field),
                before.get(field),
                "unqualified metadata changed: {slot}/{field}"
            );
        }
        let (_, actual) = build_trajectory_guard(root, Some(&a), &a_events, &marker);
        assert_eq!(actual.status, TrajectoryIntegrityStatus::Ready);
        if slot == "both" {
            let (_, actual_b) = build_trajectory_guard(root, Some(&b), &b_events, &marker);
            assert_eq!(actual_b.status, TrajectoryIntegrityStatus::Ready);
        }
    }
}
