//! Spec123 Order 14 public-surface guard regression.

use std::fs;
use std::path::Path;
use std::process::Command;

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

fn read_repo(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn public_surface_guard_script_passes() {
    let script = repo_root().join("scripts/guard-public-surface.sh");
    let output = Command::new("bash")
        .arg(script)
        .current_dir(repo_root())
        .output()
        .expect("guard should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "public-surface guard failed\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}"
    );
    assert!(stdout.contains("public-surface guard completed"));
}

#[test]
fn security_and_license_public_wording_are_sanitized() {
    let security = read_repo("SECURITY.md");
    assert!(security.contains("security@focusa.dev"));
    assert!(security.contains("Do not open public issues for suspected vulnerabilities."));
    assert!(!security.contains("placeholder"));

    let license_faq = read_repo("LICENSE-FAQ.md");
    assert!(license_faq.contains("Commercial, company, team"));
    assert!(!license_faq.contains("$384,330"));
    assert!(!license_faq.contains("license row"));

    let commercial = read_repo("COMMERCIAL.md");
    assert!(commercial.contains("https://install.focusa.dev/license"));
    assert!(commercial.contains("support@focusa.dev"));
    assert!(!commercial.contains("wpuiai.com/wp-admin"));
}

#[test]
fn public_license_commands_do_not_point_to_admin_urls() {
    let install_sh = read_repo("scripts/install-focusa.sh");
    let license_rs = read_repo("crates/focusa-cli/src/commands/license.rs");
    for src in [install_sh, license_rs] {
        assert!(!src.contains("wpuiai.com/wp-admin"));
        assert!(!src.contains("wpuiai.com/buy"));
        assert!(
            src.contains("https://focusa.dev/support")
                || src.contains("https://install.focusa.dev/license")
        );
    }
}
