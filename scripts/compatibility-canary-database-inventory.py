#!/usr/bin/env python3
"""Capture and compare non-secret SQLite compatibility-canary facts.

The receipt records schema and row-count digests, never row payloads. Comparison
is additive: migrations may add schema/rows, but cannot remove a legacy table or
reduce its durable row count. Candidate reapply additionally requires the same
schema reached by the first candidate apply.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import sqlite3
from typing import Any

SCHEMA = "focusa.compatibility_canary_database_inventory.v1"


def canonical_digest(value: Any) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def quote_identifier(value: str) -> str:
    return '"' + value.replace('"', '""') + '"'


def capture(database: pathlib.Path) -> dict[str, Any]:
    if not database.is_file():
        raise ValueError("database is missing")
    connection = sqlite3.connect(
        database.resolve(strict=True).as_uri() + "?mode=ro", uri=True, timeout=10
    )
    try:
        quick_check = connection.execute("PRAGMA quick_check").fetchone()
        if quick_check is None or quick_check[0] != "ok":
            raise ValueError("database quick_check failed")
        foreign_key_errors = sum(1 for _ in connection.execute("PRAGMA foreign_key_check"))
        schema_rows = connection.execute(
            "SELECT type, name, tbl_name, COALESCE(sql, '') "
            "FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' ORDER BY type, name"
        ).fetchall()
        schema = [list(row) for row in schema_rows]
        table_names = sorted(
            row[1] for row in schema_rows if row[0] == "table" and row[1]
        )
        required = {"events", "snapshots", "meta"}
        missing = sorted(required.difference(table_names))
        if missing:
            raise ValueError(f"database omits legacy Focusa tables: {missing}")
        row_counts: dict[str, int] = {}
        for table in table_names:
            count = connection.execute(
                f"SELECT COUNT(*) FROM {quote_identifier(table)}"
            ).fetchone()
            row_counts[table] = int(count[0])
        if sum(row_counts.values()) <= 0:
            raise ValueError("database contains no durable rows")
        page_count = int(connection.execute("PRAGMA page_count").fetchone()[0])
        page_size = int(connection.execute("PRAGMA page_size").fetchone()[0])
        user_version = int(connection.execute("PRAGMA user_version").fetchone()[0])
    finally:
        connection.close()

    return {
        "schema": SCHEMA,
        "status": "verified",
        "database_bytes": database.stat().st_size,
        "quick_check": "ok",
        "foreign_key_error_count": foreign_key_errors,
        "user_version": user_version,
        "page_count": page_count,
        "page_size": page_size,
        "table_count": len(table_names),
        "total_rows": sum(row_counts.values()),
        "schema_sha256": canonical_digest(schema),
        "row_counts_sha256": canonical_digest(row_counts),
        "tables": row_counts,
    }


def load(path: pathlib.Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict) or value.get("schema") != SCHEMA:
        raise ValueError(f"{path.name} is not a canary database inventory")
    if value.get("quick_check") != "ok" or value.get("foreign_key_error_count") != 0:
        raise ValueError(f"{path.name} is not database-clean")
    if not isinstance(value.get("tables"), dict):
        raise ValueError(f"{path.name} omits table counts")
    return value


def compare(baseline: dict[str, Any], observed: dict[str, Any], same_schema: bool) -> None:
    baseline_tables = baseline["tables"]
    observed_tables = observed["tables"]
    removed = sorted(set(baseline_tables).difference(observed_tables))
    reduced = {
        table: {"baseline": count, "observed": observed_tables.get(table)}
        for table, count in baseline_tables.items()
        if table in observed_tables and observed_tables[table] < count
    }
    if removed:
        raise ValueError(f"compatibility migration removed tables: {removed}")
    if reduced:
        raise ValueError(f"compatibility migration reduced durable rows: {reduced}")
    if same_schema and baseline.get("schema_sha256") != observed.get("schema_sha256"):
        raise ValueError("candidate reapply reached a different SQLite schema")


def main() -> int:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    copy_parser = subparsers.add_parser("copy")
    copy_parser.add_argument("--source", required=True, type=pathlib.Path)
    copy_parser.add_argument("--destination", required=True, type=pathlib.Path)
    capture_parser = subparsers.add_parser("capture")
    capture_parser.add_argument("--database", required=True, type=pathlib.Path)
    capture_parser.add_argument("--output", required=True, type=pathlib.Path)
    compare_parser = subparsers.add_parser("compare")
    compare_parser.add_argument("--baseline", required=True, type=pathlib.Path)
    compare_parser.add_argument("--observed", required=True, type=pathlib.Path)
    compare_parser.add_argument("--same-schema", action="store_true")
    args = parser.parse_args()

    if args.command == "copy":
        if not args.source.is_file() or args.destination.exists():
            raise ValueError("database copy requires an existing source and absent destination")
        source = sqlite3.connect(
            args.source.resolve(strict=True).as_uri() + "?mode=ro",
            uri=True,
            timeout=10,
        )
        destination = sqlite3.connect(args.destination)
        try:
            source.backup(destination)
            destination.commit()
        finally:
            destination.close()
            source.close()
        args.destination.chmod(0o600)
        value = capture(args.destination)
        print(json.dumps({"status": "copied", "quick_check": value["quick_check"]}, sort_keys=True))
    elif args.command == "capture":
        value = capture(args.database)
        args.output.write_text(
            json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        print(
            json.dumps(
                {
                    key: value[key]
                    for key in ("status", "table_count", "total_rows", "schema_sha256")
                },
                sort_keys=True,
            )
        )
    else:
        compare(load(args.baseline), load(args.observed), args.same_schema)
        print(
            json.dumps(
                {"status": "compatible", "same_schema_required": args.same_schema},
                sort_keys=True,
            )
        )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, sqlite3.Error, json.JSONDecodeError) as error:
        print(f"compatibility database verification failed: {error}", file=__import__("sys").stderr)
        raise SystemExit(1)
