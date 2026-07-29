#!/usr/bin/env python3
"""Spec 135J-5 runtime reuse/error envelope/drift proof."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
C=json.loads((ROOT/"docs/contracts/spec135-runtime-ownership-drift.v1.json").read_text())
assert C["operation_count"] >= 100
assert C["duplicate_operation_ids"] == []
ids=[o["operation_id"] for o in C["operations"]]
assert len(ids)==len(set(ids))
for o in C["operations"]:
    assert o["runtime_owner"], o
    assert o["core_action_ref"], o
    assert o["error_schema_ref"] == "focusa.tool_result.v1", o
    assert o["recovery_typed"] is True
E=C["error_envelope"]
assert E["recovery_actions_typed"] is True
assert E["raw_stack_visible"] is False
for field in ("error_code","scope_status","recovery_actions","evidence_refs"):
    assert field in E["required_fields"]
assert len(C["drift_gates"]) >= 6
for ref in C["proof_refs"]:
    assert (ROOT/ref).exists(), ref
print(f"Spec 135 J5 runtime ownership/error/drift gates: PASS ({len(ids)} operations)")
