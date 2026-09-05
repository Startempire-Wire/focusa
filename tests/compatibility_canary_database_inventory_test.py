#!/usr/bin/env python3
"""Regression tests for compatibility-canary SQLite evidence."""

from __future__ import annotations

import json
import pathlib
import sqlite3
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/compatibility-canary-database-inventory.py"


def invoke(*args: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["python3", str(SCRIPT), *(str(arg) for arg in args)],
        text=True,
        capture_output=True,
    )


def capture(database: pathlib.Path, output: pathlib.Path) -> None:
    result = invoke("capture", "--database", database, "--output", output)
    assert result.returncode == 0, result.stderr


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="focusa-canary-database-") as raw:
        root = pathlib.Path(raw)
        source_database = root / "legacy # source.sqlite"
        connection = sqlite3.connect(source_database)
        connection.executescript(
            """
            CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            CREATE TABLE events (id INTEGER PRIMARY KEY, payload TEXT NOT NULL);
            CREATE TABLE snapshots (id INTEGER PRIMARY KEY, payload TEXT NOT NULL);
            INSERT INTO meta VALUES ('schema', 'legacy');
            INSERT INTO events(payload) VALUES ('one');
            INSERT INTO snapshots(payload) VALUES ('one');
            """
        )
        connection.commit()
        connection.close()

        database = root / "focusa.sqlite"
        copied = invoke("copy", "--source", source_database, "--destination", database)
        assert copied.returncode == 0, copied.stderr
        assert database.stat().st_mode & 0o777 == 0o600
        duplicate = invoke("copy", "--source", source_database, "--destination", database)
        assert duplicate.returncode != 0

        baseline = root / "baseline.json"
        capture(database, baseline)
        baseline_value = json.loads(baseline.read_text(encoding="utf-8"))
        assert baseline_value["quick_check"] == "ok"
        assert baseline_value["table_count"] == 3
        assert baseline_value["total_rows"] == 3

        connection = sqlite3.connect(database)
        connection.executescript(
            """
            CREATE TABLE additive_state (id INTEGER PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO additive_state(value) VALUES ('candidate');
            INSERT INTO events(payload) VALUES ('two');
            """
        )
        connection.commit()
        connection.close()
        candidate = root / "candidate.json"
        capture(database, candidate)
        compatible = invoke("compare", "--baseline", baseline, "--observed", candidate)
        assert compatible.returncode == 0, compatible.stderr

        same_schema = invoke(
            "compare",
            "--baseline",
            baseline,
            "--observed",
            candidate,
            "--same-schema",
        )
        assert same_schema.returncode != 0
        assert "different SQLite schema" in same_schema.stderr

        reapply = root / "reapply.json"
        capture(database, reapply)
        stable = invoke(
            "compare",
            "--baseline",
            candidate,
            "--observed",
            reapply,
            "--same-schema",
        )
        assert stable.returncode == 0, stable.stderr

        reduced_value = json.loads(reapply.read_text(encoding="utf-8"))
        reduced_value["tables"]["events"] = 0
        reduced = root / "reduced.json"
        reduced.write_text(json.dumps(reduced_value), encoding="utf-8")
        rejected = invoke("compare", "--baseline", baseline, "--observed", reduced)
        assert rejected.returncode != 0
        assert "reduced durable rows" in rejected.stderr

    print("compatibility canary database inventory: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
