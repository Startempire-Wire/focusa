#!/usr/bin/env python3
"""Reject Spec 135 completion claims that lack executed runtime evidence."""
import json
import sys
from pathlib import Path

R = Path(__file__).resolve().parents[1]
matrix = json.loads((R / "docs/contracts/spec135-runtime-evidence-matrix.v2.yaml").read_text())
allowed = set(matrix["allowed_completion_evidence"])
forbidden = set(matrix["forbidden_completion_evidence"])
checks = []
for row in matrix["domains"]:
    kind = row["required_kind"]
    refs = row["evidence_refs"]
    valid_kind = kind in allowed and kind not in forbidden
    refs_exist = bool(refs) and all((R / ref).exists() for ref in refs)
    proven = row["status"] == "proven" and valid_kind and refs_exist
    checks.append({
        "domain": row["domain"],
        "owner_bead": row["owner_bead"],
        "status": "passed" if proven else "blocked",
        "required_kind": kind,
        "evidence_refs": refs,
    })
blocked = [row for row in checks if row["status"] != "passed"]
result = {
    "schema": "focusa.spec135.runtime_evidence_gate.v1",
    "status": "blocked" if blocked else "passed",
    "passed_domains": len(checks) - len(blocked),
    "total_domains": len(checks),
    "blocked_domains": blocked,
    "next_bead": blocked[0]["owner_bead"] if blocked else None,
    "evidence_ref": "docs/contracts/spec135-runtime-evidence-matrix.v2.yaml",
}
print(json.dumps(result, indent=2, sort_keys=True))
sys.exit(1 if blocked else 0)
