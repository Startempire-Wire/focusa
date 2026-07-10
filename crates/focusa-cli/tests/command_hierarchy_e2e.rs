//! E2E tests for Spec124 Order 09 command hierarchy and migration help.

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
fn help_all_exposes_target_command_hierarchy() {
    let (output, out) = run(&["help", "all"]);
    assert!(
        output.status.success(),
        "help all should exit 0, got: {out}"
    );
    for needle in [
        "FOCUSA COMMANDS",
        "focusa project",
        "focusa first-mission",
        "focusa setup",
        "focusa deck",
        "focusa workpoint",
        "focusa trajectory",
        "focusa pairing",
        "focusa help migration",
    ] {
        assert!(
            out.contains(needle),
            "help all missing {needle}, got: {out}"
        );
    }
}

#[test]
fn help_migration_exposes_old_to_new_alias_map() {
    let (output, out) = run(&["help", "migration"]);
    assert!(
        output.status.success(),
        "help migration should exit 0, got: {out}"
    );
    for needle in [
        "focusa init",
        "focusa project new / focusa setup init",
        "focusa onboard",
        "focusa setup wizard",
        "focusa pair",
        "focusa pairing start",
        "focusa pairing-doctor",
        "focusa pairing doctor",
        "focusa stack",
        "focusa focus stack",
    ] {
        assert!(
            out.contains(needle),
            "migration help missing {needle}, got: {out}"
        );
    }
}

#[test]
fn json_help_migration_is_machine_readable() {
    let (output, out) = run(&["--json", "help", "migration"]);
    assert!(
        output.status.success(),
        "json help migration should exit 0, got: {out}"
    );
    assert!(out.contains("\"schema\": \"focusa.command_help.v1\""));
    assert!(out.contains("\"migrations\""));
    assert!(out.contains("focusa pairing start"));
}

#[test]
fn pairing_start_is_canonical_command() {
    let (output, out) = run(&["pairing", "start", "--help"]);
    assert!(
        output.status.success(),
        "pairing start --help should exit 0, got: {out}"
    );
    assert!(
        out.contains("Start a Mac/phone pairing flow") || out.contains("Open a Mac Pairing Room")
    );
}

#[test]
fn cli_version_comes_from_package_version() {
    let (output, out) = run(&["--version"]);
    assert!(
        output.status.success(),
        "--version should exit 0, got: {out}"
    );
    assert!(
        out.contains(env!("CARGO_PKG_VERSION")),
        "version should be package version {}, got: {out}",
        env!("CARGO_PKG_VERSION")
    );
}
