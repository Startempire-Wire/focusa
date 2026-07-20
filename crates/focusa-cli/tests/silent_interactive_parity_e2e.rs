//! Spec 133 §24 interactive CLI parity, authorization, replay, and redaction proof.

use chrono::{Duration, Utc};
use serde_json::{Value, json};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread::JoinHandle;
use uuid::Uuid;

const FOCUSA_BIN: &str = env!("CARGO_BIN_EXE_focusa");
const RAW_SECRET: &str = "sk-must-never-reach-cli-output";

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

fn temp_json(prefix: &str, value: &Value) -> PathBuf {
    let path = std::env::temp_dir().join(format!("focusa-{prefix}-{}.json", Uuid::now_v7()));
    fs::write(&path, serde_json::to_vec(value).unwrap()).expect("write JSON fixture");
    path
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
        "fencing_token": 12,
        "acquired_at": now.to_rfc3339(),
        "heartbeat_at": now.to_rfc3339(),
        "expires_at": (now + Duration::hours(1)).to_rfc3339(),
        "adoption_policy": "operator_only"
    })
}

fn envelope(
    session_id: &str,
    run_id: &str,
    operation: &str,
    replayed: bool,
    retry_reason: &str,
) -> String {
    json!({
        "ok": true,
        "status": "accepted",
        "canonical": true,
        "advisory": false,
        "degraded": false,
        "stale": false,
        "failure_class": null,
        "retry": {
            "safe": true,
            "posture": "idempotent_read_or_guarded_write",
            "reason": retry_reason
        },
        "side_effects": if replayed {
            json!([])
        } else {
            json!([{"kind": "canonical_interaction_event", "operation": operation}])
        },
        "evidence_refs": [],
        "receipt_refs": [format!("silent-session-interaction:{}", Uuid::now_v7())],
        "next_tools": [],
        "recovery_hint": null,
        "misuse_hint": null,
        "data": {
            "session_id": session_id,
            "run_id": run_id,
            "generation": 4,
            "event_id": Uuid::now_v7(),
            "operation": operation,
            "replayed": replayed,
            "diagnostic": {"client_secret": RAW_SECRET}
        }
    })
    .to_string()
}

fn interaction_args(
    cli_operation: &str,
    session_id: &str,
    run_id: &str,
    lease_path: &Path,
    payload_path: &Path,
    json_mode: bool,
) -> Vec<String> {
    let mut args = Vec::new();
    if json_mode {
        args.push("--json".into());
    }
    args.extend([
        "silent".into(),
        cli_operation.into(),
        session_id.into(),
        "--run".into(),
        run_id.into(),
        "--generation".into(),
        "4".into(),
        "--actor-instance-ref".into(),
        "actor-instance:operator".into(),
        "--approval-id".into(),
        format!("approval:{cli_operation}"),
        "--idempotency-key".into(),
        format!("interactive-replay:{cli_operation}"),
        "--lease-file".into(),
        lease_path.display().to_string(),
        "--payload-file".into(),
        payload_path.display().to_string(),
    ]);
    if cli_operation == "key" {
        args.extend([
            "--key".into(),
            "CTRL_C".into(),
            "--key".into(),
            "ENTER".into(),
        ]);
    } else {
        args.extend([
            "--text".into(),
            format!("operator {cli_operation}: {RAW_SECRET}"),
        ]);
    }
    args
}

fn assert_success(output: &Output, server: JoinHandle<()>) -> String {
    server.join().expect("mock daemon validates request");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        output.status.success(),
        "CLI failed\nstdout={stdout}\nstderr={stderr}"
    );
    assert!(!stdout.contains(RAW_SECRET), "stdout leaked secret");
    assert!(!stderr.contains(RAW_SECRET), "stderr leaked secret");
    stdout
}

#[test]
fn interactive_commands_map_exact_authorized_requests_and_redact_json() {
    let session_id = Uuid::now_v7().to_string();
    let run_id = Uuid::now_v7().to_string();
    let lease_path = temp_json(
        "interaction-lease",
        &lease(&session_id, "actor-instance:operator"),
    );
    let payload_path = temp_json(
        "interaction-payload",
        &json!({"access_token": RAW_SECRET, "source": "operator"}),
    );

    for (cli_operation, api_operation) in [
        ("send", "input"),
        ("steer", "steer"),
        ("follow-up", "follow-up"),
        ("key", "keys"),
    ] {
        let target = format!(
            "/v1/silent-sessions/{session_id}/{api_operation}?run_id={run_id}&expected_generation=4"
        );
        let args = interaction_args(
            cli_operation,
            &session_id,
            &run_id,
            &lease_path,
            &payload_path,
            true,
        );
        let expected_cli_operation = cli_operation.to_string();
        let expected_api_operation = api_operation.to_string();
        let (output, server) = run_mocked_post(
            &args,
            &target,
            "200 OK",
            envelope(
                &session_id,
                &run_id,
                api_operation,
                false,
                "canonical_result",
            ),
            move |body| {
                assert_eq!(body["actor_instance_ref"], "actor-instance:operator");
                assert_eq!(
                    body["approval_id"],
                    format!("approval:{expected_cli_operation}")
                );
                assert_eq!(body["legacy_approved"], false);
                assert_eq!(
                    body["idempotency_key"],
                    format!("interactive-replay:{expected_cli_operation}")
                );
                assert_eq!(body["lease"]["fencing_token"], 12);
                assert_eq!(body["payload"]["access_token"], RAW_SECRET);
                if expected_api_operation == "keys" {
                    assert_eq!(body["keys"], json!(["CTRL_C", "ENTER"]));
                    assert!(body["text"].is_null());
                } else {
                    assert_eq!(
                        body["text"],
                        format!("operator {expected_cli_operation}: {RAW_SECRET}")
                    );
                    assert!(body["keys"].is_null());
                }
            },
        );
        let stdout = assert_success(&output, server);
        let value: Value = serde_json::from_str(&stdout).expect("one stable JSON envelope");
        assert_eq!(value["data"]["session_id"], session_id);
        assert_eq!(value["data"]["run_id"], run_id);
        assert_eq!(value["data"]["operation"], api_operation);
        assert_eq!(value["data"]["replayed"], false);
        assert_eq!(value["data"]["diagnostic"]["client_secret"], "[REDACTED]");
        assert_eq!(value["side_effects"][0]["operation"], api_operation);
        assert_eq!(value["receipt_refs"].as_array().unwrap().len(), 1);
    }

    for path in [lease_path, payload_path] {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn replay_is_explicit_in_human_output_and_ambiguous_replay_fails_closed() {
    let session_id = Uuid::now_v7().to_string();
    let run_id = Uuid::now_v7().to_string();
    let lease_path = temp_json(
        "replay-lease",
        &lease(&session_id, "actor-instance:operator"),
    );
    let payload_path = temp_json("replay-payload", &json!({}));
    let args = interaction_args(
        "steer",
        &session_id,
        &run_id,
        &lease_path,
        &payload_path,
        false,
    );
    let target =
        format!("/v1/silent-sessions/{session_id}/steer?run_id={run_id}&expected_generation=4");

    let (output, server) = run_mocked_post(
        &args,
        &target,
        "200 OK",
        envelope(&session_id, &run_id, "steer", true, "idempotent_replay"),
        |_| {},
    );
    let human = assert_success(&output, server);
    assert!(human.contains(&format!("Session: {session_id}")));
    assert!(human.contains(&format!("Run: {run_id}")));
    assert!(human.contains("Operation: steer"));
    assert!(human.contains("Replayed: true"));
    assert!(human.contains("Side effects: 0"));
    assert!(human.contains("silent-session-interaction:"));

    let (output, server) = run_mocked_post(
        &args,
        &target,
        "200 OK",
        envelope(&session_id, &run_id, "steer", true, "canonical_result"),
        |_| {},
    );
    server.join().expect("mock daemon validates request");
    assert!(
        !output.status.success(),
        "ambiguous replay must fail closed"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unambiguous replay safety"));
    assert!(!stderr.contains(RAW_SECRET));

    for path in [lease_path, payload_path] {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn authorization_rejection_preserves_failure_envelope_without_secret_output() {
    let session_id = Uuid::now_v7().to_string();
    let run_id = Uuid::now_v7().to_string();
    let lease_path = temp_json(
        "denied-lease",
        &lease(&session_id, "actor-instance:operator"),
    );
    let payload_path = temp_json("denied-payload", &json!({"api_key": RAW_SECRET}));
    let args = interaction_args(
        "send",
        &session_id,
        &run_id,
        &lease_path,
        &payload_path,
        true,
    );
    let target =
        format!("/v1/silent-sessions/{session_id}/input?run_id={run_id}&expected_generation=4");
    let rejection = json!({
        "ok": false,
        "status": "blocked",
        "canonical": false,
        "advisory": false,
        "degraded": true,
        "stale": false,
        "failure_class": "silent_session_authorization_denied",
        "retry": {
            "safe": false,
            "posture": "do_not_retry_unchanged",
            "reason": "authorization_denied"
        },
        "side_effects": [],
        "evidence_refs": [],
        "receipt_refs": [],
        "next_tools": [],
        "recovery_hint": "Refresh the exact principal, approval, and writer lease.",
        "misuse_hint": "Approval is not bound to the exact interaction.",
        "data": {"access_token": RAW_SECRET}
    })
    .to_string();
    let (output, server) = run_mocked_post(&args, &target, "403 Forbidden", rejection, |body| {
        assert_eq!(body["actor_instance_ref"], "actor-instance:operator");
        assert_eq!(body["approval_id"], "approval:send");
        assert_eq!(body["idempotency_key"], "interactive-replay:send");
    });
    server.join().expect("mock daemon validates request");
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains(RAW_SECRET));
    assert!(!stderr.contains(RAW_SECRET));
    let value: Value = serde_json::from_str(&stdout).expect("stable rejection envelope");
    assert_eq!(
        value["failure_class"],
        "silent_session_authorization_denied"
    );
    assert_eq!(value["retry"]["safe"], false);
    assert_eq!(value["side_effects"], json!([]));
    assert_eq!(value["receipt_refs"], json!([]));
    assert_eq!(value["data"]["access_token"], "[REDACTED]");

    for path in [lease_path, payload_path] {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn stale_lease_is_rejected_before_interaction_transport() {
    let session_id = Uuid::now_v7().to_string();
    let run_id = Uuid::now_v7().to_string();
    let mut stale_lease = lease(&session_id, "another-actor-instance");
    stale_lease["expires_at"] = json!((Utc::now() - Duration::minutes(1)).to_rfc3339());
    let lease_path = temp_json("stale-interaction-lease", &stale_lease);
    let payload_path = temp_json("stale-interaction-payload", &json!({}));
    let args = interaction_args(
        "send",
        &session_id,
        &run_id,
        &lease_path,
        &payload_path,
        true,
    );
    let output = Command::new(FOCUSA_BIN)
        .args(args)
        .env("FOCUSA_API_URL", "http://127.0.0.1:1")
        .output()
        .expect("run focusa CLI");
    assert!(!output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("CLI input rejection JSON");
    assert!(
        value["details"]["raw_error"]
            .as_str()
            .unwrap()
            .contains("CLI_STALE_SCOPE")
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains(RAW_SECRET));

    for path in [lease_path, payload_path] {
        let _ = fs::remove_file(path);
    }
}
