#!/usr/bin/env python3
"""Execute every learned release recurrence guard before immutable tagging."""

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONFIG = ROOT / "config/release-learning-guards.json"


def resolve_command(command: list[str]) -> list[str]:
    if not command or command[0] != "cargo":
        return command
    override = os.environ.get("FOCUSA_RELEASE_CARGO", "").strip()
    if override:
        return [override, *command[1:]]
    route = shutil.which("focusa-command-route")
    if route:
        return [route, "cargo", shutil.which("cargo") or "/usr/bin/cargo", *command[1:]]
    try:
        probe = subprocess.run(command[:1] + ["--version"], cwd=ROOT, text=True, capture_output=True)
    except (FileNotFoundError, PermissionError):
        probe = None
    if probe is not None and probe.returncode == 0:
        return command
    # Root's stable proxy can exist without usable cargo/rustc components.
    # Nightly is the installed fallback and satisfies the workspace rust-version;
    # operators may pin another compatible toolchain explicitly.
    toolchain = os.environ.get("FOCUSA_RELEASE_RUST_TOOLCHAIN", "nightly")
    resolved = subprocess.run(
        ["rustup", "which", "--toolchain", toolchain, "cargo"],
        cwd=ROOT,
        text=True,
        capture_output=True,
    )
    candidate = resolved.stdout.strip()
    if resolved.returncode == 0 and candidate:
        return ["rustup", "run", toolchain, "cargo", *command[1:]]
    return command


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--output")
    args = parser.parse_args()

    config = json.loads(CONFIG.read_text())
    if config.get("schema") != "focusa.release_learning_guards.v1":
        raise SystemExit("release learning guard schema mismatch")
    output = Path(
        args.output
        or os.environ.get("FOCUSA_LEARNING_GUARDS_ARTIFACT", "").strip()
        or f"/tmp/focusa-{os.getuid()}-{args.tag.removeprefix('v')}-learning-guards.json"
    )
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    if output.exists():
        cached = json.loads(output.read_text())
        if cached.get("status") == "passed" and cached.get("tag") == args.tag and cached.get("git_head") == head:
            print(json.dumps(cached, indent=2, sort_keys=True))
            return 0
    results = []
    started = time.monotonic()
    for guard in config.get("guards", []):
        configured_command = [str(part).replace("{tag}", args.tag) for part in guard["command"]]
        command = resolve_command(configured_command)
        run = subprocess.run(command, cwd=ROOT, text=True, capture_output=True)
        results.append(
            {
                "failure_class": guard["failure_class"],
                "lesson_ref": guard["lesson_ref"],
                "command": command,
                "status": "passed" if run.returncode == 0 else "blocked",
                "returncode": run.returncode,
                "result": (run.stdout or run.stderr).strip()[-300:],
            }
        )
        if run.returncode != 0:
            break
    result = {
        "schema": "focusa.release_learning_guard_result.v1",
        "status": "passed" if len(results) == len(config.get("guards", [])) and all(row["status"] == "passed" for row in results) else "blocked",
        "tag": args.tag,
        "git_head": head,
        "elapsed_seconds": round(time.monotonic() - started, 3),
        "guards_total": len(config.get("guards", [])),
        "guards_run": len(results),
        "guards": results,
    }
    output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
