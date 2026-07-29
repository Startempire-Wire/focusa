#!/usr/bin/env python3
"""Generate/check Spec 135A Work Rail preview/commit interaction contracts."""

from __future__ import annotations

import argparse
import copy
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OPENAPI = ROOT / "docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json"


def generated(current):
    result = copy.deepcopy(current)
    schemas = result["components"]["schemas"]
    interaction = {
        "additionalProperties": False,
        "properties": {
            "interaction_id": {"type": "string"},
            "action": {"type": "string"},
            "actor_ref": {"type": "string"},
            "reason": {"type": "string"},
            "receipt_ref": {"type": "string"},
            "committed_at": {"format": "date-time", "type": "string"},
        },
        "required": [
            "interaction_id",
            "action",
            "actor_ref",
            "reason",
            "receipt_ref",
            "committed_at",
        ],
        "type": "object",
    }
    schemas["focusa_work_rail_interaction_v1"] = interaction
    request = schemas["focusa_work_rail_mutation_request_v1"]
    request["properties"]["action"]["enum"] = [
        "bind",
        "activate",
        "verify_close",
        "cancel",
        "steer",
        "defer",
        "request_approval",
        "reopen",
    ]
    request["properties"].update(
        {
            "side_effect_policy": {"enum": ["preview", "commit"], "type": "string"},
            "preview_token": {"type": "string"},
            "actor_ref": {"type": "string"},
            "interaction_reason": {"type": "string"},
            "instance_id": {"type": "string"},
            "session_id": {"type": "string"},
            "work_surface_ids": {"items": {"type": "string"}, "type": "array"},
            "priority": {"type": "integer"},
            "rank": {"type": "integer"},
            "change_set_ref": {"type": "string"},
        }
    )
    if "side_effect_policy" not in request.setdefault("required", []):
        request["required"].append("side_effect_policy")
    mutation_result = schemas["focusa_work_rail_mutation_result_v1"]
    mutation_result["properties"]["committed"] = {"type": "boolean"}
    mutation_result["properties"]["preview_token"] = {"type": "string"}
    for name in ("committed", "preview_token"):
        if name not in mutation_result.setdefault("required", []):
            mutation_result["required"].append(name)

    def augment_rows(node):
        if isinstance(node, dict):
            properties = node.get("properties")
            if isinstance(properties, dict) and {
                "work_rail_id",
                "provider_item_id",
                "focusa_status",
            }.issubset(properties):
                properties.update(
                    {
                        "instance_id": {"nullable": True, "type": "string"},
                        "session_id": {"nullable": True, "type": "string"},
                        "work_surface_ids": {"items": {"type": "string"}, "type": "array"},
                        "priority": {"nullable": True, "type": "integer"},
                        "rank": {"nullable": True, "type": "integer"},
                        "change_set_ref": {"nullable": True, "type": "string"},
                        "interaction_history": {
                            "items": {"$ref": "#/components/schemas/focusa_work_rail_interaction_v1"},
                            "type": "array",
                        },
                    }
                )
            for value in node.values():
                augment_rows(value)
        elif isinstance(node, list):
            for value in node:
                augment_rows(value)

    augment_rows(schemas["focusa_work_rail_list_v1"])
    augment_rows(mutation_result)
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
        raise SystemExit("Spec 135A Work Rail OpenAPI contract is stale; run with --write")
    print(json.dumps({"status": "passed", "mode": "write" if args.write else "check"}))


if __name__ == "__main__":
    main()
