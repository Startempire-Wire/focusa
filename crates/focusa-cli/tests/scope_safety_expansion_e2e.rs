//! Spec124 Order 10 scope safety expansion tests.

use std::fs;
use std::path::Path;
use std::process::Command;

const FOCUSA_BIN: &str = env!("CARGO_BIN_EXE_focusa");

fn run(args: &[&str]) -> (std::process::Output, String) {
    let output = Command::new(FOCUSA_BIN)
        .args(args)
        .output()
        .expect("focusa CLI should execute");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output, format!("{stdout}{stderr}"))
}

#[test]
fn cleanup_rejects_unsafe_project_root_before_cleanup() {
    let (output, out) = run(&[
        "--json",
        "cleanup",
        "--safe",
        "--project-root",
        "/root",
        "--dry-run",
    ]);
    assert!(
        output.status.success(),
        "scope reject envelope should exit 0: {out}"
    );
    assert!(
        out.contains("CLI_SCOPE_REJECT"),
        "missing scope rejection: {out}"
    );
    assert!(
        out.contains("\"status\": \"blocked\""),
        "missing blocked status: {out}"
    );
    assert!(
        out.contains("unsafe_broad_project_root"),
        "missing broad-root reason: {out}"
    );
}

#[test]
fn context_cognition_rejects_unsafe_project_root_before_api_call() {
    let (output, out) = run(&[
        "context-cognition",
        "view",
        "--project-root",
        "/root",
        "--json",
    ]);
    assert!(
        output.status.success(),
        "scope reject envelope should exit 0: {out}"
    );
    assert!(
        out.contains("CLI_SCOPE_REJECT"),
        "missing scope rejection: {out}"
    );
    assert!(
        out.contains("\"status\": \"blocked\""),
        "missing blocked status: {out}"
    );
    assert!(
        out.contains("unsafe_broad_project_root"),
        "missing broad-root reason: {out}"
    );
}

#[test]
fn call_stack_rejects_unsafe_project_root_before_api_call() {
    let (output, out) = run(&[
        "call-stack",
        "list",
        "--project-root",
        "/root",
        "--limit",
        "1",
        "--json",
    ]);
    assert!(
        output.status.success(),
        "scope reject envelope should exit 0: {out}"
    );
    assert!(
        out.contains("CLI_SCOPE_REJECT"),
        "missing scope rejection: {out}"
    );
    assert!(
        out.contains("\"status\": \"blocked\""),
        "missing blocked status: {out}"
    );
}

#[test]
fn daemon_global_mutations_return_blocked_envelopes_without_api_call() {
    for args in [
        vec!["--json", "memory", "set", "foo=bar"],
        vec!["--json", "memory", "reinforce", "rule-1"],
        vec!["--json", "gate", "pin", "candidate-1"],
        vec!["--json", "gate", "resolve", "candidate-1"],
    ] {
        let (output, out) = run(&args);
        assert!(
            output.status.success(),
            "blocked envelope should exit 0: {args:?} {out}"
        );
        assert!(
            out.contains("\"status\": \"blocked\""),
            "missing blocked status: {out}"
        );
        assert!(
            out.contains("daemon_global_advisory"),
            "missing advisory authority: {out}"
        );
    }
}

#[test]
fn spec124_scope_surfaces_are_guarded_or_declared_global_advisory() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for module in [
        "focus",
        "hlt",
        "context_cognition",
        "memory",
        "turns",
        "audit",
        "cleanup",
        "lineage",
        "clt",
        "call_stack",
        "gate",
        "workpoint",
        "trajectory",
        "project",
        "recover",
    ] {
        let path = manifest_dir
            .join("src/commands")
            .join(format!("{module}.rs"));
        let src = fs::read_to_string(&path).expect("command source should be readable");
        assert!(
            src.contains("ensure_project_root_scope_safe")
                || src.contains("resolve_project_scope")
                || src.contains("daemon_global_advisory"),
            "{module} must verify project scope or declare daemon_global_advisory"
        );
    }
}
