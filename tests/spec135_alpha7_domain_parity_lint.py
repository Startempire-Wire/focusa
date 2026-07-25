#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
proof = json.loads(
    (R / "docs/contracts/spec135/generated-contract-v1/spec135-alpha7-domain-parity-proof.json").read_text()
)
assert proof["projection_sequence"] == ["general", "software", "research"]
assert len(proof["state_invariants"]) >= 7
assert all(proof["acceptance"].values())
for ref in proof["evidence_refs"]:
    assert (R / ref).exists(), ref
print("Spec 135 Alpha 7 General→Software→Research parity lint: PASS")
