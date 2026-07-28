#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
plan = json.loads((R / "docs/contracts/spec135-autonomous-workpath.v1.yaml").read_text())
issues = {row["id"]: row for row in map(json.loads, (R / ".beads/issues.jsonl").open())}
tasks = plan["tasks"]
assert plan["schema"] == "focusa.spec135.autonomous_workpath.v1"
assert plan["no_deferral"] is True
assert len(tasks) == 50
assert [row["ordinal"] for row in tasks] == list(range(1, 51))
assert len({row["bead"] for row in tasks}) == 50
seen = set()
for row in tasks:
    bead = row["bead"]
    assert bead in issues, bead
    issue = issues[bead]
    assert issue["title"] == row["title"]
    assert issue["acceptance_criteria"] == row["acceptance"]
    deps = {dep["depends_on_id"] for dep in issue.get("dependencies", [])}
    assert deps == set(row["depends_on"]), (bead, deps, row["depends_on"])
    assert all(dep in seen for dep in deps if dep.startswith("focusa-mc-full-")), (bead, deps - seen)
    seen.add(bead)
active = [row["bead"] for row in tasks if issues[row["bead"]]["status"] == "in_progress"]
assert active == [plan["current"]], active
print("Spec 135 autonomous workpath: PASS (50 ordered Beads, one active frontier)")
