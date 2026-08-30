//! Spec 133 §24 read/watch/output CLI parity over daemon APIs.

use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Output};
use std::thread::JoinHandle;
use uuid::Uuid;

const FOCUSA_BIN: &str = env!("CARGO_BIN_EXE_focusa");

fn run_mocked(
    args: &[&str],
    expected_request_target: &str,
    expected_header: Option<(&str, &str)>,
    status: &str,
    content_type: &str,
    body: String,
) -> (Output, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock daemon");
    let address = listener.local_addr().expect("mock daemon address");
    let expected_request_target = expected_request_target.to_string();
    let expected_header = expected_header
        .map(|(name, value)| (name.to_ascii_lowercase(), value.to_ascii_lowercase()));
    let status = status.to_string();
    let content_type = content_type.to_string();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept CLI request");
        let mut request = Vec::new();
        let mut buffer = [0u8; 4096];
        loop {
            let count = stream.read(&mut buffer).expect("read CLI request");
            if count == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..count]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }
        let request = String::from_utf8_lossy(&request);
        assert!(
            request.starts_with(&format!("GET {expected_request_target} HTTP/1.1")),
            "unexpected request: {request}"
        );
        if let Some((name, value)) = expected_header {
            let lower = request.to_ascii_lowercase();
            assert!(
                lower.contains(&format!("{name}: {value}")),
                "missing expected header in: {request}"
            );
        }
        write!(
            stream,
            "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
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

fn success(data: Value) -> String {
    json!({
        "ok": true,
        "status": "observed",
        "canonical": true,
        "advisory": false,
        "degraded": false,
        "stale": false,
        "failure_class": null,
        "retry": {"safe": true, "posture": "idempotent_read_or_guarded_write", "reason": "canonical_result"},
        "side_effects": [],
        "evidence_refs": [],
        "receipt_refs": [],
        "next_tools": [],
        "recovery_hint": null,
        "misuse_hint": null,
        "data": data
    })
    .to_string()
}

fn assert_success(output: &Output, server: JoinHandle<()>) -> String {
    server.join().expect("mock daemon should validate request");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        output.status.success(),
        "CLI failed\nstdout={stdout}\nstderr={stderr}"
    );
    stdout
}

#[test]
fn list_show_status_and_output_use_bounded_exact_daemon_routes() {
    let (session_id, run_id) = exact_ids();

    let list_body = success(json!({
        "schema": "focusa.silent_session_list.v1",
        "count": 1,
        "limit": 7,
        "sessions": [{
            "session_id": session_id,
            "active_run_id": run_id,
            "active_run_generation": 3,
            "display_name": "parity session",
            "lifecycle_state": "running",
            "process_status": "running",
            "completion_status": "not_completed",
            "health": "healthy",
            "api_key": "list-secret"
        }]
    }));
    let (output, server) = run_mocked(
        &["--json", "silent", "list", "--limit", "7"],
        "/v1/silent-sessions?limit=7",
        None,
        "200 OK",
        "application/json",
        list_body,
    );
    let stdout = assert_success(&output, server);
    assert!(!stdout.contains("list-secret"));
    assert!(stdout.contains("[REDACTED]"));

    let show_body = success(json!({
        "session": {
            "session_id": session_id,
            "display_name": "parity session",
            "lifecycle_state": "running"
        },
        "run": {
            "run_id": run_id,
            "generation": 3,
            "started_at": "2026-07-19T00:00:00Z",
            "ended_at": null,
            "exit_status": null
        }
    }));
    let show_target = format!("/v1/silent-sessions/{session_id}");
    let (output, server) = run_mocked(
        &["--json", "silent", "show", &session_id],
        &show_target,
        None,
        "200 OK",
        "application/json",
        show_body.clone(),
    );
    let stdout = assert_success(&output, server);
    let value: Value = serde_json::from_str(&stdout).expect("show emits one JSON envelope");
    assert_eq!(value["data"]["session"]["session_id"], session_id);
    assert_eq!(value["data"]["run"]["run_id"], run_id);

    let (output, server) = run_mocked(
        &["--json", "silent", "show", &session_id, "--run", &run_id],
        &show_target,
        None,
        "200 OK",
        "application/json",
        show_body,
    );
    assert_success(&output, server);

    let status_body = success(json!({
        "session_id": session_id,
        "run_id": run_id,
        "generation": 3,
        "lifecycle_state": "running",
        "process_status": "running",
        "completion_status": "not_completed",
        "health": "healthy",
        "current_event_seq": 8,
        "started_at": "2026-07-19T00:00:00Z",
        "ended_at": null,
        "exit_status": null
    }));
    let status_target =
        format!("/v1/silent-sessions/{session_id}/status?run_id={run_id}&generation=3");
    let (output, server) = run_mocked(
        &[
            "silent",
            "status",
            &session_id,
            "--run-id",
            &run_id,
            "--generation",
            "3",
        ],
        &status_target,
        None,
        "200 OK",
        "application/json",
        status_body,
    );
    let stdout = assert_success(&output, server);
    assert!(stdout.contains("Lifecycle state: running"));
    assert!(stdout.contains("Process status: running"));
    assert!(stdout.contains("Completion status: not_completed"));

    let event_id = Uuid::now_v7().to_string();
    let output_body = success(json!({
        "session_id": session_id,
        "run_id": run_id,
        "generation": 3,
        "stream_refs": ["stream:test"],
        "after_cursor": "opaque/cursor?x",
        "next_cursor": event_id,
        "limit": 5,
        "has_more": false,
        "events": [{
            "event_id": event_id,
            "session_id": session_id,
            "run_id": run_id,
            "seq": 8,
            "kind": "stream.stdout",
            "payload": {"text": "safe output", "access_token": "output-secret"}
        }]
    }));
    let output_target = format!(
        "/v1/silent-sessions/{session_id}/output?run_id={run_id}&generation=3&follow=false&limit=5&after=opaque%2Fcursor%3Fx"
    );
    let (output, server) = run_mocked(
        &[
            "--json",
            "silent",
            "output",
            &session_id,
            "--run",
            &run_id,
            "--generation",
            "3",
            "--after",
            "opaque/cursor?x",
            "--limit",
            "5",
        ],
        &output_target,
        None,
        "200 OK",
        "application/json",
        output_body,
    );
    let stdout = assert_success(&output, server);
    assert!(!stdout.contains("output-secret"));
    let value: Value = serde_json::from_str(&stdout).expect("output emits one JSON envelope");
    assert_eq!(value["data"]["after_cursor"], "opaque/cursor?x");
    assert_eq!(value["data"]["next_cursor"], event_id);
    assert_eq!(
        value["data"]["events"][0]["payload"]["access_token"],
        "[REDACTED]"
    );
}

#[test]
fn watch_preserves_cursor_and_advances_across_tool_filters() {
    let (session_id, run_id) = exact_ids();
    let tool_event_id = Uuid::now_v7().to_string();
    let stderr_event_id = Uuid::now_v7().to_string();
    let body = success(json!({
        "session_id": session_id,
        "run_id": run_id,
        "generation": 3,
        "events": [{
            "event_id": tool_event_id,
            "session_id": session_id,
            "run_id": run_id,
            "seq": 10,
            "kind": "tool.output",
            "payload": {"text": "safe tool output", "api_key": "watch-secret"}
        }, {
            "event_id": stderr_event_id,
            "session_id": session_id,
            "run_id": run_id,
            "seq": 11,
            "kind": "stream.stderr",
            "payload": {"text": "diagnostic"}
        }],
        "next_cursor": stderr_event_id,
        "has_more": false
    }));
    let target = format!(
        "/v1/silent-sessions/{session_id}/events?run_id={run_id}&generation=3&cursor=opaque-before&limit=2&follow=false"
    );
    let (output, server) = run_mocked(
        &[
            "--json",
            "silent",
            "watch",
            &session_id,
            "--run",
            &run_id,
            "--generation",
            "3",
            "--after",
            "opaque-before",
            "--limit",
            "2",
            "--tools",
        ],
        &target,
        None,
        "200 OK",
        "application/json",
        body,
    );
    let stdout = assert_success(&output, server);
    assert!(!stdout.contains("watch-secret"));
    let value: Value =
        serde_json::from_str(&stdout).expect("watch emits one bounded JSON envelope");
    assert_eq!(value["data"]["after_cursor"], "opaque-before");
    assert_eq!(value["data"]["next_cursor"], stderr_event_id);
    assert_eq!(value["data"]["event_count"], 1);
    assert_eq!(value["data"]["events"][0]["event_id"], tool_event_id);
    assert_eq!(
        value["data"]["events"][0]["payload"]["api_key"],
        "[REDACTED]"
    );
}

#[test]
fn empty_session_list_is_canonical_success_not_transport_failure() {
    let body = success(json!({
        "schema": "focusa.silent_session_list.v1",
        "count": 0,
        "limit": 50,
        "sessions": []
    }));
    let (output, server) = run_mocked(
        &["--json", "silent", "list"],
        "/v1/silent-sessions?limit=50",
        None,
        "200 OK",
        "application/json",
        body,
    );
    let stdout = assert_success(&output, server);
    let value: Value = serde_json::from_str(&stdout).expect("empty list emits canonical JSON");
    assert_eq!(value["status"], "observed");
    assert_eq!(value["result"]["ok"], true);
    assert_eq!(value["result"]["data"]["count"], 0);
    assert_eq!(value["result"]["data"]["sessions"], json!([]));
}

#[test]
fn cross_run_daemon_response_fails_closed_without_leaking_payload() {
    let (session_id, run_id) = exact_ids();
    let (_, wrong_run_id) = exact_ids();
    let body = success(json!({
        "session_id": session_id,
        "run_id": wrong_run_id,
        "generation": 1,
        "lifecycle_state": "running",
        "process_status": "running",
        "completion_status": "not_completed",
        "api_key": "cross-run-secret"
    }));
    let target = format!("/v1/silent-sessions/{session_id}/status?run_id={run_id}&generation=1");
    let (output, server) = run_mocked(
        &[
            "--json",
            "silent",
            "status",
            &session_id,
            "--run",
            &run_id,
            "--generation",
            "1",
        ],
        &target,
        None,
        "200 OK",
        "application/json",
        body,
    );
    server.join().expect("mock daemon validates exact request");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "cross-run response must fail closed"
    );
    assert!(!stdout.contains("cross-run-secret"));
    assert!(!stderr.contains("cross-run-secret"));
    assert!(stdout.contains("API_DECODE_ERROR"));
}

#[test]
fn daemon_rejection_keeps_shared_envelope_redacted_and_exits_nonzero() {
    let (session_id, run_id) = exact_ids();
    let body = json!({
        "ok": false,
        "status": "blocked",
        "canonical": false,
        "advisory": false,
        "degraded": true,
        "stale": true,
        "failure_class": "event_cursor_not_found",
        "retry": {"safe": false, "posture": "do_not_retry_unchanged", "reason": "wrong_run"},
        "side_effects": [],
        "evidence_refs": [],
        "receipt_refs": [],
        "next_tools": [],
        "recovery_hint": "Reload the exact run cursor.",
        "misuse_hint": "Cursor belonged to another run.",
        "data": {"access_token": "rejection-secret"}
    })
    .to_string();
    let target = format!(
        "/v1/silent-sessions/{session_id}/output?run_id={run_id}&generation=1&follow=false&limit=3"
    );
    let (output, server) = run_mocked(
        &[
            "--json",
            "silent",
            "output",
            &session_id,
            "--run",
            &run_id,
            "--generation",
            "1",
            "--limit",
            "3",
        ],
        &target,
        None,
        "409 Conflict",
        "application/json",
        body,
    );
    server
        .join()
        .expect("mock daemon validates rejection request");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !output.status.success(),
        "daemon rejection must exit nonzero"
    );
    assert!(!stdout.contains("rejection-secret"));
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(combined.contains("event_cursor_not_found"));
    assert!(combined.contains("[REDACTED]"));
}
