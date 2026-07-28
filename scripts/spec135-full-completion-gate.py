#!/usr/bin/env python3
"""Fail closed until every Spec 135-series runtime requirement has evidence."""
import json
import sys
from pathlib import Path

R = Path(__file__).resolve().parents[1]
audit = json.loads((R / "docs/contracts/spec135-runtime-conformance-audit.v2.yaml").read_text())
requirements = audit["requirements"]
workpath = json.loads((R / audit["autonomous_workpath_ref"]).read_text())
issues = {row["id"]: row for row in map(json.loads, (R / ".beads/issues.jsonl").open())}
incomplete = [
    {
        "spec": row["spec"],
        "status": row["status"],
        "bead": row.get("bead", "focusa-mc-full.1"),
        "missing_count": len(row["missing"]),
    }
    for row in requirements
    if row["status"] != "verified_complete"
]
active = [row["bead"] for row in workpath["tasks"] if issues[row["bead"]]["status"] == "in_progress"]
ready = [
    row["bead"]
    for row in workpath["tasks"]
    if issues[row["bead"]]["status"] == "open"
    and all(issues[dep]["status"] == "closed" for dep in row["depends_on"])
]
result = {
    "schema": "focusa.spec135.full_completion_gate.v1",
    "status": "blocked" if incomplete else "passed",
    "central_surface": audit["authority"]["central_surface"],
    "deferral_allowed": audit["authority"]["deferral_allowed"],
    "total_specs": len(requirements),
    "completed_specs": len(requirements) - len(incomplete),
    "incomplete_specs": incomplete,
    "next_bead": (active or ready or [None])[0],
    "detailed_tasks": len(workpath["tasks"]),
    "evidence_ref": "docs/contracts/spec135-runtime-conformance-audit.v2.yaml",
}
print(json.dumps(result, indent=2, sort_keys=True))
sys.exit(1 if incomplete else 0)
