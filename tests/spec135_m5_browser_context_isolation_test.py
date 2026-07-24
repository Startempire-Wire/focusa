#!/usr/bin/env python3
"""Spec 135 M5 static proof: browser contexts remain exact-scope UIAI-owned containers."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMAS = ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema"

request = json.loads(
    (SCHEMAS / "focusa.mission_canvas_surface_binding_mutation.request.v1.json").read_text()
)
properties = request["properties"]
assert properties["browser_isolation_class"]["enum"] == [
    "shared_authenticated",
    "isolated_authenticated",
    "ephemeral_isolated",
    "read_only_observer",
    "capture_worker",
]
assert properties["authentication_sharing"]["enum"] == ["shared_explicit", "isolated"]
assert properties["retention_policy"]["enum"] == [
    "persistent",
    "dispose_on_close",
    "manual",
]
assert any(
    rule.get("if", {}).get("properties", {}).get("binding_kind", {}).get("const")
    == "browser_context"
    and set(rule["then"]["required"])
    == {"browser_isolation_class", "authentication_sharing", "retention_policy"}
    for rule in request["allOf"]
)

route = (ROOT / "crates/focusa-api/src/routes/mission_canvas_surfaces.rs").read_text()
reducer = (ROOT / "crates/focusa-core/src/reducer.rs").read_text()
types = (ROOT / "crates/focusa-core/src/types.rs").read_text()
for marker in (
    "SharedAuthenticated",
    "IsolatedAuthenticated",
    "EphemeralIsolated",
    "ReadOnlyObserver",
    "CaptureWorker",
):
    assert marker in types
for marker in (
    "active UIAI session binding in the exact Work Surface attachment scope",
    "explicit same-project shared authentication is required",
    "Cross-surface binding mutation denied",
    "Browser target requires an active browser context owned by the exact Work Surface attachment",
):
    assert marker in route
for marker in (
    "Browser context requires an active UIAI session in exact attachment scope",
    "Browser context reuse requires exact ownership or explicit same-project sharing",
    "Browser target requires an active browser context in exact attachment scope",
):
    assert marker in reducer

generated_client = ROOT / "packages/generated/spec135/typescript/schema.d.ts"
text = generated_client.read_text()
assert "browser_isolation_class" in text
assert "authentication_sharing" in text
assert "retention_policy" in text

print("Spec 135 M5 browser context isolation and UIAI ownership: PASS")
