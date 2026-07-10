//! E2E tests for Project CLI surface parsing/help paths.

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
fn project_help_available() {
    let (output, out) = run(&["project", "--help"]);
    assert!(output.status.success(), "project --help should exit 0");
    assert!(out.contains("ProjectIdentity") || out.contains("Project"));
}

#[test]
fn project_identity_help() {
    let (output, out) = run(&["project", "identity", "--help"]);
    assert!(
        output.status.success(),
        "project identity --help should exit 0"
    );
    assert!(out.contains("project identity") || out.contains("Usage"));
}

#[test]
fn project_card_help() {
    let (output, out) = run(&["project", "card", "--help"]);
    assert!(output.status.success(), "project card --help should exit 0");
    assert!(out.contains("Project Card") || out.contains("Usage"));
}

#[test]
fn project_list_and_discover_help() {
    let (output, out) = run(&["project", "list", "--help"]);
    assert!(output.status.success(), "project list --help should exit 0");
    assert!(out.contains("project list") || out.contains("Usage"));

    let (output, out) = run(&["project", "discover", "--help"]);
    assert!(
        output.status.success(),
        "project discover --help should exit 0"
    );
    assert!(out.contains("project discover") || out.contains("Usage"));
}

#[test]
fn project_selection_alias_help() {
    for cmd in [&"use", &"bind", &"switch"] {
        let (output, out) = run(&["project", cmd, "--help"]);
        assert!(
            output.status.success(),
            "project {cmd} --help should exit 0"
        );
        assert!(out.contains("project") || out.contains("Usage"));
    }
}

#[test]
fn project_current_status_help() {
    let (output, out) = run(&["project", "current", "--help"]);
    assert!(
        output.status.success(),
        "project current --help should exit 0"
    );
    assert!(out.contains("current") || out.contains("Usage"));

    let (output, out) = run(&["project", "status", "--help"]);
    assert!(
        output.status.success(),
        "project status --help should exit 0"
    );
    assert!(out.contains("status") || out.contains("Usage"));
}

#[test]
fn project_create_settings_templates_help() {
    let (output, out) = run(&["project", "remove", "--help"]);
    assert!(
        output.status.success(),
        "project remove --help should exit 0"
    );
    assert!(out.contains("project") || out.contains("Usage"));

    let (output, out) = run(&["project", "new", "--help"]);
    assert!(output.status.success(), "project new --help should exit 0");
    assert!(out.contains("project") || out.contains("Usage"));
    assert!(
        out.contains("--working-dir"),
        "new help should expose --working-dir"
    );
    assert!(out.contains("--name"), "new help should expose --name");
    assert!(out.contains("--git"), "new help should expose --git");

    let (output, out) = run(&["project", "templates", "list", "--help"]);
    assert!(
        output.status.success(),
        "project templates list --help should exit 0"
    );
    assert!(out.contains("templates") || out.contains("Usage"));

    let (output, out) = run(&["project", "templates", "show", "--help"]);
    assert!(
        output.status.success(),
        "project templates show --help should exit 0"
    );
    assert!(out.contains("template") || out.contains("Usage"));

    let (output, out) = run(&["project", "settings", "list", "--help"]);
    assert!(
        output.status.success(),
        "project settings list --help should exit 0"
    );
    assert!(out.contains("settings") || out.contains("Usage"));

    let (output, out) = run(&["project", "settings", "get", "--help"]);
    assert!(
        output.status.success(),
        "project settings get --help should exit 0"
    );
    assert!(out.contains("settings") || out.contains("Usage"));

    let (output, out) = run(&["project", "settings", "set", "--help"]);
    assert!(
        output.status.success(),
        "project settings set --help should exit 0"
    );
    assert!(out.contains("settings") || out.contains("Usage"));

    let (output, out) = run(&["project", "settings", "unset", "--help"]);
    assert!(
        output.status.success(),
        "project settings unset --help should exit 0"
    );
    assert!(out.contains("settings") || out.contains("Usage"));
}
