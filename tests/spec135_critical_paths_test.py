#!/usr/bin/env python3
"""Prove every vertical Alpha slice has explicit edge-valid functional critical paths."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
dag = json.loads((ROOT / "docs/contracts/spec135-delivery-dag.v1.yaml").read_text())
contract = dag["critical_path_contract"]
slices = contract["slices"]
alpha_order = dag["alpha_order"]
edges = {
    (edge["from"], edge["to"]) for edge in dag["edges"] if edge["kind"] == "blocks"
}
nodes = {node["requirement_id"] for node in dag["nodes"]}

assert [item["slice_id"] for item in slices] == alpha_order
assert len(slices) == 8
required_traversal = {
    "requirement_id",
    "greater_primitive",
    "schema",
    "reducer_event",
    "typed_api",
    "generated_typescript_client",
    "generated_go_client",
    "operation_registry_action_binding",
    "a2ui_lit_renderer",
    "trusted_focusa_svelte_element",
    "real_integration",
    "focused_runtime_tests",
    "uiai_eval_when_browser_facing",
    "evidence",
    "receipt",
    "closure",
}
assert set(contract["integration_traversal"]) == required_traversal

for index, item in enumerate(slices):
    slice_id = item["slice_id"]
    assert slice_id in nodes
    assert item["operator_outcome"].strip()
    actual_gates = {source for source, target in edges if target == slice_id}
    assert set(item["merge_gates"]) == actual_gates, (
        slice_id,
        item["merge_gates"],
        actual_gates,
    )
    paths = item["feeder_paths"]
    assert paths and all(path[-1] == slice_id for path in paths)
    for path in paths:
        assert len(path) >= 2 and all(step in nodes for step in path)
        for pair in zip(path, path[1:]):
            assert pair in edges, (
                f"{slice_id}: missing blocking edge {pair[0]} -> {pair[1]}"
            )
    if index:
        assert [alpha_order[index - 1], slice_id] in paths

spec = (
    ROOT
    / "docs/135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md"
).read_text()
assert "## 10. Cross-Functional Alpha" in spec
assert "### 10.1 Critical paths to the vertical slices" in spec
for n in range(1, 9):
    assert f"Alpha {n}" in spec
assert "requirement → greater primitive → schema → reducer/event → typed API" in spec

print(
    "Spec 135 critical paths: PASS (8 Alpha slices, all merge gates and feeder edges explicit)"
)
