#!/usr/bin/env python3
"""Static contract for REL.4 all-system, non-publishing candidate artifacts."""

from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
workflow_path = ROOT / ".github/workflows/locked-release-candidate-artifacts.yml"
caller_path = ROOT / ".github/workflows/windows-ota-e2e.yml"
workflow = workflow_path.read_text()
caller = caller_path.read_text()
version_verifier = (ROOT / "scripts/verify-version-surfaces.py").read_text()
parsed = yaml.safe_load(workflow)

assert parsed[True] == {"workflow_call": None}
assert "locked-release-candidate-artifacts.yml" in caller
assert "secrets: inherit" in caller

for target in (
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "x86_64-pc-windows-msvc",
    "aarch64-pc-windows-msvc",
):
    assert target in workflow

for required in (
    "windows-11-arm",
    "macos-15-intel",
    "macos-14",
    "focusa-generated-clients-v0.9.144.tar.gz",
    "focusa-installer-v0.9.144.ps1",
    "focusa-v0.9.144.cdx.json",
    "focusa-v0.9.144.spdx.json",
    "verify-canonical-release-assets.py --dist dist --tag v0.9.144",
    "FOCUSA_RELEASE_ED25519_PRIVATE_KEY",
    "TAURI_SIGNING_PRIVATE_KEY",
    "npm ci --ignore-scripts",
    "npx --no-install tauri build",
    "release-trust-metadata.py",
    "--candidate",
    'publication_status == "candidate_only"',
    "SHA256SUMS.txt.cosign.sig",
    "sha256sum -c SHA256SUMS.txt",
    "focusa-v0.9.144-candidate-bundle",
):
    assert required in workflow

for prohibited in (
    "softprops/action-gh-release",
    "gh release create",
    "gh release edit",
    "git tag",
    "git push",
):
    assert prohibited not in workflow

assert "contents: write" not in workflow
assert "contents: read" in workflow
assert "id-token: write" in workflow
assert '.read_text(encoding="utf-8")' in version_verifier
assert ".read_text()" not in version_verifier
print("REL.4 non-publishing all-system candidate artifact contract: PASS")
