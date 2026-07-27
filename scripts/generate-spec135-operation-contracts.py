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
    if args.write:
        OPENAPI.write_text(rendered)
    if args.check and rendered != existing:
        print(json.dumps({"status":"blocked","reason":"generated_operation_contract_drift","recovery":"run scripts/generate-spec135-operation-contracts.py --write"}))
        return 1
    print(json.dumps({"status":"passed","operations":len(registry["operations"]),"mode":"write" if args.write else "check"}))
    return 0

if __name__ == "__main__":
    sys.exit(main())
