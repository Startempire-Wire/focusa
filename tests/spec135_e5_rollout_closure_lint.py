#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
contract = json.loads((R / "docs/contracts/spec135-e5-rollout-closure.v1.yaml").read_text())
rows = contract["closure_rows"]
assert len(rows) >= 8
for row in rows:
    text = json.dumps(row).lower()
    assert not any(state in text for state in contract["forbidden_states"])
    proof = R / row["proof"] if not row["proof"].startswith("spec135-") else R / "docs/contracts" / row["proof"]
    assert proof.exists(), proof
assert all(contract["acceptance"].values())
assert contract["legacy_forms"]["status"] == "non_primary_compatibility_only"
print("Spec 135 E5 expand-contract rollout closure lint: PASS")
