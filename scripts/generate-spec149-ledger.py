#!/usr/bin/env python3
"""Generate the machine-readable Spec149 feature ledger from its normative table."""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT / "docs/149-focusa-workset-flow-ledger-and-release-completion-spec.md"
OUTPUT = ROOT / "docs/contracts/spec149-complete-feature-ledger.v1.yaml"

ROW = re.compile(r"^\| (S149-R-(\d{3})) \| (.+) \|$")


def tranche(number: int) -> tuple[str, str, str]:
    if number <= 12:
        return "foundation", "sections-6-9", "focusa-core"
    if number <= 18:
        return "provider_graph", "sections-8-11", "focusa-core"
    if number <= 24:
        return "checkpoint_flows", "section-13", "focusa-core+focusa-api"
    if number <= 32:
        return "completion_release", "sections-14-15", "focusa-core+release"
    if number <= 38:
        return "preload_project_flow", "sections-16-17", "focusa-api+pi-extension"
    if number <= 45:
        return "spec135_ui", "section-18", "mission-canvas+a2ui"
    if number <= 51:
        return "runtime_governance", "section-19", "focusa-core+agent-runtime"
    if number <= 62:
        return "persistence_reliability", "sections-21-30", "focusa-core+focusa-api"
    if number == 63:
        return "release_boundary", "sections-1-2", "release-engineering"
    if number == 64:
        return "temporal", "section-19.1", "temporal-authority"
    if number == 65:
        return "epistemic", "section-19.2", "prediction+metacog"
    if number <= 73:
        return "hardening", "sections-8-14", "focusa-core"
    if number <= 77:
        return "surface_contracts", "sections-22-27", "focusa-api+all-surfaces"
    if number == 78:
        return "next_release_profile", "section-15.3", "release-engineering"
    if number == 79:
        return "call_stacks", "section-32", "architecture"
    return "promotion", "sections-32-35", "operator+spec-workbench"


def dependencies(number: int) -> list[str]:
    deps: set[int] = set()
    if number > 1:
        deps.add(3)
    if 4 <= number <= 18:
        deps.add(4)
    if 13 <= number <= 18:
        deps.update({8, 11})
    if 19 <= number <= 24:
        deps.update({17, 21 if number != 21 else 13})
    if 25 <= number <= 32:
        deps.update({19, 25 if number != 25 else 11})
    if 33 <= number <= 38:
        deps.update({17, 33 if number != 33 else 5})
    if 39 <= number <= 45:
        deps.update({38, 39 if number != 39 else 7})
    if 46 <= number <= 51:
        deps.update({3, 46 if number != 46 else 4})
    if 52 <= number <= 62:
        deps.update({7, 21, 25})
    if number == 63:
        deps.add(1)
    if number == 64:
        deps.update({19, 37})
    if number == 65:
        deps.update({23, 44})
    if 66 <= number <= 73:
        deps.update({7, 13, 19, 25})
    if 74 <= number <= 77:
        deps.update({38, 41, 51})
    if number == 78:
        deps.update({27, 63, 76})
    if number == 79:
        deps.update({74, 76})
    if number == 80:
        deps.update({63, 76, 79})
    deps.discard(number)
    return [f"S149-R-{dep:03d}" for dep in sorted(deps)]


def main() -> None:
    rows: list[dict[str, object]] = []
    for line in SPEC.read_text().splitlines():
        match = ROW.match(line)
        if not match:
            continue
        requirement_id, raw_number, text = match.groups()
        number = int(raw_number)
        phase, section_ref, owner = tranche(number)
        rows.append(
            {
                "requirement_id": requirement_id,
                "requirement": text,
                "applicability": "applicable",
                "specification_status": "specified",
                "implementation_status": "not_started",
                "release_admission": "next_release_only",
                "phase": phase,
                "section_ref": f"docs/149-focusa-workset-flow-ledger-and-release-completion-spec.md#{section_ref}",
                "owner": owner,
                "depends_on": dependencies(number),
                "positive_tests": [],
                "negative_tests": [],
                "restart_recovery_tests": [],
                "security_tests": [],
                "evidence_refs": [],
                "receipt_refs": [],
            }
        )

    expected = [f"S149-R-{number:03d}" for number in range(1, 81)]
    actual = [str(row["requirement_id"]) for row in rows]
    if actual != expected:
        raise SystemExit(f"Spec149 requirement sequence mismatch: expected 80, got {len(actual)}")

    document = {
        "schema": "focusa.spec149_complete_feature_ledger.v1",
        "spec_ref": str(SPEC.relative_to(ROOT)),
        "release_admission": "next_release_only",
        "current_release_admitted": False,
        "next_release_bead": "focusa-a89or",
        "requirement_count": len(rows),
        "allowed_specification_statuses": ["specified", "approved", "superseded"],
        "allowed_implementation_statuses": ["not_started", "active", "blocked", "verified", "not_applicable"],
        "closure_rule": "all applicable rows verified with tests, evidence, receipts, parity, migration, and no hidden deferral",
        "requirements": rows,
    }
    OUTPUT.write_text(json.dumps(document, indent=2) + "\n")
    print(json.dumps({"status": "generated", "requirements": len(rows), "output": str(OUTPUT.relative_to(ROOT))}))


if __name__ == "__main__":
    main()
