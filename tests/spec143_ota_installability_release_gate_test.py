#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
release = (ROOT / ".github/workflows/release.yml").read_text()
deploy = (ROOT / ".github/workflows/deploy-live-daemon.yml").read_text()
installer = (ROOT / "scripts/install-focusa.sh").read_text()
install_rs = (ROOT / "crates/focusa-cli/src/commands/install.rs").read_text()
update = (ROOT / "crates/focusa-cli/src/commands/update.rs").read_text()
trust = (ROOT / "crates/focusa-cli/src/commands/update_trust.rs").read_text()

assert "target: x86_64-unknown-linux-musl" in release
assert "musl: true" in release
assert '-f asset_suffix="x86_64-unknown-linux-musl"' in release
assert '-f asset_suffix="x86_64-unknown-linux-gnu"' not in release
assert "cross build --release --target ${{ matrix.target }}" in release

assert "deploy-success.json deploy-success.json.sig" in deploy
assert "Gate OTA installability against signed deployed release" in deploy
assert "ota-update-plan.json" in deploy
assert ".latest.trust.deploy_proof_verified == true" in deploy
assert ".apply_allowed == true" in deploy
assert "ota-installability-proof-${{ steps.cfg.outputs.tag }}" in deploy

assert 'Linux-x86_64)   TRIPLE="x86_64-unknown-linux-musl"' in installer
assert 'InstallTarget::Linux => "x86_64-unknown-linux-musl".to_string()' in install_rs
assert '"deploy-success.json"' in trust
assert '"deploy-success.json.sig"' in trust
assert "verify_deploy_proof" in trust

for field in [
    "installed: serde_json::Value",
    "latest: String",
    "applied: bool",
    "surfaces: Vec<String>",
    "rollback: serde_json::Value",
    "next_action: String",
    "blockers: Vec<String>",
    "error: Option<String>",
]:
    assert field in update, f"missing top-level apply result field: {field}"
assert "refresh_apply_summary(&mut apply);" in update
assert "do not bypass trust" in update
print("Spec143 OTA installability release gate: PASS")
