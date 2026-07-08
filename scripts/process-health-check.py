#!/usr/bin/env python3
"""Process health checkpoint utility for CI gates.

This script emits a compact process snapshot and can optionally run a target
command while preserving its exit code. It is used as a CI gate wrapper so unknown
process failures leave actionable evidence.
"""
from __future__ import annotations

import argparse
import subprocess
import sys
from datetime import datetime, timezone


def emit_process_snapshot(label: str) -> None:
    print(f"[process-health-check] {label} snapshot")
    print(f"[process-health-check] ts={datetime.now(timezone.utc).isoformat()}")
    cmd = [
        "ps",
        "-eo",
        "pid,ppid,stat,%cpu,%mem,etime,comm,cmd",
        "--sort=-%cpu",
        "|",
        "head",
        "-n",
        "25",
    ]
    # Use a shell pipeline for portability to avoid locale-dependent flags in
    # subprocess argv handling.
    subprocess.run(" ".join(cmd), shell=True, check=False)


def run_with_health_check(command: str, label: str) -> int:
    emit_process_snapshot(f"before::{label}")
    result = subprocess.run(command, shell=True)
    if result.returncode != 0:
        emit_process_snapshot(f"after-failure::{label} rc={result.returncode}")
    else:
        emit_process_snapshot(f"after-success::{label}")
    return result.returncode


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--label", default="ci-step")
    parser.add_argument(
        "--command",
        help="Optional command to execute under process health monitoring.",
    )
    parser.add_argument("--snapshot-only", action="store_true", help="Only emit snapshot")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    emit_process_snapshot(f"entry::{args.label}")

    if args.snapshot_only:
        return 0

    if args.command:
        return run_with_health_check(args.command, args.label)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
