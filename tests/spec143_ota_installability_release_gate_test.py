#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
release = (ROOT / ".github/workflows/release.yml").read_text()
deploy = (ROOT / ".github/workflows/deploy-live-daemon.yml").read_text()
installer = (ROOT / "scripts/install-focusa.sh").read_text()
install_rs = (ROOT / "crates/focusa-cli/src/commands/install.rs").read_text()
update = (ROOT / "crates/focusa-cli/src/commands/update.rs").read_text()
trust = (ROOT / "crates/focusa-cli/src/commands/update_trust.rs").read_text()
stamper = (ROOT / "scripts/stamp-menubar-version.py").read_text()
version_verifier = (ROOT / "scripts/verify-version-surfaces.py").read_text()

assert "target: x86_64-unknown-linux-musl" in release
assert "musl: true" in release
assert '-f asset_suffix="x86_64-unknown-linux-musl"' in release
assert '-f asset_suffix="x86_64-unknown-linux-gnu"' not in release
assert "cross build --release --target ${{ matrix.target }}" in release
assert "if: ${{ startsWith(github.ref, 'refs/tags/') }}" in release
assert "startsWith(github.ref, refs/tags/)" not in release

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

stamp = (ROOT / "scripts/stamp-menubar-version.py").read_text()
verify = (ROOT / "scripts/verify-version-surfaces.py").read_text()
tag_script = (ROOT / "scripts/create-dev-release-tag.sh").read_text()
assert "replace_extension_build" in stamp
assert "apps/pi-extension/src/auto-compaction.ts" in stamp
assert "read_extension_build_version" in verify
assert "replace_agent_card_version" in stamp
assert "docs/contracts/spec141/generated-capability-v2/agent-card.json" in verify
assert tag_script.count("apps/pi-extension/src/auto-compaction.ts") >= 2
assert tag_script.count("docs/contracts/spec141/generated-capability-v2/agent-card.json") >= 2
assert "scripts/stamp-release-version" in tag_script
assert "scripts/verify-doc-version-consistency" in tag_script
assert "validate-docs-runtime-parity.mjs" in tag_script
for docs_surface in [
    "README.md",
    "docs/current/.release-version-stamp",
    "docs/current/CURRENT_RUNTIME_STATUS.md",
]:
    assert tag_script.count(docs_surface) >= 2, docs_surface
for observability_marker in [
    "workflow_heartbeat",
    "failed_job=",
    "failed_step=",
    "workflow_error_excerpt_begin",
    "full_log_command=",
    "workflow_status_query_error",
]:
    assert observability_marker in tag_script, observability_marker
assert "gh run watch" not in tag_script
for workspace_package in ("focusa-harness-adapters", "focusa-session-runner"):
    assert workspace_package in stamper, workspace_package
    assert workspace_package in version_verifier, workspace_package
print("Spec143 OTA installability release gate: PASS")
