#!/usr/bin/env python3
"""Operation-level contract checks for the Mission Canvas API surface."""
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "docs/contracts/spec135/mission-canvas-v1/operation-registry.json"
BUNDLE = ROOT / "schemas/spec135/mission-canvas/composition-bundle.v1.schema.json"

subprocess.run(["python3", "scripts/generate-spec135-mission-canvas-operations.py", "--check"], cwd=ROOT, check=True)
registry = json.loads(REGISTRY.read_text())
definitions = json.loads(BUNDLE.read_text())["$defs"]
operations = registry["operations"]
ids = [entry["operation_id"] for entry in operations]
routes = [(entry["method"], entry["path"]) for entry in operations]

assert registry["availability"] == "available"
assert registry["promotion_owner"] == "P03 runtime implementation"
assert registry["operation_count"] == len(operations) == 25
assert len(ids) == len(set(ids))
assert len(routes) == len(set(routes))
assert all(entry["availability"] == "available" for entry in operations)
assert all(entry["implementation_phase"] == "P03" for entry in operations)

required_prefixes = {
    "focusa.mission_canvas.projection.",
    "focusa.mission_canvas.profile.",
    "focusa.mission_canvas.activity.",
    "focusa.mission_canvas.registry.",
    "focusa.mission_canvas.layout_memory.",
    "focusa.mission_canvas.rich_host.",
    "focusa.mission_canvas.draft.",
    "focusa.mission_canvas.recipient.",
    "focusa.mission_canvas.recomposition.",
}
for prefix in required_prefixes:
    assert any(operation_id.startswith(prefix) for operation_id in ids), prefix

for entry in operations:
    assert entry["scope_required"] == ["workstream"]
    assert entry["authority_chain"] == [
        "scope_ref", "project_root_key", "workstream_id", "continuity_id",
        "attachment_key", "session_id", "instance_id", "workspace_binding_id",
        "runtime_object", "work_surface_id",
    ]
    assert entry["permissions_required"]
    if entry["mode"] == "mutation":
        assert entry["requires_idempotency_key"]
        assert entry["receipt_required"]
    response = entry["response_schema_ref"].removesuffix("[]")
    if not response.startswith("focusa."):
        assert response in definitions, response

close = next(entry for entry in operations if entry["operation_id"].endswith("rich_host.close"))
assert close["confirmation"] == "explicit"
assert close["receipt_required"]

layout_mutation = next(entry for entry in operations if entry["operation_id"].endswith("layout.mutate"))
assert layout_mutation["requires_if_match_revision"]
assert layout_mutation["request_schema_ref"] == "LayoutMutationCommand"
assert layout_mutation["response_schema_ref"] == "LayoutMutationResult"

print("Spec 135 Mission Canvas operation registry: PASS")
