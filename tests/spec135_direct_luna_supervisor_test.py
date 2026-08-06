#!/usr/bin/env python3
"""Static safety contract for the direct three-minute Luna supervisor."""
from pathlib import Path
import ast

ROOT = Path(__file__).resolve().parents[1]
path = ROOT / "scripts/spec135-direct-luna-supervisor.py"
source = path.read_text()
ast.parse(source)

assert "INTERVAL_SECONDS = 180" in source
assert "MAX_WORKERS = 3" in source
assert "integrate_finished" in source
assert "select_ready" in source
assert '"gh", "pr", "create"' in source
assert '"gh", "pr", "merge"' in source
assert '"git", "push", "-u", "origin"' in source
assert '"git", "merge", "--ff-only"' in source
assert "git\", \"worktree\", \"remove\", \"--force" in source
assert "source_staging" in source
assert "process_alive" in source
assert "PROTECTED_PATHS" in source
assert "docs/contracts/spec135-svelte-task-packets/" in source
assert "INTERVAL_SECONDS = 180" in source
assert "SOURCE_STAGE_SEEDS" in source
assert "HEARTBEAT" in source
assert "TICK ERROR" in source
assert "focusa_silent_sessions" not in source
assert "focusa_work_loop" not in source

print("Spec 135 direct Luna supervisor: PASS")
