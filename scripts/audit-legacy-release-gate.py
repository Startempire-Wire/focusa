#!/usr/bin/env python3
"""Emit deterministic legacy-ref, provider, and recovery settlement evidence."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/evidence/release/141-legacy-gate-closure.json"
RESTART_PROOFS = [
    "tests/spec133_phase4_runtime_gate.sh",
    "tests/spec133_phase5_isolation_gate.sh",
    "tests/spec96_silent_session_process_failure_static_test.sh",
    "tests/spec130a_proactive_compaction_runtime_test.sh",
    "tests/spec130a_release_stress_runtime_test.mts",
]
SETTLEMENT_PREREQUISITES = [
    "focusa-vbcqu.7.1",
    "focusa-vbcqu.7.2",
    "focusa-vbcqu.7.3",
    "focusa-vbcqu.7.4",
]


def load(path: Path) -> object:
    return json.loads(path.read_text())


def jsonl(path: Path) -> list[dict[str, object]]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def git(*args: str) -> str:
    return subprocess.run(
        ["git", *args], cwd=ROOT, check=True, text=True, capture_output=True
    ).stdout.strip()


def stable_digest(value: object) -> str:
    body = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return "sha256:" + hashlib.sha256(body).hexdigest()


def build() -> dict[str, object]:
    definition = load(ROOT / "release-proof/audit/next-locked-release-workset-definition.json")
    execution = load(ROOT / "release-proof/audit/next-locked-release-execution-proof.json")
    ref_audit = load(ROOT / "docs/evidence/release/141-archived-worktree-ref-semantic-diff.json")
    members = jsonl(ROOT / "release-proof/audit/next-locked-release-workset-members.jsonl")
    issues = {row["id"]: row for row in jsonl(ROOT / ".beads/issues.jsonl")}
    missing_members = sorted(
        str(row["member_id"]) for row in members if row["member_id"] not in issues
    )
    duplicate_members = len(members) - len({row["member_id"] for row in members})
    prerequisites = {
        issue_id: str(issues[issue_id]["status"])
        for issue_id in SETTLEMENT_PREREQUISITES
    }
    unmet_prerequisites = sorted(
        issue_id for issue_id, status in prerequisites.items() if status != "closed"
    )
    missing_restart_proofs = sorted(
        ref for ref in RESTART_PROOFS if not (ROOT / ref).is_file()
    )
    beads_owner_uid = (ROOT / ".beads").stat().st_uid
    issues_owner_uid = (ROOT / ".beads/issues.jsonl").stat().st_uid
    unresolved_external = int(execution.get("unresolved_external_blockers", -1))
    errors: list[str] = []
    if ref_audit.get("status") != "verified" or ref_audit.get("unsettled_ref_count") != 0:
        errors.append("legacy_ref_audit_unsettled")
    if missing_members:
        errors.append("workset_member_missing_from_provider")
    if duplicate_members:
        errors.append("duplicate_workset_member")
    if unmet_prerequisites:
        errors.append("legacy_settlement_prerequisite_open")
    if missing_restart_proofs:
        errors.append("restart_recovery_proof_missing")
    if unresolved_external != 0:
        errors.append("unresolved_external_blocker")
    if beads_owner_uid != issues_owner_uid:
        errors.append("beads_provider_owner_mismatch")
    payload: dict[str, object] = {
        "schema": "focusa.legacy_gate_closure.v1",
        "status": "verified" if not errors else "blocked",
        "locked_release_head": ref_audit["locked_release_head"],
        "locked_release_branch": git("branch", "--show-current"),
        "workset": {
            "workset_id": definition["workset_id"],
            "member_count": len(members),
            "unique_member_count": len({row["member_id"] for row in members}),
            "missing_provider_members": missing_members,
            "duplicate_member_count": duplicate_members,
            "unresolved_external_blockers": unresolved_external,
        },
        "legacy_refs": {
            "evidence_ref": "docs/evidence/release/141-archived-worktree-ref-semantic-diff.json",
            "status": ref_audit["status"],
            "unsettled_ref_count": ref_audit["unsettled_ref_count"],
            "evidence_digest": ref_audit["evidence_digest"],
        },
        "settlement_prerequisites": prerequisites,
        "issue_provider": {
            "beads_owner_uid": beads_owner_uid,
            "issues_owner_uid": issues_owner_uid,
            "owner_consistent": beads_owner_uid == issues_owner_uid,
        },
        "restart_recovery_proofs": RESTART_PROOFS,
        "missing_restart_recovery_proofs": missing_restart_proofs,
        "zero_open_scope": {
            "unmapped_issue_count": len(missing_members),
            "duplicate_issue_count": duplicate_members,
            "unsettled_legacy_ref_count": ref_audit["unsettled_ref_count"],
            "unresolved_external_blocker_count": unresolved_external,
        },
        "errors": errors,
    }
    payload["evidence_digest"] = stable_digest(payload)
    return payload


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    payload = build()
    body = json.dumps(payload, indent=2) + "\n"
    if args.check:
        if not OUT.exists() or OUT.read_text() != body:
            raise SystemExit(f"stale legacy gate evidence: {OUT}")
    else:
        OUT.parent.mkdir(parents=True, exist_ok=True)
        OUT.write_text(body)
    print(json.dumps({"status": payload["status"], "digest": payload["evidence_digest"]}))
    return 0 if payload["status"] == "verified" else 1


if __name__ == "__main__":
    raise SystemExit(main())
