#!/usr/bin/env python3
"""Surgically update focusa-mc2 statuses in the isolated working-subpath JSONL."""
from __future__ import annotations

import argparse
import json
import os
import sqlite3
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
JSONL = ROOT / ".beads/issues.jsonl"
DB = ROOT / ".beads/beads.db"
PREFIX = "focusa-mc2"


def load_lines() -> tuple[list[str], dict[str, dict[str, Any]]]:
    lines = JSONL.read_text().splitlines(keepends=True)
    records = {record["id"]: record for record in (json.loads(line) for line in lines if line.strip())}
    return lines, records


def atomic_write(lines: list[str]) -> None:
    fd, tmp = tempfile.mkstemp(prefix="issues.jsonl.", dir=JSONL.parent)
    try:
        with os.fdopen(fd, "w") as handle:
            handle.writelines(lines)
        os.replace(tmp, JSONL)
    finally:
        if os.path.exists(tmp):
            os.unlink(tmp)


def sync_cache(records: dict[str, dict[str, Any]], ids: list[str]) -> None:
    connection = sqlite3.connect(DB)
    try:
        connection.execute("BEGIN IMMEDIATE")
        for issue_id in ids:
            record = records[issue_id]
            connection.execute(
                """
                UPDATE issues
                SET status = ?, updated_at = ?, closed_at = ?, close_reason = ?
                WHERE id = ?
                """,
                (
                    record["status"],
                    record["updated_at"],
                    record.get("closed_at"),
                    record.get("close_reason", ""),
                    issue_id,
                ),
            )
        connection.commit()
    finally:
        connection.close()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=["close"])
    parser.add_argument("ids", nargs="+")
    parser.add_argument("--reason", required=True)
    parser.add_argument("--timestamp")
    args = parser.parse_args()
    lines, records = load_lines()
    for issue_id in args.ids:
        if not (issue_id == PREFIX or issue_id.startswith(PREFIX + ".")):
            raise SystemExit(f"refusing non-{PREFIX} issue: {issue_id}")
        if issue_id not in records:
            raise SystemExit(f"unknown issue: {issue_id}")
    stamp = args.timestamp or datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")
    changed: set[str] = set()
    for issue_id in args.ids:
        record = records[issue_id]
        blockers = [
            dep["depends_on_id"]
            for dep in record.get("dependencies") or []
            if dep["type"] == "blocks" and records[dep["depends_on_id"]]["status"] != "closed"
        ]
        if blockers:
            raise SystemExit(f"dependency gate blocked for {issue_id}: {blockers}")
        record["status"] = "closed"
        record["updated_at"] = stamp
        record["closed_at"] = stamp
        record["close_reason"] = args.reason
        changed.add(issue_id)
    output: list[str] = []
    for raw in lines:
        record = json.loads(raw)
        if record["id"] in changed:
            output.append(json.dumps(records[record["id"]], ensure_ascii=False, separators=(",", ":")) + "\n")
        else:
            output.append(raw)
    atomic_write(output)
    sync_cache(records, args.ids)
    print(f"Closed {len(args.ids)} focusa-mc2 issue(s): {', '.join(args.ids)}")


if __name__ == "__main__":
    main()
