//! Actual production router, authentication middleware and SQLite round-trips.
//! Identity/entitlement are explicitly synthetic test fixtures, never production.
use super::*;
use axum::{
    body::{Body, to_bytes},
    http::Request,
};
use tower::ServiceExt;

const TOKEN: &str = "focusa-synthetic-compaction-fixture-only";
const ROOT: &str = "/tmp/focusa-compaction-http-fixture";
const CONTINUITY: &str = "synthetic-compaction-continuity";

async fn fixture() -> (Router, Arc<AppState>) {
    assert_eq!(std::env::var("FOCUSA_AUTH_TOKEN").as_deref(), Ok(TOKEN));
    let (state, _) = crate::routes::turn::tests::setup_state(Some(TOKEN.into())).await;
    state.focusa.write().await.version = 42;
    (crate::server::build_router(state.clone()), state)
}

async fn request(app: &Router, route: &str, body: Value, authorized: bool) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method("POST")
        .uri(route)
        .header("content-type", "application/json")
        .header("x-focusa-permissions", "admin:service")
        .header("x-scope-project-root", ROOT)
        .header("x-scope-continuity-id", CONTINUITY)
        .header(
            "Idempotency-Key",
            format!("synthetic-request-{}", Uuid::now_v7()),
        );
    if authorized {
        request = request.header("authorization", format!("Bearer {TOKEN}"));
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let raw = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    let result = serde_json::from_slice(&raw)
        .unwrap_or_else(|_| json!({"non_json":String::from_utf8_lossy(&raw)}));
    (status, result)
}

fn prepare_body(epoch: &str, ask: &str) -> Value {
    json!({"schema":"focusa.compaction_prepare_request.v1",
        "epoch":{"epoch_key":epoch,"session_frame_key":"synthetic-session","source_turn_id":"synthetic-turn"},
        "scope":{"root_scope":{"root_path":ROOT},"continuity_id":CONTINUITY},
        "current_ask":ask,"local_semantic_deltas":{},"adapter_capabilities":{},"native_pressure":{}})
}

fn verify_body(prepared: &Value, ask: &str) -> Value {
    json!({"schema":"focusa.compaction_verify_request.v1","epoch_id":prepared["epoch_id"],
        "expected_semantic_digest":prepared["semantic_digest"],"current_ask":ask,
        "scope":{"root_scope":{"root_path":ROOT},"continuity_id":CONTINUITY},"delivery_posture":"deferred"})
}

#[tokio::test]
async fn production_pi_caller_roundtrips_actual_authenticated_http_without_model_calls() {
    if crate::middleware::auth::isolate_synthetic_http_test(TOKEN) {
        return;
    }
    let (app, _) = fixture().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/pi-extension/tests/1051-actual-compaction-api-consumer.mjs");
    let mut command = tokio::process::Command::new("node");
    command
        .arg("--max-old-space-size=512")
        .arg(source)
        .arg(format!("http://{address}"));
    command.kill_on_drop(true);
    let output = tokio::time::timeout(std::time::Duration::from_secs(15), command.output()).await;
    server.abort();
    let output = output
        .expect("bounded consumer process")
        .expect("existing Node");
    assert!(
        output.status.success(),
        "consumer stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("round-trip passed"));
}

#[tokio::test]
async fn authenticated_actual_prepare_verify_preserves_complete_input_and_revision() {
    if crate::middleware::auth::isolate_synthetic_http_test(TOKEN) {
        return;
    }
    let (app, _) = fixture().await;
    let ask = format!(
        "{}NEVER DEPLOY OR CHANGE CUSTOMER DATA.",
        "Read-only review. ".repeat(400)
    );
    let (status, prepared) = request(
        &app,
        "/v1/compaction/prepare",
        prepare_body("actual-roundtrip", &ask),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{prepared}");
    assert_eq!(prepared["source_revision"], 42);
    assert_eq!(prepared["resume_projection"]["current_ask"]["text"], ask);
    assert_eq!(
        prepared["resume_projection"]["current_ask"]["text_complete"],
        true
    );
    assert!(
        prepared["native_compactor_instructions"]
            .as_str()
            .unwrap()
            .contains("NEVER DEPLOY OR CHANGE CUSTOMER DATA.")
    );
    let (status, verified) = request(
        &app,
        "/v1/compaction/verify",
        verify_body(&prepared, &ask),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{verified}");
    assert_eq!(verified["status"], "verified", "{verified}");
    assert_eq!(verified["semantic_digest"], prepared["semantic_digest"]);
    assert_eq!(verified["source_fields_preserved"], true);
    // Artifact/source verification is intentionally not a claim of model use.
    assert_eq!(verified["native_context_fidelity"], "unverified");
}

#[tokio::test]
async fn canonical_active_work_alone_is_not_admitted_without_its_current_operation_frontier() {
    if crate::middleware::auth::isolate_synthetic_http_test(TOKEN) {
        return;
    }
    use focusa_core::types::{
        HltStatus, TrajectoryProjectionRecord, WorkpointActionIntentRecord,
        WorkpointLifecycleStage, WorkpointRecord,
    };
    let (app, state) = fixture().await;
    let mut work = WorkpointRecord {
        project_root: Some(ROOT.into()),
        continuity_id: Some(CONTINUITY.into()),
        canonical: true,
        status: WorkpointStatus::Active,
        mission: Some("Synthetic approved review".into()),
        next_slice: Some("Review only".into()),
        ..Default::default()
    };
    let goal = TrajectoryProjectionRecord {
        trajectory_id: "synthetic-reviewed-goal".into(),
        project_root: Some(ROOT.into()),
        continuity_id: Some(CONTINUITY.into()),
        canonical: true,
        hlt_status: HltStatus::CanonicalExplicit,
        long_term_goal: "Synthetic continuity".into(),
        active_workpoint_id: Some(work.workpoint_id),
        ..Default::default()
    };
    let mut source = state.focusa.write().await;
    source.workpoint.active_workpoint_id = Some(work.workpoint_id);
    source.workpoint.records.push(work.clone());
    source.trajectory.active_trajectory_id = Some(goal.trajectory_id.clone());
    source.trajectory.records.push(goal);
    drop(source);
    let (_, without) = request(
        &app,
        "/v1/compaction/prepare",
        prepare_body("not-admitted", "review only"),
        true,
    )
    .await;
    assert_eq!(
        without["resume_projection"]["workpoint"]["action_authority"],
        false
    );
    work.action_intent = Some(WorkpointActionIntentRecord {
        action_type: "synthetic_review_only".into(),
        target_ref: Some(ROOT.into()),
        verification_hooks: vec![],
        status: None,
        lifecycle_stage: WorkpointLifecycleStage::Implement,
        lifecycle_transition_reason: None,
        lifecycle_transition_evidence_refs: vec![],
    });
    state.focusa.write().await.workpoint.records[0] = work;
    let (_, admitted) = request(
        &app,
        "/v1/compaction/prepare",
        prepare_body("admitted-context", "review only"),
        true,
    )
    .await;
    assert_eq!(
        admitted["resume_projection"]["workpoint"]["action_authority"], true,
        "{admitted}"
    );
    assert_eq!(
        admitted["resume_projection"]["workpoint"]["status"],
        "ready"
    );
}

#[tokio::test]
async fn committed_deltas_are_reconciled_without_reapplication_or_restriction_clipping() {
    if crate::middleware::auth::isolate_synthetic_http_test(TOKEN) {
        return;
    }
    use focusa_core::types::{FocusState, FrameRecord, FrameStats, FrameStatus};
    let (app, state) = fixture().await;
    let constraints: Vec<String> = (0..15)
        .map(|i| format!("Required constraint {i}"))
        .collect();
    let frame = FrameRecord {
        id: Uuid::now_v7(),
        parent_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        status: FrameStatus::Active,
        title: "Synthetic scoped review".into(),
        goal: "Review only".into(),
        beads_issue_id: "synthetic-test-item".into(),
        project_root: Some(ROOT.into()),
        continuity_id: Some(CONTINUITY.into()),
        tags: vec![],
        priority_hint: None,
        ascc_checkpoint_id: None,
        stats: FrameStats::default(),
        constraints: vec![],
        focus_state: FocusState {
            constraints: constraints.clone(),
            ..Default::default()
        },
        temporal_context: None,
        completed_at: None,
        completion_reason: None,
    };
    let mut source = state.focusa.write().await;
    source.focus_stack.active_id = Some(frame.id);
    source.focus_stack.frames.push(frame);
    drop(source);
    let mut body = prepare_body("committed", "review only");
    body["local_semantic_deltas"] = json!({"constraints":[constraints[0]]});
    let (status, prepared) = request(&app, "/v1/compaction/prepare", body, true).await;
    assert_eq!(status, StatusCode::OK, "{prepared}");
    assert_eq!(
        prepared["resume_projection"]["focus_state"]["constraints"],
        json!(constraints)
    );
    assert_eq!(
        prepared["resume_projection"]["prepare_context"]["delta_reconciliation"]["already_committed_counts"]
            ["constraints"],
        1
    );
    assert_eq!(
        prepared["resume_projection"]["prepare_context"]["delta_reconciliation"]["applied_count"],
        0
    );
    assert_eq!(state.focusa.read().await.version, 42);
}

#[tokio::test]
async fn canonical_writer_contention_waits_and_captures_one_consistent_revision() {
    if crate::middleware::auth::isolate_synthetic_http_test(TOKEN) {
        return;
    }
    let (app, state) = fixture().await;
    let writer = state.write_serial_lock.lock().await;
    let cloned = app.clone();
    let pending = tokio::spawn(async move {
        request(
            &cloned,
            "/v1/compaction/prepare",
            prepare_body("writer-contention", "review only"),
            true,
        )
        .await
    });
    tokio::task::yield_now().await;
    assert!(!pending.is_finished());
    state.focusa.write().await.version = 43;
    drop(writer);
    let (status, prepared) = pending.await.unwrap();
    assert_eq!(status, StatusCode::OK, "{prepared}");
    assert_eq!(prepared["source_revision"], 43);
}

#[tokio::test]
async fn actual_authentication_rejects_missing_credentials() {
    if crate::middleware::auth::isolate_synthetic_http_test(TOKEN) {
        return;
    }
    let (app, _) = fixture().await;
    let (status, _) = request(
        &app,
        "/v1/compaction/prepare",
        prepare_body("unauthorized", "review only"),
        false,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn actual_authenticated_read_device_cannot_request_administrative_compaction_capability() {
    if crate::middleware::auth::isolate_synthetic_http_test(TOKEN) {
        return;
    }
    let (app, _) = fixture().await;
    let token = format!("synthetic-compaction-read-{}", Uuid::now_v7());
    let pairing = crate::routes::device_pairing::shared_state();
    pairing.write().await.tokens.insert(
        token.clone(),
        focusa_core::types::DeviceToken {
            token: token.clone(),
            device_id: "synthetic-compaction-read-device".into(),
            scopes: vec!["read".into()],
            issued_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            last_used_at: None,
            issued_to: "isolated-compaction-test".into(),
        },
    );
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/compaction/prepare")
                .header("content-type", "application/json")
                .header("x-focusa-permissions", "admin:service")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::from(
                    prepare_body("read-device", "review only").to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    pairing.write().await.tokens.remove(&token);
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn actual_prepare_rejects_missing_null_and_oversized_input_without_prefixes() {
    if crate::middleware::auth::isolate_synthetic_http_test(TOKEN) {
        return;
    }
    let (app, _) = fixture().await;
    for input in [
        Value::Null,
        json!(""),
        json!(" "),
        json!("é".repeat(CURRENT_ASK_MAX_BYTES / 2 + 1)),
    ] {
        let mut body = prepare_body("invalid-current-input", "review only");
        let oversized = input
            .as_str()
            .is_some_and(|text| text.len() > CURRENT_ASK_MAX_BYTES);
        body["current_ask"] = input;
        let (status, result) = request(&app, "/v1/compaction/prepare", body, true).await;
        assert_eq!(
            status,
            if oversized {
                StatusCode::PAYLOAD_TOO_LARGE
            } else {
                StatusCode::BAD_REQUEST
            },
            "{result}"
        );
        assert_eq!(result["status"], "blocked");
    }
}

#[tokio::test]
async fn actual_epoch_retries_are_idempotent_and_changed_request_conflicts() {
    if crate::middleware::auth::isolate_synthetic_http_test(TOKEN) {
        return;
    }
    let (app, _) = fixture().await;
    let body = prepare_body("same-epoch", "review only");
    let ((s1, a), (s2, b)) = tokio::join!(
        request(&app, "/v1/compaction/prepare", body.clone(), true),
        request(&app, "/v1/compaction/prepare", body, true)
    );
    assert_eq!(s1, StatusCode::OK, "{a}");
    assert_eq!(s2, StatusCode::OK, "{b}");
    assert_eq!(a["epoch_id"], b["epoch_id"]);
    assert_eq!(a["semantic_digest"], b["semantic_digest"]);
    let (status, body) = request(
        &app,
        "/v1/compaction/prepare",
        prepare_body("same-epoch", "STOP; no changes"),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["error"], "epoch_request_changed");
}

#[tokio::test]
async fn actual_verify_rejects_digest_request_scope_and_legacy_identity_loss() {
    if crate::middleware::auth::isolate_synthetic_http_test(TOKEN) {
        return;
    }
    let (app, _) = fixture().await;
    let (_, prepared) = request(
        &app,
        "/v1/compaction/prepare",
        prepare_body("identity", "review only"),
        true,
    )
    .await;
    for field in ["expected_semantic_digest", "current_ask", "scope", "legacy"] {
        let mut body = verify_body(&prepared, "review only");
        if field == "legacy" {
            body.as_object_mut()
                .unwrap()
                .remove("expected_semantic_digest");
        } else {
            body[field] = json!("foreign");
        }
        let (status, verified) = request(&app, "/v1/compaction/verify", body, true).await;
        assert_eq!(status, StatusCode::OK, "{verified}");
        assert_eq!(verified["status"], "blocked", "{verified}");
    }
}

#[tokio::test]
async fn actual_uncommitted_delta_blocks_without_applying_or_counting_as_commit() {
    if crate::middleware::auth::isolate_synthetic_http_test(TOKEN) {
        return;
    }
    let (app, state) = fixture().await;
    let mut body = prepare_body("uncommitted", "review only");
    body["local_semantic_deltas"] = json!({"constraints":["NEVER CHANGE CUSTOMER DATA"]});
    let (status, result) = request(&app, "/v1/compaction/prepare", body, true).await;
    assert_eq!(status, StatusCode::CONFLICT, "{result}");
    assert_eq!(result["delta_reconciliation"]["applied_count"], 0);
    assert_eq!(state.focusa.read().await.version, 42);
}

#[tokio::test]
async fn unrelated_reducer_version_change_does_not_forge_scoped_drift() {
    if crate::middleware::auth::isolate_synthetic_http_test(TOKEN) {
        return;
    }
    let (app, state) = fixture().await;
    let (_, prepared) = request(
        &app,
        "/v1/compaction/prepare",
        prepare_body("same-scoped-source", "review only"),
        true,
    )
    .await;
    state.focusa.write().await.version += 1;
    let (_, result) = request(
        &app,
        "/v1/compaction/verify",
        verify_body(&prepared, "review only"),
        true,
    )
    .await;
    assert_eq!(result["status"], "verified", "{result}");
    assert_eq!(result["source_revision"], 42);
    assert_eq!(result["current_source_revision"], 43);
    assert_eq!(result["source_fields_preserved"], true);
}

#[tokio::test]
async fn durable_prepare_survives_reader_reload_and_rejects_changed_scoped_source() {
    if crate::middleware::auth::isolate_synthetic_http_test(TOKEN) {
        return;
    }
    let (app, state) = fixture().await;
    let (_, prepared) = request(
        &app,
        "/v1/compaction/prepare",
        prepare_body("durable", "review only"),
        true,
    )
    .await;
    let stored = packet_by_id_durable(
        &state.config.data_dir,
        prepared["epoch_id"].as_str().unwrap(),
    )
    .unwrap();
    assert_eq!(semantic_digest(&stored), prepared["semantic_digest"]);
    let goal = focusa_core::types::TrajectoryProjectionRecord {
        project_root: Some(ROOT.into()),
        continuity_id: Some(CONTINUITY.into()),
        canonical: true,
        trajectory_id: "changed-scoped-goal".into(),
        long_term_goal: "Changed owner scope".into(),
        ..Default::default()
    };
    let mut source = state.focusa.write().await;
    source.trajectory.records.push(goal);
    source.version += 1;
    drop(source);
    let (_, result) = request(
        &app,
        "/v1/compaction/verify",
        verify_body(&prepared, "review only"),
        true,
    )
    .await;
    assert_eq!(result["status"], "blocked", "{result}");
}
