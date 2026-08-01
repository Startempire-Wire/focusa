#!/usr/bin/env python3
"""Security, accessibility, responsive, recovery, and performance hardening gate."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
entrypoint = (ROOT / "apps/pi-extension/rich-host/host-entrypoint.mjs").read_text()
frontend = (ROOT / "apps/pi-extension/rich-host/assets/main.js").read_text()
css = (ROOT / "apps/pi-extension/rich-host/assets/styles.css").read_text()
lifecycle = (ROOT / "apps/pi-extension/src/rich-host/lifecycle.ts").read_text()
platform = (ROOT / "apps/pi-extension/src/rich-host/platform.ts").read_text()
threat = (ROOT / "docs/security/spec135-rich-host-threat-model.md").read_text()
responsive = json.loads((ROOT / "tests/fixtures/spec135-responsive-evaluations.json").read_text())

for rule in ["default-src 'self'", "connect-src http://127.0.0.1:*", "frame-ancestors 'none'", "referrer-policy", "cache-control"]:
    assert rule in entrypoint, rule
for secret_rule in ["0o600", "expires_at", "nonce", "HANDSHAKE_SHA256", "rm(handshakePath"]:
    assert secret_rule in entrypoint or secret_rule in platform, secret_rule
assert "localStorage" not in frontend
assert "sessionStorage" not in frontend
assert "innerHTML" not in frontend
assert "authorization" in frontend
assert "assertScope" in frontend
assert "scope mismatch" in frontend.lower()
assert "Ed25519" in threat
assert "verifySignature" in platform

for accessibility in ["focus-visible", "forced-colors", "prefers-reduced-motion", "text_scale_percent", "min-height: 40px", "touch-action: manipulation"]:
    assert accessibility in css or accessibility in json.dumps(responsive), accessibility
for keyboard in ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "Escape", "ctrlKey", "focus({ preventScroll: true }"]:
    assert keyboard in frontend, keyboard
for state in ["scrollPositions", "focused_semantic_target", "syncDraft", "reconnect", "shutdown", "clearInterval", "removeHandshakeFile"]:
    assert state in frontend or state in lifecycle, state
for performance in ["virtualWindow", "content-visibility", "replaceChildren", "requestAnimationFrame", "AbortController"]:
    assert performance in frontend or performance in css, performance

assert {fixture["viewport"]["platform"] for fixture in responsive} == {"macOS", "Windows", "Linux"}
assert min(fixture["viewport"]["css_width"] for fixture in responsive) == 1024
assert max(fixture["viewport"]["css_width"] for fixture in responsive) >= 1600

for audit_path in [
    ROOT / "docs/evidence/spec135-pi-extension-npm-audit.json",
    ROOT / "docs/evidence/spec135-a2ui-renderer-npm-audit.json",
]:
    audit = json.loads(audit_path.read_text())
    assert audit["metadata"]["vulnerabilities"]["total"] == 0, audit_path
for sbom_path in [
    ROOT / "docs/evidence/spec135-pi-extension-sbom.cdx.json",
    ROOT / "docs/evidence/spec135-a2ui-renderer-sbom.cdx.json",
]:
    sbom = json.loads(sbom_path.read_text())
    assert sbom["bomFormat"] == "CycloneDX"
    assert sbom["components"], sbom_path

print("Spec 135 rich-host hardening: PASS")
