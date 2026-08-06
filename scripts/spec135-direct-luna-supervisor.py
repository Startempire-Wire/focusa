#!/usr/bin/env python3
"""Three-minute direct Luna supervisor, independent of Focusa Work Loop/Silent Sessions."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parents[1]
GRAPH = ROOT / "docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-executable-callgraph.yaml"
RUNNER = ROOT / "scripts/spec135-direct-luna-runner.py"
DIRECT_ROOT = Path.home() / ".pi/agent/direct-runs/focusa-mission-canvas"
SUPERVISOR_ROOT = DIRECT_ROOT / "supervisor"
STATE_PATH = SUPERVISOR_ROOT / "state.json"
PID_PATH = SUPERVISOR_ROOT / "pid"
LOG_PATH = SUPERVISOR_ROOT / "supervisor.log"
INTERVAL_SECONDS = 180
MAX_WORKERS = 3
BASE_BRANCH = "local/spec158-desktop-pivot-audit-2026-08-04"
SOURCE_STAGE_SEEDS = {"ID-001", "ID-002", "ID-003", "ID-006", "ID-007"}
TASK_GROUPS = ("atomic_tasks", "operation_tasks", "integration_tasks")
PROTECTED_PATHS = {
    "docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-executable-callgraph.yaml",
    "docs/contracts/spec135-svelte-task-execution-index.v1.json",
    "docs/contracts/spec135-parallel-execution-plan.v1.json",
    "scripts/generate-spec135-svelte-execution-packets.py",
    "scripts/generate-spec135-parallel-execution-plan.py",
    "scripts/spec135-direct-luna-runner.py",
    "scripts/spec135-direct-luna-supervisor.py",
}


def log(message: str) -> None:
    timestamp = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    line = f"{timestamp} {message}"
    if sys.stdout.isatty():
        print(line, flush=True)
    SUPERVISOR_ROOT.mkdir(parents=True, exist_ok=True)
    with LOG_PATH.open("a") as handle:
        handle.write(line + "\n")


def load_tasks() -> dict[str, dict[str, Any]]:
    graph = yaml.safe_load(GRAPH.read_text())
    defaults = graph.get("operation_task_defaults", {})
    tasks = {}
    for group in TASK_GROUPS:
        for raw in graph.get(group, []) or []:
            task = dict(raw)
            task.setdefault("status", defaults.get("status", "blocked"))
            tasks[task["id"]] = task
    return tasks


def load_state() -> dict[str, Any]:
    if STATE_PATH.is_file():
        state = json.loads(STATE_PATH.read_text())
        state["staged"] = sorted(set(state.get("staged", [])) | SOURCE_STAGE_SEEDS)
        state.setdefault("blocked", {})
        state.setdefault("pull_requests", {})
        return state
    return {
        "schema": "focusa.direct_luna_supervisor.v1",
        "staged": sorted(SOURCE_STAGE_SEEDS),
        "blocked": {},
        "pull_requests": {},
    }


def save_state(state: dict[str, Any]) -> None:
    SUPERVISOR_ROOT.mkdir(parents=True, exist_ok=True)
    temporary = STATE_PATH.with_suffix(".tmp")
    temporary.write_text(json.dumps(state, indent=2, sort_keys=True) + "\n")
    temporary.replace(STATE_PATH)


def process_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
        return True
    except (ProcessLookupError, PermissionError):
        return False


def run_record(task_id: str) -> dict[str, Any] | None:
    path = DIRECT_ROOT / task_id / "run.json"
    return json.loads(path.read_text()) if path.is_file() else None


def exit_record(task_id: str) -> dict[str, Any] | None:
    path = DIRECT_ROOT / task_id / "exit.json"
    return json.loads(path.read_text()) if path.is_file() else None


def changed_paths(worktree: Path, base_commit: str) -> list[str]:
    output = subprocess.check_output(
        ["git", "diff", "--name-only", f"{base_commit}..HEAD"], cwd=worktree, text=True
    )
    return [line for line in output.splitlines() if line]


def unsafe_paths(paths: list[str]) -> list[str]:
    return [
        path for path in paths
        if path in PROTECTED_PATHS or path.startswith("docs/contracts/spec135-svelte-task-packets/")
    ]


def cleanup_worker(record: dict[str, Any]) -> None:
    worktree = Path(record["worktree"])
    branch = record["branch"]
    if worktree.exists():
        subprocess.run(["git", "worktree", "remove", "--force", str(worktree)], cwd=ROOT, check=True)
    subprocess.run(["git", "branch", "-D", branch], cwd=ROOT, check=False, capture_output=True)


def integrate_finished(state: dict[str, Any], tasks: dict[str, dict[str, Any]]) -> None:
    staged = set(state["staged"]) | SOURCE_STAGE_SEEDS
    state["staged"] = sorted(staged)
    blocked = state["blocked"]
    pull_requests = state.setdefault("pull_requests", {})
    for task_id in sorted(tasks):
        if task_id in staged or task_id in blocked:
            continue
        record = run_record(task_id)
        if not record:
            continue
        exit_data = exit_record(task_id)
        if not exit_data:
            continue
        worktree = Path(record["worktree"])
        branch = record["branch"]
        if not worktree.exists():
            continue
        if exit_data.get("exit_code") != 0:
            blocked[task_id] = f"worker exit {exit_data.get('exit_code')}"
            log(f"BLOCK {task_id}: {blocked[task_id]}")
            continue
        status = subprocess.check_output(["git", "status", "--porcelain"], cwd=worktree, text=True)
        if status.strip():
            blocked[task_id] = "worker exited with uncommitted changes"
            log(f"BLOCK {task_id}: {blocked[task_id]}")
            continue
        paths = changed_paths(worktree, record["base_commit"])
        forbidden = unsafe_paths(paths)
        if forbidden:
            blocked[task_id] = f"worker changed protected paths: {forbidden}"
            log(f"BLOCK {task_id}: {blocked[task_id]}")
            continue
        commits = subprocess.check_output(
            ["git", "rev-list", "--reverse", f"{record['base_commit']}..HEAD"],
            cwd=worktree,
            text=True,
        ).split()
        if not commits:
            blocked[task_id] = "worker produced no commit"
            log(f"BLOCK {task_id}: {blocked[task_id]}")
            continue
        if subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True).strip():
            log("WAIT integration: primary worktree is dirty")
            return
        evidence_path = worktree / f"docs/contracts/evidence/spec135-svelte-tasks/{task_id}.json"
        if not evidence_path.is_file():
            blocked[task_id] = "worker produced no task evidence"
            log(f"BLOCK {task_id}: {blocked[task_id]}")
            continue
        evidence = json.loads(evidence_path.read_text())
        source_ready = evidence.get("status") != "blocked" and not evidence.get("remaining_work")
        try:
            subprocess.run(["git", "push", "-u", "origin", branch], cwd=worktree, check=True, capture_output=True, text=True)
            pr_url = subprocess.check_output(
                [
                    "gh", "pr", "create", "--base", BASE_BRANCH, "--head", branch,
                    "--title", f"spec135: {task_id} {tasks[task_id].get('title', '')}",
                    "--body", (
                        f"Automated bounded Luna submission for `{task_id}`.\\n\\n"
                        f"Task packet: `{tasks[task_id].get('execution_packet_ref', '')}`\\n"
                        f"Evidence: `docs/contracts/evidence/spec135-svelte-tasks/{task_id}.json`\\n\\n"
                        "The orchestrator verified process exit, clean worktree, protected-path boundaries, and task-scoped evidence before merge."
                    ),
                ], cwd=worktree, text=True
            ).strip()
            pull_requests[task_id] = pr_url
            save_state(state)
            log(f"PR {task_id}: {pr_url}")
            if not source_ready:
                blocked[task_id] = "task evidence reports unresolved source work"
                save_state(state)
                log(f"BLOCK {task_id}: unresolved source work; PR left open for orchestrator")
                continue
            subprocess.run(
                ["gh", "pr", "merge", pr_url, "--squash", "--delete-branch"],
                cwd=ROOT, check=True, capture_output=True, text=True,
            )
            subprocess.run(["git", "fetch", "origin", BASE_BRANCH], cwd=ROOT, check=True, capture_output=True)
            subprocess.run(["git", "merge", "--ff-only", f"origin/{BASE_BRANCH}"], cwd=ROOT, check=True, capture_output=True)
        except subprocess.CalledProcessError as error:
            blocked[task_id] = f"pull request integration failed: {(error.stderr or '')[-500:]}"
            log(f"BLOCK {task_id}: pull request integration failed")
            continue
        staged.add(task_id)
        state["staged"] = sorted(staged)
        save_state(state)
        cleanup_worker(record)
        log(f"MERGED {task_id}: {pr_url}; {len(commits)} commit(s), {len(paths)} path(s)")


def lane(task_id: str) -> str:
    prefix = task_id.split("-", 1)[0]
    if task_id in {"ID-003", "ID-004", "ID-005", "ID-006", "ID-007"}:
        return task_id
    if prefix in {"DOMAIN", "ACCEPT"}:
        return task_id
    return prefix


def targets(task: dict[str, Any]) -> set[str]:
    return {
        value for value in task.get("targets", []) or []
        if isinstance(value, str) and "/" in value and " " not in value
    }


def select_ready(
    state: dict[str, Any],
    tasks: dict[str, dict[str, Any]],
    capacity: int,
    active_task_ids: set[str],
) -> list[str]:
    done = {
        task_id for task_id, task in tasks.items()
        if task["status"] == "complete"
    } | SOURCE_STAGE_SEEDS | set(state["staged"])
    blocked = set(state["blocked"])
    active = {
        task_id for task_id in tasks
        if (record := run_record(task_id))
        and not exit_record(task_id)
        and process_alive(int(record["pid"]))
    }
    ready = [
        task for task_id, task in sorted(tasks.items())
        if task_id not in done | blocked | active
        and all(dependency in done or dependency not in tasks for dependency in task.get("depends_on", []))
    ]
    selected: list[dict[str, Any]] = []
    active_tasks = [tasks[task_id] for task_id in active_task_ids if task_id in tasks]
    for task in ready:
        if len(selected) >= capacity:
            break
        occupied_tasks = [*active_tasks, *selected]
        if lane(task["id"]) in {lane(item["id"]) for item in occupied_tasks}:
            continue
        occupied = set().union(*(targets(item) for item in occupied_tasks)) if occupied_tasks else set()
        if occupied.intersection(targets(task)):
            continue
        selected.append(task)
    return [task["id"] for task in selected]


def tick() -> None:
    state = load_state()
    tasks = load_tasks()
    integrate_finished(state, tasks)
    running = []
    for task_id in tasks:
        record = run_record(task_id)
        if record and not exit_record(task_id) and process_alive(int(record["pid"])):
            running.append(task_id)
    capacity = max(0, MAX_WORKERS - len(running))
    launches = select_ready(state, tasks, capacity, set(running))
    for task_id in launches:
        log(f"LAUNCH {task_id}: source_staging")
        result = subprocess.run(
            [sys.executable, str(RUNNER), "start", task_id, "--source-stage"],
            cwd=ROOT,
            text=True,
            capture_output=True,
        )
        if result.returncode != 0:
            state["blocked"][task_id] = f"launch failed: {result.stderr[-500:]}"
            log(f"BLOCK {task_id}: launch failed")
        else:
            log(f"STARTED {task_id}")
        save_state(state)
    log(
        f"HEARTBEAT running={len(running) + len(launches)} "
        f"staged={len(state['staged'])} blocked={len(state['blocked'])}"
    )


def run_loop() -> None:
    SUPERVISOR_ROOT.mkdir(parents=True, exist_ok=True)
    PID_PATH.write_text(str(os.getpid()))
    log(f"SUPERVISOR START interval={INTERVAL_SECONDS}s max_workers={MAX_WORKERS}")
    try:
        while True:
            try:
                tick()
            except Exception as error:
                log(f"TICK ERROR {type(error).__name__}: {error}")
            time.sleep(INTERVAL_SECONDS)
    finally:
        PID_PATH.unlink(missing_ok=True)


def start_daemon() -> None:
    if PID_PATH.is_file() and process_alive(int(PID_PATH.read_text())):
        print(f"Supervisor already running pid={PID_PATH.read_text().strip()}")
        return
    SUPERVISOR_ROOT.mkdir(parents=True, exist_ok=True)
    output = LOG_PATH.open("a")
    process = subprocess.Popen(
        [sys.executable, str(Path(__file__).resolve()), "run"],
        cwd=ROOT,
        stdin=subprocess.DEVNULL,
        stdout=output,
        stderr=subprocess.STDOUT,
        start_new_session=True,
    )
    print(f"Started supervisor pid={process.pid} log={LOG_PATH}")


def show_status() -> None:
    pid = int(PID_PATH.read_text()) if PID_PATH.is_file() else 0
    state = load_state()
    print(json.dumps({
        "running": bool(pid and process_alive(pid)),
        "pid": pid or None,
        "interval_seconds": INTERVAL_SECONDS,
        "max_workers": MAX_WORKERS,
        "staged": state["staged"],
        "blocked": state["blocked"],
        "pull_requests": state.get("pull_requests", {}),
        "log": str(LOG_PATH),
    }, indent=2))


def stop_daemon() -> None:
    if not PID_PATH.is_file():
        return
    pid = int(PID_PATH.read_text())
    if process_alive(pid):
        os.killpg(pid, signal.SIGTERM)
    PID_PATH.unlink(missing_ok=True)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("start", "run", "tick", "status", "stop"))
    args = parser.parse_args()
    if args.action == "start":
        start_daemon()
    elif args.action == "run":
        run_loop()
    elif args.action == "tick":
        tick()
    elif args.action == "status":
        show_status()
    else:
        stop_daemon()


if __name__ == "__main__":
    main()
