#!/usr/bin/env python3
"""Fail-closed Work Loop normative-MUST trace and proof runner."""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_MANIFEST = ROOT / "docs/worksheets/work-loop-conformance-v1.json"
VALID_STATUSES = {"verified", "partial", "blocked", "unsupported"}
MUST_RE = re.compile(r"\bmust\b", re.IGNORECASE)


def load_manifest(path: Path) -> dict[str, Any]:
    data = json.loads(path.read_text())
    if data.get("schema") != "focusa.work_loop_conformance.v1":
        raise ValueError("unsupported conformance manifest schema")
    specs = data.get("specs")
    if not isinstance(specs, list) or not specs:
        raise ValueError("manifest specs must be a non-empty list")
    seen: set[str] = set()
    for spec in specs:
        spec_id = spec.get("id")
        if not isinstance(spec_id, str) or not spec_id or spec_id in seen:
            raise ValueError("spec ids must be unique non-empty strings")
        seen.add(spec_id)
        if spec.get("status") not in VALID_STATUSES:
            raise ValueError(f"{spec_id}: invalid status")
        proof_sets = spec.get("proofs")
        if not isinstance(proof_sets, list) or not proof_sets:
            raise ValueError(f"{spec_id}: at least one proof command is required")
        for command in proof_sets:
            if not isinstance(command, list) or not command or not all(isinstance(v, str) and v for v in command):
                raise ValueError(f"{spec_id}: proof commands must be non-empty argv arrays")
    return data


def must_records(manifest: dict[str, Any]) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    for spec in manifest["specs"]:
        path = ROOT / spec["path"]
        if not path.is_file():
            raise ValueError(f"{spec['id']}: missing normative spec {spec['path']}")
        heading = "(preamble)"
        count = 0
        for line_number, line in enumerate(path.read_text().splitlines(), 1):
            if line.startswith("#"):
                heading = line.strip()
            if MUST_RE.search(line):
                count += 1
                records.append({
                    "must_id": f"{spec['id']}:L{line_number}",
                    "spec": spec["id"],
                    "path": spec["path"],
                    "line": line_number,
                    "heading": heading,
                    "statement": line.strip(),
                    "coverage_status": spec["status"],
                    "proof_count": len(spec["proofs"]),
                    "gap": spec.get("gap"),
                })
        if count == 0:
            raise ValueError(f"{spec['id']}: normative spec contains no MUST statements")
    return records


def run_proofs(manifest: dict[str, Any], verified_only: bool) -> int:
    failures = 0
    for spec in manifest["specs"]:
        if verified_only and spec["status"] != "verified":
            continue
        for command in spec["proofs"]:
            print(f"PROOF {spec['id']}: {' '.join(command)}", flush=True)
            result = subprocess.run(command, cwd=ROOT, check=False)
            if result.returncode:
                failures += 1
                print(f"FAIL {spec['id']}: exit={result.returncode}", file=sys.stderr)
    return failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--mode", choices=("audit", "implemented", "release"), default="audit")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    try:
        manifest = load_manifest(args.manifest.resolve())
        records = must_records(manifest)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"conformance manifest rejected: {error}", file=sys.stderr)
        return 2

    pending = [record for record in records if record["coverage_status"] != "verified"]
    summary = {
        "schema": "focusa.work_loop_conformance_report.v1",
        "mode": args.mode,
        "normative_must_total": len(records),
        "verified_must_total": len(records) - len(pending),
        "pending_must_total": len(pending),
        "release_ready": not pending,
        "specs": [
            {"id": spec["id"], "status": spec["status"], "gap": spec.get("gap")}
            for spec in manifest["specs"]
        ],
    }
    if args.json:
        print(json.dumps({**summary, "must_trace": records}, indent=2))
    else:
        print(json.dumps(summary, indent=2))

    if args.mode == "audit":
        return 0
    if args.mode == "release" and pending:
        print("RELEASE BLOCKED: normative MUST coverage remains partial, blocked, or unsupported", file=sys.stderr)
        return 3
    failures = run_proofs(manifest, verified_only=False)
    receipt = {
        "schema": "focusa.work_loop_conformance_proof_receipt.v1",
        "mode": args.mode,
        "proof_status": "failed" if failures else "implemented_subset_passed",
        "proof_failures": failures,
        "release_ready": not pending and failures == 0,
    }
    print(json.dumps(receipt, indent=2))
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
