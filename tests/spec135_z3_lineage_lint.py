#!/usr/bin/env python3
import json
import subprocess
from pathlib import Path

R = Path(__file__).resolve().parents[1]
proof = json.loads(
    (R / "docs/contracts/spec135/generated-contract-v1/spec135-z3-worktree-lineage-proof.json").read_text()
)
head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=R, text=True).strip()
branch = subprocess.check_output(["git", "branch", "--show-current"], cwd=R, text=True).strip()
assert proof["head"] == head
assert proof["branch"] == branch
assert proof["merge_target"] == "origin/main"
assert proof["behind"] == 0
assert proof["clean"] is True
assert proof["stable_bead_ids"] is True
assert proof["mission_canvas_bead_count"] >= 73
for ref in proof["restart_contract_refs"]:
    assert (R / ref).exists(), ref
assert all(proof["acceptance"].values())
print("Spec 135 Z3 restart and worktree lineage lint: PASS")
