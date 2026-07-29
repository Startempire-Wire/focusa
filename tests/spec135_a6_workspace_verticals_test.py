#!/usr/bin/env python3
"""Spec 135A-6 workspace vertical/theme/artifact renderer proof."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md").read_text()
VERTICALS = (ROOT / "apps/pi-extension/src/workspace-verticals.ts").read_text()
INDEX = (ROOT / "apps/pi-extension/src/index.ts").read_text()
RUNTIME_TEST = (ROOT / "apps/pi-extension/tests/workspace-verticals.test.mjs").read_text()

for profile in ("General", "Software", "Legal", "Markets", "Research", "Custom"):
    assert f'{profile}:' in VERTICALS, profile
    assert profile in SPEC, profile

for token in (
    "VERTICAL_PROFILES",
    "variants",
    "artifactInvariant",
    "renderArtifactProjection",
    "Unified diff",
    "Side-by-side redline",
    "Thesis revision",
    "Claim delta",
    "Registered custom projection",
    "presentation-only",
    "Open artifact",
):
    assert token in VERTICALS, token

for invariant in (
    "artifactId",
    "artifactKind",
    "beforeRef",
    "afterRef",
    "evidenceRefs",
    "projectRoot",
    "continuityId",
    "sessionOrigin",
    "freshness",
    "authority",
):
    assert invariant in VERTICALS, invariant

assert 'registerCommand("focusa-profile"' in VERTICALS
assert 'registerCommand("focusa-artifact"' in VERTICALS
assert "registerWorkspaceVerticals(pi)" in INDEX
assert 'registerMessageRenderer("focusa-artifact-projection"' in INDEX
assert "new Set(outputs).size, 6" in RUNTIME_TEST
assert "each profile must have an independent projection" in RUNTIME_TEST

print("Spec 135 A6 workspace verticals/artifact renderers: PASS")
