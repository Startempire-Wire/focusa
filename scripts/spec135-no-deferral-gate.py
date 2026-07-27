#!/usr/bin/env python3
"""Verify every Spec 135 gap is owned by the dependency DAG and none is deferred."""
import json
import sys
from pathlib import Path

R = Path(__file__).resolve().parents[1]
audit = json.loads((R / "docs/contracts/spec135-runtime-conformance-audit.v2.yaml").read_text())
plan = json.loads((R / "docs/contracts/spec135-autonomous-workpath.v1.yaml").read_text())
issues = {row["id"]: row for row in map(json.loads, (R / ".beads/issues.jsonl").open())}
errors = []
if len(plan["tasks"]) != 50:
    errors.append("workpath_must_have_50_tasks")
for row in plan["tasks"]:
    issue = issues.get(row["bead"])
    if not issue:
        errors.append(f"missing:{row['bead']}")
        continue
    if "no-deferral" not in issue.get("labels", []):
        errors.append(f"no_deferral_label_missing:{row['bead']}")
    if issue.get("status") not in {"open", "in_progress", "closed"}:
        errors.append(f"unsupported_status:{row['bead']}:{issue.get('status')}")
owned_specs = {row["spec"] for row in audit["requirements"] if row.get("bead") or row["spec"] == "135"}
if owned_specs != {"135", "135A", "135B", "135C", "135D", "135E", "135F", "135G", "135H", "135I", "135J", "135K"}:
    errors.append("not_all_specs_owned")
result = {
    "schema": "focusa.spec135.no_deferral_gate.v1",
    "status": "blocked" if errors else "passed",
    "task_count": len(plan["tasks"]),
    "owned_spec_count": len(owned_specs),
    "errors": errors,
    "current_bead": plan["current"],
    "evidence_ref": "docs/contracts/spec135-autonomous-workpath.v1.yaml",
}
print(json.dumps(result, indent=2, sort_keys=True))
sys.exit(1 if errors else 0)
