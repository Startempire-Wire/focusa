#!/usr/bin/env python3
"""Validate the sealed next-release Workset against Spec149 JSON schemas."""

from __future__ import annotations

import copy
import json
from pathlib import Path

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "docs/contracts/spec149-workset.schema.v1.json"
PAYLOAD_SCHEMA_PATH = ROOT / "docs/contracts/spec149-event-payloads.schema.v1.json"
AUDIT = ROOT / "release-proof/audit"

OBJECTS = {
    "WorksetDefinition": AUDIT / "next-locked-release-workset-definition.json",
    "WorksetProviderBinding": AUDIT / "next-locked-release-workset-provider-binding.json",
    "WorksetCompletionContract": AUDIT / "next-locked-release-workset-completion-contract.json",
}
LEDGERS = {
    "WorksetMember": AUDIT / "next-locked-release-workset-members.jsonl",
    "WorksetEdge": AUDIT / "next-locked-release-workset-edges.jsonl",
    "WorksetEventEnvelope": AUDIT / "next-locked-release-workset-events.jsonl",
}


def read_json(path: Path) -> object:
    return json.loads(path.read_text())


def read_jsonl(path: Path) -> list[object]:
    return [json.loads(line) for line in path.read_text().splitlines() if line]


def validator_for(schema: dict[str, object], definition: str) -> Draft202012Validator:
    definitions = copy.deepcopy(schema["$defs"])
    if definition == "WorksetEventEnvelope":
        # Validate envelope and typed payload separately. The external payload
        # schema is projected over event_type+payload below.
        definitions[definition].pop("allOf", None)
    wrapper = {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$defs": definitions,
        "$ref": f"#/$defs/{definition}",
    }
    return Draft202012Validator(wrapper)


def validate_rows(
    errors: list[str],
    schema: dict[str, object],
    definition: str,
    path: Path,
    rows: list[object],
) -> None:
    validator = validator_for(schema, definition)
    for index, row in enumerate(rows, 1):
        for error in validator.iter_errors(row):
            location = "/".join(str(part) for part in error.absolute_path)
            errors.append(f"{path.relative_to(ROOT)}:{index}:{location}: {error.message}")


def main() -> int:
    schema = read_json(SCHEMA_PATH)
    payload_schema = read_json(PAYLOAD_SCHEMA_PATH)
    errors: list[str] = []
    counts: dict[str, int] = {}

    for definition, path in OBJECTS.items():
        rows = [read_json(path)]
        counts[definition] = len(rows)
        validate_rows(errors, schema, definition, path, rows)

    event_rows: list[object] = []
    for definition, path in LEDGERS.items():
        rows = read_jsonl(path)
        counts[definition] = len(rows)
        validate_rows(errors, schema, definition, path, rows)
        if definition == "WorksetEventEnvelope":
            event_rows = rows

    payload_validator = Draft202012Validator(payload_schema)
    for index, event in enumerate(event_rows, 1):
        projection = {"event_type": event["event_type"], "payload": event["payload"]}
        for error in payload_validator.iter_errors(projection):
            location = "/".join(str(part) for part in error.absolute_path)
            errors.append(f"typed-event:{index}:{location}: {error.message}")

    result = {
        "schema": "focusa.locked_release_workset_schema_audit.v1",
        "status": "failed" if errors else "verified",
        "spec149_schema": str(SCHEMA_PATH.relative_to(ROOT)),
        "typed_event_schema": str(PAYLOAD_SCHEMA_PATH.relative_to(ROOT)),
        "counts": counts,
        "errors": errors,
    }
    print(json.dumps(result, indent=2))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
