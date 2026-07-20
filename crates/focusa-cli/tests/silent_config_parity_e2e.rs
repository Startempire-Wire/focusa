//! Spec 133 §24 config/profile/preset CLI human/JSON parity and CAS/redaction proof.

use serde_json::{Value, json};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::thread::JoinHandle;
use uuid::Uuid;

const FOCUSA_BIN: &str = env!("CARGO_BIN_EXE_focusa");
const RAW_SECRET: &str = "sk-config-must-never-reach-cli-output";

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

fn envelope(status: &str, data: Value, side_effects: Value) -> String {
    json!({
        "ok": true,
        "status": status,
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
        "side_effects": side_effects,
        "evidence_refs": [],
        "receipt_refs": [],
        "next_tools": [],
        "recovery_hint": null,
        "misuse_hint": null,
        "data": data
    })
    .to_string()
}

fn temp_json(prefix: &str, value: &Value) -> PathBuf {
    let path = std::env::temp_dir().join(format!("focusa-{prefix}-{}.json", Uuid::now_v7()));
    fs::write(&path, serde_json::to_vec(value).unwrap()).expect("write JSON fixture");
    path
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
fn profile_and_preset_lists_have_redacted_human_json_parity() {
    let fixtures = [
        (
            "profile",
            "/v1/silent-sessions/profiles",
            "local_pi_isolated",
            envelope(
                "observed",
                json!({"profiles": [{
                    "profile_id": "local_pi_isolated",
                    "persistent_defaults": {
                        "harness": {"kind": "pi"},
                        "api_key": RAW_SECRET
                    }
                }]}),
                json!([]),
            ),
            "Profiles: 1",
        ),
        (
            "preset",
            "/v1/silent-sessions/presets",
            "audit",
            envelope(
                "observed",
                json!({"presets": [{
                    "preset_id": "audit",
                    "invocation_patch": {
                        "governance": {"completion_receipt_required": true},
                        "access_token": RAW_SECRET
                    }
                }]}),
                json!([]),
            ),
            "Presets: 1",
        ),
    ];

    for (command, target, stable_id, response, count_line) in fixtures {
        for json_mode in [false, true] {
            let cli_args = args(json_mode, &[command, "list"]);
            let (output, server) = run_mocked(
                &cli_args,
                "GET",
                target,
                "200 OK",
                response.clone(),
                |body| {
                    assert!(body.is_none());
                },
            );
            let stdout = assert_success(&output, server);
            assert!(stdout.contains(stable_id));
            assert!(stdout.contains("[REDACTED]"));
            if json_mode {
                let value: Value = serde_json::from_str(&stdout).expect("stable JSON envelope");
                assert_eq!(value["status"], "observed");
            } else {
                assert!(stdout.contains("Status: observed"));
                assert!(stdout.contains(count_line));
            }
        }
    }
}

#[test]
fn resolve_diff_apply_and_rollback_expose_hashes_provenance_and_exact_cas() {
    let session_id = Uuid::now_v7().to_string();
    let initial_revision_id = Uuid::now_v7().to_string();
    let applied_revision_id = Uuid::now_v7().to_string();
    let rollback_revision_id = Uuid::now_v7().to_string();
    let rollback_target_id = Uuid::now_v7().to_string();
    let initial_hash = "a".repeat(64);
    let effective_hash = "b".repeat(64);

    let resolve_request = json!({
        "requested_config": {"identity": {"display_name": "safe"}, "api_key": RAW_SECRET},
        "profile_id": "local_pi_isolated"
    });
    let preview_request = json!({
        "expected_revision_id": initial_revision_id,
        "expected_effective_hash": initial_hash,
        "preset_id": "audit",
        "layers": [{"source": "explicit_override", "patch": {"notifications": {"completed": false}}}]
    });
    let apply_request = json!({
        "actor_instance_ref": "operator:config:instance",
        "expected_revision_id": initial_revision_id,
        "expected_effective_hash": initial_hash,
        "operator_approval_ref": "approval:config-apply",
        "layers": [{"source": "operator_edit", "patch": {"notifications": {"completed": false}}}]
    });
    let rollback_request = json!({
        "actor_instance_ref": "operator:config:instance",
        "expected_revision_id": applied_revision_id,
        "expected_effective_hash": effective_hash,
        "target_revision_id": rollback_target_id,
        "operator_approval_ref": "approval:config-rollback"
    });
    let requests = [
        temp_json("config-resolve", &resolve_request),
        temp_json("config-diff", &preview_request),
        temp_json("config-apply", &apply_request),
        temp_json("config-rollback", &rollback_request),
    ];

    let effective_data = json!({
        "requested_config": {"identity": {"display_name": "safe"}},
        "effective_config": {"identity": {"display_name": "safe"}, "api_key": RAW_SECRET},
        "field_provenance": {
            "/identity/display_name": format!("CurrentRevision:{initial_revision_id}"),
            "/notifications/completed": "ExplicitOverride"
        },
        "policy_locks": [{"json_pointer": "/governance/destructive_actions_allowed", "source": "durable_session_policy"}],
        "mutation_classes": {"/notifications/completed": "hot_mutable"},
        "warnings": ["bounded test warning"],
        "validation": {"valid": true, "errors": [], "warnings": ["bounded test warning"]},
        "redacted_config_hash": effective_hash
    });

    for json_mode in [false, true] {
        let path = requests[0].display().to_string();
        let cli_args = args(json_mode, &["config", "resolve", "--request-file", &path]);
        let expected = resolve_request.clone();
        let (output, server) = run_mocked(
            &cli_args,
            "POST",
            "/v1/silent-sessions/config/resolve",
            "200 OK",
            envelope("resolved", effective_data.clone(), json!([])),
            move |body| assert_eq!(body.unwrap(), expected),
        );
        let stdout = assert_success(&output, server);
        assert!(stdout.contains(&effective_hash));
        assert!(stdout.contains("/notifications/completed"));
        assert!(stdout.contains("ExplicitOverride"));
        if json_mode {
            let value: Value = serde_json::from_str(&stdout).expect("stable config JSON");
            assert_eq!(value["data"]["effective_config"]["api_key"], "[REDACTED]");
        } else {
            assert!(stdout.contains("Action: config resolve"));
            assert!(stdout.contains("Validation: true"));
        }
    }

    for json_mode in [false, true] {
        let path = requests[1].display().to_string();
        let cli_args = args(
            json_mode,
            &["config", "diff", &session_id, "--request-file", &path],
        );
        let expected = preview_request.clone();
        let target = format!("/v1/silent-sessions/{session_id}/config/preview");
        let (output, server) = run_mocked(
            &cli_args,
            "POST",
            &target,
            "200 OK",
            envelope("previewed", effective_data.clone(), json!([])),
            move |body| assert_eq!(body.unwrap(), expected),
        );
        let stdout = assert_success(&output, server);
        assert!(stdout.contains(&effective_hash));
        assert!(stdout.contains("/notifications/completed"));
        if !json_mode {
            assert!(stdout.contains("Action: config diff"));
            assert!(stdout.contains("Mutation classes: 1"));
        }
    }

    let apply_data = json!({
        "revision": {
            "schema": "focusa.silent_session_config_revision.v1",
            "revision_id": applied_revision_id,
            "session_id": session_id,
            "parent_revision_id": initial_revision_id,
            "requested_changes": {},
            "effective_diff": {},
            "field_provenance": {"/notifications/completed": "OperatorEdit"},
            "policy_lock_results": {},
            "operator_approval_ref": "approval:config-apply",
            "validation_result": {"valid": true, "errors": [], "warnings": []},
            "applied_at": "2026-07-19T00:00:00Z",
            "rollback_target": null,
            "config": {"api_key": RAW_SECRET}
        },
        "redacted_config_hash": effective_hash
    });
    for json_mode in [false, true] {
        let path = requests[2].display().to_string();
        let cli_args = args(
            json_mode,
            &["config", "apply", &session_id, "--request-file", &path],
        );
        let expected = apply_request.clone();
        let target = format!("/v1/silent-sessions/{session_id}/config/revisions");
        let (output, server) = run_mocked(
            &cli_args,
            "POST",
            &target,
            "200 OK",
            envelope(
                "applied",
                apply_data.clone(),
                json!([{"kind": "config_revision_applied"}]),
            ),
            move |body| assert_eq!(body.unwrap(), expected),
        );
        let stdout = assert_success(&output, server);
        assert!(stdout.contains(&applied_revision_id));
        assert!(stdout.contains(&effective_hash));
        assert!(stdout.contains("OperatorEdit"));
        if json_mode {
            let value: Value = serde_json::from_str(&stdout).expect("stable apply JSON");
            assert_eq!(value["data"]["revision"]["config"]["api_key"], "[REDACTED]");
        } else {
            assert!(stdout.contains("Action: config apply"));
            assert!(stdout.contains("Side effects: 1"));
        }
    }

    let rollback_data = json!({
        "revision": {
            "schema": "focusa.silent_session_config_revision.v1",
            "revision_id": rollback_revision_id,
            "session_id": session_id,
            "parent_revision_id": applied_revision_id,
            "requested_changes": {},
            "effective_diff": {},
            "field_provenance": {"/notifications/completed": "RollbackTarget"},
            "policy_lock_results": {},
            "operator_approval_ref": "approval:config-rollback",
            "validation_result": {"valid": true, "errors": [], "warnings": []},
            "applied_at": "2026-07-19T00:01:00Z",
            "rollback_target": rollback_target_id,
            "config": {"refresh_token": RAW_SECRET}
        },
        "redacted_config_hash": initial_hash
    });
    for json_mode in [false, true] {
        let path = requests[3].display().to_string();
        let cli_args = args(
            json_mode,
            &["config", "rollback", &session_id, "--request-file", &path],
        );
        let expected = rollback_request.clone();
        let target = format!("/v1/silent-sessions/{session_id}/config/rollback");
        let (output, server) = run_mocked(
            &cli_args,
            "POST",
            &target,
            "200 OK",
            envelope(
                "rolled_back",
                rollback_data.clone(),
                json!([{"kind": "config_revision_rolled_back"}]),
            ),
            move |body| assert_eq!(body.unwrap(), expected),
        );
        let stdout = assert_success(&output, server);
        assert!(stdout.contains(&rollback_revision_id));
        assert!(stdout.contains(&rollback_target_id));
        assert!(stdout.contains(&initial_hash));
        if json_mode {
            let value: Value = serde_json::from_str(&stdout).expect("stable rollback JSON");
            assert_eq!(
                value["data"]["revision"]["config"]["refresh_token"],
                "[REDACTED]"
            );
        } else {
            assert!(stdout.contains("Action: config rollback"));
            assert!(stdout.contains("Side effects: 1"));
        }
    }

    for path in requests {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn config_cas_conflicts_are_stable_and_secret_free_in_human_and_json() {
    let session_id = Uuid::now_v7().to_string();
    let revision_id = Uuid::now_v7().to_string();
    let request = json!({
        "actor_instance_ref": "operator:config:instance",
        "expected_revision_id": revision_id,
        "expected_effective_hash": "c".repeat(64),
        "operator_approval_ref": "approval:stale-config",
        "layers": []
    });
    let request_path = temp_json("config-cas-conflict", &request);
    let rejection = json!({
        "ok": false,
        "status": "blocked",
        "canonical": false,
        "advisory": false,
        "degraded": true,
        "stale": false,
        "failure_class": "silent_session_config_conflict",
        "retry": {
            "safe": false,
            "posture": "do_not_retry_unchanged",
            "reason": "stale_config_revision"
        },
        "side_effects": [],
        "evidence_refs": [],
        "receipt_refs": [],
        "next_tools": [],
        "recovery_hint": "Reload the current config revision and effective hash before retrying.",
        "misuse_hint": "Config details are intentionally not echoed.",
        "data": {"access_token": RAW_SECRET}
    })
    .to_string();

    for json_mode in [false, true] {
        let path = request_path.display().to_string();
        let cli_args = args(
            json_mode,
            &["config", "apply", &session_id, "--request-file", &path],
        );
        let expected = request.clone();
        let target = format!("/v1/silent-sessions/{session_id}/config/revisions");
        let (output, server) = run_mocked(
            &cli_args,
            "POST",
            &target,
            "409 Conflict",
            rejection.clone(),
            move |body| assert_eq!(body.unwrap(), expected),
        );
        server.join().expect("mock daemon validates request");
        assert!(!output.status.success(), "CAS conflict must fail");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stdout.contains(RAW_SECRET));
        assert!(!stderr.contains(RAW_SECRET));
        assert!(stdout.contains("silent_session_config_conflict"));
        assert!(stdout.contains("Reload the current config revision"));
        if json_mode {
            let value: Value = serde_json::from_str(&stdout).expect("stable conflict JSON");
            assert_eq!(value["retry"]["safe"], false);
            assert_eq!(value["data"]["access_token"], "[REDACTED]");
        } else {
            assert!(stdout.contains("Status: blocked"));
            assert!(stdout.contains("do_not_retry_unchanged"));
        }
    }

    let _ = fs::remove_file(request_path);
}
