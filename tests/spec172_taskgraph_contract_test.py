#!/usr/bin/env python3
"""Validate the weak-model-safe Spec 172 implementation overlay graph."""

import hashlib
import json
from collections import Counter, defaultdict, deque
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INDEX_PATH = ROOT / "docs/contracts/spec172-implementation-taskgraph.v1.json"
INDEX = json.loads(INDEX_PATH.read_text(encoding="utf-8"))

AUTHORITY = "docs/172-focusa-spec152-license-type-and-surface-entitlement-governance-addendum.md"
PARENT = "focusa-vbcqu.20.15"
PHASE_COUNTS = {"00": 5, "01": 4, "02": 8, "03": 7, "04": 7, "05": 11}
REQUIRED_FIELDS = {
    "id", "code", "phase", "title", "lane", "owner", "exact_surfaces",
    "required_steps", "required_output", "verification", "evidence_path",
    "done_condition", "dependencies", "estimate_minutes",
}

assert INDEX["schema"] == "focusa.spec172_implementation_taskgraph_index.v1"
assert INDEX["authority"] == AUTHORITY
assert INDEX["parent"] == PARENT
assert INDEX["task_count"] == 42
assert INDEX["first_task"] == f"{PARENT}.1"
assert INDEX["final_task"] == f"{PARENT}.42"
assert INDEX["phase_counts"] == PHASE_COUNTS
assert set(INDEX["phase_files"]) == set(PHASE_COUNTS)
assert set(INDEX["phase_file_sha256"]) == set(PHASE_COUNTS)
assert len(INDEX_PATH.read_bytes().splitlines()) < 500

phase_docs = []
for phase, relative_path in sorted(INDEX["phase_files"].items()):
    path = ROOT / relative_path
    raw = path.read_bytes()
    assert len(raw.splitlines()) < 500, path
    assert hashlib.sha256(raw).hexdigest() == INDEX["phase_file_sha256"][phase]
    doc = json.loads(raw)
    assert doc["schema"] == "focusa.spec172_implementation_phase.v1"
    assert doc["authority"] == AUTHORITY
    assert doc["parent"] == PARENT
    assert doc["phase"] == phase
    assert doc["task_count"] == PHASE_COUNTS[phase]
    phase_docs.append(doc)

tasks = [task for doc in phase_docs for task in doc["tasks"]]
assert len(tasks) == 42
assert [task["id"] for task in tasks] == [f"{PARENT}.{n}" for n in range(1, 43)]
assert len({task["code"] for task in tasks}) == 42
assert Counter(task["phase"] for task in tasks) == Counter(PHASE_COUNTS)
assert {task["lane"] for task in tasks} == {
    "governance", "spec152e", "spec152f", "surfaces", "acceptance"
}

by_id = {task["id"]: task for task in tasks}
for task in tasks:
    assert set(task) == REQUIRED_FIELDS, task["id"]
    for field in REQUIRED_FIELDS - {"dependencies", "estimate_minutes"}:
        assert task[field], f"{task['id']}: empty {field}"
    assert task["estimate_minutes"] > 0
    assert task["evidence_path"] == f"docs/evidence/spec172/{task['id']}-acceptance.txt"
    assert task["verification"].startswith(("python3 ", "cargo ", "php ", "node ", "bash "))
    assert "exact" in task["done_condition"].lower() or task["done_condition"].strip()

internal_edges = []
external_edges = []
for task in tasks:
    for dependency in task["dependencies"]:
        edge = {"blocked": task["id"], "blocker": dependency}
        if dependency.startswith(f"{PARENT}."):
            internal_edges.append(edge)
        else:
            external_edges.append(edge)

assert len(internal_edges) == INDEX["internal_dependency_edge_count"] == 114
assert len(external_edges) == INDEX["external_dependency_edge_count"] == 56
assert len({(e["blocked"], e["blocker"]) for e in internal_edges}) == len(internal_edges)
assert len({(e["blocked"], e["blocker"]) for e in external_edges}) == len(external_edges)
for edge in internal_edges:
    assert edge["blocked"] in by_id
    assert edge["blocker"] in by_id
for edge in external_edges:
    assert edge["blocker"].startswith("focusa-vbcqu.")

# Kahn traversal proves the internal graph is acyclic.
indegree = {task_id: 0 for task_id in by_id}
dependents = defaultdict(list)
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
assert len(visited) == 42, "Spec 172 internal dependency cycle detected"

expected_downstream = {
    ("focusa-vbcqu.20.13.3", f"{PARENT}.8"),
    ("focusa-vbcqu.20.13.20", f"{PARENT}.10"),
    ("focusa-vbcqu.20.14.6", f"{PARENT}.18"),
    ("focusa-vbcqu.20.14.7", f"{PARENT}.18"),
    ("focusa-vbcqu.20.14.49", f"{PARENT}.40"),
    ("focusa-vbcqu.20.14.52", f"{PARENT}.42"),
}
assert {(e["blocked"], e["blocker"]) for e in INDEX["downstream_edges"]} == expected_downstream

overlays = INDEX["existing_task_overlays"]
overlay_ids = {entry["task_id"] for entry in overlays}
assert len(overlays) == len(overlay_ids) == 14
assert {
    "focusa-vbcqu.20.13.3", "focusa-vbcqu.20.13.20",
    "focusa-vbcqu.20.13.47", "focusa-vbcqu.20.13.49",
    "focusa-vbcqu.20.13.58", "focusa-vbcqu.20.14.6",
    "focusa-vbcqu.20.14.7", "focusa-vbcqu.20.14.8",
    "focusa-vbcqu.20.14.10", "focusa-vbcqu.20.14.11",
    "focusa-vbcqu.20.14.35", "focusa-vbcqu.20.14.43",
    "focusa-vbcqu.20.14.45", "focusa-vbcqu.20.14.46",
} == overlay_ids
for entry in overlays:
    text = entry["binding_overlay"]
    assert text.startswith("Binding Spec172 overlay:")
    assert any(term in text for term in ("Evaluation", "Operator", "Bundle", "sales", "family", "families", "limited"))

# Commercial constants and hole-closure surfaces must be explicit in task bodies.
corpus = "\n".join(
    task[field]
    for task in tasks
    for field in ("title", "exact_surfaces", "required_steps", "done_condition")
)
for required in (
    "verified_no_license", "$1,254.60", "Focusa Operator", "UIAI Operator",
    "Navigator", "Download 453", "Gravity", "Stripe", "one mutable project",
    "Focusa Desktop", "Cockpit", "menubar", "dynamic", "signed manifest",
    "hosted", "three shared nodes", "whole-order", "no-sales", "exact-SHA",
):
    assert required.lower() in corpus.lower(), required

contract = INDEX["weaker_model_contract"]
assert len(contract["before_start"]) >= 3
assert len(contract["during_work"]) >= 3
assert len(contract["before_close"]) >= 3
assert "one atomic task" in INDEX["execution_rule"]
assert any("exact verification" in item for item in contract["before_close"])
assert any("Do not implement adjacent tasks" in item for item in contract["during_work"])

print("Spec 172 implementation task graph: PASS")
print("tasks=42 phases=6 internal_edges=114 external_edges=56 overlays=14 cycles=0")
print("final_task=focusa-vbcqu.20.15.42")
