use super::*;
use axum::{
    body::{Body, to_bytes},
    http::Request,
};
use chrono::Utc;
use focusa_core::{
    agent_runtime_constitution::*,
    agent_runtime_constitution_authority::discover_project_instructions,
    agent_runtime_constitution_orchestrator::*, types::*,
};
use std::collections::{BTreeMap, BTreeSet};
use tower::ServiceExt;
const TOKEN: &str = "focusa-synthetic-compaction-fixture-only";

async fn fixture() -> (Router, Arc<AppState>, Value) {
    let (state, _) = crate::routes::turn::tests::setup_state(Some(TOKEN.into())).await;
    let root =
        std::env::temp_dir().join(format!("focusa-preload-fixture-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::create_dir(root.join(".beads")).unwrap();
    std::fs::write(
        root.join(".git/config"),
        "[remote \"origin\"]\n url = https://example.test/preload.git\n",
    )
    .unwrap();
    std::fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
    std::fs::write(root.join(".focusa-project.json"),r#"{"schema":"focusa.project.v1","project_id":"synthetic-preload","canonical_name":"Synthetic preload"}"#).unwrap();
    std::fs::write(
        root.join("AGENTS.md"),
        "# Rules\n- MUST verify evidence before completion.\n",
    )
    .unwrap();
    let root = root.to_str().unwrap().to_string();
    let sources = discover_project_instructions(std::path::Path::new(&root), 256 * 1024).unwrap();
    let approved_claim_refs = sources
        .claims
        .iter()
        .map(|claim| claim.claim_id.clone())
        .collect();
    let input = CristRuntimeInput {
        project_ref: root.clone(),
        genesis_ref: "synthetic-genesis".into(),
        approved_spec_ref: "synthetic-spec111".into(),
        operator_confirmed: true,
        agent_identity_ref: AgentIdentityReference("synthetic-agent".into()),
        kernel_ref: ConstitutionalKernelReference("synthetic-kernel".into()),
        role_ref: RoleProfileReference("synthetic-reviewer".into()),
        mission: "Synthetic review only".into(),
        responsibilities: vec!["Verify source".into()],
        non_responsibilities: vec!["No real activation or model calls".into()],
        authority_order: vec!["operator".into()],
        execution_boundaries: vec!["isolated fixture".into()],
        output_contracts: vec!["exact proof".into()],
        awareness_ref: RuntimeAwarenessContractReference("synthetic-awareness".into()),
        instruction_sources: sources.sources,
        target_profile: TargetCapabilityProfile {
            profile_version: "1".into(),
            target: "pi".into(),
            supported_layers: [
                PromptLayer::HarnessSystem,
                PromptLayer::FocusaKernel,
                PromptLayer::ProjectConstitution,
            ]
            .into_iter()
            .collect::<BTreeSet<_>>(),
            supported_features: ["append".into()].into_iter().collect(),
            unsupported_features: BTreeMap::new(),
            max_prompt_bytes: 64 * 1024,
            supports_session_pinning: true,
        },
        changed_paths: vec![],
    };
    let mut c = compose_runtime_constitution(input).unwrap().constitution;
    c.claim_refs = approved_claim_refs;
    state.persistence.save_runtime_constitution(&c).unwrap();
    let version = RuntimeConstitutionVersion {
        version: c.revision.to_string(),
        parent_version: None,
        content_sha256: c
            .instruction_sources
            .iter()
            .map(|s| s.content_sha256.as_str())
            .collect::<Vec<_>>()
            .join(":"),
        lifecycle: RuntimeConstitutionLifecycleState::Approved,
        created_at: Utc::now(),
    };
    state
        .persistence
        .append_runtime_constitution_event(
            &uuid::Uuid::now_v7().to_string(),
            &c.constitution_id,
            "synthetic-approval",
            &RuntimeConstitutionEvent::RuntimeConstitutionApproved(version.clone()),
        )
        .unwrap();
    state
        .persistence
        .append_runtime_constitution_event(
            &uuid::Uuid::now_v7().to_string(),
            &c.constitution_id,
            "synthetic-activation",
            &RuntimeConstitutionEvent::RuntimeConstitutionActivated(RuntimeConstitutionVersion {
                lifecycle: RuntimeConstitutionLifecycleState::Active,
                ..version
            }),
        )
        .unwrap();
    let work = WorkpointRecord {
        project_root: Some(root.clone()),
        continuity_id: Some("synthetic-preload-stream".into()),
        canonical: true,
        status: WorkpointStatus::Active,
        mission: Some("Saved mission is not current request".into()),
        next_slice: Some("Review only".into()),
        action_intent: Some(WorkpointActionIntentRecord {
            action_type: "synthetic_review".into(),
            target_ref: Some(root.clone()),
            verification_hooks: vec![],
            status: None,
            lifecycle_stage: WorkpointLifecycleStage::Implement,
            lifecycle_transition_reason: None,
            lifecycle_transition_evidence_refs: vec![],
        }),
        ..Default::default()
    };
    let goal = TrajectoryProjectionRecord {
        trajectory_id: "synthetic-preload-goal".into(),
        project_root: Some(root.clone()),
        continuity_id: Some("synthetic-preload-stream".into()),
        canonical: true,
        hlt_status: HltStatus::CanonicalExplicit,
        long_term_goal: "Synthetic approved purpose".into(),
        active_workpoint_id: Some(work.workpoint_id),
        ..Default::default()
    };
    let mut source = state.focusa.write().await;
    source.version = 42;
    source.workpoint.active_workpoint_id = Some(work.workpoint_id);
    source.workpoint.records.push(work);
    source.trajectory.active_trajectory_id = Some(goal.trajectory_id.clone());
    source.trajectory.records.push(goal);
    drop(source);
    let body = json!({"artifact_version":2,"profile":"rules_and_context","project_root":root,"continuity_id":"synthetic-preload-stream","working_subpath_id":"primary","constitution_id":c.constitution_id,"current_ask":"Review only: preserve the late restriction, no installation."});
    (crate::server::build_router(state.clone()), state, body)
}
async fn call(app: &Router, action: &str, body: Value) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/preload/{action}"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {TOKEN}"))
                .header(
                    "X-Focusa-Permissions",
                    "admin:service,work-loop:read,work-loop:write",
                )
                .header("Idempotency-Key", uuid::Uuid::now_v7().to_string())
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}
fn bound(mut request: Value, result: &Value) -> Value {
    request["artifact"] = result["artifact"].clone();
    request["artifact_ref"] = result["artifact"]["artifact_ref"].clone();
    request["expected_digest"] = result["artifact"]["content_sha256"].clone();
    request
}
#[tokio::test]
async fn production_registered_preload_tools_roundtrip_actual_api_without_model_calls() {
    let (app, _, request) = fixture().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/pi-extension/tests/1078-actual-scoped-preload-consumer.mjs");
    let output = tokio::process::Command::new("node")
        .arg(script)
        .arg(format!("http://{address}"))
        .env("FOCUSA_PRELOAD_TEST_REQUEST", request.to_string())
        .output()
        .await
        .unwrap();
    server.abort();
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
#[tokio::test]
async fn scoped_build_renders_composed_current_request_and_all_inspectors_validate_exact_artifact()
{
    let (app, _, req) = fixture().await;
    let (status, built) = call(&app, "build", req.clone()).await;
    assert_eq!(status, StatusCode::OK, "{built}");
    assert!(
        built["artifact"]["packet"]["rendered"]
            .as_str()
            .unwrap()
            .contains("late restriction")
    );
    assert!(
        !built["artifact"]["packet"]["rendered"]
            .as_str()
            .unwrap()
            .contains("bounded_context_item_")
    );
    for action in ["render", "verify", "doctor"] {
        let (status, check) = call(&app, action, bound(req.clone(), &built)).await;
        assert_eq!(status, StatusCode::OK, "{check}");
        assert_eq!(check["status"], "verified");
        assert_eq!(check["delivery_verified"], false);
        assert_eq!(check["current_process_refresh_verified"], false);
    }
}
#[tokio::test]
async fn cross_scope_tampered_partial_legacy_and_changed_request_artifacts_are_denied() {
    let (app, _, req) = fixture().await;
    let (_, built) = call(&app, "build", req.clone()).await;
    assert_eq!(built["status"], "completed", "{built}");
    for field in [
        "artifact",
        "artifact_ref",
        "expected_digest",
        "constitution_id",
    ] {
        let mut b = bound(req.clone(), &built);
        b.as_object_mut().unwrap().remove(field);
        assert!(call(&app, "verify", b).await.0.is_client_error());
    }
    let mut changed = bound(req.clone(), &built);
    changed["current_ask"] = json!("Different complete request");
    assert!(call(&app, "verify", changed).await.0.is_client_error());
    let mut tamper = bound(req.clone(), &built);
    tamper["artifact"]["packet"]["rendered"] = json!("Lost restriction");
    assert!(call(&app, "render", tamper).await.0.is_client_error());
    let mut foreign = bound(req.clone(), &built);
    foreign["continuity_id"] = json!("foreign-stream");
    assert!(call(&app, "verify", foreign).await.0.is_client_error());
    let (_, legacy) = call(&app, "verify", json!({"profile":"rules_only"})).await;
    assert_eq!(legacy["context_kind"], "profile_template");
    assert_eq!(legacy["canonical"], false);
}
#[tokio::test]
async fn exact_write_receipt_replay_file_tamper_and_unsafe_targets_are_checked() {
    let (app, _, req) = fixture().await;
    let (_, built) = call(&app, "build", req.clone()).await;
    assert_eq!(built["status"], "completed", "{built}");
    let target = format!("/tmp/focusa-preload/synthetic-{}.txt", uuid::Uuid::now_v7());
    let mut write = bound(req.clone(), &built);
    write["profile_id"] = req["profile"].clone();
    write["target_path"] = json!(target);
    write["idempotency_key"] = json!(uuid::Uuid::now_v7().to_string());
    let (status, delivered) = call(&app, "write", write.clone()).await;
    assert_eq!(status, StatusCode::OK, "{delivered}");
    assert_eq!(delivered["delivery_verified"], true);
    assert_eq!(delivered["current_process_refresh_verified"], false);
    let (_, replay) = call(&app, "write", write.clone()).await;
    assert_eq!(replay["idempotent_replay"], true);
    let other = format!("/tmp/focusa-preload/synthetic-{}.txt", uuid::Uuid::now_v7());
    let mut conflicting_write = write.clone();
    conflicting_write["target_path"] = json!(other);
    assert!(
        call(&app, "write", conflicting_write)
            .await
            .0
            .is_client_error()
    );
    assert!(
        !std::path::Path::new(&other).exists(),
        "key conflict must fail before creating another target"
    );
    let mut receipt = bound(req.clone(), &built);
    receipt["delivery_target"] = json!(target);
    receipt["idempotency_key"] = write["idempotency_key"].clone();
    let (status, committed) = call(&app, "receipt-commit", receipt.clone()).await;
    assert_eq!(status, StatusCode::OK, "{committed}");
    let (_, replay) = call(&app, "receipt-commit", receipt.clone()).await;
    assert_eq!(replay["idempotent_replay"], true);
    for unsafe_path in [
        "/tmp/focusa-preload/../outside.txt",
        "/etc/not-authorized.txt",
    ] {
        write["target_path"] = json!(unsafe_path);
        assert!(call(&app, "write", write.clone()).await.0.is_client_error());
    }
    std::fs::write(&target, "Lost exact content").unwrap();
    assert!(
        call(&app, "receipt-commit", receipt)
            .await
            .0
            .is_client_error()
    );
}
#[tokio::test]
async fn concurrent_delivery_keys_bind_one_target_but_remain_project_scoped() {
    let (app, _, req) = fixture().await;
    let (_, built) = call(&app, "build", req.clone()).await;
    let key = uuid::Uuid::now_v7().to_string();
    let targets = [
        format!("/tmp/focusa-preload/race-{}.txt", uuid::Uuid::now_v7()),
        format!("/tmp/focusa-preload/race-{}.txt", uuid::Uuid::now_v7()),
    ];
    let mut first = bound(req.clone(), &built);
    first["profile_id"] = req["profile"].clone();
    first["target_path"] = json!(targets[0]);
    first["idempotency_key"] = json!(key);
    let mut second = first.clone();
    second["target_path"] = json!(targets[1]);
    let (a, b) = tokio::join!(call(&app, "write", first), call(&app, "write", second));
    assert_eq!(
        [a.0, b.0]
            .iter()
            .filter(|status| status.is_success())
            .count(),
        1
    );
    assert_eq!(
        targets
            .iter()
            .filter(|target| std::path::Path::new(target).exists())
            .count(),
        1
    );
    let (other_app, _, other_req) = fixture().await;
    let (_, other_build) = call(&other_app, "build", other_req.clone()).await;
    let mut foreign = bound(other_req.clone(), &other_build);
    foreign["profile_id"] = other_req["profile"].clone();
    foreign["target_path"] = json!(format!(
        "/tmp/focusa-preload/other-project-{}.txt",
        uuid::Uuid::now_v7()
    ));
    foreign["idempotency_key"] = json!(key);
    let (status, body) = call(&other_app, "write", foreign).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}
#[tokio::test]
async fn revoked_constitution_cannot_verify_previously_valid_artifact() {
    let (app, state, req) = fixture().await;
    let (_, built) = call(&app, "build", req.clone()).await;
    state
        .persistence
        .append_runtime_constitution_event(
            &uuid::Uuid::now_v7().to_string(),
            req["constitution_id"].as_str().unwrap(),
            "synthetic-revocation",
            &RuntimeConstitutionEvent::RuntimeConstitutionRevoked(PromptRevocation {
                revocation_id: "synthetic-revoked".into(),
                version: "1".into(),
                reason_code: "test".into(),
                effective_at: Utc::now(),
            }),
        )
        .unwrap();
    assert!(
        call(&app, "verify", bound(req, &built))
            .await
            .0
            .is_client_error()
    );
}
#[tokio::test]
async fn source_and_constitution_drift_reject_stale_identity() {
    let (app, state, req) = fixture().await;
    let (_, built) = call(&app, "build", req.clone()).await;
    assert_eq!(built["status"], "completed", "{built}");
    state.focusa.write().await.version += 1;
    assert!(
        call(&app, "verify", bound(req.clone(), &built))
            .await
            .0
            .is_client_error()
    );
    let (_, fresh) = call(&app, "build", req.clone()).await;
    assert_ne!(
        built["artifact"]["artifact_ref"],
        fresh["artifact"]["artifact_ref"]
    );
    std::fs::write(
        std::path::Path::new(req["project_root"].as_str().unwrap()).join("AGENTS.md"),
        "Changed approved source\n",
    )
    .unwrap();
    assert!(call(&app, "build", req).await.0.is_client_error());
}
