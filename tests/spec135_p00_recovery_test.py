#!/usr/bin/env python3
"""Validate Spec 135 P00 scope/task-provider recovery without claiming the blocked Workpoint."""
from __future__ import annotations

import json
import subprocess
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MARKER = json.loads((ROOT / ".focusa-project.json").read_text())
BASELINE = json.loads((ROOT / "docs/contracts/spec135-p00-recovery-baseline.v1.json").read_text())
GRAPH = json.loads((ROOT / "docs/contracts/spec135-mission-canvas-completion-dag.v2.json").read_text())
JSONL = [json.loads(line) for line in (ROOT / ".beads/issues.jsonl").read_text().splitlines() if line.strip()]

assert MARKER["schema"] == "focusa.project.v1"
assert MARKER["project_id"] == "focusa"
assert "project_root" not in MARKER, "tracked project marker must not embed a host-specific absolute path"

assert GRAPH["status"] == "operator_approved_p00_execution"
assert BASELINE["schema"] == "focusa.spec135.p00_recovery_baseline.v1"
assert BASELINE["status"] == "p00_complete_p01_authority_ready"
assert BASELINE["focusa_authority"]["workpoint_status"] == "canonical_active"
assert BASELINE["focusa_authority"]["workpoint_id"] == "019fb3a9-5b29-7db3-84e4-bbb507cbe411"
assert BASELINE["focusa_authority"]["writer_lease"]["acquired"] is True
assert BASELINE["task_provider_after"]["new_materialized_records"] == 394
assert BASELINE["task_provider_after"]["new_dependency_edges"] == 1014

valid_types = {"task", "bug", "feature", "epic", "chore"}
valid_statuses = {"open", "in_progress", "blocked", "closed"}
assert {record["issue_type"] for record in JSONL} <= valid_types
assert {record["status"] for record in JSONL} <= valid_statuses

legacy_type_labels = Counter(
    label
    for record in JSONL
    for label in record.get("labels") or []
    if label.startswith("legacy-type:")
)
legacy_status_labels = Counter(
    label
    for record in JSONL
    for label in record.get("labels") or []
    if label.startswith("legacy-status:")
)
assert legacy_type_labels["legacy-type:security"] == 3
assert legacy_type_labels["legacy-type:improvement"] == 5
assert legacy_status_labels["legacy-status:deferred"] == 2

by_id = {record["id"]: record for record in JSONL}
new_ids = {issue_id for issue_id in by_id if issue_id == "focusa-mc2" or issue_id.startswith("focusa-mc2.")}
assert len(new_ids) == 394
assert sum(len(by_id[issue_id].get("dependencies") or []) for issue_id in new_ids) == 1016

superseded = [record for record in JSONL if record["id"].startswith("focusa-mc-full")]
assert len(superseded) == 63
assert all(record["status"] == "closed" for record in superseded)
assert all(record.get("superseded_by") == "focusa-mc2" for record in superseded)

for completed in (
    "focusa-mc2.1.001",
    "focusa-mc2.1.002",
    "focusa-mc2.1.003",
    "focusa-mc2.1.004",
    "focusa-mc2.1.005",
    "focusa-mc2.1.006",
    "focusa-mc2.1.008",
    "focusa-mc2.1.009",
    "focusa-mc2.1.010",
    "focusa-mc2.1.011",
    "focusa-mc2.1.012",
    "focusa-mc2.1.013",
    "focusa-mc2.1.014",
    "focusa-mc2.1.015",
    "focusa-mc2.1.016",
    "focusa-mc2.1.017",
):
    assert by_id[completed]["status"] == "closed"
for successor in ("focusa-mc2.2.018", "focusa-mc2.2.019", "focusa-mc2.2.020", "focusa-mc2.2.021"):
    assert successor in by_id
    assert all(
        by_id[dependency["depends_on_id"]]["status"] == "closed"
        for dependency in by_id[successor].get("dependencies") or []
        if dependency["type"] == "blocks"
    )

subprocess.run(
    ["python3", "scripts/materialize-spec135-mission-canvas-completion-beads.py", "--check"],
    cwd=ROOT,
    check=True,
)
print("Spec 135 P00 recovery: PASS (P00 complete; P01 authority wave ready)")
