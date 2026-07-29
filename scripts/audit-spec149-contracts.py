#!/usr/bin/env python3
"""Fail-closed static audit for Spec149 prose and machine contracts."""

from __future__ import annotations

import json
import re
from collections import defaultdict, deque
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT / "docs/149-focusa-workset-flow-ledger-and-release-completion-spec.md"
CONTRACTS = ROOT / "docs/contracts"
FILES = {
    "workset_schema": CONTRACTS / "spec149-workset.schema.v1.json",
    "event_schema": CONTRACTS / "spec149-event-payloads.schema.v1.json",
    "openapi": CONTRACTS / "spec149-openapi.v1.yaml",
    "operations": CONTRACTS / "spec149-operation-contracts.v1.yaml",
    "compatibility": CONTRACTS / "spec149-spec135-compatibility-packet.v1.yaml",
    "ledger": CONTRACTS / "spec149-complete-feature-ledger.v1.yaml",
    "profile": CONTRACTS / "spec149-next-release-profile.v1.yaml",
    "promotion": CONTRACTS / "spec149-reference-promotion-audit.v1.yaml",
}
ROW = re.compile(r"^\| (S149-R-\d{3}) \| (.+) \|$")
EVENT = re.compile(r"^workset\.[a-z0-9_]+$")


def fail(errors: list[str], message: str) -> None:
    errors.append(message)


def collect_event_types(value: object, output: set[str]) -> None:
    if isinstance(value, dict):
        if "const" in value and isinstance(value["const"], str) and EVENT.match(value["const"]):
            output.add(value["const"])
        if "enum" in value and isinstance(value["enum"], list):
            output.update(item for item in value["enum"] if isinstance(item, str) and EVENT.match(item))
        for child in value.values():
            collect_event_types(child, output)
    elif isinstance(value, list):
        for child in value:
            collect_event_types(child, output)


def collect_refs(value: object, output: set[str]) -> None:
    if isinstance(value, dict):
        ref = value.get("$ref")
        if isinstance(ref, str):
            output.add(ref)
        for child in value.values():
            collect_refs(child, output)
    elif isinstance(value, list):
        for child in value:
            collect_refs(child, output)


def collect_openapi_operations(value: object, output: set[str]) -> None:
    if isinstance(value, dict):
        direct = value.get("x-focusa-operation")
        if isinstance(direct, str):
            output.add(direct)
        by_action = value.get("x-focusa-operation-by-action")
        if isinstance(by_action, dict):
            output.update(item for item in by_action.values() if isinstance(item, str))
        for child in value.values():
            collect_openapi_operations(child, output)
    elif isinstance(value, list):
        for child in value:
            collect_openapi_operations(child, output)


def has_cycle(rows: list[dict[str, object]]) -> bool:
    ids = {str(row["requirement_id"]) for row in rows}
    indegree = {item: 0 for item in ids}
    children: dict[str, list[str]] = defaultdict(list)
    for row in rows:
        item = str(row["requirement_id"])
        for dep in row.get("depends_on", []):
            dep = str(dep)
            if dep not in ids:
                return True
            children[dep].append(item)
            indegree[item] += 1
    queue = deque(item for item, degree in indegree.items() if degree == 0)
    visited = 0
    while queue:
        item = queue.popleft()
        visited += 1
        for child in children[item]:
            indegree[child] -= 1
            if indegree[child] == 0:
                queue.append(child)
    return visited != len(ids)


def main() -> None:
    errors: list[str] = []
    for name, path in FILES.items():
        if not path.is_file():
            fail(errors, f"missing_contract:{name}:{path.relative_to(ROOT)}")

    text = SPEC.read_text()
    prose_rows = [match.groups() for line in text.splitlines() if (match := ROW.match(line))]
    expected_ids = [f"S149-R-{number:03d}" for number in range(1, 81)]
    actual_ids = [row[0] for row in prose_rows]
    if actual_ids != expected_ids:
        fail(errors, f"prose_requirement_sequence:{len(actual_ids)}")

    workset_schema = json.loads(FILES["workset_schema"].read_text())
    event_schema = json.loads(FILES["event_schema"].read_text())
    openapi = yaml.safe_load(FILES["openapi"].read_text())
    operations = yaml.safe_load(FILES["operations"].read_text())
    compatibility = yaml.safe_load(FILES["compatibility"].read_text())
    ledger = yaml.safe_load(FILES["ledger"].read_text())
    profile = yaml.safe_load(FILES["profile"].read_text())
    promotion = yaml.safe_load(FILES["promotion"].read_text())

    ledger_rows = ledger.get("requirements", [])
    if [row.get("requirement_id") for row in ledger_rows] != expected_ids:
        fail(errors, "ledger_requirement_sequence")
    if has_cycle(ledger_rows):
        fail(errors, "ledger_dependency_cycle_or_unknown_ref")
    if ledger.get("current_release_admitted") is not False:
        fail(errors, "ledger_current_release_admission")
    if profile.get("release_boundary", {}).get("current_release_admitted") is not False:
        fail(errors, "profile_current_release_admission")
    if profile.get("release_boundary", {}).get("current_release_membership_import") != "forbidden":
        fail(errors, "profile_current_membership_import")

    if compatibility.get("unknown_impacts"):
        fail(errors, "spec135_unknown_impacts")
    if compatibility.get("blockers"):
        fail(errors, "spec135_blockers")
    impacted = {str(item.get("spec")) for item in compatibility.get("impacts", [])}
    expected_impacts = {"135", "135A", "135B", "135C", "135D", "135E", "135F", "135G", "135H", "135I", "135J", "135K"}
    if impacted != expected_impacts:
        fail(errors, f"spec135_impact_coverage:{sorted(expected_impacts - impacted)}")

    if openapi.get("openapi") != "3.1.0" or not openapi.get("paths"):
        fail(errors, "openapi_contract_invalid")
    operation_ids = [str(item.get("id")) for item in operations.get("operations", [])]
    if len(operation_ids) != len(set(operation_ids)):
        fail(errors, "duplicate_operation_id")
    permitted = {operation for group in operations.get("permissions", []) for operation in group.get("operations", [])}
    missing_permissions = sorted(set(operation_ids) - permitted)
    missing_implementations = sorted(permitted - set(operation_ids))
    if missing_permissions:
        fail(errors, f"operations_without_permission:{missing_permissions}")
    if missing_implementations:
        fail(errors, f"permission_operations_without_contract:{missing_implementations}")
    if len(operations.get("call_stacks", [])) != 5:
        fail(errors, "call_stack_count")
    openapi_operations: set[str] = set()
    collect_openapi_operations(openapi, openapi_operations)
    missing_openapi = sorted((set(operation_ids) - {"release.cycle.execute"}) - openapi_operations)
    if missing_openapi:
        fail(errors, f"operations_missing_openapi_binding:{missing_openapi}")

    prose_events: set[str] = set()
    in_events = False
    for line in text.splitlines():
        if line.strip() in {"Minimum events:", "### 21.2 Minimum events"}:
            in_events = True
            continue
        if in_events and line.strip() == "```text":
            continue
        if in_events and line.strip() == "```":
            break
        if in_events and EVENT.match(line.strip()):
            prose_events.add(line.strip())
    schema_events: set[str] = set()
    collect_event_types(event_schema, schema_events)
    if prose_events != schema_events:
        fail(errors, f"event_schema_drift:prose_only={sorted(prose_events-schema_events)}:schema_only={sorted(schema_events-prose_events)}")

    for schema_name, schema in [("workset", workset_schema), ("event", event_schema)]:
        if schema.get("$schema") != "https://json-schema.org/draft/2020-12/schema":
            fail(errors, f"schema_draft:{schema_name}")
        refs: set[str] = set()
        collect_refs(schema, refs)
        definitions = schema.get("$defs", {})
        missing_internal = sorted(ref for ref in refs if ref.startswith("#/$defs/") and ref.split("/")[-1] not in definitions)
        if missing_internal:
            fail(errors, f"missing_internal_schema_refs:{schema_name}:{missing_internal}")

    refs = promotion.get("reference_audit", {}).get("focusa_doc_refs", []) + promotion.get("reference_audit", {}).get("code_refs", [])
    missing_refs = [ref for ref in refs if not (ROOT / ref).exists()]
    if missing_refs:
        fail(errors, f"missing_reference_refs:{missing_refs}")
    if promotion.get("unresolved_blockers"):
        fail(errors, "promotion_unresolved_blockers")
    if promotion.get("whole_spec_reconciliation", {}).get("status") != "passed":
        fail(errors, "whole_spec_reconciliation")

    required_terms = ["Specs 137/137a", "Specs 138/138a", "RFC 8785", "next-release", "current release remains locked"]
    missing_terms = [term for term in required_terms if term not in text]
    if missing_terms:
        fail(errors, f"missing_prose_terms:{missing_terms}")

    result = {
        "schema": "focusa.spec149_contract_audit.v1",
        "status": "verified" if not errors else "failed",
        "requirements": len(actual_ids),
        "dependency_cycles": 0 if not has_cycle(ledger_rows) else 1,
        "operations": len(operation_ids),
        "openapi_paths": len(openapi.get("paths", {})),
        "events": len(schema_events),
        "spec135_impacts": len(impacted),
        "call_stacks": len(operations.get("call_stacks", [])),
        "current_release_admitted": False,
        "unresolved_blockers": len(errors),
        "errors": errors,
    }
    print(json.dumps(result, indent=2))
    if errors:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
