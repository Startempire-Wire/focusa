#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
B = R / "docs/contracts/spec135/generated-contract-v1"
bundle = json.loads((B / "spec135-z2-permanent-integration-evidence.json").read_text())
assert bundle["chain"] == [
    "onboarding", "context", "role", "interview", "spec", "tasks",
    "workpoint", "evidence", "receipt", "artifact", "mission_canvas",
]
for ref in bundle["real_execution_refs"]:
    result = json.loads((B / ref).read_text())
    assert result["status"] == "passed", ref
for ref in bundle["durability_refs"]:
    assert (R / ref).exists(), ref
assert (B / bundle["permanent_path_ref"]).exists()
assert all(bundle["continuity_proof"].values())
assert all(bundle["acceptance"].values())
assert bundle["evidence_ref"] and bundle["receipt_ref"]
print("Spec 135 Z2 permanent integration evidence lint: PASS")
