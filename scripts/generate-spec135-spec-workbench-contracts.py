#!/usr/bin/env python3
"""Generate/check Spec 135B C.R.I.S.T. → Spec 120 handoff contracts."""

from __future__ import annotations

import argparse
import copy
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OPENAPI = ROOT / "docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json"


def refs(max_items=64):
    return {"items": {"type": "string"}, "maxItems": max_items, "type": "array"}


def handoff_schema():
    return {
        "additionalProperties": False,
        "properties": {
            "schema": {"const": "focusa.crist_spec_handoff.v1", "type": "string"},
            "project_root": {"type": "string"},
            "continuity_id": {"type": "string"},
            "current_ask": {"type": "string"},
            "workspace_profile_ref": {"type": "string"},
            "active_domain_pack_refs": refs(),
            "semantic_registry_version": {"type": "string"},
            "context_pack_refs": refs(),
            "accepted_project_claim_refs": refs(),
            "role_profile_ref": {"type": "string"},
            "interview_session_refs": refs(),
            "unresolved_questions": refs(),
            "known_contradictions": refs(),
            "desired_spec_template": {"const": "project_genesis", "type": "string"},
        },
        "required": [
            "schema",
            "project_root",
            "continuity_id",
            "current_ask",
            "workspace_profile_ref",
            "active_domain_pack_refs",
            "semantic_registry_version",
            "context_pack_refs",
            "accepted_project_claim_refs",
            "role_profile_ref",
            "interview_session_refs",
            "unresolved_questions",
            "known_contradictions",
            "desired_spec_template",
        ],
        "type": "object",
    }


def generated(current):
    result = copy.deepcopy(current)
    schemas = result["components"]["schemas"]
    schemas["focusa_crist_spec_handoff_v1"] = handoff_schema()
    mutation = schemas["focusa_spec_workbench_mutation_request_v1"]
    mutation["properties"]["desired_spec_template"] = {
        "enum": ["adversarial", "project_genesis"],
        "type": "string",
    }

    def augment(node):
        if isinstance(node, dict):
            properties = node.get("properties")
            if isinstance(properties, dict):
                if {"title", "section_kind", "order_index", "docs_only"}.issubset(properties):
                    properties["reality_classification"] = {
                        "enum": [
                            "implemented",
                            "partial",
                            "docs_only",
                            "normative_target",
                            "planned",
                            "speculative",
                            "stale",
                            "blocked",
                            "unknown",
                        ],
                        "type": "string",
                    }
                if {"workbench_session_id", "current_ask", "sections"}.issubset(properties):
                    properties["desired_spec_template"] = {"type": "string"}
                    properties["crist_handoff"] = {
                        "$ref": "#/components/schemas/focusa_crist_spec_handoff_v1"
                    }
                    required = node.setdefault("required", [])
                    for name in ("desired_spec_template", "crist_handoff"):
                        if name not in required:
                            required.append(name)
            for value in node.values():
                augment(value)
        elif isinstance(node, list):
            for value in node:
                augment(value)

    augment(schemas["focusa_spec_workbench_mutation_request_v1"])
    augment(schemas["focusa_spec_workbench_session_list_v1"])
    augment(schemas["focusa_spec_workbench_mutation_result_v1"])
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
        raise SystemExit("Spec 135B Spec Workbench OpenAPI contract is stale; run with --write")
    print(json.dumps({"status": "passed", "mode": "write" if args.write else "check"}))


if __name__ == "__main__":
    main()
