#!/usr/bin/env python3
"""Validate the permanent A2UI v0.9.1 web_core + Lit renderer."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT / "packages/a2ui-renderer"
manifest = json.loads((PACKAGE / "package.json").read_text())
lock = json.loads((PACKAGE / "package-lock.json").read_text())
source = (PACKAGE / "src/index.ts").read_text()
renderer_test = (PACKAGE / "tests/renderer.test.mjs").read_text()
snapshot = json.loads((PACKAGE / "fixtures/mission-snapshot.json").read_text())
delta = json.loads((PACKAGE / "fixtures/mission-delta.json").read_text())
catalog = json.loads(
    (
        ROOT / "docs/contracts/spec135/generated-contract-v1/a2ui-catalog.json"
    ).read_text()
)
compatibility = json.loads(
    (
        ROOT / "docs/contracts/spec135/generated-contract-v1/compatibility-lock.yaml"
    ).read_text()
)

assert manifest["dependencies"]["@a2ui/web_core"] == "0.9.1"
assert manifest["dependencies"]["@a2ui/lit"] == "0.9.1"
assert manifest["dependencies"]["lit"] == "3.3.1"
assert "svelte" not in json.dumps(manifest).lower()
assert "playwright" not in json.dumps(lock).lower()
assert lock["packages"]["node_modules/@a2ui/web_core"]["version"] == "0.9.1"
assert lock["packages"]["node_modules/@a2ui/lit"]["version"] == "0.9.1"

for marker in (
    'from "@a2ui/web_core/v0_9"',
    'from "@a2ui/lit/v0_9"',
    "class FocusaA2uiRenderer",
    "new MessageProcessor",
    "basicCatalog",
    'document.createElement("a2ui-surface")',
    "processSnapshot",
    "processDelta",
    "maxSerializedBytes",
    "Unsupported A2UI protocol version",
):
    assert marker in source
assert (
    "A2UI v0.9.1 snapshot and delta render deterministically through Lit"
    in renderer_test
)
assert "a2ui-basic-text" in renderer_test

assert snapshot[0]["version"] == "v0.9"
assert snapshot[0]["createSurface"]["surfaceId"] == "mission-canvas"
assert snapshot[1]["updateComponents"]["components"][0]["text"] == "Mission ready"
assert delta[0]["updateComponents"]["components"][0]["text"] == "Mission resumed"
assert catalog["schema"] == "focusa.a2ui_catalog.v1"
assert catalog["protocol_version"] == "v0.9"
assert catalog["renderer"] == "lit"
assert catalog["package_lock"] == {
    "@a2ui/lit": "0.9.1",
    "@a2ui/web_core": "0.9.1",
    "@focusa/elements": "0.9.120-dev",
    "lit": "3.3.1",
    "svelte": "5.55.9",
}
assert catalog["capabilities"]["v0.9"]["supportedCatalogIds"]
assert catalog["capabilities"]["v0.9"]["inlineCatalogs"]
assert compatibility["a2ui_protocol"] == "0.9.1"
assert compatibility["a2ui_catalog"] == "0.9.1"

print(
    "Spec 135 A2UI Lit renderer: PASS (v0.9.1 snapshot/delta, permanent bounded renderer)"
)
