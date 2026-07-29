#!/usr/bin/env python3
"""Generate/check Spec 135B Genesis first-Workpoint response contracts."""

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
    schemas["focusa_genesis_first_workpoint_v1"] = {
        "additionalProperties": False,
        "properties": {
            "workpoint_id": {"type": "string"},
            "work_item_id": {"nullable": True, "type": "string"},
            "project_root": {"type": "string"},
            "continuity_id": {"type": "string"},
            "status": {"const": "active", "type": "string"},
            "canonical": {"const": True, "type": "boolean"},
            "acceptance_criteria": {"items": {"type": "string"}, "type": "array"},
            "evidence_refs": {"items": {"type": "string"}, "minItems": 1, "type": "array"},
        },
        "required": [
            "workpoint_id",
            "project_root",
            "continuity_id",
            "status",
            "canonical",
            "acceptance_criteria",
            "evidence_refs",
        ],
        "type": "object",
    }
    response = schemas["focusa_project_genesis_response_v1"]
    response["properties"]["first_workpoint"] = {
        "allOf": [{"$ref": "#/components/schemas/focusa_genesis_first_workpoint_v1"}],
        "nullable": True,
    }
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
        raise SystemExit("Spec 135B Genesis OpenAPI contract is stale; run with --write")
    print(json.dumps({"status": "passed", "mode": "write" if args.write else "check"}))


if __name__ == "__main__":
    main()
