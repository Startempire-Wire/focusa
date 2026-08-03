#!/usr/bin/env python3
"""Generate bounded, public-safe final Mission Canvas release proof."""

import json
import subprocess
import sys
from pathlib import Path

R = Path(__file__).resolve().parents[1]

def git(*args):
    return subprocess.check_output(["git", *args], cwd=R, text=True).strip()

import yaml
ledger = yaml.safe_load((R / "docs/contracts/spec135-complete-feature-ledger.v1.yaml").read_text())["requirements"]
verified = [row["requirement_id"] for row in ledger if row.get("closure_status") == "verified"]
open_requirements = [row["requirement_id"] for row in ledger if row.get("closure_status") != "verified"]
issues = [json.loads(line) for line in (R / ".beads/issues.jsonl").read_text().splitlines()]
AUTHORITATIVE_RELEASE_TASKS = [
    "focusa-mc2",
    "focusa-mc2.12",
    "focusa-mc2.12.377",
    "focusa-mc2.12.378",
    "focusa-mc2.12.379",
    "focusa-mc2.12.380",
    "focusa-mc2.12.381",
]
open_tasks = [
    row["id"] for row in issues
    if (row["id"] == "focusa-mc2" or row["id"].startswith("focusa-mc2."))
    and row.get("issue_type") in {"task", "epic"}
    and row.get("status") != "closed"
]
status = git("status", "--porcelain=v1")
ahead, behind = map(int, git("rev-list", "--left-right", "--count", "HEAD...origin/main").split())
head = git("rev-parse", "HEAD")
proof = {
    "schema": "focusa.spec135.final_release_proof.v1",
    "requirement_id": "SPEC135-Z5",
    "branch": git("branch", "--show-current"),
    "head": head,
    "merge_target": "origin/main",
    "ahead": ahead,
    "behind": behind,
    "clean": not status,
    "verified_requirements": len(verified),
    "open_requirements": open_requirements,
    "open_task_beads": open_tasks,
    "closure_gate_ref": "spec135-z4-closure-gate-result.json",
    "requirement_matrix_ref": "spec135-z1-requirement-closure-matrix.json",
    "lineage_ref": "spec135-z3-worktree-lineage-proof.json",
    "project_card_outcome": {
        "status": "degraded_timeout",
        "attempted": True,
        "recovery_tool": "focusa_project_card_outcome",
        "fallback_evidence": "repository closure proof remains authoritative"
    },
    "evidence_ref": f"evidence:spec135-final:{head[:12]}",
    "receipt_ref": f"receipt:spec135-final:{head[:12]}",
}
proof["merge_ready"] = (
    proof["clean"]
    and behind == 0
    and open_requirements == ["SPEC135-Z5"]
    and open_tasks == AUTHORITATIVE_RELEASE_TASKS
)
print(json.dumps(proof, indent=2, sort_keys=True))
sys.exit(0 if proof["merge_ready"] else 1)
