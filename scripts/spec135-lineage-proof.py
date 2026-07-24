#!/usr/bin/env python3
import json
import subprocess
import sys
from pathlib import Path

R = Path(__file__).resolve().parents[1]

def git(*args):
    return subprocess.check_output(["git", *args], cwd=R, text=True).strip()

head = git("rev-parse", "HEAD")
branch = git("branch", "--show-current")
merge_target = "origin/main"
merge_base = git("merge-base", "HEAD", merge_target)
status = git("status", "--porcelain=v1")
ahead, behind = map(int, git("rev-list", "--left-right", "--count", f"HEAD...{merge_target}").split())
issues = []
for line in (R / ".beads/issues.jsonl").read_text().splitlines():
    issue = json.loads(line)
    if issue["id"].startswith("focusa-mc-"):
        issues.append({"id": issue["id"], "status": issue["status"]})
ids = [row["id"] for row in issues]
proof = {
    "schema": "focusa.spec135.worktree_lineage_proof.v1",
    "requirement_id": "SPEC135-Z3",
    "branch": branch,
    "head": head,
    "merge_target": merge_target,
    "merge_base": merge_base,
    "ahead": ahead,
    "behind": behind,
    "clean": not status,
    "stable_bead_ids": len(ids) == len(set(ids)) and len(ids) >= 73,
    "mission_canvas_bead_count": len(ids),
    "open_bead_ids": [row["id"] for row in issues if row["status"] != "closed"][:32],
    "restart_contract_refs": [
        "docs/contracts/spec135-q4-recovery-matrix.v1.yaml",
        "docs/contracts/spec135/generated-contract-v1/spec135-z2-permanent-integration-evidence.json"
    ],
    "evidence_ref": f"evidence:spec135-lineage:{head[:12]}",
    "acceptance": {
        "exact_head_and_merge_base_recorded": True,
        "worktree_clean": not status,
        "stable_bead_ids": len(ids) == len(set(ids)) and len(ids) >= 73,
        "merge_target_explicit": merge_target == "origin/main"
    }
}
print(json.dumps(proof, indent=2, sort_keys=True))
sys.exit(0 if all(proof["acceptance"].values()) else 1)
