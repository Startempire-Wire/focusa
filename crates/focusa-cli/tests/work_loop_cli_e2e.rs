//! GitHub #132 Work Loop CLI route, scope, and writer-fencing parity.

use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Output};
use std::thread::JoinHandle;

const FOCUSA_BIN: &str = env!("CARGO_BIN_EXE_focusa");

fn run_mocked(
    args: &[&str],
    assertion: impl FnOnce(&str) + Send + 'static,
    body: Value,
) -> (Output, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock daemon");
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept request");
        let mut request = Vec::new();
        let mut buffer = [0u8; 8192];
        loop {
            let count = stream.read(&mut buffer).expect("read request");
            if count == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..count]);
            let headers_done = request.windows(4).any(|window| window == b"\r\n\r\n");
            let expected_body = String::from_utf8_lossy(&request)
                .split("content-length:")
                .nth(1)
                .and_then(|tail| tail.lines().next())
                .and_then(|line| line.trim().parse::<usize>().ok())
                .unwrap_or(0);
            let header_end = request
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .map(|i| i + 4)
                .unwrap_or(0);
            if headers_done && request.len() >= header_end + expected_body {
                break;
            }
        }
        let request = String::from_utf8_lossy(&request).to_string();
        assertion(&request);
        let body = body.to_string();
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
    });
    let output = Command::new(FOCUSA_BIN)
        .args(args)
        .env("FOCUSA_API_URL", format!("http://{address}"))
        .env("FOCUSA_API_TIMEOUT", "5")
        .output()
        .expect("run focusa CLI");
    (output, server)
}

fn assert_success(output: Output, server: JoinHandle<()>) -> Value {
    server.join().expect("mock assertion");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stdout={stdout}\nstderr={stderr}");
    serde_json::from_str(&stdout).expect("one JSON response")
}

fn assert_scope(request: &str) {
    let request = request.to_ascii_lowercase();
    assert!(request.contains("x-scope-project-root: /project"));
    assert!(request.contains("x-scope-continuity-id: spec172"));
}

#[test]
fn status_is_discoverable_and_uses_explicit_scope() {
    let (output, server) = run_mocked(
        &[
            "--json",
            "work-loop",
            "status",
            "--project-root",
            "/project",
            "--continuity-id",
            "spec172",
        ],
        |request| {
            assert!(request.starts_with("GET /v1/work-loop/status?summary_only=true HTTP/1.1"));
            assert_scope(request);
        },
        json!({"schema":"focusa.work_loop_status.v3","status":"idle","canonical":true}),
    );
    let value = assert_success(output, server);
    assert_eq!(value["status"], "idle");
}

#[test]
fn enable_requires_approval_and_returns_the_writer_lease() {
    let (output, server) = run_mocked(
        &[
            "--json",
            "work-loop",
            "enable",
            "--project-root",
            "/project",
            "--continuity-id",
            "spec172",
            "--root-work-item-id",
            "focusa-root",
            "--writer-id",
            "worker-one",
            "--approve",
            "--idempotency-key",
            "enable-001",
        ],
        |request| {
            assert!(request.starts_with("POST /v1/work-loop/enable HTTP/1.1"));
            assert_scope(request);
            let lower = request.to_ascii_lowercase();
            assert!(lower.contains("x-focusa-writer-id: worker-one"));
            assert!(lower.contains("x-focusa-approval: approved"));
            assert!(lower.contains("idempotency-key: enable-001"));
            assert!(request.contains("\"root_work_item_id\":\"focusa-root\""));
        },
        json!({"ok":true,"writer_id":"worker-one","fencing_token":17,"orphaned_scope_recovered":false}),
    );
    let value = assert_success(output, server);
    assert_eq!(value["fencing_token"], 17);
}

#[test]
fn checkpoint_carries_exact_writer_and_fencing_token() {
    let (output, server) = run_mocked(
        &[
            "--json",
            "work-loop",
            "checkpoint",
            "--project-root",
            "/project",
            "--continuity-id",
            "spec172",
            "--writer-id",
            "worker-one",
            "--fencing-token",
            "17",
            "--summary",
            "bounded checkpoint",
            "--idempotency-key",
            "checkpoint-001",
        ],
        |request| {
            assert!(request.starts_with("POST /v1/work-loop/checkpoint HTTP/1.1"));
            assert_scope(request);
            let lower = request.to_ascii_lowercase();
            assert!(lower.contains("x-focusa-writer-id: worker-one"));
            assert!(lower.contains("x-focusa-fencing-token: 17"));
            assert!(lower.contains("idempotency-key: checkpoint-001"));
            assert!(request.contains("\"summary\":\"bounded checkpoint\""));
        },
        json!({"ok":true,"checkpoint_id":"019fd75a-3b5f-7b20-b80c-a8866995259f"}),
    );
    let value = assert_success(output, server);
    assert_eq!(value["ok"], true);
}
