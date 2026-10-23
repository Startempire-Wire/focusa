fn write_genesis_status_fixture(root: &Path) -> (Value, Value) {
    let root_string = root.to_string_lossy().to_string();
    let mut req = complete_request(root);
    req.continuity_id = "status-b".into();
    let mut packet = build_staged_packet(root, &req, None);
    packet["status"] = json!("ready");
    packet["first_workpoint"] = json!({"workpoint_id":"status-b-workpoint", "canonical":true});
    packet["readiness_receipt"] = json!({"receipt_id":"status-b-receipt", "trajectory_id":"status-b-trajectory", "workpoint_id":"status-b-workpoint", "marker_guard":"verified"});
    let marker = json!({
        "schema":"focusa.project.v1", "project_id":"synthetic-genesis-status",
        "project_root":root_string,
        "genesis_binding":{
            "status":"ready", "continuity_id":"status-b",
            "workpoint_id":"status-b-workpoint", "receipt_id":"status-b-receipt",
            "trajectory_id":"status-b-trajectory"
        }
    });
    write_json_atomic(&root.join(".focusa-project.json"), &marker).unwrap();
    write_json_atomic(&packet_path(root), &packet).unwrap();
    (marker, packet)
}

#[tokio::test]
async fn status_qualifies_requested_workstream_without_relabeling_project_history() {
    let root = test_root("status-workstream-qualification")
        .canonicalize()
        .unwrap();
    let (marker, packet) = write_genesis_status_fixture(&root);
    let marker_before = std::fs::read(root.join(".focusa-project.json")).unwrap();
    let packet_before = std::fs::read(packet_path(&root)).unwrap();
    let (state, _) = crate::routes::turn::tests::setup_state(None).await;
    for (query_continuity, header_continuity, expected_match) in [
        (Some("status-a"), Some("status-a"), false),
        (Some("status-b"), Some("status-b"), true),
        (None, Some("status-b"), true),
        (Some("status-unknown"), Some("status-unknown"), false),
        (None, None, false),
        (Some("status-a"), Some("status-b"), false),
    ] {
        let (_, body) = read_genesis_status_fixture(
            state.clone(),
            &root,
            query_continuity,
            header_continuity,
            None,
        )
        .await;
        if expected_match {
            assert_eq!(body["canonical"], true, "matched status refused: {body}");
            assert_eq!(body["status"], "ready");
            assert_eq!(body["resolved_scope"]["continuity_id"], "status-b");
            assert_eq!(body["authority"]["continuity_id"], "status-b");
        } else {
            assert_ne!(
                body["status"], "ready",
                "Genesis status borrowed canonical ready from another or unqualified workstream: {body}"
            );
            assert_eq!(body["canonical"], false);
        }
        assert_eq!(
            std::fs::read(root.join(".focusa-project.json")).unwrap(),
            marker_before
        );
        assert_eq!(std::fs::read(packet_path(&root)).unwrap(), packet_before);
    }
    for session in ["status-session-a", "status-session-b"] {
        let (_, body) = read_genesis_status_fixture(
            state.clone(),
            &root,
            Some("status-b"),
            Some("status-b"),
            Some(session),
        )
        .await;
        assert_eq!(body["canonical"], true);
        assert_eq!(body["requested_scope"]["session_id"], session);
        assert!(body["resolved_scope"]["session_id"].is_null());
        assert_eq!(body["scope_validation"]["execution_admission"], false);
    }
    for variant in [
        "stale-marker",
        "missing-continuity",
        "ambiguous-root",
        "unsupported-schema",
        "uncommitted-marker",
    ] {
        let mut changed_packet = packet.clone();
        let mut changed_marker = marker.clone();
        match variant {
            "stale-marker" => {
                changed_marker["genesis_binding"]["receipt_id"] = json!("other-receipt")
            }
            "missing-continuity" => changed_packet["authority"]["continuity_id"] = Value::Null,
            "ambiguous-root" => {
                changed_packet["project_identity"]["project_root"] = json!("/foreign")
            }
            "unsupported-schema" => changed_packet["schema"] = json!("unsupported"),
            _ => changed_marker["genesis_binding"]["status"] = json!("staged"),
        }
        write_json_atomic(&root.join(".focusa-project.json"), &changed_marker).unwrap();
        write_json_atomic(&packet_path(&root), &changed_packet).unwrap();
        let current_marker = std::fs::read(root.join(".focusa-project.json")).unwrap();
        let current_packet = std::fs::read(packet_path(&root)).unwrap();
        let (_, body) = read_genesis_status_fixture(
            state.clone(),
            &root,
            Some("status-b"),
            Some("status-b"),
            None,
        )
        .await;
        assert_eq!(body["canonical"], false, "{variant}: {body}");
        assert_ne!(body["status"], "ready", "{variant}: {body}");
        assert_eq!(
            std::fs::read(root.join(".focusa-project.json")).unwrap(),
            current_marker
        );
        assert_eq!(std::fs::read(packet_path(&root)).unwrap(), current_packet);
    }
}

async fn read_genesis_status_fixture(
    state: Arc<AppState>,
    root: &Path,
    query_continuity: Option<&str>,
    header_continuity: Option<&str>,
    session: Option<&str>,
) -> (StatusCode, Value) {
    use tower::ServiceExt;
    let mut uri = reqwest::Url::parse("http://fixture/v1/project/genesis/status").unwrap();
    uri.query_pairs_mut()
        .append_pair("project_root", root.to_string_lossy().as_ref());
    if let Some(continuity) = query_continuity {
        uri.query_pairs_mut()
            .append_pair("continuity_id", continuity);
    }
    let mut request = axum::http::Request::builder().uri(uri.as_str());
    if let Some(continuity) = header_continuity {
        request = request
            .header("x-scope-project-root", root.to_string_lossy().as_ref())
            .header("x-scope-continuity-id", continuity);
    }
    if let Some(session) = session {
        request = request.header("x-scope-session-id", session);
    }
    let response = router()
        .with_state(state)
        .oneshot(request.body(axum::body::Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn status_uses_existing_worktree_normalization_without_copying_session_authority() {
    let fixture = tempfile::tempdir().unwrap();
    let primary = fixture.path().join("primary");
    let worktree_a = fixture.path().join("worktree-a");
    let worktree_b = fixture.path().join("worktree-b");
    fs::create_dir(&primary).unwrap();
    let primary = fs::canonicalize(primary).unwrap();
    write_genesis_status_fixture(&primary);
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(&primary)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git fixture: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["init", "--initial-branch=main"]);
    git(&["add", ".focusa-project.json"]);
    git(&[
        "-c",
        "user.name=GenesisFixture",
        "-c",
        "user.email=genesis-fixture@example.invalid",
        "commit",
        "-m",
        "fixture",
    ]);
    git(&["worktree", "add", "--detach", worktree_a.to_str().unwrap()]);
    git(&["worktree", "add", "--detach", worktree_b.to_str().unwrap()]);
    let worktree_a = fs::canonicalize(worktree_a).unwrap();
    let worktree_b = fs::canonicalize(worktree_b).unwrap();
    let marker_before = fs::read(primary.join(".focusa-project.json")).unwrap();
    let packet_before = fs::read(packet_path(&primary)).unwrap();
    let (state, _) = crate::routes::turn::tests::setup_state(None).await;
    let mut subpaths = Vec::new();
    for (root, session) in [
        (&worktree_a, "worktree-session-a"),
        (&worktree_b, "worktree-session-b"),
    ] {
        let (_, body) = read_genesis_status_fixture(
            state.clone(),
            root,
            Some("status-b"),
            Some("status-b"),
            Some(session),
        )
        .await;
        assert_eq!(body["canonical"], true, "{body}");
        assert_eq!(
            body["resolved_scope"]["project_root"],
            primary.to_string_lossy().as_ref()
        );
        assert_eq!(
            body["requested_scope"]["project_root"],
            root.to_string_lossy().as_ref()
        );
        assert_eq!(body["requested_scope"]["session_id"], session);
        assert!(body["resolved_scope"]["session_id"].is_null());
        assert_eq!(body["scope_validation"]["execution_admission"], false);
        let subpath = body["requested_scope"]["working_subpath_id"]
            .as_str()
            .expect("existing working-context owner resolved the subpath");
        subpaths.push(subpath.to_string());
        assert_eq!(
            fs::read(primary.join(".focusa-project.json")).unwrap(),
            marker_before
        );
        assert_eq!(fs::read(packet_path(&primary)).unwrap(), packet_before);
    }
    assert_ne!(subpaths[0], subpaths[1]);
    let (_, foreign) = read_genesis_status_fixture(
        state.clone(),
        &worktree_a,
        Some("status-a"),
        Some("status-a"),
        Some("worktree-session-a"),
    )
    .await;
    assert_eq!(foreign["canonical"], false);
    assert_eq!(foreign["requested_scope"]["continuity_id"], "status-a");
    assert_eq!(foreign["resolved_scope"]["continuity_id"], "status-b");
    let mut uri = reqwest::Url::parse("http://fixture/v1/project/genesis/status").unwrap();
    uri.query_pairs_mut()
        .append_pair("project_root", worktree_a.to_string_lossy().as_ref())
        .append_pair("continuity_id", "status-b");
    use tower::ServiceExt;
    let response = router()
        .with_state(state)
        .oneshot(
            axum::http::Request::builder()
                .uri(uri.as_str())
                .header("x-scope-project-root", primary.to_string_lossy().as_ref())
                .header("x-scope-continuity-id", "status-b")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(
        fs::read(primary.join(".focusa-project.json")).unwrap(),
        marker_before
    );
    assert_eq!(fs::read(packet_path(&primary)).unwrap(), packet_before);
}
