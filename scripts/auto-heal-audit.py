#!/usr/bin/env python3
"""Focusa auto-heal audit hook.

Reads the append-only audit ledger and emits one self-heal synthesis
row per failure that does not yet have a matching self_heal row.

Usage:
  python3 scripts/auto-heal-audit.py [audit.jsonl]
"""
from __future__ import annotations

import json
import sys
import time
from pathlib import Path

DEFAULT_AUDIT_FILE = "release-proof/audit/audit.jsonl"


def load_entries(path: Path) -> list[dict]:
    out: list[dict] = []
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


def heal_id(failure_id: str) -> str:
    return f"heal-{failure_id}" if failure_id else "heal-unknown"


def synthesize(failure: dict, audit_path: Path) -> dict:
    return {
        "ts": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "event": "self_heal",
        "subsystem": "ops",
        "scope": failure.get("scope", ""),
        "category": failure.get("category", ""),
        "derived_from": failure.get("id", ""),
        "symptom": failure.get("symptom", ""),
        "root_cause": failure.get("root_cause", ""),
        "fix": failure.get("fix", ""),
        "guard": failure.get("guard", ""),
        "test": failure.get("test", ""),
        "linked_run": failure.get("linked_run", ""),
        "auto_generated": True,
    }


def write_synthesis(audit_path: Path, row: dict) -> None:
    with audit_path.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(row, separators=(",", ":")) + "\n")


def main() -> int:
    if len(sys.argv) > 1:
        audit_path = Path(sys.argv[1])
    else:
        repo_root = Path(__file__).resolve().parents[1]
        audit_path = repo_root / DEFAULT_AUDIT_FILE

    if not audit_path.exists():
        print(f"audit file missing: {audit_path}", file=sys.stderr)
        return 1

    entries = load_entries(audit_path)
    heals = {e.get("derived_from") for e in entries if e.get("event") == "self_heal"}
    failures = [e for e in entries if e.get("event") == "failure"]

    written = 0
    for f in failures:
        fid = f.get("id")
        if not fid or fid in heals:
            continue
        row = synthesize(f, audit_path)
        write_synthesis(audit_path, row)
        print(f"[focusa-audit] self_heal synthesized for {fid} ({f.get('category', 'unknown')})")
        heals.add(fid)
        written += 1

    if written == 0:
        print("[focusa-audit] audit trail fully self-heal-synchronized")
    else:
        print(f"[focusa-audit] synthesized {written} self_heal row(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())