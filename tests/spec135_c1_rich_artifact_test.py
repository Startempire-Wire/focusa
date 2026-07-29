#!/usr/bin/env python3
"""Spec 135C-1 rich artifact contract + Pi renderer proof."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md").read_text()
SCHEMA = json.loads(
    (ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.workspace_artifact_descriptor.v1.json").read_text()
)
REGISTRY = json.loads(
    (ROOT / "docs/contracts/spec135-rich-artifact-renderer-registry.v1.json").read_text()
)
RENDERERS = (ROOT / "apps/pi-extension/src/rich-artifact-renderers.ts").read_text()
INDEX = (ROOT / "apps/pi-extension/src/index.ts").read_text()
RUNTIME = (ROOT / "apps/pi-extension/tests/rich-artifact-renderers.test.mjs").read_text()

assert SCHEMA["$schema"] == "https://json-schema.org/draft/2020-12/schema"
assert SCHEMA["title"] == "Focusa Workspace Artifact Descriptor v1"
required = set(SCHEMA["required"])
for field in (
    "artifact_id", "artifact_kind", "before_ref", "after_ref", "evidence_refs",
    "project_root", "continuity_id", "session_origin", "freshness", "authority",
    "render_safe",
):
    assert field in required, field

for kind in ("image", "markdown", "dataset", "diff", "browser_snapshot", "diagnostics", "chart", "document", "media", "fpv_session"):
    assert kind in SCHEMA["properties"]["artifact_kind"]["enum"], kind

assert REGISTRY["schema"] == "focusa.spec135.rich_artifact_renderer_registry.v1"
assert REGISTRY["fallback_rule"] == "No client may silently discard an artifact because it cannot render the preferred format."
kinds = {k["kind"] for k in REGISTRY["artifact_kinds"]}
assert kinds == set(SCHEMA["properties"]["artifact_kind"]["enum"])

for token in (
    "ArtifactKind",
    "RichArtifactDescriptor",
    "RICH_ARTIFACT_RENDERERS",
    "renderRichArtifact",
    "fallbackSafeRender",
    "RENDER_BLOCKED: render_safe is false; fallback required",
    "Fallback if unavailable",
    "registerRichArtifactRenderers",
):
    assert token in RENDERERS, token

assert "registerRichArtifactRenderers(pi)" in INDEX
assert 'registerMessageRenderer("focusa-rich-artifact"' in INDEX
assert "10 kinds" in RUNTIME
assert "never discarded" in RUNTIME
assert "render_safe is false" in RUNTIME

for spec_text in (
    "Artifact kinds and required renderers",
    "No client may silently discard an artifact because it cannot render the preferred format",
    "Workspace Artifact descriptor",
    "provenance",
):
    assert spec_text in SPEC, spec_text

print("Spec 135 C1 rich artifact contract and Pi renderers: PASS")