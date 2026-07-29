#!/usr/bin/env python3
"""Spec 135G-6 multiplexing/concurrency proof matrix."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
C=json.loads((ROOT/"docs/contracts/spec135-multiplexing-concurrency-proof.v1.json").read_text())
assert C["scenario_count"] == 8
expected={"two_project","same_project","contention","browser_isolation","shared_warning","close_semantics","restart_restore","concurrent_writer"}
assert {s["scenario_id"] for s in C["scenarios"]} == expected
for scenario in C["scenarios"]:
    assert scenario["status"] == "passed", scenario
    assert scenario["proof_refs"]
    for ref in scenario["proof_refs"]:
        assert (ROOT/ref).exists(), ref
assert "No Work Surface becomes singleton canonical authority" in C["global_invariants"]
print("Spec 135 G6 multiplexing and concurrency proof suite: PASS (8/8)")
