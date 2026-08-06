#!/usr/bin/env python3
"""Generate a dependency-safe, collision-bounded Spec 135 parallel execution plan."""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parents[1]
GRAPH_PATH = ROOT / "docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-executable-callgraph.yaml"
OUTPUT_PATH = ROOT / "docs/contracts/spec135-parallel-execution-plan.v1.json"
TASK_GROUPS = ("atomic_tasks", "operation_tasks", "integration_tasks")

LANE_OVERRIDES = {
    "ID-003": "attachment-identity",
    "ID-004": "project-state-partition",
    "ID-005": "request-context",
    "ID-006": "migration-mapping",
    "ID-007": "legacy-quarantine",
}

LANES = {
    "ID": "identity-migration",
    "CORE": "core-reducer",
    "OPS": "api-operations",
    "UI": "desktop-svelte",
    "PROFILE": "desktop-svelte",
    "LIVE": "desktop-runtime",
    "BROWSER": "browser-artifacts",
    "CONTRACT": "generated-contracts",
    "DOMAIN": "domain-renderers",
    "GEN": "generated-ui",
    "PTY": "pty-runtime",
    "ACCEPT": "acceptance",
    "CUT": "integration-cutover",
    "POLISH": "release-polish",
    "FIXTURE": "fixtures",
    "EX": "examples",
}


def load_tasks() -> tuple[dict[str, Any], list[dict[str, Any]]]:
    graph = yaml.safe_load(GRAPH_PATH.read_text())
    operation_defaults = graph.get("operation_task_defaults", {})
    tasks: list[dict[str, Any]] = []
    for group in TASK_GROUPS:
        for source in graph.get(group, []) or []:
            task = dict(source)
            task.setdefault("status", operation_defaults.get("status", "blocked"))
            task.setdefault("title", task.get("operation_id", task["id"]))
            task["group"] = group
            tasks.append(task)
    return graph, tasks


def lane_for(task_id: str) -> str:
    if task_id in LANE_OVERRIDES:
        return LANE_OVERRIDES[task_id]
    prefix = task_id.split("-", 1)[0]
    if prefix in {"DOMAIN", "ACCEPT"}:
        return f"{LANES[prefix]}/{task_id.lower()}"
    return LANES.get(prefix, "integration")


def concrete_targets(task: dict[str, Any]) -> list[str]:
    targets: list[str] = []
    for value in task.get("targets", []) or []:
        if not isinstance(value, str):
            continue
        if "/" not in value or " " in value:
            continue
        targets.append(value.strip())
    return sorted(set(targets))


def topological_waves(tasks: list[dict[str, Any]]) -> list[list[dict[str, Any]]]:
    by_id = {task["id"]: task for task in tasks}
    closed = {task["id"] for task in tasks if task["status"] == "complete"}
    remaining = set(by_id) - closed
    waves: list[list[dict[str, Any]]] = []
    while remaining:
        ready = sorted(
            (
                by_id[task_id]
                for task_id in remaining
                if all(dep in closed or dep not in by_id for dep in by_id[task_id].get("depends_on", []))
            ),
            key=lambda task: task["id"],
        )
        if not ready:
            unresolved = {task_id: by_id[task_id].get("depends_on", []) for task_id in sorted(remaining)}
            raise RuntimeError(f"dependency cycle or unknown blocked chain: {unresolved}")
        waves.append(ready)
        closed.update(task["id"] for task in ready)
        remaining.difference_update(task["id"] for task in ready)
    return waves


def collision_batches(tasks: list[dict[str, Any]]) -> list[list[dict[str, Any]]]:
    batches: list[list[dict[str, Any]]] = []
    for task in tasks:
        lane = lane_for(task["id"])
        targets = set(concrete_targets(task))
        for batch in batches:
            occupied_lanes = {lane_for(item["id"]) for item in batch}
            occupied_targets = {target for item in batch for target in concrete_targets(item)}
            if lane not in occupied_lanes and not targets.intersection(occupied_targets):
                batch.append(task)
                break
        else:
            batches.append([task])
    return batches


def task_projection(task: dict[str, Any], wave: int, batch: int) -> dict[str, Any]:
    task_id = task["id"]
    return {
        "task_id": task_id,
        "title": task["title"],
        "source_status": task["status"],
        "lane": lane_for(task_id),
        "depends_on": task.get("depends_on", []),
        "task_packet_ref": task.get(
            "execution_packet_ref", f"docs/contracts/spec135-svelte-task-packets/{task_id}.json"
        ),
        "targets": task.get("targets", []),
        "concrete_targets": concrete_targets(task),
        "check": task.get("check"),
        "evidence_ref": f"docs/contracts/evidence/spec135-svelte-tasks/{task_id}.json",
        "branch": f"agents/spec135-{task_id.lower()}",
        "worktree_slot": f"$FOCUSA_WORKTREE_ROOT/spec135-{task_id.lower()}",
        "wave": wave,
        "batch": batch,
    }


def generate() -> dict[str, Any]:
    graph, tasks = load_tasks()
    waves = topological_waves(tasks)
    projected_waves = []
    remaining_count = sum(task["status"] != "complete" for task in tasks)
    for wave_index, wave_tasks in enumerate(waves, start=1):
        batches = collision_batches(wave_tasks)
        projected_waves.append(
            {
                "wave": wave_index,
                "dependency_ready_after_wave": wave_index - 1,
                "task_count": len(wave_tasks),
                "max_safe_concurrency": max((len(batch) for batch in batches), default=0),
                "batches": [
                    {
                        "batch": batch_index,
                        "tasks": [
                            task_projection(task, wave_index, batch_index) for task in batch
                        ],
                    }
                    for batch_index, batch in enumerate(batches, start=1)
                ],
            }
        )

    return {
        "schema": "focusa.spec135.parallel_execution_plan.v1",
        "status": "active_direct_pi_workaround",
        "translation_contract_ref": graph["cardinal_translation_contract"]["id"],
        "source_graph": str(GRAPH_PATH.relative_to(ROOT)),
        "source_task_count": len(tasks),
        "complete_task_count": len(tasks) - remaining_count,
        "remaining_task_count": remaining_count,
        "activation_gate": {
            "issue_ref": "https://github.com/Startempire-Wire/focusa/issues/132",
            "required_state": "fix merged and canary orchestration receipt verified",
            "focusa_orchestration_enabled": False,
            "direct_pi_workaround_enabled": True,
            "direct_runner": "scripts/spec135-direct-luna-runner.py",
        },
        "scheduler_policy": {
            "dependency_rule": "A wave starts only after every dependency task is integrated and complete.",
            "collision_rule": "At most one task per ownership lane and no shared concrete target in one batch.",
            "worker_rule": "One bounded task packet, branch, worktree, commit, and evidence artifact per worker.",
            "integration_rule": "Workers never edit central task status; the integration writer regenerates it after verification.",
            "failure_rule": "Failed or blocked work checkpoints and exits without selecting a speculative dependent task.",
            "release_rule": "Release remains blocked until all checks, Cargo gates, UIAI evidence, acceptance tasks, and receipts pass.",
        },
        "waves": projected_waves,
    }


def main() -> None:
    plan = generate()
    OUTPUT_PATH.write_text(json.dumps(plan, indent=2) + "\n")
    print(
        f"Spec 135 parallel plan: {plan['remaining_task_count']} remaining tasks, "
        f"{len(plan['waves'])} dependency waves"
    )


if __name__ == "__main__":
    main()
