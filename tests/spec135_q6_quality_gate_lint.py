#!/usr/bin/env python3
import json
import subprocess
from pathlib import Path

R = Path(__file__).resolve().parents[1]
contract = json.loads((R / "docs/contracts/spec135-q6-quality-gate.v1.yaml").read_text())
run = subprocess.run(
    ["python3", str(R / contract["runner"])],
    cwd=R,
    check=False,
    capture_output=True,
    text=True,
)
assert run.returncode == 0, run.stdout + run.stderr
result = json.loads(run.stdout)
assert result["status"] == "completed"
assert result["failed_count"] == 0
assert len(result["checks"]) <= contract["output"]["bounded_checks"]
assert len(result["recovery"]) <= contract["output"]["bounded_recovery_actions"]
assert result["evidence_ref"]
print("Spec 135 Q6 fail-closed aggregate quality gate lint: PASS")
