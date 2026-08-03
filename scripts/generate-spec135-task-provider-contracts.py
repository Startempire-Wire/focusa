#!/usr/bin/env python3
"""Generate/check Spec 135B task-provider capability truth contracts."""

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
    capability = {
        "additionalProperties": False,
        "properties": {
            "provider": {
                "enum": ["beads", "github_issues", "linear", "asana", "markdown_checklist"],
                "type": "string",
            },
            "status": {
                "enum": [
                    "configured and operational",
                    "configured but unhealthy",
                    "read-only",
                    "credentials missing",
                    "adapter unavailable",
                    "schema-only support",
                    "mutation approval required",
                ],
                "type": "string",
            },
            "read_write_posture": {"enum": ["read-only", "read-write"], "type": "string"},
            "configured": {"type": "boolean"},
            "credential_reference_present": {"type": "boolean"},
            "mutation_approval_required": {"type": "boolean"},
            "adapter_ref": {"type": "string"},
            "recovery_action": {"type": "string"},
        },
        "required": [
            "provider",
            "status",
            "read_write_posture",
            "configured",
            "credential_reference_present",
            "mutation_approval_required",
            "adapter_ref",
            "recovery_action",
        ],
        "type": "object",
    }
    schemas["focusa_task_provider_capability_truth_v1"] = capability
    listing = schemas["focusa_provider_neutral_task_plan_list_v1"]
    listing["properties"]["provider_capabilities"] = {
        "items": {"$ref": "#/components/schemas/focusa_task_provider_capability_truth_v1"},
        "maxItems": 5,
        "minItems": 5,
        "type": "array",
    }
    if "provider_capabilities" not in listing.setdefault("required", []):
        listing["required"].append("provider_capabilities")
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
        raise SystemExit("Spec 135B task-provider OpenAPI contract is stale; run with --write")
    print(json.dumps({"status": "passed", "mode": "write" if args.write else "check"}))


if __name__ == "__main__":
    main()
