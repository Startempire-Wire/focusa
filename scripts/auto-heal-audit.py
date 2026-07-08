#!/usr/bin/env python3
"""Focusa auto-heal audit hook (proactive, in-place behavior change).

This script synthesizes self-heal rows only when a failure class crosses the
retry threshold and the class has a mapped system-level deliverable.
Passive mirroring rows are suppressed.
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any

DEFAULT_AUDIT_FILE = "release-proof/audit/audit.jsonl"
DEFAULT_WINDOW_DAYS = 7
DEFAULT_THRESHOLD = 3

KNOWN_DELIVERABLES: dict[str, dict[str, str]] = {
    "ci_clippy_failure": {
        "type": "lint",
        "ref": "clippy.toml",
        "change_summary": "Create/add clippy lint deny rule for the recurring warning class."
    },
    "unknown_process_failure": {
        "type": "ci_gate",
        "ref": ".github/workflows/audit-recorder.yml",
        "change_summary": "Add a pre-failing process health gate for the unknown-process class."
    },
    "rust_compile_failure": {
        "type": "type",
        "ref": "crates/focusa-core/src/types/mod.rs",
        "change_summary": "Add a compile-time safe type/validator so this class fails earlier."
    },
    "transient_github_or_network_failure": {
        "type": "retry",
        "ref": "scripts/retry.sh",
        "change_summary": "Add a shared retry helper and wire CI/deploy jobs to it."
    },
    "ci_test_failure": {
        "type": "test",
        "ref": "tests/self-heal-regression-test.rs",
        "change_summary": "Add a regression test that reproduces and catches this failure class."
    },
    "deploy_health_failure": {
        "type": "ci_gate",
        "ref": ".github/workflows/deploy.yml",
        "change_summary": "Add a deploy preflight health gate with a circuit breaker."
    },
}


def parse_ts(value: str | None) -> datetime | None:
    if not value:
        return None
    value = value.strip()
    if not value:
        return None
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        pass
    try:
        return datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
    except Exception:
        return None


def load_entries(path: Path) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                out.append(json.loads(line))
            except json.JSONDecodeError:
                print(f"[focusa-audit] skipping malformed line: {line[:80]!r}", file=sys.stderr)
    return out


def failure_class(row: dict[str, Any]) -> str:
    return row.get("failure_class") or row.get("category") or "unknown"


def is_in_window(ts_value: str | None, cutoff: datetime) -> bool:
    dt = parse_ts(ts_value)
    return dt is not None and dt >= cutoff


def build_class_groups(rows: list[dict[str, Any]]) -> dict[str, list[dict[str, Any]]]:
    buckets: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for entry in rows:
        if entry.get("event") != "failure":
            continue
        cls = failure_class(entry)
        buckets[cls].append(entry)
    for bucket in buckets.values():
        bucket.sort(key=lambda item: parse_ts(item.get("ts", "")) or datetime.min.replace(tzinfo=timezone.utc))
    return buckets


def read_existing_heals(rows: list[dict[str, Any]]) -> tuple[set[str], dict[str, list[dict[str, Any]]]]:
    healed_ids = {row.get("derived_from") for row in rows if row.get("event") == "self_heal" and row.get("derived_from")}
    by_class: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for row in rows:
        if row.get("event") != "self_heal":
            continue
        cls = row.get("failure_class") or row.get("category") or "unknown"
        by_class[cls].append(row)
    for bucket in by_class.values():
        bucket.sort(key=lambda item: parse_ts(item.get("ts", "")) or datetime.min.replace(tzinfo=timezone.utc))
    return healed_ids, by_class


def deliverable_for_class(fail_class: str) -> dict[str, str] | None:
    return KNOWN_DELIVERABLES.get(fail_class)


def synthesize_row(
    failure: dict[str, Any],
    failures_count: int,
    escalations: int,
    deliverable: dict[str, str],
) -> dict[str, Any]:
    scope = failure.get("scope", "")
    symptom = failure.get("symptom", "")
    failure_sym = symptom[:180]
    fail_class = failure_class(failure)

    return {
        "ts": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "event": "self_heal",
        "category": failure.get("category", "ops"),
        "subsystem": failure.get("subsystem", "ops"),
        "scope": scope,
        "derived_from": failure.get("id", ""),
        "symptom": failure_sym,
        "root_cause": failure.get("root_cause", ""),
        "fix": (
            f"Immediate fix for this occurrence + system fix for {fail_class}: "
            f"{deliverable['change_summary']}"
        ),
        "guard": (
            f"Add {deliverable['type']} guard at {deliverable['ref']} so this class cannot recur "
            "without operator-visible prevention."
        ),
        "test": f"Re-run class reproduction guard for {fail_class}",
        "linked_run": failure.get("linked_run", "open"),
        "auto_generated": True,
        "failure_class": fail_class,
        "fail_count_7d": failures_count,
        "deliverable": {
            "type": deliverable["type"],
            "ref": deliverable["ref"],
            "change_summary": deliverable["change_summary"],
        },
        "escalation_count": escalations,
        "closed": False,
        "operator_reviewed": False,
    }


def write_synthesis(audit_path: Path, row: dict[str, Any]) -> None:
    with audit_path.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(row, separators=(",", ":")) + "\n")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("audit_path", nargs="?", default=DEFAULT_AUDIT_FILE)
    parser.add_argument("--window-days", type=int, default=DEFAULT_WINDOW_DAYS)
    parser.add_argument("--threshold", type=int, default=DEFAULT_THRESHOLD)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    repo_root = Path(__file__).resolve().parents[1]
    audit_path = Path(args.audit_path)
    if not audit_path.is_absolute():
        audit_path = repo_root / audit_path
    if not audit_path.exists():
        print(f"audit file missing: {audit_path}", file=sys.stderr)
        return 1

    entries = load_entries(audit_path)
    failures_by_class = build_class_groups(entries)
    healed_ids, heals_by_class = read_existing_heals(entries)
    cutoff = datetime.now(timezone.utc) - timedelta(days=args.window_days)

    written = 0
    suppressed = 0
    for fail_class, failures in sorted(failures_by_class.items()):
        recent = [f for f in failures if is_in_window(f.get("ts"), cutoff)]
        if len(recent) < args.threshold:
            continue

        latest_failure = recent[-1]
        latest_failure_id = latest_failure.get("id")
        if not latest_failure_id or latest_failure_id in healed_ids:
            suppressed += 1
            continue

        previous_heals = heals_by_class.get(fail_class, [])
        if previous_heals:
            last_heal_ts = parse_ts(previous_heals[-1].get("ts"))
            latest_failure_ts = parse_ts(latest_failure.get("ts"))
            if last_heal_ts and latest_failure_ts and latest_failure_ts <= last_heal_ts:
                suppressed += 1
                continue

        deliverable = deliverable_for_class(fail_class)
        if not deliverable:
            print(f"[focusa-audit] escalate-only class {fail_class} reached threshold ({len(recent)}x), no auto-deliverable mapping")
            suppressed += 1
            continue

        row = synthesize_row(latest_failure, len(recent), len(previous_heals), deliverable)
        write_synthesis(audit_path, row)
        healed_ids.add(latest_failure_id)
        writeable_id = latest_failure.get("id")
        if writeable_id:
            healed_ids.add(writeable_id)
        print(
            "[focusa-audit] self_heal synthesized "
            f"for {latest_failure_id} ({fail_class}) with deliverable={deliverable['type']}"
        )
        written += 1

    if written == 0:
        print(f"[focusa-audit] no proactive self_heal rows written; suppressed {suppressed} eligible cases")
    else:
        print(f"[focusa-audit] synthesized {written} proactive self_heal row(s)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
