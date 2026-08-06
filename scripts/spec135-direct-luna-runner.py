#!/usr/bin/env python3
"""Direct Pi/Luna worker runner that does not use Work Loop or Silent Sessions."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shlex
import signal
import subprocess
import sys
import time
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
PLAN_PATH = ROOT / "docs/contracts/spec135-parallel-execution-plan.v1.json"
RUN_ROOT = Path.home() / ".pi/agent/direct-runs/focusa-mission-canvas"
WORKTREE_ROOT = Path.home() / "focusa-agent-worktrees"
PI = Path("/Applications/ServBay/package/node/22/22.23.1/bin/pi")
PROVIDER = "openai-codex"
MODEL = "gpt-5.6-luna"
THINKING = "max"


def plan_tasks() -> dict[str, dict[str, Any]]:
    plan = json.loads(PLAN_PATH.read_text())
    return {
        task["task_id"]: task
        for wave in plan["waves"]
        for batch in wave["batches"]
        for task in batch["tasks"]
    }


def pid_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except (ProcessLookupError, PermissionError):
        return False
    return True


def metadata(task_id: str) -> dict[str, Any] | None:
    path = RUN_ROOT / task_id / "run.json"
    return json.loads(path.read_text()) if path.is_file() else None


def build_prompt(task: dict[str, Any], base_commit: str) -> str:
    return f"""You are a bounded Luna Max implementation worker for Focusa Mission Canvas.

Execute exactly {task['task_id']}: {task['title']}.
Task packet: {task['task_packet_ref']}
Evidence destination: {task['evidence_ref']}
Base commit: {base_commit}
Branch: {task['branch']}

Read AGENTS.md, the complete task packet, and every required source before editing.
Follow CARDINAL-135-SVELTE-001: Mission Canvas Pi-overlay behavior translates to the Focusa Desktop Svelte Mission Canvas tab; Agent TUI remains separate.
Work only on this task and its exact targets. Do not infer missing identity, bindings, operations, paths, or acceptance criteria.
Do not edit the central executable graph, execution index, generated task packets, or another task's evidence; the integration writer owns them.
Do not run Cargo commands: the operator's pre-50-percent Cargo prohibition remains binding.
Do not merge, rebase, push, release, or modify another worktree.
Run all permitted non-Cargo checks from the packet. Add or update only {task['evidence_ref']} with truthful partial/verified status.
Commit bounded changes on your branch. If no safe change remains, write a concise result to DIRECT_LUNA_RESULT.md, commit it, and stop.
If blocked by a dependency or authority boundary, record the exact blocker in DIRECT_LUNA_RESULT.md, commit it, and stop without selecting another task.
"""


def create_worktree(task: dict[str, Any], base_commit: str) -> Path:
    worktree = WORKTREE_ROOT / f"spec135-{task['task_id'].lower()}"
    branch = task["branch"]
    WORKTREE_ROOT.mkdir(parents=True, exist_ok=True)
    if worktree.exists():
        current_branch = subprocess.check_output(
            ["git", "branch", "--show-current"], cwd=worktree, text=True
        ).strip()
        if current_branch != branch:
            raise SystemExit(
                f"Existing worktree branch mismatch for {task['task_id']}: {current_branch} != {branch}"
            )
    else:
        subprocess.run(
            ["git", "worktree", "add", "-b", branch, str(worktree), base_commit],
            cwd=ROOT,
            check=True,
        )
    status = subprocess.check_output(
        ["git", "status", "--porcelain"], cwd=worktree, text=True
    )
    if status.strip():
        raise SystemExit(
            f"Refusing to launch {task['task_id']} from an incomplete or dirty worktree: {worktree}"
        )
    actual_head = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=worktree, text=True
    ).strip()
    if actual_head != base_commit:
        raise SystemExit(
            f"Worktree base mismatch for {task['task_id']}: {actual_head} != {base_commit}"
        )
    return worktree


def start(task_id: str) -> None:
    tasks = plan_tasks()
    if task_id not in tasks:
        raise SystemExit(f"Task is not in the remaining plan: {task_id}")
    prior = metadata(task_id)
    if prior and pid_alive(int(prior["pid"])):
        raise SystemExit(f"Worker already running for {task_id}: pid={prior['pid']}")

    base_commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    task = tasks[task_id]
    worktree = create_worktree(task, base_commit)
    run_dir = RUN_ROOT / task_id
    run_dir.mkdir(parents=True, exist_ok=True)
    session_dir = run_dir / "sessions"
    session_dir.mkdir(exist_ok=True)
    prompt = build_prompt(task, base_commit)
    (run_dir / "prompt.txt").write_text(prompt)
    log = (run_dir / "output.log").open("ab", buffering=0)
    exit_path = run_dir / "exit.json"
    exit_path.unlink(missing_ok=True)

    pi_command = [
        str(PI),
        "-p",
        "--provider", PROVIDER,
        "--model", MODEL,
        "--thinking", THINKING,
        "--no-extensions",
        "--no-skills",
        "--no-prompt-templates",
        "--no-themes",
        "--tools", "read,bash,edit,write",
        "--session-dir", str(session_dir),
        "--name", f"spec135-{task_id.lower()}",
        prompt,
    ]
    wrapped = (
        f"{shlex.join(pi_command)}; rc=$?; "
        f"printf '{{\"exit_code\":%s,\"finished_at\":\"%s\"}}\\n' \"$rc\" \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\" > {shlex.quote(str(exit_path))}; "
        "exit $rc"
    )
    process = subprocess.Popen(
        ["/bin/zsh", "-lc", wrapped],
        cwd=worktree,
        stdin=subprocess.DEVNULL,
        stdout=log,
        stderr=subprocess.STDOUT,
        start_new_session=True,
    )
    record = {
        "schema": "focusa.direct_luna_run.v1",
        "task_id": task_id,
        "pid": process.pid,
        "provider": PROVIDER,
        "model": MODEL,
        "thinking": THINKING,
        "base_commit": base_commit,
        "branch": task["branch"],
        "worktree": str(worktree),
        "log": str(run_dir / "output.log"),
        "session_dir": str(session_dir),
        "started_at_epoch": time.time(),
        "bypasses_focusa_work_loop": True,
        "bypasses_focusa_silent_sessions": True,
    }
    (run_dir / "run.json").write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps(record, indent=2))


def status(task_id: str | None) -> None:
    ids = [task_id] if task_id else sorted(path.name for path in RUN_ROOT.iterdir()) if RUN_ROOT.exists() else []
    rows = []
    for current in ids:
        record = metadata(current)
        if not record:
            continue
        exit_path = RUN_ROOT / current / "exit.json"
        exit_data = json.loads(exit_path.read_text()) if exit_path.is_file() else None
        rows.append({
            "task_id": current,
            "pid": record["pid"],
            "running": pid_alive(int(record["pid"])) and exit_data is None,
            "exit": exit_data,
            "branch": record["branch"],
            "worktree": record["worktree"],
            "log": record["log"],
        })
    print(json.dumps(rows, indent=2))


def tail(task_id: str, lines: int) -> None:
    record = metadata(task_id)
    if not record:
        raise SystemExit(f"No run metadata for {task_id}")
    log_path = Path(record["log"])
    if not log_path.is_file():
        return
    content = log_path.read_text(errors="replace").splitlines()
    print("\n".join(content[-lines:]))


def stop(task_id: str) -> None:
    record = metadata(task_id)
    if not record:
        raise SystemExit(f"No run metadata for {task_id}")
    pid = int(record["pid"])
    if pid_alive(pid):
        os.killpg(pid, signal.SIGTERM)
        print(f"Stopped {task_id} pid={pid}")


def main() -> None:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action", required=True)
    start_parser = sub.add_parser("start")
    start_parser.add_argument("task_id")
    status_parser = sub.add_parser("status")
    status_parser.add_argument("task_id", nargs="?")
    tail_parser = sub.add_parser("tail")
    tail_parser.add_argument("task_id")
    tail_parser.add_argument("--lines", type=int, default=80)
    stop_parser = sub.add_parser("stop")
    stop_parser.add_argument("task_id")
    args = parser.parse_args()
    if args.action == "start":
        start(args.task_id)
    elif args.action == "status":
        status(args.task_id)
    elif args.action == "tail":
        tail(args.task_id, args.lines)
    elif args.action == "stop":
        stop(args.task_id)


if __name__ == "__main__":
    main()
