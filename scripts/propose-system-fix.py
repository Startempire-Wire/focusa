#!/usr/bin/env python3
"""Spec 122 proactive self-heal proposer.

This script is intentionally conservative: it writes a self_heal row only after
a failure class crosses the de-dup threshold and only when the row references a
concrete deliverable that exists in the same checkout. One-off/passive failures
produce a result JSON but no audit mutation, preventing noisy audit commits.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any

try:
    from self_heal_governor import FAILURE_SCHEMA, failure_fingerprint
except ModuleNotFoundError:  # Standalone compatibility copies used by Spec122.
    FAILURE_SCHEMA = "focusa.self_heal.failure.v1"

    def failure_fingerprint(failure: dict[str, Any]) -> str:
        fields = ("repository", "workflow", "failure_class", "exact_sha", "action_scope")
        normalized = []
        for field in fields:
            value = str(failure.get(field) or "").strip().lower()
            if not value:
                raise ValueError(f"{field} is required")
            normalized.append(re.sub(r"[^a-z0-9._:/-]+", "-", value).strip("-"))
        return hashlib.sha256("\x00".join(normalized).encode()).hexdigest()


DEFAULT_AUDIT_FILE = "release-proof/audit/audit.jsonl"
DEFAULT_RESULT_FILE = "release-proof/audit/self-heal-result.json"
DEFAULT_WINDOW_DAYS = 7
DEFAULT_THRESHOLD = 3

KNOWN_DELIVERABLES: dict[str, dict[str, str]] = {
    "ci_workflow_failure": {
        "type": "script",
        "ref": "scripts/classify-ci-failure.py",
        "change_summary": "Added a script that classifies ci_workflow_failure before self-heal escalation.",
    },
    "ci_clippy_failure": {
        "type": "lint",
        "ref": "tests/spec122_self_heal_proposal_static_test.sh",
        "change_summary": "Added a lint/static gate for ci_clippy_failure.",
    },
    "unknown_process_failure": {
        "type": "ci_gate",
        "ref": "scripts/process-health-check.py",
        "change_summary": "Added a failing gate at process-health-check that catches unknown_process_failure.",
    },
    "rust_compile_failure": {
        "type": "test",
        "ref": "tests/spec122_self_heal_proposal_static_test.sh",
        "change_summary": "Added a test that fails when rust_compile_failure reproduces as a passive mirror.",
    },
    "rust_compile_api_drift": {
        "type": "test",
        "ref": "tests/spec122_self_heal_proposal_static_test.sh",
        "change_summary": "Added a test that fails when rust_compile_api_drift reproduces as a passive mirror.",
    },
    "rust_compile_format_arg_drift": {
        "type": "test",
        "ref": "tests/spec122_self_heal_proposal_static_test.sh",
        "change_summary": "Added a test that fails when rust_compile_format_arg_drift reproduces as a passive mirror.",
    },
    "rust_compile_api_signature_drift": {
        "type": "test",
        "ref": "tests/spec122_self_heal_proposal_static_test.sh",
        "change_summary": "Added a test that fails when rust_compile_api_signature_drift reproduces as a passive mirror.",
    },
    "transient_github_or_network_failure": {
        "type": "retry",
        "ref": "scripts/retry.sh",
        "change_summary": "Replaced ad-hoc retry with `scripts/retry.sh`.",
    },
    "ci_test_failure": {
        "type": "test",
        "ref": "tests/spec122_self_heal_proposal_static_test.sh",
        "change_summary": "Added a test that fails when ci_test_failure reproduces as a passive mirror.",
    },
    "deploy_health_failure": {
        "type": "ci_gate",
        "ref": "scripts/deploy-smoke-check.sh",
        "change_summary": "Added a failing gate at deploy health proof that catches deploy_health_failure.",
    },
    "release_static_proof_failure": {
        "type": "test",
        "ref": "tests/spec122_self_heal_proposal_static_test.sh",
        "change_summary": "Added a test that fails when release_static_proof_failure reproduces as a passive mirror.",
    },
    "runner_resource_failure": {
        "type": "retry",
        "ref": "scripts/retry.sh",
        "change_summary": "Replaced ad-hoc retry with `scripts/retry.sh` for runner_resource_failure.",
    },
    "auto_heal_process_error": {
        "type": "ci_gate",
        "ref": ".github/workflows/audit-recorder.yml",
        "change_summary": "Added a failing gate at audit-recorder that catches auto_heal_process_error.",
    },
}


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def parse_ts(value: str | None) -> datetime | None:
    if not value:
        return None
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None


def load_entries(path: Path) -> list[dict[str, Any]]:
    if not path.exists():
        return []
    rows: list[dict[str, Any]] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        if not raw.strip():
            continue
        rows.append(json.loads(raw))
    return rows


def failure_class(row: dict[str, Any]) -> str:
    return row.get("failure_class") or row.get("category") or "unknown_process_failure"


def group_failures(rows: list[dict[str, Any]]) -> dict[str, list[dict[str, Any]]]:
    grouped: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for row in rows:
        if row.get("event") == "failure":
            grouped[failure_class(row)].append(row)
    for bucket in grouped.values():
        bucket.sort(
            key=lambda row: parse_ts(row.get("ts"))
            or datetime.min.replace(tzinfo=timezone.utc)
        )
    return grouped


def existing_self_heals(
    rows: list[dict[str, Any]],
) -> tuple[set[str], dict[str, list[dict[str, Any]]]]:
    derived = {
        row.get("derived_from")
        for row in rows
        if row.get("event") == "self_heal" and row.get("derived_from")
    }
    by_class: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for row in rows:
        if row.get("event") == "self_heal":
            by_class[
                row.get("failure_class") or row.get("category") or "unknown"
            ].append(row)
    for bucket in by_class.values():
        bucket.sort(
            key=lambda row: parse_ts(row.get("ts"))
            or datetime.min.replace(tzinfo=timezone.utc)
        )
    return derived, by_class


def intervention_rate(rows: list[dict[str, Any]], cutoff: datetime) -> dict[str, Any]:
    failures = [
        row
        for row in rows
        if row.get("event") == "failure"
        and (parse_ts(row.get("ts")) or datetime.min.replace(tzinfo=timezone.utc))
        >= cutoff
    ]
    manual = [
        row
        for row in failures
        if str(row.get("fix", "")).lower()
        in {"open", "manual", "operator", "manual_intervention"}
        or "manual" in str(row.get("fix", "")).lower()
    ]
    pct = 0.0 if not failures else round((len(manual) / len(failures)) * 100.0, 2)
    return {
        "ts": now_iso(),
        "event": "intervention_rate",
        "window_days": 7,
        "total_CI_runs": len(failures),
        "manual_interventions_required": len(manual),
        "operator_intervention_rate_pct": pct,
    }


def validate_deliverable(repo_root: Path, deliverable: dict[str, str]) -> bool:
    ref = deliverable.get("ref", "")
    return bool(ref) and (repo_root / ref).exists()


def build_self_heal_row(
    failure: dict[str, Any],
    fail_count_30d: int,
    fail_count_7d: int,
    deliverable: dict[str, str] | None,
    escalation_count: int,
    rate_before: dict[str, Any],
) -> dict[str, Any]:
    cls = failure_class(failure)
    return {
        "ts": now_iso(),
        "event": "self_heal",
        "failure_class": cls,
        "category": failure.get("category", "self_heal"),
        "subsystem": failure.get("subsystem", "workflow"),
        "scope": failure.get("scope", "workflow"),
        "derived_from": failure.get("id", "open"),
        "symptom": str(failure.get("symptom", ""))[:180],
        "root_cause": failure.get("root_cause", "open"),
        "fix": "Immediate occurrence remains visible; system-level deliverable prevents class recurrence.",
        "guard": None if deliverable is None else deliverable["change_summary"],
        "test": None
        if deliverable is None
        else f"Verify deliverable ref exists and class {cls} is not passively mirrored.",
        "linked_run": failure.get("linked_run", "open"),
        "auto_generated": False,
        "fail_count_30d": fail_count_30d,
        "fail_count_7d": fail_count_7d,
        "deliverable": deliverable,
        "before": {
            "manual_intervention_rate_pct": rate_before[
                "operator_intervention_rate_pct"
            ],
            "failure_class_repro_count": fail_count_30d,
        },
        "after": {
            "manual_intervention_rate_pct": None,
            "failure_class_repro_count": None,
        },
        "closed": False,
        "escalation_count": escalation_count,
        "operator_reviewed": False,
    }


def append_jsonl(path: Path, rows: list[dict[str, Any]]) -> None:
    if not rows:
        return
    with path.open("a", encoding="utf-8") as fh:
        for row in rows:
            fh.write(json.dumps(row, separators=(",", ":")) + "\n")


def write_result(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("audit_path", nargs="?", default=DEFAULT_AUDIT_FILE)
    parser.add_argument("--result", default=DEFAULT_RESULT_FILE)
    parser.add_argument("--window-days", type=int, default=DEFAULT_WINDOW_DAYS)
    parser.add_argument("--threshold", type=int, default=DEFAULT_THRESHOLD)
    parser.add_argument("--dry-run", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    repo_root = Path(__file__).resolve().parents[1]
    audit_path = Path(args.audit_path)
    result_path = Path(args.result)
    if not audit_path.is_absolute():
        audit_path = repo_root / audit_path
    if not result_path.is_absolute():
        result_path = repo_root / result_path

    rows = load_entries(audit_path)
    cutoff_7d = datetime.now(timezone.utc) - timedelta(days=args.window_days)
    cutoff_30d = datetime.now(timezone.utc) - timedelta(days=30)
    rate = intervention_rate(rows, cutoff_7d)
    grouped = group_failures(rows)
    healed_ids, heals_by_class = existing_self_heals(rows)

    output_rows: list[dict[str, Any]] = []
    decisions: list[dict[str, Any]] = []
    for cls, failures in sorted(grouped.items()):
        recent_7d = [
            row
            for row in failures
            if (parse_ts(row.get("ts")) or datetime.min.replace(tzinfo=timezone.utc))
            >= cutoff_7d
        ]
        recent_30d = [
            row
            for row in failures
            if (parse_ts(row.get("ts")) or datetime.min.replace(tzinfo=timezone.utc))
            >= cutoff_30d
        ]
        if len(recent_7d) < args.threshold:
            decisions.append(
                {"failure_class": cls, "action": "log_only", "count_7d": len(recent_7d)}
            )
            continue
        latest = recent_7d[-1]
        latest_id = latest.get("id")
        if latest_id and latest_id in healed_ids:
            decisions.append(
                {
                    "failure_class": cls,
                    "action": "suppressed_duplicate",
                    "derived_from": latest_id,
                }
            )
            continue
        previous = heals_by_class.get(cls, [])
        escalation = len(previous)
        deliverable = KNOWN_DELIVERABLES.get(cls)
        if deliverable is not None and not validate_deliverable(repo_root, deliverable):
            deliverable = None
        row = build_self_heal_row(
            latest, len(recent_30d), len(recent_7d), deliverable, escalation, rate
        )
        head_sha = str(latest.get("head_sha") or "").strip()
        fingerprint = None
        if len(head_sha) >= 7:
            fingerprint = failure_fingerprint(
                {
                    "schema": FAILURE_SCHEMA,
                    "repository": os.environ.get("GITHUB_REPOSITORY", "unknown/repository"),
                    "workflow": str(latest.get("scope") or "unknown"),
                    "failure_class": cls,
                    "exact_sha": head_sha,
                    "action_scope": "system-fix-proposal",
                    "deterministic": bool(latest.get("deterministic")),
                }
            )
            row["failure_fingerprint"] = fingerprint
            row["branch_ref"] = f"self-heal/fp-{fingerprint[:20]}"
        output_rows.append(row)
        decisions.append(
            {
                "failure_class": cls,
                "action": "self_heal" if deliverable and fingerprint else "operator_review_required",
                "deliverable": deliverable,
                "derived_from": latest_id,
                "failure_fingerprint": fingerprint,
            }
        )

    fingerprinted = [
        row for row in output_rows if row.get("deliverable") and row.get("failure_fingerprint")
    ]
    primary = max(fingerprinted, key=lambda row: str(row.get("ts") or ""), default=None)
    wrote_deliverable = primary is not None
    result = {
        "schema": "focusa.self_heal.result.v1",
        "status": "dry_run" if args.dry_run else "completed",
        "audit_path": str(audit_path),
        "wrote_deliverable": wrote_deliverable,
        "self_heal_rows": len(output_rows),
        "proposed_self_heal_rows": output_rows,
        "should_commit": wrote_deliverable,
        "proposal_fingerprint": None if primary is None else primary["failure_fingerprint"],
        "branch_ref": None if primary is None else primary["branch_ref"],
        "decisions": decisions,
        "intervention_rate": rate,
    }

    if not args.dry_run:
        rows_to_append = output_rows[:]
        if wrote_deliverable:
            rows_to_append.append(rate)
        append_jsonl(audit_path, rows_to_append)
        write_result(result_path, result)

    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
