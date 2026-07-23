#!/usr/bin/env python3
"""Fixture regression tests for scripts/classify-ci-failure.py.

Each fixture has:
  <name>.log
  <name>.expected.json

Expected JSON is a subset of the classifier payload. The test enforces exact
matches for listed fields and requires all core self-heal failure classes to be
covered by fixtures.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "tests" / "fixtures" / "self-heal-classifier"
CLASSIFIER = ROOT / "scripts" / "classify-ci-failure.py"

REQUIRED_CLASSES = {
    "rust_compile_api_drift",
    "rust_compile_format_arg_drift",
    "ci_clippy_failure",
    "ci_test_failure",
    "release_static_proof_failure",
    "deploy_health_failure",
    "runner_resource_failure",
    "auto_heal_process_error",
    "transient_github_or_network_failure",
    "unknown_process_failure",
}


def classify(log_path: Path) -> dict:
    proc = subprocess.run(
        [sys.executable, str(CLASSIFIER), str(log_path)],
        check=True,
        text=True,
        capture_output=True,
    )
    return json.loads(proc.stdout)


def main() -> int:
    expected_files = sorted(FIXTURES.glob("*.expected.json"))
    if not expected_files:
        raise SystemExit(f"no classifier fixtures found under {FIXTURES}")

    seen_classes: set[str] = set()
    failures: list[str] = []
    for expected_path in expected_files:
        log_path = expected_path.with_name(
            expected_path.name.removesuffix(".expected.json") + ".log"
        )
        if not log_path.exists():
            failures.append(
                f"{expected_path.name}: missing log fixture {log_path.name}"
            )
            continue
        expected = json.loads(expected_path.read_text())
        actual = classify(log_path)
        seen_classes.add(actual.get("failure_class", ""))
        for key, expected_value in expected.items():
            actual_value = actual.get(key)
            if actual_value != expected_value:
                failures.append(
                    f"{expected_path.name}: {key} mismatch\n"
                    f"  expected={expected_value!r}\n"
                    f"  actual  ={actual_value!r}"
                )
    missing_classes = sorted(REQUIRED_CLASSES - seen_classes)
    if missing_classes:
        failures.append(f"missing required fixture classes: {missing_classes}")
    if failures:
        print("=== self-heal classifier fixture failures ===", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    print(f"Self-heal classifier fixtures: PASS ({len(expected_files)} fixtures)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
