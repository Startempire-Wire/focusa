//! E2E tests for Spec124 first-mission CLI surface.

use std::path::PathBuf;
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

fn repo_root() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_string_lossy()
        .to_string()
}

#[test]
fn first_mission_help_exposes_spec_flags() {
    let (output, out) = run(&["first-mission", "--help"]);
    assert!(
        output.status.success(),
        "first-mission --help should exit 0"
    );
    for flag in [
        "--project-root",
        "--project",
        "--continuity-id",
        "--yes",
        "--dry-run",
        "--open-deck",
        "--no-animation",
    ] {
        assert!(out.contains(flag), "help should expose {flag}, got: {out}");
    }
}

#[test]
fn first_mission_dry_run_json_does_not_require_daemon() {
    let root = repo_root();
    let (output, out) = run(&[
        "--json",
        "first-mission",
        "--project-root",
        root.as_str(),
        "--dry-run",
        "--no-animation",
    ]);
    assert!(output.status.success(), "dry-run should exit 0, got: {out}");
    assert!(out.contains("\"schema\": \"focusa.first_mission.v1\""));
    assert!(out.contains("\"dry_run\": true"));
    assert!(out.contains("\"mutated\": false"));
    assert!(out.contains("\"no_animation\": true"));
    assert!(out.contains("/v1/workpoint/checkpoint"));
    assert!(out.contains("Project status shown"));
    assert!(out.contains("focusa project status"));
}

#[test]
fn setup_wizard_routes_to_first_mission_dry_run() {
    let root = repo_root();
    let (output, out) = run(&[
        "--json",
        "setup",
        "wizard",
        "--project-root",
        root.as_str(),
        "--dry-run",
        "--no-animation",
    ]);
    assert!(
        output.status.success(),
        "setup wizard dry-run should exit 0, got: {out}"
    );
    assert!(out.contains("\"schema\": \"focusa.first_mission.v1\""));
    assert!(out.contains("\"dry_run\": true"));
    assert!(out.contains("\"mutated\": false"));
    assert!(out.contains("Project status shown"));
}

#[test]
fn setup_help_exposes_wizard() {
    let (output, out) = run(&["setup", "--help"]);
    assert!(output.status.success(), "setup --help should exit 0");
    assert!(out.contains("wizard"));
}
