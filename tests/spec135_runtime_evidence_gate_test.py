#!/usr/bin/env python3
import json
import subprocess
import sys
from pathlib import Path

R = Path(__file__).resolve().parents[1]
matrix = json.loads((R / "docs/contracts/spec135-runtime-evidence-matrix.v2.yaml").read_text())
assert len(matrix["domains"]) == 12
assert not (set(matrix["allowed_completion_evidence"]) & set(matrix["forbidden_completion_evidence"]))
for row in matrix["domains"]:
    assert row["required_kind"] in matrix["allowed_completion_evidence"]
    if row["status"] == "proven":
        assert row["evidence_refs"]
        assert all((R / ref).exists() for ref in row["evidence_refs"])
run = subprocess.run([sys.executable, str(R / "scripts/spec135-runtime-evidence-gate.py")], capture_output=True, text=True)
result = json.loads(run.stdout)
assert run.returncode == 1
assert result["status"] == "blocked"
assert result["passed_domains"] == 1
assert result["total_domains"] == 12
print("Spec 135 runtime evidence gate: PASS (static completion rejected; 11 runtime domains remain blocked)")
