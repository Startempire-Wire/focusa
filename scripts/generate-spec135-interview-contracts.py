#!/usr/bin/env python3
"""Generate/check Spec 135B Interview closure provenance contract."""

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
    provenance = {
        "additionalProperties": False,
        "properties": {
            "answer_id": {"type": "string"},
            "operator_id": {"type": "string"},
            "status": {"type": "string"},
            "confidence": {"nullable": True, "type": "number"},
            "created_at": {"format": "date-time", "type": "string"},
            "supersedes": {"nullable": True, "type": "string"},
        },
        "required": ["answer_id", "operator_id", "status", "created_at"],
        "type": "object",
    }
    schemas["focusa_interview_answer_provenance_v1"] = provenance
    compendium_entry = {
        "additionalProperties": False,
        "properties": {
            "question_id": {"type": "string"},
            "question": {"type": "string"},
            "answer": {},
            "answer_provenance": {
                "allOf": [
                    {"$ref": "#/components/schemas/focusa_interview_answer_provenance_v1"}
                ],
                "nullable": True,
            },
            "notes": {"nullable": True, "type": "string"},
            "attachment_refs": {"items": {"type": "string"}, "type": "array"},
            "context_refs": {"items": {"type": "string"}, "type": "array"},
            "spec_sections": {"items": {"type": "string"}, "type": "array"},
        },
        "required": [
            "question_id",
            "question",
            "answer",
            "answer_provenance",
            "notes",
            "attachment_refs",
            "context_refs",
            "spec_sections",
        ],
        "type": "object",
    }
    schemas["focusa_interview_compendium_entry_v1"] = compendium_entry
    closure = schemas["focusa_interview_closure_package_v1"]
    closure["properties"]["compendium"]["items"] = {
        "$ref": "#/components/schemas/focusa_interview_compendium_entry_v1"
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
        raise SystemExit("Spec 135B Interview OpenAPI contract is stale; run with --write")
    print(json.dumps({"status": "passed", "mode": "write" if args.write else "check"}))


if __name__ == "__main__":
    main()
