#!/usr/bin/env python3
"""Static contract for the Spec 177 AppVeyor full-release fallback."""

from __future__ import annotations

import json
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
APPVEYOR = (ROOT / ".appveyor.yml").read_text()
TOPOLOGY = json.loads((ROOT / "config/focusa-release-topology.json").read_text())
TRUSTED_KEYS = json.loads(
    (ROOT / "config/focusa-trusted-release-keys.json").read_text()
)
TRUST_SCRIPT = (ROOT / "scripts/release-trust-metadata.py").read_text()
INSTALL_SH = (ROOT / "scripts/install-focusa.sh").read_text()
INSTALL_PS1 = (ROOT / "scripts/install-focusa.ps1").read_text()
BG = (ROOT / "crates/focusa-cli/src/commands/bg.rs").read_text()
ROUTE = (ROOT / "crates/focusa-api/src/routes/background_jobs.rs").read_text()
STORE = (ROOT / "crates/focusa-core/src/background_job_store.rs").read_text()

EXPECTED_GATES = [
    "release-contract",
    "source-ci",
    "strict-spec-gates",
    "terminal-windows",
    "terminal-linux",
    "terminal-macos",
    "rust-macos",
    "rust-linux",
    "rust-windows",
    "desktop-macos",
    "desktop-windows",
    "portable-surfaces",
    "trust-and-publication",
    "ota-deploy-live",
]

provider_contract = TOPOLOGY["provider_contract"]
assert TOPOLOGY["provider"] == "github_actions"
assert provider_contract["primary"] == "github_actions"
appveyor = next(
    item for item in provider_contract["emergency"] if item["provider"] == "appveyor"
)
assert appveyor["project"] == "verioussmith/focusa"
assert appveyor["required_gate_receipts"] == EXPECTED_GATES
assert appveyor["signing"]["cosign_oidc_issuer"] == "https://accounts.google.com"
assert appveyor["signing"]["rekor_required"] is True
assert appveyor["publication"]["draft_until_all_gates_green"] is True

for gate in EXPECTED_GATES:
    assert f"GATE: {gate}" in APPVEYOR, f"AppVeyor matrix missing {gate}"

for target in [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "x86_64-pc-windows-msvc",
    "aarch64-pc-windows-msvc",
]:
    assert target in APPVEYOR, f"AppVeyor target missing {target}"

for image in ["Visual Studio 2022", "Ubuntu", "macos-sonoma"]:
    assert image in APPVEYOR, f"AppVeyor image missing {image}"

for surface in [
    "focusa-daemon",
    "focusa-tui",
    "pi-extension",
    "generated-clients",
    "agent-context",
    "SHA256SUMS",
    "release-manifest",
    "release-provenance",
    "release-intelligence",
    "SBOM",
    "Tauri",
    "DMG",
    "MSI",
]:
    assert surface.lower() in APPVEYOR.lower(), f"surface missing: {surface}"

for guard in [
    "job_depends_on: Release contract",
    "job_depends_on: Proofs",
    "APPVEYOR_REPO_TAG",
    "APPVEYOR_REPO_COMMIT",
    "APPVEYOR_API_TOKEN",
    "FOCUSA_RELEASE_ED25519_PRIVATE_KEY",
    "FOCUSA_APPVEYOR_SIGNER_JSON_BASE64",
    "TAURI_SIGNING_PRIVATE_KEY",
    "GITHUB_RELEASE_TOKEN",
    "focusa.release_gate_receipt.v1",
    "focusa.release_gate_ledger.v1",
    "scripts/verify-canonical-release-assets.py",
    "scripts/generate-release-notes.py",
    "scripts/release-trust-metadata.py",
    "--identity-token",
    "https://accounts.google.com",
    "isPrerelease",
    "isLatest",
]:
    assert guard in APPVEYOR, f"publication guard missing: {guard}"

assert "deploy: off" not in APPVEYOR
assert "skip_tags: false" in APPVEYOR
assert "matrix:\n  fast_finish: true" in APPVEYOR

identities = {item["provider"]: item for item in TRUSTED_KEYS["sigstore_identities"]}
assert identities["appveyor"]["issuer"] == "https://accounts.google.com"
assert identities["appveyor"]["identity"].startswith(
    "focusa-appveyor-release-signer@"
)
assert identities["github-actions"]["issuer"] == (
    "https://token.actions.githubusercontent.com"
)

for marker in [
    'choices=("github-actions", "appveyor")',
    "--provider-receipt",
    "focusa.release_gate_ledger.v1",
    '"builder": args.builder',
    '"provider_evidence": provider_evidence',
]:
    assert marker in TRUST_SCRIPT

for installer in [INSTALL_SH, INSTALL_PS1]:
    assert "focusa-appveyor-release-signer@tech-empire-258307.iam.gserviceaccount.com" in installer
    assert "https://accounts.google.com" in installer
    assert "https://token.actions.githubusercontent.com" in installer
    assert "certificate-identity" in installer
    assert "certificate-oidc-issuer" in installer

for marker in [
    "CREATE_NEW_PROCESS_GROUP",
    "DETACHED_PROCESS",
    "current_dir(cwd)",
    '"output_tail": output_tail',
]:
    assert marker in BG
assert "bounded_output_tail(&body.output_tail, 4096)" in ROUTE
assert 'output_tail TEXT NOT NULL DEFAULT \'\'' in STORE
assert '"completion_event": completion_event' in ROUTE

print("PASS: Spec177 AppVeyor fallback covers 14 gates, every target/surface, trusted publication, and bg portability")
