#!/usr/bin/env python3
"""Fail-closed structural audit for Spec133 §§32-33 acceptance."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUDIT = json.loads(
    (ROOT / "docs/worksheets/spec133-final-acceptance-audit-v1.json").read_text()
)

assert AUDIT["schema"] == "focusa.spec133_final_acceptance_audit.v1"
assert AUDIT["work_item_id"] == "focusa-a6yq6.10.7"
assert AUDIT["release_ready"] is False
assert "fail_closed" in AUDIT["release_gate"]
assert len(AUDIT["acceptance_categories"]) == 10
assert {row["id"] for row in AUDIT["gap_closure_matrix"]} == set(range(1, 16))
assert any(row["status"] == "partial" for row in AUDIT["acceptance_categories"])
assert any(row["status"] == "partial" for row in AUDIT["gap_closure_matrix"])
assert "real Pi lifecycle evidence is absent" in AUDIT["not_done_if"]
assert "tests/spec133_phase9_final_gate.sh executed as one clean proof chain" in AUDIT[
    "implementation_proof"
]["does_not_prove"]

print("Spec133 final acceptance audit remains evidence-backed and fail-closed: PASS")
