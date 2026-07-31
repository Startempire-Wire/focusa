#!/usr/bin/env python3
"""Fail closed unless Specs 137/137A/138/138A/144/150 permeate runtime and tools."""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"

SPECS = {
    "137+137A": {
        "path": CONTRACTS / "spec137-complete-feature-ledger.v1.yaml",
        "lists": ("requirements", "spec137a_requirement_rows"),
        "count": 258,
    },
    "138+138A": {
        "path": CONTRACTS / "spec138-complete-feature-ledger.v1.yaml",
        "lists": ("requirements",),
        "count": 542,
    },
    "144": {
        "path": CONTRACTS / "spec144-complete-feature-ledger.v1.yaml",
        "lists": ("requirements",),
        "count": 677,
    },
    "150": {
        "path": CONTRACTS / "spec150-complete-feature-ledger.v1.yaml",
        "lists": ("requirements",),
        "count": None,
    },
}
TOOL_MATRIX = CONTRACTS / "spec137-138-144-150-tool-grounding-matrix.v1.yaml"
COMPLETE = {"verified_complete", "implemented", "passed", "not_applicable_proven"}


def values(row: dict[str, Any], keys: tuple[str, ...]) -> list[Any]:
    result: list[Any] = []
    for key in keys:
        value = row.get(key)
        if isinstance(value, list):
            result.extend(item for item in value if item not in (None, ""))
        elif value not in (None, ""):
            result.append(value)
    return result


def row_status(row: dict[str, Any]) -> str:
    return str(row.get("runtime_status") or row.get("status") or "missing")


def main() -> int:
    failures: list[str] = []
    summary: dict[str, Any] = {}
    for spec, contract in SPECS.items():
        path: Path = contract["path"]
        if not path.is_file():
            failures.append(f"{spec}: missing complete feature ledger {path.relative_to(ROOT)}")
            summary[spec] = {"status": "missing"}
            continue
        payload = yaml.safe_load(path.read_text()) or {}
        rows = [row for key in contract["lists"] for row in payload.get(key, [])]
        if contract["count"] is not None and len(rows) != contract["count"]:
            failures.append(f"{spec}: requirement count {len(rows)} != {contract['count']}")
        incomplete: list[str] = []
        missing_runtime: list[str] = []
        missing_tests: list[str] = []
        missing_evidence: list[str] = []
        for index, row in enumerate(rows):
            requirement_id = str(row.get("requirement_id") or f"row-{index + 1}")
            if row_status(row) not in COMPLETE:
                incomplete.append(requirement_id)
            if not values(
                row,
                (
                    "implementation_refs",
                    "core_types",
                    "reducer_events",
                    "persistence",
                    "api_operations",
                    "cli_commands",
                    "pi_tools",
                    "ui_surfaces",
                ),
            ):
                missing_runtime.append(requirement_id)
            if not values(
                row,
                (
                    "test_refs",
                    "positive_tests",
                    "negative_tests",
                    "restart_recovery_tests",
                    "security_tests",
                    "accessibility_tests",
                ),
            ):
                missing_tests.append(requirement_id)
            if not values(row, ("evidence_refs", "receipt_refs")):
                missing_evidence.append(requirement_id)
        for label, ids in (
            ("incomplete runtime status", incomplete),
            ("missing runtime integration", missing_runtime),
            ("missing tests", missing_tests),
            ("missing evidence/receipts", missing_evidence),
        ):
            if ids:
                failures.append(f"{spec}: {label}: {len(ids)}")
        summary[spec] = {
            "requirements": len(rows),
            "incomplete": len(incomplete),
            "missing_runtime": len(missing_runtime),
            "missing_tests": len(missing_tests),
            "missing_evidence": len(missing_evidence),
        }

    if not TOOL_MATRIX.is_file():
        failures.append(
            "all Focusa tools: missing exhaustive applicability and grounding matrix"
        )
        summary["tool_grounding"] = {"status": "missing"}
    else:
        matrix = yaml.safe_load(TOOL_MATRIX.read_text()) or {}
        tools = matrix.get("tools", [])
        unresolved = [
            str(row.get("tool_name") or "unknown")
            for row in tools
            if row.get("status") not in COMPLETE
            or not row.get("applicability_decision")
            or not row.get("machine_readable_contract_refs")
            or not row.get("focus_stack_refs")
            or not row.get("reducer_event_refs")
            or not row.get("projection_replay_refs")
            or not row.get("awareness_refs")
            or not row.get("runbook_refs")
            or not row.get("recovery_refs")
            or not row.get("evidence_refs")
        ]
        if unresolved:
            failures.append(f"all Focusa tools: unresolved grounding rows: {len(unresolved)}")
        summary["tool_grounding"] = {
            "tools": len(tools),
            "unresolved": len(unresolved),
        }

    result = {
        "schema": "focusa.cross_spec_reality_grounding_audit.v1",
        "status": "verified" if not failures else "blocked",
        "specs": summary,
        "failures": failures,
    }
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if not failures else 1


if __name__ == "__main__":
    raise SystemExit(main())
