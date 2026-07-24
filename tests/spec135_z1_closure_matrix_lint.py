#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
matrix = json.loads(
    (R / "docs/contracts/spec135/generated-contract-v1/spec135-z1-requirement-closure-matrix.json").read_text()
)
assert matrix["requirement_count"] == 73 == len(matrix["rows"])
assert len({row["requirement_id"] for row in matrix["rows"]}) == 73
assert all(row["implementation_tasks"] for row in matrix["rows"])
assert all(row["tests"] for row in matrix["rows"])
assert all(row["evidence_requirements"] for row in matrix["rows"])
assert matrix["audit"] == {
    "missing_evidence": [],
    "missing_implementation_tasks": [],
    "missing_tests": [],
    "mock_only": [],
    "silently_deferred": [],
}
assert matrix["evidence_ref"] and matrix["receipt_ref"]
print("Spec 135 Z1 complete requirement closure matrix lint: PASS")
