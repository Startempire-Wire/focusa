#!/usr/bin/env python3
"""Canonical audit-trail schema for `release-proof/audit/audit.jsonl`.

Every row in `audit.jsonl` MUST conform to one of four canonical shapes:

    FailureRow:
      id (str, required, unique): "fail-YYYY-MM-DD-<short>"
      ts (str, required, ISO 8601 UTC): "2026-06-30T11:30:00Z"
      event (str, required, literal): "failure"
      category (str, required): one of categories in categories.md
      subsystem (str, required): "deploy" | "ci" | "release" | "ops" | "daemon"
      scope (str, required): file path or workflow name
      symptom (str, required): what the operator saw
      root_cause (str, required): why it happened
      fix (str, required): how it was fixed (or "open" if not yet fixed)
      guard (str, required): what prevents regression
      test (str, required): which static test asserts the guard
      linked_run (str, required): comma-separated GitHub run ids or "open"

    AdditionRow:
      id (str, required, unique): "add-YYYY-MM-DD-<short>"
      ts (str, required, ISO 8601 UTC)
      event (str, required, literal): "addition"
      category (str, required): same enum as FailureRow.category
      subsystem (str, required)
      scope (str, required)
      change (str, required): one-sentence description of the addition
      guard (str, optional): what the addition protects
      test (str, optional): which static test asserts it
      linked_run (str, optional): comma-separated run ids

    SelfHealRow (synthesized by scripts/propose-system-fix.py):
      ts (str, required, ISO 8601 UTC)
      event (str, required, literal): "self_heal"
      failure_class (str, required)
      scope (str, required)
      derived_from (str, required): the failure row's id
      fail_count_30d (int, required)
      deliverable (object|null, required)
      before (object, required)
      after (object, required)
      closed (bool, required)
      linked_run (str, required)
      escalation_count (int, required)
      operator_reviewed (bool, required)

    InterventionRateRow:
      ts (str, required, ISO 8601 UTC)
      event (str, required, literal): "intervention_rate"
      total_CI_runs (int, required)
      manual_interventions_required (int, required)
      operator_intervention_rate_pct (number, required)

Run from repo root:

    python3 scripts/audit-schema.py validate release-proof/audit/audit.jsonl
    python3 scripts/audit-schema.py migrate release-proof/audit/audit.jsonl
    python3 scripts/audit-schema.py stats release-proof/audit/audit.jsonl

The script is idempotent: `migrate` is safe to run repeatedly and never
demotes an already-canonical row.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterable

VALID_CATEGORIES = {
    "brittle_regex_match",
    "stale_version_surface",
    "missing_ci_gate_passing",
    "infrastructure_blocked",
    "disk_pressure",
    "permission_denied",
    "hostname_assumption",
    "policy_violation",
    "stale_execut",
    "hang_or_oom",
    "health_timeout",
    "intermittent_health",
    "artifact_already_exists",
    "strict_mode_kill",
    "self_test",
    "ci_workflow_failure",
    "self_heal",
    "intervention_rate",
}

VALID_SUBSYSTEMS = {
    "deploy",
    "ci",
    "release",
    "ops",
    "daemon",
    "audit",
    "runner",
    "workflow",
    "git",
}

REQUIRED_FAILURE = {
    "id",
    "ts",
    "event",
    "category",
    "subsystem",
    "scope",
    "symptom",
    "root_cause",
    "fix",
    "guard",
    "test",
    "linked_run",
}

REQUIRED_ADDITION = {
    "id",
    "ts",
    "event",
    "category",
    "subsystem",
    "scope",
    "change",
}

REQUIRED_SELF_HEAL = {
    "ts",
    "event",
    "derived_from",
    "category",
    "subsystem",
    "scope",
    "symptom",
    "root_cause",
    "fix",
    "guard",
    "test",
    "linked_run",
    "failure_class",
    "fail_count_30d",
    "deliverable",
    "before",
    "after",
    "closed",
    "escalation_count",
    "operator_reviewed",
}

REQUIRED_INTERVENTION_RATE = {
    "ts",
    "event",
    "total_CI_runs",
    "manual_interventions_required",
    "operator_intervention_rate_pct",
}

REQUIRED_LEGACY_SELF_HEAL = {
    "ts",
    "event",
    "derived_from",
    "category",
    "subsystem",
    "scope",
    "symptom",
    "root_cause",
    "fix",
    "guard",
    "test",
    "linked_run",
    "auto_generated",
}


def iter_rows(path: str) -> Iterable[dict[str, Any]]:
    with open(path, "r", encoding="utf-8") as fh:
        for ln, raw in enumerate(fh, 1):
            raw = raw.strip()
            if not raw:
                continue
            try:
                yield json.loads(raw), ln
            except json.JSONDecodeError as exc:
                raise SystemExit(f"{path}:{ln}: invalid JSON: {exc}") from exc


def _row_kind(row: dict[str, Any]) -> str | None:
    event = row.get("event")
    if event == "failure":
        return "failure"
    if event == "addition":
        return "addition"
    if event == "self_heal":
        return "self_heal"
    if event == "intervention_rate":
        return "intervention_rate"
    return None


def _validate_required(
    row: dict[str, Any], required: set[str], path: str, ln: int
) -> list[str]:
    missing = sorted(required - set(row.keys()))
    if missing:
        return [f"{path}:{ln}: missing fields {missing}"]
    return []


def validate(path: str) -> int:
    errors: list[str] = []
    seen_ids: set[str] = set()
    counts: Counter[str] = Counter()
    for row, ln in iter_rows(path):
        kind = _row_kind(row)
        if kind is None:
            errors.append(f"{path}:{ln}: missing or invalid `event` field")
            continue
        counts[kind] += 1
        if kind == "failure":
            errors.extend(_validate_required(row, REQUIRED_FAILURE, path, ln))
            if row.get("category") not in VALID_CATEGORIES:
                errors.append(
                    f"{path}:{ln}: category '{row.get('category')}' not in canonical set"
                )
        elif kind == "addition":
            errors.extend(_validate_required(row, REQUIRED_ADDITION, path, ln))
        elif kind == "self_heal":
            if "fail_count_30d" not in row and row.get("auto_generated") is True:
                errors.extend(
                    _validate_required(row, REQUIRED_LEGACY_SELF_HEAL, path, ln)
                )
            else:
                errors.extend(_validate_required(row, REQUIRED_SELF_HEAL, path, ln))
                deliverable = row.get("deliverable")
                if deliverable is not None:
                    if not isinstance(deliverable, dict):
                        errors.append(
                            f"{path}:{ln}: self_heal deliverable must be object or null"
                        )
                    else:
                        for key in ("type", "ref", "change_summary"):
                            if not deliverable.get(key):
                                errors.append(
                                    f"{path}:{ln}: self_heal deliverable missing {key}"
                                )
        elif kind == "intervention_rate":
            errors.extend(_validate_required(row, REQUIRED_INTERVENTION_RATE, path, ln))
        if row.get("subsystem") and row["subsystem"] not in VALID_SUBSYSTEMS:
            errors.append(
                f"{path}:{ln}: subsystem '{row['subsystem']}' not in canonical set"
            )
        rid = row.get("id")
        if rid is not None:
            if rid in seen_ids:
                errors.append(f"{path}:{ln}: duplicate id '{rid}'")
            seen_ids.add(rid)
    if errors:
        for e in errors:
            print(f"  {e}", file=sys.stderr)
        return 1
    print(f"audit schema OK ({sum(counts.values())} rows: {dict(counts)})")
    return 0


def migrate(path: str) -> int:
    """Migrate legacy rows to canonical schema in place. Idempotent."""
    rows: list[dict[str, Any]] = []
    with open(path, "r", encoding="utf-8") as fh:
        for raw in fh:
            raw = raw.strip()
            if not raw:
                continue
            rows.append(json.loads(raw))

    seen_ids: set[str] = set()
    deduplicated: list[dict[str, Any]] = []
    for row in rows:
        rid = row.get("id")
        if rid is not None and rid in seen_ids:
            # Drop exact duplicates; if we want to preserve both we can
            # rename later. For now dedupe is the safer behavior.
            continue
        if rid is not None:
            seen_ids.add(rid)
        deduplicated.append(row)
    rows = deduplicated

    migrated = 0
    for row in rows:
        kind = _row_kind(row)
        if kind is None:
            # New-shape rows use `summary`+`date` instead of `event`.
            # Promote them by setting event from id prefix.
            if "id" in row:
                rid = row["id"]
                if rid.startswith("fail-"):
                    row["event"] = "failure"
                    kind = "failure"
                elif rid.startswith("add-"):
                    row["event"] = "addition"
                    kind = "addition"
        if kind == "failure":
            if "id" not in row:
                row["id"] = f"fail-{row.get('date', '1970-01-01')}-unanchored"
            if "ts" not in row:
                row["ts"] = _date_to_ts(row.get("date", ""))
            change_text = row.get("change", row.get("summary", ""))
            row.setdefault("category", _infer_category(row, change_text))
            row.setdefault("subsystem", _infer_subsystem(row))
            row.setdefault("scope", row.get("scope", "unknown"))
            row.setdefault("symptom", row.get("summary", change_text or "see change"))
            row.setdefault("root_cause", row.get("root_cause", "see change"))
            row.setdefault("fix", row.get("fix", change_text or "open"))
            row.setdefault("guard", row.get("guard", "open"))
            row.setdefault("test", row.get("test", "open"))
            row.setdefault("linked_run", row.get("linked_run", "open"))
            migrated += 1
        elif kind == "addition":
            if "id" not in row:
                row["id"] = f"add-{row.get('date', '1970-01-01')}-unanchored"
            if "ts" not in row:
                row["ts"] = _date_to_ts(row.get("date", ""))
            change_text = row.get("change", row.get("summary", ""))
            row.setdefault("category", _infer_category(row, change_text))
            row.setdefault("subsystem", _infer_subsystem(row))
            row.setdefault("scope", row.get("scope", "unknown"))
            row.setdefault("change", row.get("change", change_text or "open"))
            migrated += 1
        else:
            continue
    with open(path, "w", encoding="utf-8") as fh:
        for row in rows:
            fh.write(json.dumps(row, separators=(",", ":")) + "\n")
    print(f"migrated {migrated} legacy rows to canonical schema")
    return 0


def _date_to_ts(date_str: str) -> str:
    """Convert `YYYY-MM-DD` to `YYYY-MM-DDT00:00:00Z`."""
    if not date_str:
        return "1970-01-01T00:00:00Z"
    return f"{date_str}T00:00:00Z"


def _infer_category(row: dict[str, Any], change_text: str) -> str:
    """Heuristically map a legacy row's `change` field onto a canonical category."""
    blob = (change_text + " " + str(row.get("scope", ""))).lower()
    if any(k in blob for k in ("rg -q", "grep -fq", "regex")):
        return "brittle_regex_match"
    if "execstart" in blob or "systemd" in blob:
        return "stale_execut"
    if "permission_denied" in blob or "permission" in blob and "denied" in blob:
        return "permission_denied"
    if "disk" in blob or "cleanup" in blob:
        return "disk_pressure"
    if "self-heal" in blob or "auto-heal" in blob or "self_heal" in blob:
        return "self_heal"
    if "kill" in blob and ("strict" in blob or "set -e" in blob):
        return "strict_mode_kill"
    if "ci" in blob and ("gate" in blob or "blocked" in blob):
        return "missing_ci_gate_passing"
    if "audit" in blob:
        return "policy_violation"
    return "policy_violation"


def _infer_subsystem(row: dict[str, Any]) -> str:
    sub = row.get("subsystem")
    if sub in VALID_SUBSYSTEMS:
        return sub
    scope = (row.get("scope") or "").lower()
    if "deploy" in scope or "install-daemon" in scope or "release" in scope:
        return "deploy"
    if "ci" in scope or ".github/workflows/ci" in scope:
        return "ci"
    if "runner" in scope:
        return "runner"
    if "audit" in scope or "audit.jsonl" in scope:
        return "audit"
    if ".github" in scope:
        return "workflow"
    if "git" in scope:
        return "git"
    return "ops"


def stats(path: str) -> int:
    counts: Counter[str] = Counter()
    category_counts: Counter[str] = Counter()
    for row, _ in iter_rows(path):
        kind = _row_kind(row)
        if kind:
            counts[kind] += 1
            if "category" in row:
                category_counts[row["category"]] += 1
    print(f"total: {sum(counts.values())}")
    for kind, n in counts.most_common():
        print(f"  {kind}: {n}")
    print("by category:")
    for cat, n in category_counts.most_common():
        print(f"  {cat}: {n}")
    return 0


def spec104_audit(path: str) -> int:
    """Run Spec 104 static surface audit and record results as audit addition rows."""
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
    try:
        from tests.spec104_deep_focusa_surface_sweep import static_audit, warnings

        static_audit()
    except Exception as e:
        print(f"Spec104 static audit execution failed: {e}")
        return 1

    new_globals = sum(1 for w in warnings if w["kind"].startswith("new_singleton"))
    missing_routes = sum(1 for w in warnings if w["kind"] == "uncatalogued_route")
    missing_cmds = sum(1 for w in warnings if w["kind"] == "uncatalogued_cli_command")

    addition_id = (
        f"add-spec104-static-{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S')}"
    )
    addition_row = {
        "id": addition_id,
        "ts": f"{datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')}",
        "event": "addition",
        "category": "missing_ci_gate_passing",
        "subsystem": "audit",
        "scope": "docs/104-typed-scoped-runtime-and-singleton-elimination-spec.md",
        "change": f"Spec104 static audit: new_globals={new_globals} missing_routes={missing_routes} missing_cmds={missing_cmds}",
        "guard": "tests/spec104_deep_focusa_surface_sweep.py --static-only fails on new singletons",
        "test": "tests/spec104_deep_focusa_surface_sweep.py --static-only",
        "linked_run": os.environ.get("GITHUB_RUN_ID", "local"),
    }

    audit_dir = Path(path).parent
    audit_dir.mkdir(parents=True, exist_ok=True)
    existing = []
    if Path(path).exists():
        with open(path) as f:
            for line in f:
                line = line.strip()
                if line:
                    existing.append(json.loads(line))
    existing.append(addition_row)
    with open(path, "w") as f:
        for row in existing:
            f.write(json.dumps(row, sort_keys=True) + "\n")

    print(f"Spec104 audit recorded: {addition_id}")
    print(
        f"  new_globals={new_globals} missing_routes={missing_routes} missing_cmds={missing_cmds}"
    )
    if new_globals > 0:
        print(f"WARNING: {new_globals} new authority-bearing singleton(s) detected!")
        return 1
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = parser.add_subparsers(dest="cmd", required=True)
    p_val = sub.add_parser("validate", help="validate rows against canonical schema")
    p_val.add_argument("path")
    p_mig = sub.add_parser("migrate", help="migrate legacy rows in place")
    p_mig.add_argument("path")
    p_st = sub.add_parser("stats", help="print row counts")
    p_st.add_argument("path")
    p_s104 = sub.add_parser(
        "spec104", help="run Spec104 static audit and record in audit.jsonl"
    )
    p_s104.add_argument("path", nargs="?", default="release-proof/audit/audit.jsonl")
    args = parser.parse_args(argv)
    if args.cmd == "validate":
        return validate(args.path)
    if args.cmd == "migrate":
        return migrate(args.path)
    if args.cmd == "stats":
        return stats(args.path)
    if args.cmd == "spec104":
        return spec104_audit(args.path)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
