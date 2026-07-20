#!/usr/bin/env python3
"""Validate the single Spec 135 Operation Registry and generated UI bindings."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "docs/contracts/spec135/generated-contract-v1"
registry = json.loads((BUNDLE / "operation-registry.json").read_text())
openapi = json.loads((BUNDLE / "openapi-3.0.3.json").read_text())
bindings = json.loads((BUNDLE / "ui-action-bindings.fixture.json").read_text())

assert registry["schema"] == "focusa.operation_registry.v1"
assert registry["operation_count"] == len(registry["operations"]) >= 48
operations = {operation["operation_id"]: operation for operation in registry["operations"]}
assert len(operations) == registry["operation_count"]
route_keys = {(operation["method"], operation["path"]) for operation in operations.values()}
assert len(route_keys) == len(operations), "duplicate method/path authority"

required_extensions = {
    "x-focusa-subsystem", "x-focusa-core-action", "x-focusa-scope-keys",
    "x-focusa-capabilities", "x-focusa-permissions", "x-focusa-mode",
    "x-focusa-confirmation", "x-focusa-idempotency", "x-focusa-concurrency",
    "x-focusa-receipt", "x-focusa-reversible", "x-focusa-generated-ui",
    "x-focusa-plain-label", "x-focusa-advanced-only", "x-focusa-sensitive",
}
openapi_operations = {}
for path, path_item in openapi["paths"].items():
    for method, operation in path_item.items():
        if method not in {"get", "post", "put", "patch", "delete"}:
            continue
        operation_id = operation["operationId"]
        openapi_operations[operation_id] = operation
        assert required_extensions <= operation.keys(), (operation_id, required_extensions - operation.keys())
        descriptor = operations[operation_id]
        assert operation["x-focusa-core-action"] == descriptor["ownership"]["core_action_ref"]
        assert operation["x-focusa-scope-keys"] == descriptor["scope"]["required_keys"]
        assert operation["x-focusa-permissions"] == descriptor["control"]["permission_scopes"]

assert openapi_operations.keys() == operations.keys()
for operation_id, descriptor in operations.items():
    assert descriptor["schema"] == "focusa.operation_descriptor.v1"
    assert descriptor["ownership"]["subsystem"]
    assert descriptor["contracts"]["input_schema_ref"]
    assert descriptor["contracts"]["output_schema_ref"]
    assert descriptor["contracts"]["error_schema_ref"] == "focusa.tool_result.v1"
    assert descriptor["control"]["capability_refs"]
    assert descriptor["control"]["mode"] in {"read", "preview", "commit"}
    assert descriptor["control"]["confirmation"] in {"none", "simple", "consequential"}
    assert descriptor["ui"]["default_label"]
    if descriptor["method"] != "GET":
        assert descriptor["control"]["receipt_required"] is True

assert bindings["schema"] == "focusa.ui_action_binding_index.v1"
assert bindings["binding_count"] == len(bindings["bindings"])
binding_ids = {binding["action_id"] for binding in bindings["bindings"]}
allowed_ids = {operation_id for operation_id, operation in operations.items() if operation["ui"]["allowed_in_generated_ui"]}
assert binding_ids == allowed_ids
for binding in bindings["bindings"]:
    assert binding["schema"] == "focusa.ui_action_binding.v1"
    assert binding["contracts"] == operations[binding["action_id"]]["contracts"]
    assert binding["result_envelope_ref"] == "focusa.tool_result.v1"
    assert binding["recovery_envelope_ref"] == "focusa.tool_result.v1"

print(f"Spec 135 Operation Registry: PASS ({len(operations)} operations, {len(binding_ids)} UI bindings)")
