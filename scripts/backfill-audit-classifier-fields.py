#!/usr/bin/env python3
"""Append classifier-field backfill rows for historical audit failures.

Preserves audit immutability: never rewrites historical `failure` rows.  `--apply`
appends one `addition` row per failure that lacks classifier metadata; readers can
overlay those rows by `derived_from`.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
from pathlib import Path

DEFAULT_AUDIT = "release-proof/audit/audit.jsonl"
SCHEMA = "focusa.release_failure_classifier.v1"
FIELDS = [
    "classifier_schema",
    "failure_class",
    "retry_policy",
    "deterministic",
    "safe_to_rerun_unchanged",
    "source_refs",
    "remediation_template",
    "classifier_signals",
]


def load_rows(path: Path) -> list[dict]:
    if not path.exists():
        return []
    rows: list[dict] = []
    for raw in path.read_text().splitlines():
        raw = raw.strip()
        if raw:
            rows.append(json.loads(raw))
    return rows


def row_text(row: dict) -> str:
    parts = []
    for key in (
        "category",
        "subsystem",
        "scope",
        "symptom",
        "root_cause",
        "fix",
        "guard",
        "test",
        "log_url",
    ):
        value = row.get(key)
        if value:
            parts.append(str(value))
    return "\n".join(parts).lower()


def existing_backfills(rows: list[dict]) -> set[str]:
    return {
        str(row.get("derived_from"))
        for row in rows
        if row.get("event") == "addition"
        and row.get("classifier_schema") == SCHEMA
        and row.get("derived_from")
    }


def has_classifier(row: dict) -> bool:
    return bool(
        row.get("classifier_schema")
        or row.get("failure_class")
        or row.get("retry_policy")
    )


def source_refs(row: dict) -> list[str]:
    value = row.get("source_refs")
    if isinstance(value, list):
        return [str(item) for item in value if str(item).strip()]
    if isinstance(value, str):
        return [part.strip() for part in value.split(",") if part.strip()]
    refs: list[str] = []
    for key in ("scope", "guard", "test"):
        item = row.get(key)
        if isinstance(item, str) and (
            "/" in item or item.endswith((".py", ".rs", ".yml", ".sh"))
        ):
            refs.append(item)
    return list(dict.fromkeys(refs))[:5]


def infer(row: dict) -> dict:
    text = row_text(row)
    cls = "unknown_process_failure"
    retry = "rerun_once"
    deterministic = False
    safe = True
    remediation = "Retry once; if repeated, inspect linked logs and add a specific classifier case."
    signals = ["historical_failure_row"]

    if "clippy" in text:
        cls = "ci_clippy_failure"
        retry = "hard_failure_no_rerun"
        deterministic = True
        safe = False
        remediation = "Patch clippy violations; do not rerun unchanged CI."
        signals.append("clippy")
    elif re.search(r"\b(test|tests)\b", text) and (
        "fail" in text or "panicked" in text
    ):
        cls = "ci_test_failure"
        retry = "hard_failure_no_rerun"
        deterministic = True
        safe = False
        remediation = "Patch failing tests; do not rerun unchanged CI."
        signals.append("test_failure")
    elif "error[" in text or "cargo" in text or "rust" in text or ".rs:" in text:
        cls = "rust_compile_api_drift"
        retry = "hard_failure_no_rerun"
        deterministic = True
        safe = False
        remediation = "Patch Rust API/build drift; do not rerun unchanged CI."
        signals.append("rust_compile")
    elif (
        "deploy_health" in text
        or "/v1/health" in text
        or "health" in text
        and "deploy" in text
    ):
        cls = "deploy_health_failure"
        retry = "rerun_once"
        deterministic = False
        safe = True
        remediation = "Retry deploy once; if repeated, inspect daemon journal and deploy health logs."
        signals.append("deploy_health")
    elif any(
        token in text
        for token in (
            "github",
            "network",
            "timeout",
            "5xx",
            "artifact",
            "upload",
            "rate limit",
        )
    ):
        cls = "transient_github_or_network_failure"
        retry = "rerun_once"
        deterministic = False
        safe = True
        remediation = "Retry once; if repeated, inspect GitHub/runner/network status."
        signals.append("transient_infra")
    elif row.get("category") == "ci_workflow_failure":
        cls = "unknown_process_failure"
        retry = "rerun_once"
        deterministic = False
        safe = True
        remediation = "Historical CI failure lacks logs; rerun once only, then classify from linked run logs."
        signals.append("ci_workflow_failure")

    return {
        "classifier_schema": SCHEMA,
        "failure_class": cls,
        "retry_policy": retry,
        "deterministic": deterministic,
        "safe_to_rerun_unchanged": safe,
        "source_refs": source_refs(row),
        "remediation_template": remediation,
        "classifier_signals": signals,
    }


def backfill_id(failure_id: str) -> str:
    safe = re.sub(r"[^A-Za-z0-9_.:-]+", "-", failure_id)[:120]
    return f"add-backfill-classifier-{safe}"


def make_backfill_row(failure: dict, ts: str) -> dict:
    fields = infer(failure)
    fid = str(failure.get("id", "unknown"))
    return {
        "id": backfill_id(fid),
        "ts": ts,
        "event": "addition",
        "category": "self_heal",
        "subsystem": "audit",
        "scope": "release-proof/audit/audit.jsonl",
        "change": f"Backfilled classifier fields for historical failure {fid}: {fields['failure_class']} ({fields['retry_policy']}).",
        "derived_from": fid,
        "guard": "scripts/backfill-audit-classifier-fields.py",
        "test": "tests/release_deploy_automation_static_test.sh",
        **fields,
    }


def candidate_rows(rows: list[dict]) -> list[dict]:
    done = existing_backfills(rows)
    out = []
    for row in rows:
        fid = row.get("id")
        if row.get("event") != "failure" or not fid:
            continue
        if has_classifier(row) or str(fid) in done:
            continue
        out.append(row)
    return out


def append_rows(path: Path, rows: list[dict]) -> None:
    with path.open("a", encoding="utf-8") as fh:
        for row in rows:
            fh.write(json.dumps(row, separators=(",", ":"), sort_keys=True) + "\n")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--audit", default=DEFAULT_AUDIT)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--dry-run", action="store_true")
    mode.add_argument("--apply", action="store_true")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)

    path = Path(args.audit)
    rows = load_rows(path)
    ts = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    additions = [make_backfill_row(row, ts) for row in candidate_rows(rows)]
    payload = {
        "schema": "focusa.audit_classifier_backfill.v1",
        "audit": str(path),
        "mode": "apply" if args.apply else "dry_run",
        "candidate_count": len(additions),
        "append_count": len(additions) if args.apply else 0,
        "failure_classes": {},
        "rows": additions,
    }
    for row in additions:
        cls = row["failure_class"]
        payload["failure_classes"][cls] = payload["failure_classes"].get(cls, 0) + 1
    if args.apply and additions:
        append_rows(path, additions)
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        print(
            f"audit classifier backfill: {payload['mode']} candidates={len(additions)} appended={payload['append_count']}"
        )
        for cls, count in sorted(payload["failure_classes"].items()):
            print(f"  - {cls}: {count}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
