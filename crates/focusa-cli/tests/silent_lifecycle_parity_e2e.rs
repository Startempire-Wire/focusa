//! Spec 133 §24 lifecycle CLI parity over one-to-one daemon routes.

use chrono::{Duration, Utc};
use serde_json::{Value, json};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::thread::JoinHandle;
use uuid::Uuid;

const FOCUSA_BIN: &str = env!("CARGO_BIN_EXE_focusa");

fn run_mocked_post<F>(
    args: &[String],
    expected_target: &str,
    status: &str,
    response_body: String,
    assert_body: F,
) -> (Output, JoinHandle<()>)
where
    F: FnOnce(Value) + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock daemon");
    let address = listener.local_addr().expect("mock daemon address");
    let expected_target = expected_target.to_string();
    let status = status.to_string();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept CLI request");
        let mut request = Vec::new();
        let mut buffer = [0u8; 4096];
        let header_end = loop {
            let count = stream.read(&mut buffer).expect("read CLI request");
            assert!(count > 0, "request ended before headers");
            request.extend_from_slice(&buffer[..count]);
            if let Some(index) = request.windows(4).position(|window| window == b"\r\n\r\n") {
                break index + 4;
            }
        };
        let headers = String::from_utf8_lossy(&request[..header_end]);
        assert!(
            headers.starts_with(&format!("POST {expected_target} HTTP/1.1")),
            "unexpected request: {headers}"
        );
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().expect("content length"))
            })
            .expect("JSON POST has content length");
        while request.len() - header_end < content_length {
            let count = stream.read(&mut buffer).expect("read CLI request body");
            assert!(count > 0, "request body ended early");
            request.extend_from_slice(&buffer[..count]);
        }
        let body: Value = serde_json::from_slice(&request[header_end..header_end + content_length])
            .expect("CLI sends JSON body");
        assert_body(body);
        write!(
            stream,
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
            response_body.len()
        )
        .expect("write mock daemon response");
    });

    let output = Command::new(FOCUSA_BIN)
        .args(args)
        .env("FOCUSA_API_URL", format!("http://{address}"))
        .env("FOCUSA_API_TIMEOUT", "5")
        .output()
        .expect("run focusa CLI");
    (output, server)
}

fn exact_ids() -> (String, String) {
    (Uuid::now_v7().to_string(), Uuid::now_v7().to_string())
}

fn temp_json(prefix: &str, value: &Value) -> PathBuf {
    let path = std::env::temp_dir().join(format!("focusa-{prefix}-{}.json", Uuid::now_v7()));
    fs::write(&path, serde_json::to_vec(value).unwrap()).expect("write JSON fixture");
    path
}

fn context_authority() -> Value {
    json!({
        "verdict_ref": "context-authority:fresh",
        "allowed": true,
        "project_identity_ref": "project:test",
        "continuity_id": "continuity:test",
        "workpoint_ref": "workpoint:test",
        "expires_at": (Utc::now() + Duration::hours(1)).to_rfc3339(),
    })
}

fn lease(session_id: &str, actor: &str) -> Value {
    let now = Utc::now();
    json!({
        "schema": "focusa.silent_session_lease.v1",
        "lease_id": Uuid::now_v7(),
        "session_id": session_id,
        "project_root": "/tmp/focusa-project",
        "project_identity_ref": "project:test",
        "continuity_id": "continuity:test",
        "work_item_ref": "focusa-test",
        "workspace_ref": "workspace:test",
        "path_intents": ["/tmp/focusa-project"],
        "writer_role": "exclusive_writer",
        "owner_actor_instance_ref": actor,
        "fencing_token": 9,
        "acquired_at": now.to_rfc3339(),
        "heartbeat_at": now.to_rfc3339(),
        "expires_at": (now + Duration::hours(1)).to_rfc3339(),
        "adoption_policy": "operator_only"
    })
}

fn success(data: Value, side_effects: Value, receipts: Value) -> String {
    json!({
        "ok": true,
        "status": "accepted",
        "canonical": true,
        "advisory": false,
        "degraded": false,
        "stale": false,
        "failure_class": null,
        "retry": {"safe": true, "posture": "idempotent_read_or_guarded_write", "reason": "canonical_result"},
        "side_effects": side_effects,
        "evidence_refs": ["evidence:authority"],
        "receipt_refs": receipts,
        "next_tools": [],
        "recovery_hint": null,
        "misuse_hint": null,
        "data": data
    })
    .to_string()
}

fn assert_success(output: &Output, server: JoinHandle<()>) -> String {
    server.join().expect("mock daemon validates request");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        output.status.success(),
        "CLI failed\nstdout={stdout}\nstderr={stderr}"
    );
    stdout
}

fn control_args(
    operation: &str,
    session_id: &str,
    run_id: &str,
    lease_path: &std::path::Path,
    json_mode: bool,
) -> Vec<String> {
    let mut args = Vec::new();
    if json_mode {
        args.push("--json".into());
    }
    args.extend([
        "silent".into(),
        operation.into(),
        session_id.into(),
        "--run".into(),
        run_id.into(),
        "--generation".into(),
        "3".into(),
        "--actor-instance-ref".into(),
        "actor-instance:test".into(),
        "--approval-id".into(),
        format!("approval:{operation}"),
        "--lease-file".into(),
        lease_path.display().to_string(),
        "--reason-code".into(),
        format!("test_{operation}"),
    ]);
    args
}

#[test]
fn preflight_create_and_controls_map_one_to_one_to_daemon_routes() {
    let context = context_authority();
    let preflight_config = json!({
        "identity": {
            "project_root": "/tmp/focusa-project",
            "project_identity_ref": "project:test",
            "continuity_id": "continuity:test",
            "work_item_ref": "focusa-test",
            "mission": "Prove lifecycle parity"
        }
    });
    let preflight_path = temp_json("preflight-config", &preflight_config);
    let preflight_response = success(
        json!({"resolved_effective_config": preflight_config}),
        json!([]),
        json!([]),
    );
    let preflight_args = vec![
        "--json".into(),
        "silent".into(),
        "preflight".into(),
        "--config-file".into(),
        preflight_path.display().to_string(),
    ];
    let expected_preflight_config = preflight_config.clone();
    let (output, server) = run_mocked_post(
        &preflight_args,
        "/v1/silent-sessions/preflight",
        "200 OK",
        preflight_response,
        move |body| {
            assert_eq!(body["config"], expected_preflight_config);
            assert_eq!(body["layers"], json!([]));
            assert!(body.get("approval_id").is_none());
            assert!(body.get("context_authority").is_none());
        },
    );
    let stdout = assert_success(&output, server);
    let preflight_value: Value = serde_json::from_str(&stdout).unwrap();
    assert!(
        preflight_value["receipt_refs"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let (session_id, run_id) = exact_ids();
    let create_request = json!({
        "actor_instance_ref": "actor-instance:test",
        "approval_id": "approval:create",
        "context_authority": context,
        "session": {
            "session_id": session_id,
            "active_run_id": run_id,
            "lifecycle_state": "draft",
            "project_root": "/tmp/focusa-project",
            "project_identity_ref": "project:test",
            "continuity_id": "continuity:test",
            "workpoint_ref": "workpoint:test",
            "work_item_ref": "focusa-test",
            "mission": "Prove lifecycle parity"
        },
        "run": {"run_id": run_id, "session_id": session_id, "generation": 1},
        "initial_config": {"identity": {
            "project_root": "/tmp/focusa-project",
            "project_identity_ref": "project:test",
            "continuity_id": "continuity:test",
            "work_item_ref": "focusa-test",
            "mission": "Prove lifecycle parity"
        }},
        "legacy_approved": false
    });
    let create_path = temp_json("create", &create_request);
    let create_response = success(
        json!({
            "session_id": session_id,
            "run_id": run_id,
            "generation": 1,
            "lifecycle_state": "draft",
            "event_id": Uuid::now_v7()
        }),
        json!([{"kind": "canonical_session_created", "lifecycle_state": "draft"}]),
        json!(["silent-session-lifecycle:create"]),
    );
    let create_args = vec![
        "--json".into(),
        "silent".into(),
        "create".into(),
        "--request-file".into(),
        create_path.display().to_string(),
    ];
    let expected_session = session_id.clone();
    let expected_run = run_id.clone();
    let (output, server) = run_mocked_post(
        &create_args,
        "/v1/silent-sessions",
        "200 OK",
        create_response,
        move |body| {
            assert_eq!(body["session"]["session_id"], expected_session);
            assert_eq!(body["run"]["run_id"], expected_run);
            assert_eq!(body["approval_id"], "approval:create");
        },
    );
    assert_success(&output, server);

    let lease_path = temp_json("lease", &lease(&session_id, "actor-instance:test"));
    for operation in [
        "start",
        "pause",
        "resume",
        "interrupt",
        "cancel",
        "restart",
        "adopt",
    ] {
        let returned_run_id = if operation == "restart" {
            Uuid::now_v7().to_string()
        } else {
            run_id.clone()
        };
        let returned_generation = if operation == "restart" { 4 } else { 3 };
        let response = success(
            json!({
                "session_id": session_id,
                "run_id": returned_run_id,
                "generation": returned_generation,
                "lifecycle_state": if operation == "restart" { "failed" } else { "running" },
                "event_id": Uuid::now_v7()
            }),
            json!([{"kind": "canonical_lifecycle_transition", "operation": operation}]),
            json!([format!("silent-session-lifecycle:{operation}")]),
        );
        let target = format!(
            "/v1/silent-sessions/{session_id}/{operation}?run_id={run_id}&expected_generation=3"
        );
        let args = control_args(operation, &session_id, &run_id, &lease_path, true);
        let expected_operation = operation.to_string();
        let (output, server) = run_mocked_post(&args, &target, "200 OK", response, move |body| {
            assert_eq!(body["actor_instance_ref"], "actor-instance:test");
            assert_eq!(
                body["approval_id"],
                format!("approval:{expected_operation}")
            );
            assert_eq!(body["legacy_approved"], false);
            assert_eq!(body["lease"]["fencing_token"], 9);
            assert_eq!(body["reason_code"], format!("test_{expected_operation}"));
        });
        let stdout = assert_success(&output, server);
        let value: Value = serde_json::from_str(&stdout).expect("control emits one envelope");
        assert_eq!(value["data"]["generation"], returned_generation);
        assert_eq!(value["receipt_refs"].as_array().unwrap().len(), 1);
        assert_eq!(value["side_effects"].as_array().unwrap().len(), 1);
    }

    for path in [preflight_path, create_path, lease_path] {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn approval_preview_and_create_map_to_exact_daemon_routes() {
    let (session_id, run_id) = exact_ids();
    let approval_id = Uuid::now_v7().to_string();
    let expires_at = (Utc::now() + Duration::hours(1)).to_rfc3339();
    let preview_request = json!({
        "approval_id": approval_id,
        "run_id": run_id,
        "generation": 1,
        "action": "start",
        "risk_class": "read_only_verifier",
        "expires_at": expires_at,
        "requested_side_effects": ["runner_start_request"]
    });
    let preview_path = temp_json("approval-preview", &preview_request);
    let preview_args = vec![
        "--json".into(),
        "silent".into(),
        "approval".into(),
        "preview".into(),
        session_id.clone(),
        "--request-file".into(),
        preview_path.display().to_string(),
    ];
    let expected_preview = preview_request.clone();
    let (output, server) = run_mocked_post(
        &preview_args,
        &format!("/v1/silent-sessions/{session_id}/approvals/preview"),
        "200 OK",
        success(
            json!({"approval": {"action_digest": "digest:start"}, "persisted": false}),
            json!([]),
            json!([]),
        ),
        move |body| assert_eq!(body, expected_preview),
    );
    assert_success(&output, server);

    let create_request = json!({
        "request": {
            "approval_id": approval_id,
            "run_id": run_id,
            "generation": 1,
            "action": "start",
            "risk_class": "read_only_verifier",
            "expires_at": expires_at,
            "requested_side_effects": ["runner_start_request"]
        },
        "expected_action_digest": "digest:start"
    });
    let create_path = temp_json("approval-create", &create_request);
    let create_args = vec![
        "--json".into(),
        "silent".into(),
        "approval".into(),
        "create".into(),
        session_id.clone(),
        "--request-file".into(),
        create_path.display().to_string(),
    ];
    let expected_create = create_request.clone();
    let (output, server) = run_mocked_post(
        &create_args,
        &format!("/v1/silent-sessions/{session_id}/approvals"),
        "201 Created",
        success(
            json!({"approval": {"approval_id": approval_id}, "persisted": true}),
            json!([{"effect": "durable_approval_create", "status": "persisted"}]),
            json!([format!("silent-session-approval:{approval_id}")]),
        ),
        move |body| assert_eq!(body, expected_create),
    );
    assert_success(&output, server);

    for path in [preview_path, create_path] {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn lifecycle_human_and_json_modes_preserve_receipts_side_effects_and_recovery() {
    let (session_id, run_id) = exact_ids();
    let lease_path = temp_json("parity-lease", &lease(&session_id, "actor-instance:test"));
    let response = || {
        success(
            json!({
                "session_id": session_id,
                "run_id": run_id,
                "generation": 3,
                "lifecycle_state": "pausing",
                "event_id": Uuid::now_v7()
            }),
            json!([{"kind": "canonical_lifecycle_transition", "from": "running", "to": "pausing"}]),
            json!(["silent-session-lifecycle:pause-receipt"]),
        )
    };
    let target =
        format!("/v1/silent-sessions/{session_id}/pause?run_id={run_id}&expected_generation=3");

    let human_args = control_args("pause", &session_id, &run_id, &lease_path, false);
    let (output, server) = run_mocked_post(&human_args, &target, "200 OK", response(), |_| {});
    let human = assert_success(&output, server);
    assert!(human.contains(&format!("Session: {session_id}")));
    assert!(human.contains(&format!("Run: {run_id}")));
    assert!(human.contains("Generation: 3"));
    assert!(human.contains("Lifecycle state: pausing"));
    assert!(human.contains("canonical_lifecycle_transition"));
    assert!(human.contains("silent-session-lifecycle:pause-receipt"));
    assert!(human.contains("Recovery: none"));

    let json_args = control_args("pause", &session_id, &run_id, &lease_path, true);
    let (output, server) = run_mocked_post(&json_args, &target, "200 OK", response(), |_| {});
    let machine = assert_success(&output, server);
    let value: Value = serde_json::from_str(&machine).expect("machine envelope");
    assert_eq!(value["data"]["session_id"], session_id);
    assert_eq!(value["data"]["run_id"], run_id);
    assert_eq!(value["data"]["generation"], 3);
    assert_eq!(value["data"]["lifecycle_state"], "pausing");
    assert_eq!(
        value["side_effects"][0]["kind"],
        "canonical_lifecycle_transition"
    );
    assert_eq!(
        value["receipt_refs"][0],
        "silent-session-lifecycle:pause-receipt"
    );
    assert!(value["recovery_hint"].is_null());

    let _ = fs::remove_file(lease_path);
}

#[test]
fn stale_scope_and_generation_fail_closed_even_when_http_is_successful() {
    let (session_id, run_id) = exact_ids();
    let lease_path = temp_json("stale-lease", &lease(&session_id, "actor-instance:test"));
    let args = control_args("pause", &session_id, &run_id, &lease_path, true);
    let target =
        format!("/v1/silent-sessions/{session_id}/pause?run_id={run_id}&expected_generation=3");
    let stale_scope = json!({
        "ok": false,
        "status": "blocked",
        "canonical": false,
        "advisory": false,
        "degraded": true,
        "stale": true,
        "failure_class": "silent_session_authorization_denied",
        "retry": {"safe": false, "posture": "do_not_retry_unchanged", "reason": "stale_scope"},
        "side_effects": [],
        "evidence_refs": [],
        "receipt_refs": [],
        "next_tools": [],
        "recovery_hint": "Refresh the exact durable principal, approval, and writer lease.",
        "misuse_hint": "Scope no longer matches.",
        "data": {"access_token": "must-not-leak"}
    })
    .to_string();
    let (output, server) = run_mocked_post(&args, &target, "200 OK", stale_scope, |_| {});
    server.join().expect("mock daemon validates stale request");
    assert!(
        !output.status.success(),
        "stale 2xx envelope must fail closed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(!combined.contains("must-not-leak"));
    assert!(combined.contains("silent_session_authorization_denied"));

    let generation_conflict = json!({
        "ok": false,
        "status": "blocked",
        "canonical": false,
        "advisory": false,
        "degraded": true,
        "stale": true,
        "failure_class": "generation_conflict",
        "retry": {"safe": false, "posture": "do_not_retry_unchanged", "reason": "generation_conflict"},
        "side_effects": [],
        "evidence_refs": [],
        "receipt_refs": [],
        "next_tools": [],
        "recovery_hint": "Reload the exact session run and retry with its current generation.",
        "misuse_hint": "A stale writer attempted a mutation.",
        "data": null
    })
    .to_string();
    let (output, server) =
        run_mocked_post(&args, &target, "409 Conflict", generation_conflict, |_| {});
    server
        .join()
        .expect("mock daemon validates stale generation");
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("generation_conflict"));

    let _ = fs::remove_file(lease_path);
}

#[test]
fn create_rejects_mismatched_context_scope_before_any_daemon_side_effect() {
    let (session_id, run_id) = exact_ids();
    let request = json!({
        "actor_instance_ref": "actor-instance:test",
        "approval_id": "approval:create",
        "context_authority": context_authority(),
        "session": {
            "session_id": session_id,
            "active_run_id": run_id,
            "lifecycle_state": "draft",
            "project_root": "/tmp/focusa-project",
            "project_identity_ref": "project:stale",
            "continuity_id": "continuity:test",
            "workpoint_ref": "workpoint:test",
            "work_item_ref": "focusa-test",
            "mission": "Reject stale scope"
        },
        "run": {"run_id": run_id, "session_id": session_id, "generation": 1},
        "initial_config": {"identity": {
            "project_root": "/tmp/focusa-project",
            "project_identity_ref": "project:stale",
            "continuity_id": "continuity:test",
            "work_item_ref": "focusa-test",
            "mission": "Reject stale scope"
        }}
    });
    let request_path = temp_json("stale-create", &request);
    let output = Command::new(FOCUSA_BIN)
        .args([
            "--json",
            "silent",
            "create",
            "--request-file",
            &request_path.display().to_string(),
        ])
        .env("FOCUSA_API_URL", "http://127.0.0.1:1")
        .output()
        .expect("run focusa CLI");
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("CLI_STALE_SCOPE"));
    let _ = fs::remove_file(request_path);
}
