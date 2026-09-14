//! Spec 133 §24 Evidence, receipt, and export CLI exact-run parity proof.

use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Output};
use std::thread::JoinHandle;
use uuid::Uuid;

const FOCUSA_BIN: &str = env!("CARGO_BIN_EXE_focusa");
const RAW_SECRET: &str = "sk-proof-export-must-never-print";

fn run_mocked<F>(
    args: &[String],
    method: &str,
    expected_target: &str,
    status: &str,
    response_body: String,
    assert_body: F,
) -> (Output, JoinHandle<()>)
where
    F: FnOnce(Option<Value>) + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock daemon");
    let address = listener.local_addr().expect("mock daemon address");
    let method = method.to_string();
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
            headers.starts_with(&format!("{method} {expected_target} HTTP/1.1")),
            "unexpected request: {headers}"
        );
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().expect("content length"))
            })
            .unwrap_or(0);
        while request.len() - header_end < content_length {
            let count = stream.read(&mut buffer).expect("read CLI request body");
            assert!(count > 0, "request body ended early");
            request.extend_from_slice(&buffer[..count]);
        }
        let body = (content_length > 0).then(|| {
            serde_json::from_slice(&request[header_end..header_end + content_length])
                .expect("CLI sends JSON body")
        });
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

fn args(json_mode: bool, tail: &[&str]) -> Vec<String> {
    let mut args = Vec::new();
    if json_mode {
        args.push("--json".into());
    }
    args.push("silent".into());
    args.extend(tail.iter().map(|value| (*value).to_string()));
    args
}

fn envelope(data: Value, evidence_refs: Value, receipt_refs: Value) -> String {
    json!({
        "ok": true,
        "status": "observed",
        "canonical": true,
        "advisory": false,
        "degraded": false,
        "stale": false,
        "failure_class": null,
        "retry": {
            "safe": true,
            "posture": "idempotent_read_or_guarded_write",
            "reason": "canonical_result"
        },
        "side_effects": [],
        "evidence_refs": evidence_refs,
        "receipt_refs": receipt_refs,
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
    assert!(!stdout.contains(RAW_SECRET), "stdout leaked secret");
    assert!(!stderr.contains(RAW_SECRET), "stderr leaked secret");
    stdout
}

#[test]
fn evidence_and_receipts_are_bounded_redacted_and_exact_run_scoped_in_both_modes() {
    let session_id = Uuid::now_v7().to_string();
    let run_id = Uuid::now_v7().to_string();
    let next_cursor = Uuid::now_v7().to_string();
    let evidence_ref =
        "artifact:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let receipt_ref = format!("silent-session-lifecycle:{next_cursor}");

    for json_mode in [false, true] {
        let response = envelope(
            json!({
                "session_id": session_id,
                "run_id": run_id,
                "generation": 4,
                "after_cursor": "opaque/cursor?1",
                "next_cursor": next_cursor,
                "limit": 3,
                "has_more": true,
                "artifact_refs": [evidence_ref],
                "api_key": RAW_SECRET
            }),
            json!([evidence_ref]),
            json!([]),
        );
        let target = format!(
            "/v1/silent-sessions/{session_id}/artifacts?run_id={run_id}&generation=4&limit=3&after=opaque%2Fcursor%3F1"
        );
        let cli_args = args(
            json_mode,
            &[
                "evidence",
                &session_id,
                "--run",
                &run_id,
                "--generation",
                "4",
                "--after",
                "opaque/cursor?1",
                "--limit",
                "3",
            ],
        );
        let (output, server) = run_mocked(&cli_args, "GET", &target, "200 OK", response, |body| {
            assert!(body.is_none())
        });
        let stdout = assert_success(&output, server);
        assert!(stdout.contains(&session_id));
        assert!(stdout.contains(&run_id));
        assert!(stdout.contains(evidence_ref));
        if json_mode {
            let value: Value = serde_json::from_str(&stdout).expect("Evidence JSON envelope");
            assert_eq!(value["data"]["limit"], 3);
            assert_eq!(value["data"]["has_more"], true);
            assert_eq!(value["data"]["api_key"], "[REDACTED]");
        } else {
            assert!(stdout.contains("Evidence: 1"));
            assert!(stdout.contains("Has more: true"));
        }
    }

    for json_mode in [false, true] {
        let response = envelope(
            json!({
                "session_id": session_id,
                "run_id": run_id,
                "generation": 4,
                "after_cursor": null,
                "next_cursor": next_cursor,
                "limit": 2,
                "has_more": false,
                "receipt_refs": [receipt_ref],
                "events": [{
                    "event_id": next_cursor,
                    "session_id": session_id,
                    "run_id": run_id,
                    "seq": 9,
                    "kind": "receipt.committed",
                    "payload": {"access_token": RAW_SECRET}
                }]
            }),
            json!([]),
            json!([receipt_ref]),
        );
        let target = format!(
            "/v1/silent-sessions/{session_id}/receipts?run_id={run_id}&generation=4&limit=2"
        );
        let cli_args = args(
            json_mode,
            &[
                "receipt",
                &session_id,
                "--run-id",
                &run_id,
                "--generation",
                "4",
                "--limit",
                "2",
            ],
        );
        let (output, server) = run_mocked(&cli_args, "GET", &target, "200 OK", response, |body| {
            assert!(body.is_none())
        });
        let stdout = assert_success(&output, server);
        assert!(stdout.contains(&session_id));
        assert!(stdout.contains(&run_id));
        assert!(stdout.contains(&receipt_ref));
        if json_mode {
            let value: Value = serde_json::from_str(&stdout).expect("receipt JSON envelope");
            assert_eq!(
                value["data"]["events"][0]["payload"]["access_token"],
                "[REDACTED]"
            );
        } else {
            assert!(stdout.contains("Receipts: 1"));
            assert!(stdout.contains("kind=receipt.committed"));
        }
    }
}

#[test]
fn evidence_and_receipt_require_exact_generation_before_network_access() {
    let session_id = Uuid::now_v7().to_string();
    let run_id = Uuid::now_v7().to_string();

    for command in ["evidence", "receipt"] {
        let output = Command::new(FOCUSA_BIN)
            .args(["silent", command, &session_id, "--run", &run_id])
            .output()
            .expect("run missing-generation parser check");
        assert!(
            !output.status.success(),
            "missing generation must fail closed"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("--generation"),
            "CLI must identify the missing exact generation"
        );
    }
}

#[test]
fn export_posts_only_a_redacted_bounded_exact_run_request_with_human_json_parity() {
    let session_id = Uuid::now_v7().to_string();
    let run_id = Uuid::now_v7().to_string();
    let next_cursor = Uuid::now_v7().to_string();
    let export_ref = "silent-session-export:sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let artifact_ref =
        "artifact:sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    let manifest_sha256 = "d".repeat(64);

    for json_mode in [false, true] {
        let response = envelope(
            json!({
                "schema": "focusa.silent_session_export.v1",
                "session_id": session_id,
                "run_id": run_id,
                "generation": 4,
                "after_cursor": "export/cursor?1",
                "next_cursor": next_cursor,
                "limit": 5,
                "event_count": 5,
                "has_more": true,
                "redacted": true,
                "export_ref": export_ref,
                "manifest_sha256": manifest_sha256,
                "artifact_refs": [artifact_ref],
                "credentials": RAW_SECRET
            }),
            json!([artifact_ref]),
            json!(["receipt:export-proof"]),
        );
        let target = format!("/v1/silent-sessions/{session_id}/export?run_id={run_id}");
        let cli_args = args(
            json_mode,
            &[
                "export",
                &session_id,
                "--run",
                &run_id,
                "--after",
                "export/cursor?1",
                "--limit",
                "5",
            ],
        );
        let expected_run_id = run_id.clone();
        let (output, server) = run_mocked(
            &cli_args,
            "POST",
            &target,
            "200 OK",
            response,
            move |body| {
                let body = body.expect("export request body");
                assert_eq!(body["schema"], "focusa.silent_session_export_request.v1");
                assert_eq!(body["run_id"], expected_run_id);
                assert_eq!(body["after_cursor"], "export/cursor?1");
                assert_eq!(body["event_limit"], 5);
                assert_eq!(body["redaction_required"], true);
            },
        );
        let stdout = assert_success(&output, server);
        assert!(stdout.contains(&session_id));
        assert!(stdout.contains(&run_id));
        assert!(stdout.contains(export_ref));
        assert!(stdout.contains(artifact_ref));
        if json_mode {
            let value: Value = serde_json::from_str(&stdout).expect("export JSON envelope");
            assert_eq!(value["data"]["redacted"], true);
            assert_eq!(value["data"]["credentials"], "[REDACTED]");
        } else {
            assert!(stdout.contains("Redacted: true"));
            assert!(stdout.contains(&format!("Manifest SHA-256: {manifest_sha256}")));
            assert!(stdout.contains("Has more: true"));
        }
    }
}

#[test]
fn proof_and_export_fail_closed_on_cross_run_or_unbounded_daemon_responses() {
    let session_id = Uuid::now_v7().to_string();
    let run_id = Uuid::now_v7().to_string();
    let wrong_run_id = Uuid::now_v7().to_string();
    let response = envelope(
        json!({
            "session_id": session_id,
            "run_id": wrong_run_id,
            "generation": 1,
            "after_cursor": null,
            "next_cursor": null,
            "limit": 1,
            "has_more": false,
            "artifact_refs": ["artifact:wrong-run"]
        }),
        json!(["artifact:wrong-run"]),
        json!([]),
    );
    let target =
        format!("/v1/silent-sessions/{session_id}/artifacts?run_id={run_id}&generation=1&limit=1");
    let cli_args = args(
        true,
        &[
            "evidence",
            &session_id,
            "--run",
            &run_id,
            "--generation",
            "1",
            "--limit",
            "1",
        ],
    );
    let (output, server) = run_mocked(&cli_args, "GET", &target, "200 OK", response, |body| {
        assert!(body.is_none())
    });
    server.join().expect("mock daemon validates request");
    assert!(
        !output.status.success(),
        "cross-run response must fail closed"
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("artifact:wrong-run"),
        "cross-run artifact refs must not reach stdout"
    );

    let wrong_generation_response = envelope(
        json!({
            "session_id": session_id,
            "run_id": run_id,
            "generation": 2,
            "limit": 1,
            "artifact_refs": ["artifact:wrong-generation"]
        }),
        json!(["artifact:wrong-generation"]),
        json!([]),
    );
    let generation_target =
        format!("/v1/silent-sessions/{session_id}/artifacts?run_id={run_id}&generation=1&limit=1");
    let generation_args = args(
        true,
        &[
            "evidence",
            &session_id,
            "--run",
            &run_id,
            "--generation",
            "1",
            "--limit",
            "1",
        ],
    );
    let (generation_output, generation_server) = run_mocked(
        &generation_args,
        "GET",
        &generation_target,
        "200 OK",
        wrong_generation_response,
        |body| assert!(body.is_none()),
    );
    generation_server
        .join()
        .expect("mock daemon validates generation request");
    assert!(
        !generation_output.status.success(),
        "cross-generation response must fail closed"
    );
    assert!(
        !String::from_utf8_lossy(&generation_output.stdout).contains("artifact:wrong-generation"),
        "cross-generation artifact refs must not reach stdout"
    );

    let output = Command::new(FOCUSA_BIN)
        .args([
            "silent",
            "export",
            &session_id,
            "--run",
            &run_id,
            "--limit",
            "1001",
        ])
        .output()
        .expect("run bounded export parser check");
    assert!(
        !output.status.success(),
        "unbounded export must fail locally"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("1-1000")
            || String::from_utf8_lossy(&output.stderr).contains("between 1 and 1000")
    );
}
