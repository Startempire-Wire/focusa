#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
audit = json.loads((R / "docs/contracts/spec135-runtime-conformance-audit.v2.yaml").read_text())
assert audit["schema"] == "focusa.spec135.runtime_conformance_audit.v2"
assert audit["authority"]["central_surface"] == "Pi-native Mission Canvas TUI"
assert audit["authority"]["deferral_allowed"] is False
assert audit["authority"]["delivery"] == "pull_request_only"
assert audit["overall_status"] == "incomplete"
assert audit["detailed_task_count"] == 50
assert (R / audit["autonomous_workpath_ref"]).exists()
requirements = audit["requirements"]
assert [row["spec"] for row in requirements] == [
    "135", "135A", "135B", "135C", "135D", "135E", "135F", "135G", "135H", "135I", "135J", "135K"
]
assert all(row["status"] in {"partial", "incomplete", "in_progress", "verified_complete"} for row in requirements)
assert all(row["implemented_evidence"] for row in requirements)
assert all((not row["missing"]) == (row["status"] == "verified_complete") for row in requirements)
assert len(audit["completion_path"]) == 12
assert len(set(audit["completion_path"])) == 12
for ref in {e for row in requirements for e in row["implemented_evidence"]}:
    assert (R / ref).exists(), ref
incomplete = sum(row["status"] != "verified_complete" for row in requirements)
print(f"Spec 135 runtime conformance audit: PASS ({incomplete}/12 specs remain explicitly incomplete)")
