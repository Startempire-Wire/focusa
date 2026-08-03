#!/usr/bin/env python3
"""Spec 135H-2 Alpha 5-8 isolation/parity/dogfood proof."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
C=json.loads((ROOT/"docs/contracts/spec135-alpha5-8-production-proof.v1.json").read_text())
assert [w["alpha"] for w in C["workflows"]] == [5,6,7,8]
for row in C["workflows"]:
    assert row["status"] == "passed", row
    assert (ROOT/row["proof_ref"]).exists()
assert C["static_marker_substitution"] is False
assert C["waived_defects"] == []
assert C["blockers"] == []
for ref in C["runtime_proof_refs"]: assert (ROOT/ref).exists(), ref
print("Spec 135 H2 Alpha 5-8 isolation, parity, dogfood: PASS")
