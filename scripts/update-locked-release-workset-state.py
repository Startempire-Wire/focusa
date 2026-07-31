#!/usr/bin/env python3
"""Refresh locked-release Workset projections and ready frontier from local Beads truth."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUDIT = ROOT / "release-proof/audit"
MEMBERS_PATH = AUDIT / "next-locked-release-workset-members.jsonl"
EDGES_PATH = AUDIT / "next-locked-release-workset-edges.jsonl"
DEFINITION_PATH = AUDIT / "next-locked-release-workset-definition.json"
PROOF_PATH = AUDIT / "next-locked-release-execution-proof.json"
ISSUES_PATH = ROOT / ".beads/issues.jsonl"


def jsonl(path: Path) -> list[dict[str, object]]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def digest(value: object) -> str:
    body = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return "sha256:" + hashlib.sha256(body).hexdigest()


def main() -> int:
    members = jsonl(MEMBERS_PATH)
    edges = jsonl(EDGES_PATH)
    issues = {row["id"]: row for row in jsonl(ISSUES_PATH)}
    member_ids = {str(row["member_id"]) for row in members}
    incoming: dict[str, list[str]] = {member_id: [] for member_id in member_ids}
    for edge in edges:
        incoming[str(edge["to_member_ref"])].append(str(edge["from_member_ref"]))
    for member in members:
        status = str(issues[str(member["member_id"])]["status"])
        member["current_status_projection"] = status
        member["current_status_freshness"] = "current"
        member["disposition"] = "completed" if status == "closed" else "pending"
    MEMBERS_PATH.write_text(
        "\n".join(json.dumps(row, separators=(",", ":")) for row in members) + "\n"
    )
    membership_digest = digest(members)
    definition = json.loads(DEFINITION_PATH.read_text())
    definition["membership_digest"] = membership_digest
    DEFINITION_PATH.write_text(json.dumps(definition, indent=2) + "\n")
    closed = lambda issue_id: issues[issue_id]["status"] == "closed"
    ready = sorted(
        member_id
        for member_id in member_ids
        if not closed(member_id) and all(closed(parent) for parent in incoming[member_id])
    )
    phase_by_id = {
        str(row["member_id"]): int(str(row["task_plan_ref"]).rsplit(":", 1)[-1])
        for row in members
    }
    active_phase = min((phase_by_id[member_id] for member_id in ready), default=8)
    proof = json.loads(PROOF_PATH.read_text())
    proof["membership_digest"] = membership_digest
    proof["unique_ready_frontier"] = ready
    proof["active_phase"] = active_phase
    PROOF_PATH.write_text(json.dumps(proof, indent=2) + "\n")
    print(json.dumps({"status": "updated", "ready": ready, "digest": membership_digest}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
