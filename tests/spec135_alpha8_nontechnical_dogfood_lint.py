#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
B = R / "docs/contracts/spec135/generated-contract-v1"
proof = json.loads((B / "spec135-alpha8-nontechnical-dogfood-proof.json").read_text())
registry = json.loads((B / "operation-registry.json").read_text())
bindings = json.loads((B / "ui-action-bindings.fixture.json").read_text())
operations = {row["operation_id"] for row in registry["operations"]}
actions = {row["action_id"] for row in bindings["bindings"]}
assert len(proof["path"]) >= 9
for step in proof["path"]:
    assert step["action_id"] in operations
    assert step["action_id"] in actions
    assert step["visible_result"] and step["recovery"]
for ref in proof["uiai_eval_refs"]:
    result = json.loads((B / ref).read_text())
    assert result["status"] == "passed", ref
assert all(proof["acceptance"].values())
print("Spec 135 Alpha 8 permanent nontechnical dogfood path lint: PASS")
