#!/usr/bin/env python3
"""Spec 135H-1 Alpha 1-4 production workflow proof."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
C=json.loads((ROOT/"docs/contracts/spec135-alpha1-4-production-proof.v1.json").read_text())
assert [w["alpha"] for w in C["workflows"]] == [1,2,3,4]
for row in C["workflows"]:
    assert row["status"] in {"passed","verified"}, row
    assert (ROOT/row["proof_ref"]).exists()
assert C["blockers"] == []
for ref in C["runtime_test_refs"]+C["pi_ui_refs"]:
    assert (ROOT/ref).exists(), ref
assert len(C["runtime_test_refs"]) >= 6
print("Spec 135 H1 Alpha 1-4 production workflows: PASS")
