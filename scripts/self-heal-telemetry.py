#!/usr/bin/env python3
"""Report self-heal health telemetry from release-proof audit JSONL."""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from datetime import datetime, timezone, timedelta
from pathlib import Path

DEFAULT_AUDIT = "release-proof/audit/audit.jsonl"


def parse_ts(value: str) -> datetime | None:
    if not value:
        return None
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None


def load_rows(path: Path) -> list[dict]:
    if not path.exists():
        return []
    rows: list[dict] = []
    for raw in path.read_text().splitlines():
        raw = raw.strip()
        if raw:
            rows.append(json.loads(raw))
    return rows


def build_payload(rows: list[dict], stale_hours: float) -> dict:
    failures = [row for row in rows if row.get("event") == "failure"]
    heals = [row for row in rows if row.get("event") == "self_heal"]
    healed_ids = {row.get("derived_from") for row in heals if row.get("derived_from")}
    class_counts = Counter(row.get("failure_class") or "unclassified" for row in failures)
    retry_counts = Counter(row.get("retry_policy") or "unknown" for row in failures)
    repeated_classes = {cls: count for cls, count in sorted(class_counts.items()) if count > 1}
    repair_needed = [
        row for row in failures
        if row.get("retry_policy") == "hard_failure_no_rerun"
        or row.get("deterministic") is True
    ]
    cutoff = datetime.now(timezone.utc) - timedelta(hours=stale_hours)
    stale_unhealed = []
    for row in failures:
        ts = parse_ts(row.get("ts", ""))
        if row.get("id") not in healed_ids and ts and ts < cutoff:
            stale_unhealed.append(row)
    heal_times = [ts for ts in (parse_ts(row.get("ts", "")) for row in heals) if ts]
    latest_heal_ts = max(heal_times).isoformat().replace("+00:00", "Z") if heal_times else ""
    status = "ok"
    if stale_unhealed:
        status = "stale_unhealed_failures"
    elif repair_needed:
        status = "repair_needed"
    return {
        "schema": "focusa.self_heal_telemetry.v1",
        "status": status,
        "failure_count": len(failures),
        "self_heal_count": len(heals),
        "class_counts": dict(sorted(class_counts.items())),
        "retry_policy_counts": dict(sorted(retry_counts.items())),
        "repeated_classes": repeated_classes,
        "open_repair_needed": [row.get("id") for row in repair_needed],
        "stale_unhealed_failures": [row.get("id") for row in stale_unhealed],
        "latest_heal_ts": latest_heal_ts,
    }


def render_human(payload: dict) -> str:
    out = [f"self-heal telemetry: {payload['status']}"]
    out.append(f"failures={payload['failure_count']} self_heals={payload['self_heal_count']}")
    out.append("class_counts:")
    for cls, count in payload["class_counts"].items():
        out.append(f"  - {cls}: {count}")
    out.append("retry_policy_counts:")
    for retry, count in payload["retry_policy_counts"].items():
        out.append(f"  - {retry}: {count}")
    if payload["repeated_classes"]:
        out.append("repeated_classes:")
        for cls, count in payload["repeated_classes"].items():
            out.append(f"  - {cls}: {count}")
    out.append("open_repair_needed: " + (", ".join(payload["open_repair_needed"]) or "none"))
    out.append("stale_unhealed_failures: " + (", ".join(payload["stale_unhealed_failures"]) or "none"))
    out.append("latest_heal_ts: " + (payload["latest_heal_ts"] or "none"))
    return "\n".join(out)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--audit", default=DEFAULT_AUDIT)
    parser.add_argument("--stale-hours", type=float, default=24.0)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)
    payload = build_payload(load_rows(Path(args.audit)), args.stale_hours)
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        print(render_human(payload))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
