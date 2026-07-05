#!/usr/bin/env python3
"""Summarize recent release audit failures by classifier output.

Read-only operator triage helper for `release-proof/audit/audit.jsonl`.
Shows failure_class, retry_policy, source_refs, remediation, run id, and URL
without opening raw JSONL.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Iterable

DEFAULT_AUDIT = "release-proof/audit/audit.jsonl"


def load_rows(path: Path) -> list[dict]:
    rows: list[dict] = []
    if not path.exists():
        return rows
    with path.open("r", encoding="utf-8") as fh:
        for raw in fh:
            raw = raw.strip()
            if not raw:
                continue
            rows.append(json.loads(raw))
    return rows


def recent_failures(rows: Iterable[dict], failure_class: str | None, limit: int) -> list[dict]:
    failures = [row for row in rows if row.get("event") == "failure"]
    if failure_class:
        failures = [row for row in failures if row.get("failure_class") == failure_class]
    failures.sort(key=lambda row: row.get("ts", ""), reverse=True)
    return failures[:limit]


def normalize_source_refs(value: object) -> list[str]:
    if value is None:
        return []
    if isinstance(value, str):
        return [part.strip() for part in value.split(",") if part.strip()]
    if isinstance(value, list):
        return [str(part) for part in value if str(part).strip()]
    return [str(value)]


def summarize(rows: list[dict]) -> dict:
    counts: dict[str, int] = {}
    retry_counts: dict[str, int] = {}
    for row in rows:
        cls = row.get("failure_class") or "unclassified"
        counts[cls] = counts.get(cls, 0) + 1
        retry = row.get("retry_policy") or "unknown"
        retry_counts[retry] = retry_counts.get(retry, 0) + 1
    return {"count": len(rows), "failure_classes": counts, "retry_policies": retry_counts, "failures": rows}


def render_human(payload: dict) -> str:
    out: list[str] = []
    out.append(f"audit failures: {payload['count']}")
    if payload["failure_classes"]:
        out.append("failure classes:")
        for cls, count in sorted(payload["failure_classes"].items()):
            out.append(f"  - {cls}: {count}")
    if payload["retry_policies"]:
        out.append("retry policies:")
        for retry, count in sorted(payload["retry_policies"].items()):
            out.append(f"  - {retry}: {count}")
    for row in payload["failures"]:
        rid = row.get("id", "(no-id)")
        ts = row.get("ts", "")
        cls = row.get("failure_class") or "unclassified"
        retry = row.get("retry_policy") or "unknown"
        run = row.get("linked_run") or "open"
        out.append(f"- {rid} {ts} class={cls} retry={retry} run={run}")
        symptom = row.get("symptom") or ""
        if symptom:
            out.append(f"  symptom: {symptom}")
        remediation = row.get("remediation_template") or row.get("fix") or ""
        if remediation:
            out.append(f"  remediation: {remediation}")
        refs = normalize_source_refs(row.get("source_refs"))
        if refs:
            out.append("  source_refs: " + ", ".join(refs[:5]))
        url = row.get("log_url") or ""
        if url:
            out.append(f"  log: {url}")
    return "\n".join(out)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--audit", default=DEFAULT_AUDIT)
    parser.add_argument("--class", dest="failure_class", help="Filter by failure_class")
    parser.add_argument("--limit", type=int, default=10)
    parser.add_argument("--json", action="store_true", help="Emit JSON instead of human text")
    args = parser.parse_args(argv)

    rows = recent_failures(load_rows(Path(args.audit)), args.failure_class, args.limit)
    payload = summarize(rows)
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        print(render_human(payload))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
