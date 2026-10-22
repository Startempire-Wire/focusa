//! Producer regressions using synthetic workspace data, without a live daemon.

#[cfg(unix)]
async fn assert_slow_metadata_keeps_http_responsive(endpoint: &str, offload: bool) {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };
    use std::os::unix::fs::OpenOptionsExt;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use tower::ServiceExt;

    let fixture = tempfile::tempdir().unwrap();
    let project = fixture.path().join("synthetic-project");
    std::fs::create_dir_all(project.join(".git")).unwrap();
    let marker = project.join(".focusa-project.json");
    nix::unistd::mkfifo(
        &marker,
        nix::sys::stat::Mode::S_IRUSR | nix::sys::stat::Mode::S_IWUSR,
    )
    .unwrap();

    // Release the synthetic slow read even if the async runtime cannot progress.
    // Nonblocking open also bounds fixture failure if a handler never reads it.
    let released = Arc::new(AtomicBool::new(false));
    let writer_released = released.clone();
    let (opened_tx, opened_rx) = tokio::sync::oneshot::channel();
    let writer = std::thread::spawn(move || -> std::io::Result<()> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        let pipe = loop {
            match std::fs::OpenOptions::new()
                .write(true)
                .custom_flags(nix::libc::O_NONBLOCK)
                .open(&marker)
            {
                Ok(pipe) => break pipe,
                Err(error)
                    if error.raw_os_error() == Some(nix::libc::ENXIO)
                        && std::time::Instant::now() < deadline =>
                {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(error) => return Err(error),
            }
        };
        // The identity path may inspect the same marker more than once. Keep
        // the first opened descriptor slow, but make later opens regular reads.
        let replacement = marker.with_extension("regular.json");
        std::fs::write(&replacement, b"{}")?;
        std::fs::rename(&replacement, &marker)?;
        let _ = opened_tx.send(());
        std::thread::sleep(std::time::Duration::from_secs(1));
        writer_released.store(true, Ordering::SeqCst);
        drop(pipe);
        Ok(())
    });

    let discovery_route = if offload {
        get(super::discover_projects)
    } else {
        get(
            |axum::extract::Query(query): axum::extract::Query<super::ProjectDiscoverQuery>| async move {
                super::discover_projects_inspection(query)
            },
        )
    };
    let app = Router::new()
        .route("/v1/project/discover", discovery_route)
        .route("/v1/project/identity", get(super::identity))
        .route("/v1/project/list", get(super::list_projects))
        .route("/unrelated-health-probe", get(|| async { StatusCode::OK }));
    let query = if endpoint == "/v1/project/discover" {
        format!(
            "from={}&max_depth=0&max_results=1",
            fixture.path().display()
        )
    } else {
        format!("project_root={}", project.display())
    };
    let request = Request::builder()
        .uri(format!("{endpoint}?{query}"))
        .body(Body::empty())
        .unwrap();
    let inspection = tokio::spawn(app.clone().oneshot(request));
    let opened = opened_rx.await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/unrelated-health-probe")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let responsive_while_inspection_blocked = !released.load(Ordering::SeqCst);

    // Drain owned work before asserting; an expected failure leaks no fixture.
    let inspection_response = tokio::time::timeout(std::time::Duration::from_secs(3), inspection)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let writer_result = writer.join().unwrap();
    assert!(
        opened.is_ok(),
        "the selected handler did not exercise the synthetic slow read"
    );
    writer_result.unwrap();
    assert_eq!(inspection_response.status(), StatusCode::OK);
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        responsive_while_inspection_blocked, offload,
        "project inspection scheduling did not match unrelated HTTP responsiveness"
    );
}

#[cfg(unix)]
#[tokio::test(flavor = "current_thread")]
async fn slow_workspace_metadata_does_not_starve_unrelated_http_requests() {
    assert_slow_metadata_keeps_http_responsive("/v1/project/discover", true).await;
}

#[cfg(unix)]
#[tokio::test(flavor = "current_thread")]
async fn slow_workspace_metadata_identity_keeps_http_responsive() {
    assert_slow_metadata_keeps_http_responsive("/v1/project/identity", true).await;
}

#[cfg(unix)]
#[tokio::test(flavor = "current_thread")]
async fn slow_workspace_metadata_list_keeps_http_responsive() {
    assert_slow_metadata_keeps_http_responsive("/v1/project/list", true).await;
}

#[cfg(unix)]
#[tokio::test(flavor = "current_thread")]
async fn slow_workspace_metadata_negative_control_detects_inline_inspection() {
    assert_slow_metadata_keeps_http_responsive("/v1/project/discover", false).await;
}

#[tokio::test]
async fn inspection_success_preserves_unverified_identity_payload() {
    let original = serde_json::json!({
        "status": "ambiguous_project_binding",
        "canonical": false,
        "degraded": true,
        "verification": {"verified": false},
        "binding_decision": {"requires_confirmation": true}
    });
    let supplied = original.clone();
    let axum::Json(actual) = super::project_inspection_response(move || axum::Json(supplied)).await;
    assert_eq!(actual, original);
}

#[tokio::test]
async fn inspection_worker_failure_never_claims_verified_identity() {
    let axum::Json(result) =
        super::project_inspection_response(|| panic!("synthetic project inspection failure")).await;
    assert_eq!(result["status"], "error");
    assert_eq!(result["ok"], false);
    assert_eq!(result["canonical"], false);
    assert_eq!(result["failure_class"], "null_response");
    assert_eq!(result["error"]["code"], "PROJECT_INSPECTION_WORKER_FAILED");
    assert!(result.get("verification").is_none());
}
