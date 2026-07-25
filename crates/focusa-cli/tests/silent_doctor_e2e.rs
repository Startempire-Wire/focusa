//! Spec 133 §24 Silent Session doctor readiness and recovery parity proof.

use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Output};
use std::thread::JoinHandle;

const FOCUSA_BIN: &str = env!("CARGO_BIN_EXE_focusa");
const RAW_SECRET: &str = "sk-doctor-must-never-print";

struct MockResponse {
    target: &'static str,
    status: &'static str,
    body: Value,
}

fn envelope(canonical: bool, status: &str, data: Value) -> Value {
    json!({
        "ok": true,
        "status": status,
        "canonical": canonical,
        "advisory": !canonical,
        "degraded": status == "degraded" || status == "unknown",
        "stale": false,
        "failure_class": null,
        "retry": {
            "safe": canonical,
            "posture": if canonical { "idempotent_read_or_guarded_write" } else { "recover_then_recheck" },
            "reason": if canonical { "canonical_result" } else { "probe_unavailable" }
        },
        "side_effects": [],
        "evidence_refs": [],
        "receipt_refs": [],
        "next_tools": [],
        "recovery_hint": null,
        "misuse_hint": null,
        "data": data,
    })
}

fn blocked_envelope(failure_class: &str, recovery_hint: &str) -> Value {
    json!({
        "ok": false,
        "status": "blocked",
        "canonical": false,
        "advisory": false,
        "degraded": true,
        "stale": false,
        "failure_class": failure_class,
        "retry": {
            "safe": false,
            "posture": "do_not_retry_unchanged",
            "reason": failure_class,
        },
        "side_effects": [],
        "evidence_refs": [],
        "receipt_refs": [],
        "next_tools": [],
        "recovery_hint": recovery_hint,
        "misuse_hint": "Do not infer readiness from a failed probe.",
        "data": null,
    })
}

fn healthy_responses() -> Vec<MockResponse> {
    vec![
        MockResponse {
            target: "/health",
            status: "200 OK",
            body: json!({
                "status": "ok",
                "daemon": {"ok": true, "version": "0.9.test"},
            }),
        },
        MockResponse {
            target: "/harnesses",
            status: "200 OK",
            body: envelope(
                true,
                "observed",
                json!({
                    "schema": "focusa.harness_catalog.v1",
                    "harnesses": [{
                        "harness": "pi",
                        "availability": "available",
                        "freshness": {"status": "live"},
                    }],
                }),
            ),
        },
        MockResponse {
            target: "/providers",
            status: "200 OK",
            body: envelope(
                true,
                "observed",
                json!({
                    "schema": "focusa.provider_catalog.v1",
                    "providers": [{
                        "provider": "openai",
                        "catalog_status": "ready",
                        "auth_status": "authenticated",
                        "entitlement_status": "entitled",
                        "capability_status": "supported",
                    }],
                }),
            ),
        },
        MockResponse {
            target: "/silent-sessions/profiles",
            status: "200 OK",
            body: envelope(
                true,
                "observed",
                json!({
                    "profiles": [{
                        "profile_id": "local_pi_isolated",
                        "persistent_defaults": {"harness": {"kind": "pi"}},
                    }],
                }),
            ),
        },
        MockResponse {
            target: "/silent-sessions/presets",
            status: "200 OK",
            body: envelope(
                true,
                "observed",
                json!({
                    "presets": [{
                        "preset_id": "conservative",
                        "invocation_patch": {"governance": {"destructive_actions_allowed": false}},
                    }],
                }),
            ),
        },
        MockResponse {
            target: "/silent-sessions/capabilities",
            status: "200 OK",
            body: envelope(true, "observed", json!({"capabilities": []})),
        },
    ]
}

fn run_mocked(args: &[&str], responses: Vec<MockResponse>) -> (Output, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock daemon");
    let address = listener.local_addr().expect("mock daemon address");
    let server = std::thread::spawn(move || {
        for response in responses {
            let (mut stream, _) = listener.accept().expect("accept CLI request");
            let mut request = Vec::new();
            let mut buffer = [0u8; 4096];
            loop {
                let count = stream.read(&mut buffer).expect("read CLI request");
                assert!(count > 0, "request ended before headers");
                request.extend_from_slice(&buffer[..count]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            let request = String::from_utf8_lossy(&request);
            assert!(
                request.starts_with(&format!("GET {} HTTP/1.1", response.target)),
                "unexpected request: {request}"
            );
            let body = response.body.to_string();
            write!(
                stream,
                "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response.status,
                body.len(),
                body,
            )
            .expect("write mock daemon response");
        }
    });

    let output = Command::new(FOCUSA_BIN)
        .args(args)
        .env("FOCUSA_API_URL", format!("http://{address}"))
        .env("FOCUSA_API_TIMEOUT", "5")
        .output()
        .expect("run focusa CLI");
    (output, server)
}

fn stdout(output: Output, server: JoinHandle<()>) -> String {
    server.join().expect("mock daemon validates requests");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        output.status.success(),
        "CLI failed\nstdout={stdout}\nstderr={stderr}"
    );
    assert!(!stdout.contains(RAW_SECRET), "stdout leaked a secret");
    assert!(!stderr.contains(RAW_SECRET), "stderr leaked a secret");
    stdout
}

fn check<'a>(report: &'a Value, component: &str) -> &'a Value {
    report["data"]["checks"]
        .as_array()
        .expect("doctor checks array")
        .iter()
        .find(|check| check["component"] == component)
        .unwrap_or_else(|| panic!("missing {component} doctor check"))
}

#[test]
fn doctor_has_stable_human_and_json_readiness_parity() {
    let (output, server) = run_mocked(&["--json", "silent", "doctor"], healthy_responses());
    let json_stdout = stdout(output, server);
    let report: Value = serde_json::from_str(&json_stdout).expect("one JSON doctor report");
    assert_eq!(report["schema"], "focusa.silent_cli_doctor.v1");
    assert_eq!(report["status"], "ready");
    assert_eq!(report["ok"], true);
    assert_eq!(report["canonical"], true);
    assert_eq!(report["degraded"], false);
    assert_eq!(report["side_effects"], json!([]));
    assert_eq!(report["data"]["read_only"], true);
    assert_eq!(report["data"]["checks"].as_array().unwrap().len(), 5);
    for component in ["daemon", "harness", "provider", "config", "capabilities"] {
        let check = check(&report, component);
        assert_eq!(check["status"], "ok", "{component} should be ready");
        assert_eq!(check["retry"]["safe"], true);
    }

    let (output, server) = run_mocked(&["silent", "doctor"], healthy_responses());
    let human = stdout(output, server);
    assert!(human.contains("Silent Session Doctor"));
    assert!(human.contains("Status: ready"));
    for component in ["daemon", "harness", "provider", "config", "capabilities"] {
        assert!(human.contains(&format!("[ok] {component}:")));
    }
    assert!(human.contains("idempotent_recheck"));
}

#[test]
fn doctor_reports_harness_provider_and_config_faults_with_exact_recovery() {
    let mut responses = healthy_responses();
    responses[1] = MockResponse {
        target: "/harnesses",
        status: "503 Service Unavailable",
        body: blocked_envelope(
            "transport_degraded",
            "Reconnect the Pi harness transport, then repeat the harness probe.",
        ),
    };
    responses[2] = MockResponse {
        target: "/providers",
        status: "200 OK",
        body: envelope(
            false,
            "unknown",
            json!({
                "providers": [{
                    "provider": "openai",
                    "catalog_status": "unknown",
                    "auth_status": "unknown",
                    "entitlement_status": "unknown",
                    "capability_status": "unknown",
                    "api_key": RAW_SECRET,
                }],
            }),
        ),
    };
    responses[3] = MockResponse {
        target: "/silent-sessions/profiles",
        status: "200 OK",
        body: envelope(true, "observed", json!({"profiles": []})),
    };

    let (output, server) = run_mocked(&["--json", "silent", "doctor"], responses);
    let output = stdout(output, server);
    let report: Value = serde_json::from_str(&output).expect("one JSON doctor report");
    assert_eq!(report["status"], "blocked");
    assert_eq!(report["canonical"], false);
    assert_eq!(report["degraded"], true);
    assert_eq!(report["retry"]["safe"], false);
    assert_eq!(report["retry"]["posture"], "recover_then_recheck");
    assert_eq!(check(&report, "daemon")["status"], "ok");
    assert_eq!(
        check(&report, "harness")["failure_class"],
        "transport_degraded"
    );
    assert_eq!(
        check(&report, "provider")["failure_class"],
        "provider_unverified"
    );
    assert_eq!(
        check(&report, "config")["failure_class"],
        "config_catalog_invalid"
    );
    for component in ["harness", "provider", "config"] {
        let fault = check(&report, component);
        assert_eq!(fault["retry"]["safe"], false);
        assert!(
            fault["recovery_hint"].as_str().is_some_and(|hint| hint
                .contains("focusa silent doctor")
                || hint.contains("repeat")),
            "{component} fault needs actionable recovery: {fault}"
        );
    }
    assert!(!output.contains(RAW_SECRET));
}

#[test]
fn doctor_turns_daemon_connection_failure_into_bounded_recovery_report() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve unavailable daemon address");
    let address = listener.local_addr().unwrap();
    drop(listener);

    let output = Command::new(FOCUSA_BIN)
        .args(["--json", "silent", "doctor"])
        .env("FOCUSA_API_URL", format!("http://{address}"))
        .env("FOCUSA_API_TIMEOUT", "1")
        .output()
        .expect("run focusa CLI");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "doctor should report transport faults\nstdout={stdout}\nstderr={stderr}"
    );
    let report: Value = serde_json::from_str(&stdout).expect("bounded JSON fault report");
    assert_eq!(report["status"], "blocked");
    assert_eq!(report["failure_class"], "daemon_unreachable");
    assert_eq!(report["data"]["checks"].as_array().unwrap().len(), 4);
    for component in ["daemon", "harness", "provider", "config"] {
        let fault = check(&report, component);
        assert_eq!(fault["status"], "blocked");
        assert!(
            fault["recovery_hint"]
                .as_str()
                .is_some_and(|hint| !hint.is_empty())
        );
    }
}
