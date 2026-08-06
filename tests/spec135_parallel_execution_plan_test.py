#!/usr/bin/env python3
"""Validate dependency and collision safety of the Spec 135 parallel plan."""
from pathlib import Path
import json
import subprocess
import yaml

ROOT = Path(__file__).resolve().parents[1]
subprocess.run(
    ["python3", "scripts/generate-spec135-parallel-execution-plan.py"],
    cwd=ROOT,
    check=True,
    capture_output=True,
    text=True,
)
plan = json.loads((ROOT / "docs/contracts/spec135-parallel-execution-plan.v1.json").read_text())
graph = yaml.safe_load(
    (ROOT / "docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-executable-callgraph.yaml").read_text()
)

tasks = []
for group in ("atomic_tasks", "operation_tasks", "integration_tasks"):
    tasks.extend(graph.get(group, []) or [])
all_ids = {task["id"] for task in tasks}
complete_ids = {task["id"] for task in tasks if task.get("status") == "complete"}
remaining_ids = all_ids - complete_ids

projected = [
    task
    for wave in plan["waves"]
    for batch in wave["batches"]
    for task in batch["tasks"]
]
projected_ids = [task["task_id"] for task in projected]
assert len(projected_ids) == len(set(projected_ids))
assert set(projected_ids) == remaining_ids
assert plan["remaining_task_count"] == len(remaining_ids)
assert plan["complete_task_count"] == len(complete_ids)
assert plan["activation_gate"]["launch_enabled"] is False
assert plan["activation_gate"]["issue_ref"].endswith("/issues/132")

wave_by_id = {task["task_id"]: task["wave"] for task in projected}
for task in projected:
    assert (ROOT / task["task_packet_ref"]).is_file(), task["task_id"]
    for dependency in task["depends_on"]:
        if dependency in remaining_ids:
            assert wave_by_id[dependency] < task["wave"], (task["task_id"], dependency)

for wave in plan["waves"]:
    for batch in wave["batches"]:
        lanes = [task["lane"] for task in batch["tasks"]]
        assert len(lanes) == len(set(lanes)), (wave["wave"], batch["batch"], lanes)
        targets = [target for task in batch["tasks"] for target in task["concrete_targets"]]
        assert len(targets) == len(set(targets)), (wave["wave"], batch["batch"], targets)

first_tasks = [task["task_id"] for task in plan["waves"][0]["batches"][0]["tasks"]]
assert first_tasks == ["ID-001"]
assert plan["waves"][7]["task_count"] == 5
assert plan["waves"][8]["task_count"] == 9
assert plan["waves"][9]["task_count"] == 8

print(
    "Spec 135 parallel execution plan: PASS "
    f"({len(remaining_ids)} tasks, {len(plan['waves'])} waves)"
)
