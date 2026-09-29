//! GH#592: paid one-request redeem fails closed without embedded roots.
//!
//! The `--license-key` fast path must refuse before contacting the authority
//! when the build carries no embedded production trust roots (upstream #376):
//! exit code 2, TRUST_ROOTS_UNCLASSIFIED-style typed envelope, and proof no
//! request was sent. Marked `#[ignore]` by default: release/CI builds embed
//! roots and would proceed to the live authority; run explicitly on
//! rootless dev builds with --ignored.

use std::process::Command;

fn isolated_home() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("gh592-redeem-probe-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("isolated HOME");
    dir
}

const FOCUSA_BIN: &str = env!("CARGO_BIN_EXE_focusa");

#[ignore = "requires a build without embedded production roots; run with --ignored"]
#[test]
fn redeem_fast_path_refuses_before_submission_without_roots() {
    let home = isolated_home();
    let output = Command::new(FOCUSA_BIN)
        .env("HOME", &home)
        .args([
            "--json",
            "license",
            "activate-flow",
            "--license-key",
            "GH592-PROBE-KEY",
        ])
        .output()
        .expect("run redeem probe");
    assert_eq!(
        output.status.code(),
        Some(2),
        "redeem without roots must exit 2, got {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
    );
    let envelope: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("typed JSON envelope on stdout");
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["code"], "TRUST_ROOTS_UNAVAILABLE");
    assert_eq!(envelope["authority_request_sent"], false);
}
