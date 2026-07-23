#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
B = R / "docs/contracts/spec135/generated-contract-v1"
a = json.loads((B / "spec135-alpha2-role-interview-proof.json").read_text())
assert a["status"] == "passed" and a["merge_gate"]["all_feeders_closed"] is True
assert a["merge_gate"]["required_feeders"] == [
    "SPEC135-ALPHA1",
    "SPEC135-RI1",
    "SPEC135-RI2",
    "SPEC135-RI3",
]
assert a["merge_gate"]["critical_path"] == [
    "SPEC135-ALPHA1",
    "SPEC135-RI1",
    "SPEC135-RI2",
    "SPEC135-RI3",
    "SPEC135-ALPHA2",
]
proofs = {
    k: json.loads((B / v["proof_ref"]).read_text())
    for k, v in a["acceptance_mapping"].items()
}
assert all(p["status"] == "passed" for p in proofs.values())
assert (
    "Role responsibility never grants operational permission"
    in proofs["role"]["invariants"]
)
assert "one primary question" in " ".join(proofs["strategy"]["invariants"]).lower()
assert "Reopen resumes the exact retained pointers" in proofs["interview"]["invariants"]
for ref in a["generated_ui_eval_refs"]:
    assert json.loads((B / ref).read_text())["status"] == "passed"
print(
    "Spec 135 Alpha 2 closure: PASS (approved Role, retrieval-first Grill, durable exact Interview resume)"
)
