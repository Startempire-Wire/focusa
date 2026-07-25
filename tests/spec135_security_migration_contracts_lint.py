#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
C = R / "docs/contracts"

q2 = json.loads((C / "spec135-q2-security-privacy-gates.v1.yaml").read_text())
assert q2["mutation_gate"]["failure_posture"] == "fail_closed_with_tool_result"
assert q2["stream_gate"]["failure_posture"] == "deny_subscription"
assert all(q2["data_minimization"].values())
for ref in q2["evidence_refs"]:
    assert (R / ref).exists(), ref

migration = json.loads((C / "spec135-e2-e4-migration-plan.v1.yaml").read_text())
assert migration["requirements"] == ["SPEC135-E2", "SPEC135-E3", "SPEC135-E4"]
assert [row["phase"] for row in migration["expand_contract_phases"]] == [
    "expand",
    "prove",
    "contract",
]
assert migration["compatibility_guarantees"]["old_and_new_readers_coexist_until_proof"]
assert migration["compatibility_guarantees"]["single_canonical_writer"] == "focusa_core_reducer"
assert migration["acceptance"]["native_ui_path_primary"]
for ref in migration["evidence_refs"]:
    assert (R / ref).exists(), ref

print("Spec 135 Q2 and E2-E4 security/migration contract lint: PASS")
