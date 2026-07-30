#!/usr/bin/env python3
"""Validate the Spec 135 Mission Canvas completion pivot DAG."""
from __future__ import annotations

import json
import subprocess
from collections import Counter, deque
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GRAPH = ROOT / "docs/contracts/spec135-mission-canvas-completion-dag.v2.json"
GENERATOR = ROOT / "scripts/generate-spec135-mission-canvas-completion-dag.py"

subprocess.run(["python3", str(GENERATOR), "--check"], cwd=ROOT, check=True)
graph = json.loads(GRAPH.read_text())

assert graph["schema"] == "focusa.spec135.mission_canvas_completion_dag.v2"
assert graph["status"] == "operator_approval_required_before_implementation"
assert graph["task_count_excluding_gates"] >= 300
assert graph["node_count"] == len(graph["nodes"])
assert graph["edge_count"] == len(graph["edges"])
assert graph["operator_confirmations"] == {
    "replacement_text_outranks_images_and_older_contracts_for_occupancy": True,
    "images_are_populated_examples_not_fixed_inventory": True,
    "quality_compromise_allowed": False,
    "implementation_owner": "Pi extension",
    "required_platforms": ["macOS", "Windows", "Linux"],
    "release_path": "canonical Git/GitHub release pipeline only",
}
assert graph["trajectory_alignment"]["mlg"].startswith("Close all 30 remaining")
assert graph["trajectory_alignment"]["ready_frontier"] == [
    "SPEC135-RI4",
    "SPEC135-V3",
    "SPEC135-V4",
    "SPEC135-V5",
    "SPEC135-P2",
    "SPEC135-P4",
    "SPEC135-Q1",
    "SPEC135-Q3",
    "SPEC135-Q4",
    "SPEC135-E1",
]

nodes = {node["id"]: node for node in graph["nodes"]}
assert len(nodes) == graph["node_count"]
assert graph["implementation_start_gate"] in nodes
assert graph["final_gate"] in nodes
assert graph["critical_path"] == [f"P{index:02d}-GATE" for index in range(12)]

indegree = Counter()
children: dict[str, list[str]] = {node_id: [] for node_id in nodes}
for edge in graph["edges"]:
    assert edge["from"] in nodes
    assert edge["to"] in nodes
    children[edge["from"]].append(edge["to"])
    indegree[edge["to"]] += 1
queue = deque(node_id for node_id in nodes if indegree[node_id] == 0)
visited: list[str] = []
while queue:
    node_id = queue.popleft()
    visited.append(node_id)
    for child in children[node_id]:
        indegree[child] -= 1
        if indegree[child] == 0:
            queue.append(child)
assert len(visited) == len(nodes), "completion DAG contains a cycle"

all_text = json.dumps(graph, ensure_ascii=False).lower()
for required in (
    "resolvedworkspaceprojection",
    "no-dead-chrome",
    "focusa_pi_rich_window",
    "pi extension",
    "macos",
    "windows",
    "linux",
    "work surface",
    "a2ui",
    "crist",
    "uiai",
    "draft",
    "reconnect",
    "accessibility",
    "performance",
    "receipt",
):
    assert required in all_text, required

for forbidden_completion in (
    "terminal shell as complete gui",
    "static screenshots and handwritten proof json",
    "fixed-slot and process-local layout assumptions",
):
    assert forbidden_completion in all_text

finding_classes = {finding["classification"] for finding in graph["current_state_findings"]}
assert {
    "closure_drift",
    "rich_host_missing",
    "provider_blocked",
    "baseline_failure",
    "proof_missing",
    "cross_platform_boundary",
}.issubset(finding_classes)

print(
    "Spec 135 Mission Canvas completion DAG: PASS "
    f"({graph['task_count_excluding_gates']} tasks, "
    f"{graph['node_count']} nodes, {graph['edge_count']} edges)"
)
