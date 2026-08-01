#!/usr/bin/env python3
"""Generate a source-complete, implementation-open Spec150 feature ledger."""
from __future__ import annotations

import argparse
import hashlib
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "docs/150-focusa-guided-install-first-project-and-lifecycle-master-spec.md"
OUTPUT = ROOT / "docs/contracts/spec150-complete-feature-ledger.v1.yaml"


def source_rows() -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    in_fence = False
    section = "0"
    for line_number, raw in enumerate(SOURCE.read_text().splitlines(), 1):
        text = raw.strip()
        if text.startswith("```"):
            in_fence = not in_fence
            continue
        if not text or in_fence or text.startswith("# "):
            continue
        if text.startswith("## ") or text.startswith("### "):
            section = text.lstrip("#").strip().split(maxsplit=1)[0].rstrip(".")
            continue
        normalized = text.removeprefix("- ").strip()
        digest = hashlib.sha256(f"{line_number}:{normalized}".encode()).hexdigest()[:12]
        rows.append(
            {
                "requirement_id": f"S150-R-{line_number:04d}-{digest}",
                "source_line": line_number,
                "spec_section": section,
                "requirement_text": normalized,
                "requirement_text_sha256": "sha256:"
                + hashlib.sha256(normalized.encode()).hexdigest(),
                "applicability_decision": "decision_required",
                "runtime_status": "implementation_open",
                "implementation_refs": [],
                "test_refs": [],
                "evidence_refs": [],
                "receipt_refs": [],
                "platform_refs": [],
                "focus_stack_refs": [],
                "reducer_event_refs": [],
                "awareness_refs": [],
                "runbook_refs": [],
            }
        )
    return rows


def build() -> dict[str, object]:
    rows = source_rows()
    return {
        "schema": "focusa.spec150.complete_feature_ledger.v1",
        "spec_ref": SOURCE.relative_to(ROOT).as_posix(),
        "spec_hash": "sha256:" + hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
        "source_atom_count": len(rows),
        "runtime_status": "implementation_open",
        "requirements": rows,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    expected = build()
    rendered = yaml.safe_dump(expected, sort_keys=False, width=100)
    if args.check:
        if not OUTPUT.is_file():
            print("Spec150 complete feature ledger is stale or missing")
            return 1
        actual = yaml.safe_load(OUTPUT.read_text())
        immutable_keys = ("schema", "spec_ref", "spec_hash", "source_atom_count")
        if any(actual.get(key) != expected.get(key) for key in immutable_keys):
            print("Spec150 complete feature ledger is stale or missing")
            return 1
        source_keys = (
            "requirement_id",
            "source_line",
            "spec_section",
            "requirement_text",
            "requirement_text_sha256",
            "applicability_decision",
        )
        actual_rows = actual.get("requirements", [])
        expected_rows = expected["requirements"]
        if len(actual_rows) != len(expected_rows) or any(
            any(actual_row.get(key) != expected_row.get(key) for key in source_keys)
            for actual_row, expected_row in zip(actual_rows, expected_rows)
        ):
            print("Spec150 complete feature ledger is stale or missing")
            return 1
        if actual.get("runtime_status") == "verified_complete" and any(
            row.get("runtime_status") != "verified_complete"
            or not row.get("implementation_refs")
            or not row.get("test_refs")
            or not row.get("evidence_refs")
            or not row.get("receipt_refs")
            for row in actual_rows
        ):
            print("Spec150 activated ledger has incomplete runtime evidence")
            return 1
        print("Spec150 complete feature ledger: current")
        return 0
    OUTPUT.write_text(rendered)
    print(f"wrote {OUTPUT.relative_to(ROOT)} with {len(build()['requirements'])} rows")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
