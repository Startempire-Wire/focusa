#!/usr/bin/env python3
"""Validate the executable Mission Canvas Desktop callgraph."""
from pathlib import Path
import json
import yaml

ROOT = Path(__file__).resolve().parents[1]
GRAPH_PATH = ROOT / "docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-executable-callgraph.yaml"
PARENT_PATH = ROOT / "docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-desktop-callgraph.yaml"
DESKTOP_PATH = ROOT / "docs/transitions/FOCUSA-TRANSITION-001-desktop-implementation-task-graph.yaml"
REGISTRY_PATH = ROOT / "docs/contracts/spec135/mission-canvas-v1/operation-registry.json"
PROFILE_MATRIX_PATH = ROOT / "tests/fixtures/spec135-profile-activity-matrix.json"
COMPLETION_DAG_PATH = ROOT / "docs/contracts/spec135-mission-canvas-completion-dag.v2.json"
CARDINAL_TRANSLATION_REF = "CARDINAL-135-SVELTE-001"
TRANSLATION_MATRIX = "docs/transitions/FOCUSA-TRANSITION-001-spec135-svelte-translation-matrix.md"

G = yaml.safe_load(GRAPH_PATH.read_text())
PARENT = yaml.safe_load(PARENT_PATH.read_text())
DESKTOP = yaml.safe_load(DESKTOP_PATH.read_text())
REGISTRY = json.loads(REGISTRY_PATH.read_text())
MATRIX = json.loads(PROFILE_MATRIX_PATH.read_text())
COMPLETION_DAG = json.loads(COMPLETION_DAG_PATH.read_text())

tasks = G["atomic_tasks"] + G["operation_tasks"] + G["integration_tasks"]
ids = [task["id"] for task in tasks]
id_set = set(ids)
assert len(ids) == len(id_set), "duplicate executable task IDs"

missing_dependencies = {
    dependency
    for task in tasks
    for dependency in task.get("depends_on", [])
    if dependency not in id_set
}
assert not missing_dependencies, f"unknown dependencies: {sorted(missing_dependencies)}"

ordered = [task_id for group in G["execution_order"].values() for task_id in group]
assert len(ordered) == len(set(ordered)), "task appears more than once in execution_order"
assert set(ordered) == id_set, "execution_order must contain every task exactly once"

operation_tasks = [task for task in G["operation_tasks"] if task["id"].startswith("OPS-")]
registry_operations = {operation["operation_id"] for operation in REGISTRY["operations"]}
graph_operations = {task["operation"] for task in operation_tasks}
assert len(operation_tasks) == 25, "all 25 generated Mission Canvas operations need atomic tasks"
assert graph_operations == registry_operations, "operation tasks must match the generated registry exactly"

assert G["execution_order"]["completed_alignment"] == ["EX-001", "FIXTURE-001"]
assert G["execution_order"]["ready_now"] == []
assert next(task for task in tasks if task["id"] == "EX-001")["status"] == "complete"
assert next(task for task in tasks if task["id"] == "FIXTURE-001")["status"] == "complete"

assert PARENT["executable_child"] == str(GRAPH_PATH.relative_to(ROOT))
assert DESKTOP["mission_canvas_executable_callgraph"] == str(GRAPH_PATH.relative_to(ROOT))

assert (ROOT / TRANSLATION_MATRIX).is_file()
assert G["shared_sources"]["cardinal_translation"] == TRANSLATION_MATRIX
assert PARENT["source_precedence"][1] == TRANSLATION_MATRIX
assert G["cardinal_translation_contract"]["id"] == CARDINAL_TRANSLATION_REF
assert PARENT["cardinal_translation_contract"]["id"] == CARDINAL_TRANSLATION_REF
assert DESKTOP["cardinal_translation_contract"]["id"] == CARDINAL_TRANSLATION_REF
assert COMPLETION_DAG["cardinal_translation_rule"]["id"] == CARDINAL_TRANSLATION_REF
assert all(task["translation_contract_ref"] == CARDINAL_TRANSLATION_REF for task in tasks)
assert all(node["translation_contract_ref"] == CARDINAL_TRANSLATION_REF for node in PARENT["nodes"])
assert all(node["translation_contract_ref"] == CARDINAL_TRANSLATION_REF for node in DESKTOP["nodes"])
assert all(node["translation_contract_ref"] == CARDINAL_TRANSLATION_REF for node in COMPLETION_DAG["nodes"])

expected_profiles = {"general", "software", "legal", "markets", "research", "custom"}
expected_activities = {
    "overview", "context", "role", "interview", "spec", "tasks",
    "sessions", "documents", "research", "evidence", "history", "controls",
}
assert set(MATRIX["profiles"]) == expected_profiles
assert set(MATRIX["activities"]) == expected_activities

for required in (
    "fixed dashboard layouts",
    "client-local contribution eligibility",
    "handwritten duplicate DTOs",
    "inferred Workstream or Attachment identity",
):
    assert required in G["agent_contract"]["forbidden"]

print(f"Spec 158 Mission Canvas executable callgraph: PASS ({len(tasks)} atomic tasks, 25 operations)")
