// Actual-handler transaction regression, using synthetic project data only.
#[tokio::test]
async fn trajectory_guard_respects_project_transaction_before_shared_marker_update() {
    let directory = tempfile::Builder::new()
        .prefix("focusa-marker-transaction-")
        .tempdir()
        .unwrap();
    let root = directory.path().canonicalize().unwrap();
    let root = root.to_string_lossy().to_string();
    let (state, persistence) = crate::routes::turn::tests::setup_state(None).await;
    let marker = directory.path().join(".focusa-project.json");
    std::fs::write(
        &marker,
        serde_json::to_vec(&json!({
            "schema":"focusa.project.v1", "project_id":"marker-transaction",
            "project_root":root
        }))
        .unwrap(),
    )
    .unwrap();
    for continuity in ["transaction-a", "transaction-b"] {
        let (mut goal, mut events) =
            workstream_marker_fixture(&format!("trajectory:{continuity}"), continuity);
        goal.project_root = Some(root.clone());
        goal.canonical = true;
        for event in &mut events {
            event.project_root = root.clone();
        }
        state.focusa.write().await.trajectory.records.push(goal);
        persistence
            .append_trajectory_ladder_events(&events)
            .unwrap();
    }
    let request = |continuity: &str| TrajectoryGuardRequest {
        action: Some("migrate".into()),
        project_root: Some(root.clone()),
        continuity_id: Some(continuity.into()),
        confirm: Some(true),
        ..Default::default()
    };
    let transaction =
        crate::routes::project_bootstrap_safety::BootstrapLock::acquire(directory.path()).unwrap();
    let before = std::fs::read(&marker).unwrap();
    let events_before = persistence
        .read_trajectory_ladder_events(&root, None, 500)
        .unwrap()
        .len();
    let (status, Json(reply)) = trajectory_guard(
        ScopeContext {
            project_root: Some(root.clone()),
            continuity_id: Some("transaction-a".into()),
            ..Default::default()
        },
        State(state.clone()),
        Json(request("transaction-a")),
    )
    .await;
    assert_eq!(
        status,
        axum::http::StatusCode::CONFLICT,
        "confirmed marker mutation bypassed the existing project transaction: {reply}"
    );
    assert_eq!(reply["code"], "PROJECT_TRANSACTION_ACTIVE");
    assert_eq!(std::fs::read(&marker).unwrap(), before);
    assert_eq!(
        persistence
            .read_trajectory_ladder_events(&root, None, 500)
            .unwrap()
            .len(),
        events_before
    );
    drop(transaction);
    for continuity in ["transaction-a", "transaction-b"] {
        let (status, Json(reply)) = trajectory_guard(
            ScopeContext {
                project_root: Some(root.clone()),
                continuity_id: Some(continuity.into()),
                ..Default::default()
            },
            State(state.clone()),
            Json(request(continuity)),
        )
        .await;
        assert!(status.is_success(), "{continuity}: {reply}");
    }
    let marker: Value = serde_json::from_slice(&std::fs::read(marker).unwrap()).unwrap();
    let entries = marker["workstream_trajectory_bindings"]["entries"]
        .as_object()
        .unwrap();
    assert_eq!(entries.len(), 2);
    for continuity in ["transaction-a", "transaction-b"] {
        assert_eq!(entries[continuity]["continuity_id"], continuity);
    }
    let _read_transaction =
        crate::routes::project_bootstrap_safety::BootstrapLock::acquire(directory.path()).unwrap();
    for continuity in ["transaction-a", "transaction-b"] {
        let mut verify = request(continuity);
        verify.action = Some("verify".into());
        let (status, Json(reply)) = trajectory_guard(
            ScopeContext {
                project_root: Some(root.clone()),
                continuity_id: Some(continuity.into()),
                ..Default::default()
            },
            State(state.clone()),
            Json(verify),
        )
        .await;
        assert!(
            status.is_success(),
            "read-only verification was locked: {reply}"
        );
    }
    for continuity in ["transaction-a", "transaction-b"] {
        for action in ["verify", "preview"] {
            let mut scoped = request(continuity);
            scoped.action = Some(action.into());
            scoped.continuity_id = None;
            let (status, Json(reply)) = trajectory_guard(
                ScopeContext {
                    project_root: Some(root.clone()),
                    continuity_id: Some(continuity.into()),
                    source: crate::scope::ScopeSource::Header,
                    ..Default::default()
                },
                State(state.clone()),
                Json(scoped),
            )
            .await;
            assert!(
                status.is_success(),
                "guard ignored current request workstream {continuity}/{action}: {reply}"
            );
            assert_eq!(
                reply["trajectory_binding"]["active_trajectory_id"],
                format!("trajectory:{continuity}"),
                "guard borrowed another workstream instead of the request scope: {reply}"
            );
            assert_eq!(reply["trajectory_integrity_guard"]["status"], "READY");
        }
    }
    let mut unqualified = request("transaction-a");
    unqualified.action = Some("verify".into());
    unqualified.continuity_id = None;
    let (status, Json(reply)) = trajectory_guard(
        ScopeContext::default(),
        State(state.clone()),
        Json(unqualified),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CONFLICT);
    assert_eq!(reply["code"], "WORKSTREAM_SCOPE_REQUIRED");
    assert_eq!(reply["canonical"], false);
}

#[tokio::test]
async fn trajectory_guard_read_only_and_unconfirmed_requests_leave_no_transaction_artifact() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let root = root.to_string_lossy().to_string();
    let (state, _) = crate::routes::turn::tests::setup_state(None).await;
    let marker = directory.path().join(".focusa-project.json");
    let before = serde_json::to_vec(&json!({
        "schema":"focusa.project.v1", "project_id":"read-only-transaction",
        "project_root":root
    }))
    .unwrap();
    std::fs::write(&marker, &before).unwrap();
    for (action, confirm) in [
        ("verify", Some(true)),
        ("preview", Some(true)),
        ("migrate", None),
        ("repair", Some(false)),
    ] {
        let (status, _) = trajectory_guard(
            ScopeContext::default(),
            State(state.clone()),
            Json(TrajectoryGuardRequest {
                action: Some(action.into()),
                project_root: Some(root.clone()),
                continuity_id: Some("read-only-fixture".into()),
                confirm,
                ..Default::default()
            }),
        )
        .await;
        if matches!(action, "migrate" | "repair") {
            assert_eq!(status, axum::http::StatusCode::PRECONDITION_REQUIRED);
        }
        assert_eq!(std::fs::read(&marker).unwrap(), before);
        assert!(!directory.path().join(".focusa-bootstrap.lock").exists());
    }
}
