#!/usr/bin/env python3
"""Spec 135 master final 12/12 acceptance gate."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
C=json.loads((ROOT/"docs/contracts/spec135-master-final-acceptance.v1.json").read_text())
assert C["gate_count"] == 12
assert C["passed_count"] == 12
assert len(C["checks"]) == 12
assert len({r["check_id"] for r in C["checks"]}) == 12
for row in C["checks"]:
    assert row["status"] == "passed", row
    assert (ROOT/row["evidence_ref"]).exists(), row
assert C["branch_policy"] == "feature branch + PR only; never direct commit to main"
assert C["go_sdk"] == "excluded; Pi TUI uses TypeScript"
assert "provider-owned" in C["beads_closure_authority"]
assert len(C["merge_ready_conditions"]) == 6
print("Spec 135 master final acceptance: PASS (12/12)")
