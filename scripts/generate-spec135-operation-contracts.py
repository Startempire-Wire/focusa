#!/usr/bin/env python3
"""Generate OpenAPI operation metadata from the canonical Spec 135 Operation Registry."""
import argparse
import json
import sys
from pathlib import Path

R = Path(__file__).resolve().parents[1]
B = R / "docs/contracts/spec135/generated-contract-v1"
REGISTRY = B / "operation-registry.json"
OPENAPI = B / "openapi-3.0.3.json"
UI_BINDINGS = B / "ui-action-bindings.fixture.json"


def generated(registry, openapi):
    operations = registry["operations"]
    by_id = {row["operation_id"]: row for row in operations}
    if len(by_id) != len(operations):
        raise ValueError("duplicate operation_id in canonical registry")
    routes = {}
    for path, path_item in openapi["paths"].items():
        for method, operation in path_item.items():
            if method not in {"get", "post", "put", "patch", "delete"}:
                continue
            operation_id = operation["operationId"]
            if operation_id in routes:
                raise ValueError(f"duplicate OpenAPI operationId: {operation_id}")
            routes[operation_id] = (method.upper(), path, operation)
    if set(routes) != set(by_id):
        raise ValueError(
            f"registry/OpenAPI operation mismatch missing={sorted(set(by_id)-set(routes))} extra={sorted(set(routes)-set(by_id))}"
        )
    for operation_id, descriptor in by_id.items():
        method, path, operation = routes[operation_id]
        if method != descriptor["method"] or path != descriptor["path"]:
            raise ValueError(f"route drift for {operation_id}")
        control = descriptor["control"]
        ui = descriptor["ui"]
        operation.update({
            "x-focusa-subsystem": descriptor["ownership"]["subsystem"],
            "x-focusa-core-action": descriptor["ownership"]["core_action_ref"],
            "x-focusa-scope-keys": descriptor["scope"]["required_keys"],
            "x-focusa-permissions": control["permission_scopes"],
            "x-focusa-capabilities": control["capability_refs"],
            "x-focusa-mode": control["mode"],
            "x-focusa-confirmation": control["confirmation"],
            "x-focusa-idempotency": control["idempotency_required"],
            "x-focusa-concurrency": control["optimistic_concurrency_required"],
            "x-focusa-receipt": control["receipt_required"],
            "x-focusa-reversible": control["reversible"],
            "x-focusa-generated-ui": ui["allowed_in_generated_ui"],
            "x-focusa-plain-label": ui["default_label"],
            "x-focusa-advanced-only": ui["advanced_only"],
            "x-focusa-sensitive": ui["sensitivity"],
            "x-focusa-result-envelope": "focusa.tool_result.v1",
            "x-focusa": {
                "family": descriptor["family"],
                "canonical": descriptor["canonical"],
                "budget_profile": descriptor["budget_profile"],
                "materialization_mode": descriptor["materialization_mode"],
                "side_effect_profile": descriptor["side_effect_profile"],
                "supports_side_effect_policy": descriptor["supports_side_effect_policy"],
                "requires_preview_token": descriptor["requires_preview_token"],
                "deprecation": descriptor["deprecation"],
            },
        })
    openapi["x-focusa-operation-registry-ref"] = "operation-registry.json"
    return openapi


def generated_ui_bindings(registry, current):
    """Keep reviewed bindings stable and derive any newly allowed operation bindings."""
    existing = {row["action_id"]: row for row in current["bindings"]}
    bindings = []
    example_values = {
        "project_root": "/example",
        "continuity_id": "example",
        "attachment_id": "example",
        "session_id": "example",
        "origin": "https://example.invalid",
    }
    for descriptor in registry["operations"]:
        if not descriptor["ui"]["allowed_in_generated_ui"]:
            continue
        operation_id = descriptor["operation_id"]
        binding = existing.get(operation_id, {})
        required_keys = descriptor["scope"]["required_keys"]
        scope = {key: example_values.get(key, "example") for key in required_keys}
        scope["required_keys"] = required_keys
        confirmation = descriptor["control"]["confirmation"]
        binding.update(
            {
                "action_id": operation_id,
                "canonical_revision": descriptor["operation_version"],
                "capability_refs": descriptor["control"]["capability_refs"],
                "contracts": descriptor["contracts"],
                "control": {
                    "confirmation": "none" if confirmation == "none" else "explicit",
                    "idempotency_required": descriptor["control"]["idempotency_required"],
                    "mode": "read" if descriptor["control"]["mode"] == "read" else "write",
                    "optimistic_concurrency_required": descriptor["control"]["optimistic_concurrency_required"],
                    "receipt_required": descriptor["control"]["receipt_required"],
                    "reversible": descriptor["control"]["reversible"],
                },
                "operation_descriptor_ref": f"/v1/agent/operations#{operation_id}",
                "permission_scopes": descriptor["control"]["permission_scopes"],
                "presentation": descriptor["ui"],
                "recovery_envelope_ref": descriptor["contracts"]["error_schema_ref"],
                "result_envelope_ref": "focusa.tool_result.v1",
                "schema": "focusa.ui_action_binding.v1",
                "scope": scope,
            }
        )
        bindings.append(binding)
    current["bindings"] = bindings
    current["binding_count"] = len(bindings)
    return current


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    registry = json.loads(REGISTRY.read_text())
    current = json.loads(OPENAPI.read_text())
    expected = generated(registry, json.loads(json.dumps(current)))
    rendered = json.dumps(expected, indent=2) + "\n"
    existing = json.dumps(current, indent=2) + "\n"
    current_bindings = json.loads(UI_BINDINGS.read_text())
    expected_bindings = generated_ui_bindings(
        registry, json.loads(json.dumps(current_bindings))
    )
    rendered_bindings = json.dumps(expected_bindings, indent=2) + "\n"
    existing_bindings = json.dumps(current_bindings, indent=2) + "\n"
    if args.write:
        OPENAPI.write_text(rendered)
        UI_BINDINGS.write_text(rendered_bindings)
    if args.check and (
        rendered != existing or rendered_bindings != existing_bindings
    ):
        print(json.dumps({"status":"blocked","reason":"generated_operation_contract_drift","recovery":"run scripts/generate-spec135-operation-contracts.py --write"}))
        return 1
    print(json.dumps({"status":"passed","operations":len(registry["operations"]),"mode":"write" if args.write else "check"}))
    return 0

if __name__ == "__main__":
    sys.exit(main())
