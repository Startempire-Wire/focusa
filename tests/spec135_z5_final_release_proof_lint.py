#!/usr/bin/env python3
import json
import subprocess
from pathlib import Path

R = Path(__file__).resolve().parents[1]
B = R / "docs/contracts/spec135/generated-contract-v1"
proof = json.loads((B / "spec135-z5-final-release-proof.json").read_text())
assert proof["merge_ready"] is True
assert proof["verified_requirements"] == 72
assert proof["open_requirements"] == ["SPEC135-Z5"]
assert proof["open_task_beads"] == [
    "focusa-mc2",
    "focusa-mc2.12",
    "focusa-mc2.12.377",
    "focusa-mc2.12.378",
    "focusa-mc2.12.379",
    "focusa-mc2.12.380",
    "focusa-mc2.12.381",
]
assert proof["behind"] == 0 and proof["clean"] is True
for ref in (proof["closure_gate_ref"], proof["requirement_matrix_ref"], proof["lineage_ref"]):
    assert (B / ref).exists(), ref
assert proof["project_card_outcome"]["attempted"] is True
assert proof["evidence_ref"] and proof["receipt_ref"]
ancestor = subprocess.run(["git", "merge-base", "--is-ancestor", proof["head"], "HEAD"], cwd=R)
assert ancestor.returncode == 0
print("Spec 135 Z5 final merge-ready release proof lint: PASS")
