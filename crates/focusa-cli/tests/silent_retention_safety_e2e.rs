//! Spec 133 §24 retention CLI destructive-safety and status-separation proof.

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
const RAW_SECRET: &str = "sk-retention-must-never-print";
const ADMIN_SCOPE: &str = "silent_sessions:admin";
const FORENSICS_SCOPE: &str = "silent_sessions:forensics";

fn run_mocked<F>(
    args: &[String],
    method: &str,
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
            .expect("retention request has a JSON body");
        while request.len() - header_end < content_length {
            let count = stream.read(&mut buffer).expect("read CLI request body");
            assert!(count > 0, "request body ended early");
            request.extend_from_slice(&buffer[..count]);
        }
        let body = serde_json::from_slice(&request[header_end..header_end + content_length])
            .expect("CLI sends JSON body");
        assert_body(body);
        write!(
            stream,
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
            response_body.len()
        )
        .expect("write mock response");
    });

    let output = Command::new(FOCUSA_BIN)
        .args(args)
        .env("FOCUSA_API_URL", format!("http://{address}"))
        .env("FOCUSA_API_TIMEOUT", "5")
        .output()
        .expect("run focusa CLI");
    (output, server)
}

fn temp_context_authority() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "focusa-retention-context-authority-{}.json",
        Uuid::now_v7()
    ));
    fs::write(
        &path,
        serde_json::to_vec(&json!({
            "verdict_ref": "context-authority:retention:fresh",
            "allowed": true,
            "project_identity_ref": "project:test",
            "continuity_id": "continuity:test",
            "workpoint_ref": "workpoint:test",
            "expires_at": (Utc::now() + Duration::hours(1)).to_rfc3339(),
        }))
        .unwrap(),
    )
    .expect("write Context Authority fixture");
    path
}

fn retention_args(
    operation: &str,
    session_id: &str,
    run_id: &str,
    context_path: &Path,
    json_mode: bool,
    dry_run: bool,
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
        "7".into(),
        "--actor-instance-ref".into(),
        "actor-instance:retention-test".into(),
        "--approval-id".into(),
        format!(
            "approval:{operation}:{}",
            if dry_run { "preview" } else { "apply" }
        ),
        "--context-authority-file".into(),
        context_path.display().to_string(),
        if dry_run { "--dry-run" } else { "--apply" }.into(),
        "--reason-code".into(),
        format!("operator_{operation}"),
    ]);
    args
}

fn retention_envelope(
    operation: &str,
    authority_scope: &str,
    session_id: &str,
    run_id: &str,
    dry_run: bool,
    impact_preview_ref: Option<&str>,
) -> String {
    let side_effect = match operation {
        "hold" => "evidence_hold_placed",
        "delete" => "active_projection_deleted",
        "purge" => "silent_session_purged",
        _ => unreachable!(),
    };
    let data = json!({
        "session_id": session_id,
        "run_id": run_id,
        "generation": 7,
        "operation": operation,
        "required_authority_scope": authority_scope,
        "dry_run": dry_run,
        "lifecycle_state": "completed",
        "process_status": "exited",
        "completion_status": "completed",
        "process_termination_performed": false,
        "completion_transition_performed": false,
        "lifecycle_transition_performed": false,
        "impact_preview_ref": impact_preview_ref,
        "evidence_hold_active": operation == "hold" && !dry_run,
        "receipt_integrity_preserved": true,
        "event_integrity_preserved": true,
        "evidence_hold_checked": operation == "purge",
        "purge_eligible": operation == "purge",
        "irreversible": operation == "purge",
        "forensic_reconstruction_may_be_impossible": operation == "purge",
        "api_key": RAW_SECRET,
    });
    json!({
        "ok": true,
        "status": if dry_run { "previewed" } else { "accepted" },
        "canonical": true,
        "advisory": false,
        "degraded": false,
        "stale": false,
        "failure_class": null,
        "retry": {
            "safe": dry_run,
            "posture": if dry_run { "accept_preview_before_commit" } else { "do_not_retry_unchanged" },
            "reason": "canonical_retention_result"
        },
        "side_effects": if dry_run { json!([]) } else { json!([{"kind": side_effect}]) },
        "evidence_refs": ["evidence:retention-safety"],
        "receipt_refs": [format!("silent-session-retention:{operation}:{}", Uuid::now_v7())],
        "next_tools": [],
        "recovery_hint": null,
        "misuse_hint": null,
        "data": data,
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
fn hold_preview_and_apply_require_admin_approval_without_lifecycle_side_effects() {
    let session_id = Uuid::now_v7().to_string();
    let run_id = Uuid::now_v7().to_string();
    let context_path = temp_context_authority();
    let target = format!(
        "/v1/silent-sessions/{session_id}/evidence-hold?run_id={run_id}&expected_generation=7"
    );

    for json_mode in [false, true] {
        for dry_run in [true, false] {
            let preview_ref = dry_run.then_some("retention-preview:hold:stable");
            let response = retention_envelope(
                "hold",
                ADMIN_SCOPE,
                &session_id,
                &run_id,
                dry_run,
                preview_ref,
            );
            let cli_args = retention_args(
                "hold",
                &session_id,
                &run_id,
                &context_path,
                json_mode,
                dry_run,
            );
            let expected_session = session_id.clone();
            let expected_run = run_id.clone();
            let (output, server) = run_mocked(
                &cli_args,
                "POST",
                &target,
                "200 OK",
                response,
                move |body| {
                    assert_eq!(body["operation"], "hold");
                    assert_eq!(body["session_id"], expected_session);
                    assert_eq!(body["run_id"], expected_run);
                    assert_eq!(body["expected_generation"], 7);
                    assert_eq!(
                        body["approval_id"],
                        format!(
                            "approval:hold:{}",
                            if dry_run { "preview" } else { "apply" }
                        )
                    );
                    assert_eq!(body["required_authority_scope"], ADMIN_SCOPE);
                    assert_eq!(body["context_authority"]["allowed"], true);
                    assert_eq!(body["dry_run"], dry_run);
                    assert_eq!(
                        body["side_effect_policy"],
                        if dry_run { "preview" } else { "commit" }
                    );
                    assert_eq!(body["evidence_hold"], true);
                    assert_eq!(body["process_termination_allowed"], false);
                    assert_eq!(body["completion_transition_allowed"], false);
                    assert_eq!(body["lifecycle_transition_allowed"], false);
                    assert_eq!(body["legacy_approved"], false);
                    assert!(body["confirmation"].is_null());
                },
            );
            let stdout = assert_success(&output, server);
            assert!(stdout.contains(&session_id));
            assert!(stdout.contains(&run_id));
            assert!(stdout.contains("exited"));
            assert!(stdout.contains("completed"));
            if json_mode {
                let value: Value = serde_json::from_str(&stdout).expect("retention JSON");
                assert_eq!(value["data"]["api_key"], "[REDACTED]");
                assert_eq!(value["data"]["dry_run"], dry_run);
            } else {
                assert!(stdout.contains(if dry_run {
                    "Side-effect policy: dry-run"
                } else {
                    "Side-effect policy: apply"
                }));
                assert!(stdout.contains("Process status: exited"));
                assert!(stdout.contains("Completion status: completed"));
                assert!(stdout.contains("Receipts: 1"));
            }
        }
    }
    fs::remove_file(context_path).ok();
}

#[test]
fn ordinary_delete_is_preview_bound_and_uses_delete_without_terminating_or_completing() {
    let session_id = Uuid::now_v7().to_string();
    let run_id = Uuid::now_v7().to_string();
    let context_path = temp_context_authority();
    let preview_ref = "retention-preview:delete:stable";
    let target = format!("/v1/silent-sessions/{session_id}?run_id={run_id}&expected_generation=7");

    for json_mode in [false, true] {
        for dry_run in [true, false] {
            let response = retention_envelope(
                "delete",
                ADMIN_SCOPE,
                &session_id,
                &run_id,
                dry_run,
                Some(preview_ref),
            );
            let mut cli_args = retention_args(
                "delete",
                &session_id,
                &run_id,
                &context_path,
                json_mode,
                dry_run,
            );
            if !dry_run {
                cli_args.extend([
                    "--impact-preview-ref".into(),
                    preview_ref.into(),
                    "--confirm-delete".into(),
                    session_id.clone(),
                ]);
            }
            let expected_session = session_id.clone();
            let (output, server) = run_mocked(
                &cli_args,
                "DELETE",
                &target,
                "200 OK",
                response,
                move |body| {
                    assert_eq!(body["operation"], "delete");
                    assert_eq!(body["required_authority_scope"], ADMIN_SCOPE);
                    assert_eq!(body["dry_run"], dry_run);
                    assert_eq!(body["process_termination_allowed"], false);
                    assert_eq!(body["completion_transition_allowed"], false);
                    if dry_run {
                        assert!(body["impact_preview_ref"].is_null());
                        assert!(body["confirmation"].is_null());
                    } else {
                        assert_eq!(body["impact_preview_ref"], preview_ref);
                        assert_eq!(body["confirmation"]["session_id"], expected_session);
                        assert_eq!(
                            body["confirmation"]["active_projection_removal_acknowledged"],
                            true
                        );
                        assert_eq!(
                            body["confirmation"]["irreversible_forensic_loss_acknowledged"],
                            false
                        );
                    }
                },
            );
            let stdout = assert_success(&output, server);
            assert!(stdout.contains("completed"));
            if !dry_run {
                assert!(stdout.contains(preview_ref));
            }
        }
    }
    fs::remove_file(context_path).ok();
}

#[test]
fn purge_requires_forensics_scope_preview_binding_hold_check_and_irreversible_acknowledgement() {
    let session_id = Uuid::now_v7().to_string();
    let run_id = Uuid::now_v7().to_string();
    let context_path = temp_context_authority();
    let preview_ref = "retention-preview:purge:stable";
    let target =
        format!("/v1/silent-sessions/{session_id}/purge?run_id={run_id}&expected_generation=7");

    for json_mode in [false, true] {
        for dry_run in [true, false] {
            let response = retention_envelope(
                "purge",
                FORENSICS_SCOPE,
                &session_id,
                &run_id,
                dry_run,
                Some(preview_ref),
            );
            let mut cli_args = retention_args(
                "purge",
                &session_id,
                &run_id,
                &context_path,
                json_mode,
                dry_run,
            );
            if !dry_run {
                cli_args.extend([
                    "--impact-preview-ref".into(),
                    preview_ref.into(),
                    "--confirm-irreversible-purge".into(),
                    session_id.clone(),
                ]);
            }
            let expected_session = session_id.clone();
            let (output, server) = run_mocked(
                &cli_args,
                "POST",
                &target,
                "200 OK",
                response,
                move |body| {
                    assert_eq!(body["operation"], "purge");
                    assert_eq!(body["required_authority_scope"], FORENSICS_SCOPE);
                    assert_eq!(
                        body["context_authority"]["verdict_ref"],
                        "context-authority:retention:fresh"
                    );
                    assert_eq!(body["process_termination_allowed"], false);
                    assert_eq!(body["completion_transition_allowed"], false);
                    if dry_run {
                        assert!(body["confirmation"].is_null());
                    } else {
                        assert_eq!(body["impact_preview_ref"], preview_ref);
                        assert_eq!(body["confirmation"]["session_id"], expected_session);
                        assert_eq!(
                            body["confirmation"]["irreversible_forensic_loss_acknowledged"],
                            true
                        );
                        assert_eq!(
                            body["confirmation"]["active_projection_removal_acknowledged"],
                            false
                        );
                    }
                },
            );
            let stdout = assert_success(&output, server);
            assert!(stdout.contains("completed"));
            if json_mode {
                let value: Value = serde_json::from_str(&stdout).unwrap();
                assert_eq!(
                    value["data"]["forensic_reconstruction_may_be_impossible"],
                    true
                );
            } else {
                assert!(stdout.contains("Forensic reconstruction may be impossible: true"));
            }
        }
    }
    fs::remove_file(context_path).ok();
}

fn run_local_failure(args: &[String]) -> Output {
    Command::new(FOCUSA_BIN)
        .args(args)
        .env("FOCUSA_API_URL", "http://127.0.0.1:1")
        .env("FOCUSA_API_TIMEOUT", "1")
        .output()
        .expect("run local retention rejection")
}

#[test]
fn destructive_commands_fail_locally_without_explicit_mode_preview_and_exact_confirmation() {
    let session_id = Uuid::now_v7().to_string();
    let wrong_session_id = Uuid::now_v7().to_string();
    let run_id = Uuid::now_v7().to_string();
    let context_path = temp_context_authority();

    let mut no_mode = retention_args("delete", &session_id, &run_id, &context_path, false, true);
    no_mode.retain(|arg| arg != "--dry-run");
    let output = run_local_failure(&no_mode);
    assert!(!output.status.success(), "retention mode must be explicit");
    assert!(String::from_utf8_lossy(&output.stderr).contains("--dry-run"));

    let missing_preview =
        retention_args("delete", &session_id, &run_id, &context_path, false, false);
    let output = run_local_failure(&missing_preview);
    assert!(
        !output.status.success(),
        "delete apply needs a dry-run preview"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("--impact-preview-ref"));

    let mut wrong_confirmation =
        retention_args("purge", &session_id, &run_id, &context_path, false, false);
    wrong_confirmation.extend([
        "--impact-preview-ref".into(),
        "retention-preview:purge:stable".into(),
        "--confirm-irreversible-purge".into(),
        wrong_session_id,
    ]);
    let output = run_local_failure(&wrong_confirmation);
    assert!(
        !output.status.success(),
        "purge confirmation must bind exact session"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("must exactly equal the canonical session_id")
    );

    let mut ambiguous_preview =
        retention_args("delete", &session_id, &run_id, &context_path, false, true);
    ambiguous_preview.extend([
        "--impact-preview-ref".into(),
        "retention-preview:delete:stable".into(),
        "--confirm-delete".into(),
        session_id,
    ]);
    let output = run_local_failure(&ambiguous_preview);
    assert!(
        !output.status.success(),
        "dry-run must reject commit acknowledgements"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("dry-run cannot accept"));

    fs::remove_file(context_path).ok();
}

#[test]
fn daemon_responses_fail_closed_on_termination_dry_run_effects_or_held_purge() {
    let session_id = Uuid::now_v7().to_string();
    let run_id = Uuid::now_v7().to_string();
    let context_path = temp_context_authority();

    let mut conflated: Value = serde_json::from_str(&retention_envelope(
        "delete",
        ADMIN_SCOPE,
        &session_id,
        &run_id,
        true,
        Some("retention-preview:delete:unsafe"),
    ))
    .unwrap();
    conflated["data"]["process_termination_performed"] = json!(true);
    let target = format!("/v1/silent-sessions/{session_id}?run_id={run_id}&expected_generation=7");
    let cli_args = retention_args("delete", &session_id, &run_id, &context_path, true, true);
    let (output, server) = run_mocked(
        &cli_args,
        "DELETE",
        &target,
        "200 OK",
        conflated.to_string(),
        |_| {},
    );
    server.join().expect("mock daemon responds");
    assert!(
        !output.status.success(),
        "retention cannot terminate a process"
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("silent-session-retention:delete"),
        "unsafe retention receipt must not be rendered: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let mut effectful_preview: Value = serde_json::from_str(&retention_envelope(
        "hold",
        ADMIN_SCOPE,
        &session_id,
        &run_id,
        true,
        Some("retention-preview:hold:unsafe"),
    ))
    .unwrap();
    effectful_preview["side_effects"] = json!([{"kind": "evidence_hold_placed"}]);
    let target = format!(
        "/v1/silent-sessions/{session_id}/evidence-hold?run_id={run_id}&expected_generation=7"
    );
    let cli_args = retention_args("hold", &session_id, &run_id, &context_path, true, true);
    let (output, server) = run_mocked(
        &cli_args,
        "POST",
        &target,
        "200 OK",
        effectful_preview.to_string(),
        |_| {},
    );
    server.join().expect("mock daemon responds");
    assert!(
        !output.status.success(),
        "dry-run cannot report side effects"
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("silent-session-retention:hold"),
        "effectful preview receipt must not be rendered: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let preview_ref = "retention-preview:purge:held";
    let mut held_purge: Value = serde_json::from_str(&retention_envelope(
        "purge",
        FORENSICS_SCOPE,
        &session_id,
        &run_id,
        false,
        Some(preview_ref),
    ))
    .unwrap();
    held_purge["data"]["evidence_hold_active"] = json!(true);
    held_purge["data"]["purge_eligible"] = json!(false);
    let target =
        format!("/v1/silent-sessions/{session_id}/purge?run_id={run_id}&expected_generation=7");
    let mut cli_args = retention_args("purge", &session_id, &run_id, &context_path, true, false);
    cli_args.extend([
        "--impact-preview-ref".into(),
        preview_ref.into(),
        "--confirm-irreversible-purge".into(),
        session_id,
    ]);
    let (output, server) = run_mocked(
        &cli_args,
        "POST",
        &target,
        "200 OK",
        held_purge.to_string(),
        |_| {},
    );
    server.join().expect("mock daemon responds");
    assert!(
        !output.status.success(),
        "active Evidence hold must block purge"
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("silent-session-retention:purge"),
        "held purge receipt must not be rendered: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_file(context_path).ok();
}
