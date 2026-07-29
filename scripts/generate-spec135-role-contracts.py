#!/usr/bin/env python3
"""Generate/check Spec 135B Role Composer alternative schemas."""

from __future__ import annotations

import argparse
import copy
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OPENAPI = ROOT / "docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json"


def string(max_length=512):
    return {"maxLength": max_length, "type": "string"}


def alternative_schema():
    refs = {"items": string(), "maxItems": 32, "minItems": 1, "type": "array"}
    return {
        "additionalProperties": False,
        "properties": {
            "alternative_id": string(256),
            "title": string(200),
            "purpose": string(2000),
            "tradeoffs": copy.deepcopy(refs),
            "grounding_refs": copy.deepcopy(refs),
        },
        "required": ["title", "purpose", "tradeoffs", "grounding_refs"],
        "type": "object",
    }


def generated(current):
    result = copy.deepcopy(current)
    schemas = result["components"]["schemas"]
    draft = schemas["focusa_project_agent_role_profile_draft_request_v1"]
    input_alternative = alternative_schema()
    input_alternative["properties"].pop("alternative_id")
    draft["properties"]["alternatives"] = {
        "items": input_alternative,
        "maxItems": 16,
        "type": "array",
    }

    schemas["focusa_role_alternative_v1"] = alternative_schema()

    def augment_profiles(node):
        if isinstance(node, dict):
            properties = node.get("properties")
            if isinstance(properties, dict) and {
                "role_profile_id",
                "grounding",
                "grants_permissions",
            }.issubset(properties):
                properties["alternatives"] = {
                    "items": {"$ref": "#/components/schemas/focusa_role_alternative_v1"},
                    "type": "array",
                }
                if "alternatives" not in node.setdefault("required", []):
                    node["required"].append("alternatives")
            for value in node.values():
                augment_profiles(value)
        elif isinstance(node, list):
            for value in node:
                augment_profiles(value)

    augment_profiles(schemas["focusa_project_agent_role_profile_mutation_result_v1"])
    augment_profiles(schemas["focusa_project_agent_role_profile_list_v1"])
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    current = json.loads(OPENAPI.read_text())
    expected = generated(current)
    rendered = json.dumps(expected, indent=2) + "\n"
    existing = json.dumps(current, indent=2) + "\n"
    if args.write:
        OPENAPI.write_text(rendered)
    if args.check and rendered != existing:
        raise SystemExit("Spec 135B Role Composer OpenAPI contract is stale; run with --write")
    print(json.dumps({"status": "passed", "mode": "write" if args.write else "check"}))


if __name__ == "__main__":
    main()
