#!/usr/bin/env python3
"""Validate the atomic, weaker-model-safe Spec 152F implementation graph."""

import hashlib
import json
from collections import Counter, defaultdict, deque
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATH = ROOT / "docs/contracts/spec152f-implementation-taskgraph.v1.json"
GRAPH = json.loads(PATH.read_text(encoding="utf-8"))
assert len(PATH.read_bytes().splitlines()) < 500

assert GRAPH["schema"] == "focusa.spec152f_implementation_taskgraph_index.v1"
assert GRAPH["authority"] == "docs/152f-simple-entitlement-gating-and-future-granularity-addendum.md"
assert GRAPH["parent"] == "focusa-vbcqu.20.14"
assert GRAPH["task_count"] == 52
assert GRAPH["first_task"] == "focusa-vbcqu.20.14.1"
assert GRAPH["final_task"] == "focusa-vbcqu.20.14.52"
assert GRAPH["phase_counts"] == {"00": 4, "01": 9, "02": 8, "03": 8, "04": 7, "05": 6, "06": 10}
assert GRAPH["internal_dependency_edge_count"] == 123
assert set(GRAPH["phase_files"]) == set(GRAPH["phase_counts"])
assert set(GRAPH["phase_file_sha256"]) == set(GRAPH["phase_counts"])

phase_documents = []
for phase, relative_path in sorted(GRAPH["phase_files"].items()):
    raw = (ROOT / relative_path).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == GRAPH["phase_file_sha256"][phase]
    document = json.loads(raw)
    assert document["schema"] == "focusa.spec152f_implementation_phase.v1"
    assert document["authority"] == GRAPH["authority"]
    assert document["parent"] == GRAPH["parent"]
    assert document["phase"] == phase
    assert document["task_count"] == GRAPH["phase_counts"][phase]
    assert len(raw.splitlines()) < 500
    phase_documents.append(document)

required_task_fields = {
    "id",
    "code",
    "phase",
    "title",
    "owner",
    "exact_surfaces",
    "required_steps",
    "required_output",
    "verification",
    "evidence_path",
    "done_condition",
    "dependencies",
    "estimate_minutes",
}
tasks = [task for document in phase_documents for task in document["tasks"]]
assert len(tasks) == 52
assert [task["id"] for task in tasks] == [f"focusa-vbcqu.20.14.{n}" for n in range(1, 53)]
assert len({task["code"] for task in tasks}) == 52
assert Counter(task["phase"] for task in tasks) == Counter(GRAPH["phase_counts"])

by_id = {task["id"]: task for task in tasks}
for task in tasks:
    assert set(task) == required_task_fields, task["id"]
    for field in required_task_fields - {"dependencies", "estimate_minutes"}:
        assert task[field], f"{task['id']}: empty {field}"
    assert task["estimate_minutes"] > 0
    assert task["evidence_path"] == f"docs/evidence/spec152f/{task['id']}-acceptance.txt"
    assert "&&" in task["verification"] or task["verification"].startswith(
        ("python3 ", "cargo ", "node ", "bash ")
    ), task["id"]

internal_edges = [
    {"blocked": task["id"], "blocker": dependency}
    for task in tasks
    for dependency in task["dependencies"]
    if dependency.startswith("focusa-vbcqu.20.14.")
]
assert len(internal_edges) == GRAPH["internal_dependency_edge_count"] == 123
assert len({(edge["blocked"], edge["blocker"]) for edge in internal_edges}) == 123
for edge in internal_edges:
    assert edge["blocked"] in by_id
    assert edge["blocker"] in by_id
    assert edge["blocker"] in by_id[edge["blocked"]]["dependencies"]

external_edges = GRAPH["external_dependency_edges"]
assert len(external_edges) == 8
for edge in external_edges:
    assert edge["blocked"] in by_id
    assert edge["blocker"].startswith("focusa-vbcqu.20.13.")
    assert edge["blocker"] in by_id[edge["blocked"]]["dependencies"]

assert GRAPH["downstream_release_edges"] == [
    {"blocked": "focusa-vbcqu.20.13.63", "blocker": "focusa-vbcqu.20.14.52"}
]

# Kahn traversal proves the internal graph is acyclic and fully reachable from roots.
indegree = {task_id: 0 for task_id in by_id}
dependents: dict[str, list[str]] = defaultdict(list)
for edge in internal_edges:
    indegree[edge["blocked"]] += 1
    dependents[edge["blocker"]].append(edge["blocked"])
queue = deque(sorted(task_id for task_id, degree in indegree.items() if degree == 0))
visited = []
while queue:
    task_id = queue.popleft()
    visited.append(task_id)
    for dependent in dependents[task_id]:
        indegree[dependent] -= 1
        if indegree[dependent] == 0:
            queue.append(dependent)
assert len(visited) == 52, "Spec 152F internal dependency cycle detected"
assert "focusa-vbcqu.20.14.1" in visited
assert "focusa-vbcqu.20.14.52" in visited

contract = GRAPH["weaker_model_contract"]
assert contract["before_start"]
assert contract["during_work"]
assert contract["before_close"]
assert any("one task at a time" in value for value in [GRAPH["execution_rule"]])
assert any("close only after technical acceptance" in value for value in contract["before_close"])

print("Spec 152F implementation task graph: PASS")
print("tasks=52 internal_edges=123 external_edges=8 cycles=0")
print("final_task=focusa-vbcqu.20.14.52")
