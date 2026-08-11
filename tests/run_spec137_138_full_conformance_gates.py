#!/usr/bin/env python3
"""Run every Spec137/137A/138/138A full-conformance gate without pytest."""
from __future__ import annotations

import runpy
import sys
import traceback
from collections.abc import Callable, Mapping, Sequence
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REQUIRED_SUITES = (
    "tests/spec137_full_conformance_gate.py",
    "tests/spec137a_applicability_decision_gate.py",
    "tests/spec138_full_conformance_gate.py",
    "tests/spec138a_full_conformance_gate.py",
)
FUNCTION_SUITES = frozenset(
    {
        "tests/spec138_full_conformance_gate.py",
        "tests/spec138a_full_conformance_gate.py",
    }
)


def validate_suite_manifest(suites: Sequence[str]) -> None:
    configured = tuple(suites)
    if configured != REQUIRED_SUITES:
        missing = sorted(set(REQUIRED_SUITES) - set(configured))
        extra = sorted(set(configured) - set(REQUIRED_SUITES))
        raise RuntimeError(
            f"full-conformance suite manifest mismatch: missing={missing}, extra={extra}"
        )


def run_test_functions(namespace: Mapping[str, object], suite: str) -> list[str]:
    tests: list[tuple[str, Callable[[], object]]] = sorted(
        (name, value)
        for name, value in namespace.items()
        if name.startswith("test_") and callable(value)
    )
    if not tests:
        raise RuntimeError(f"{suite}: direct Python load discovered zero test functions")

    failures: list[str] = []
    for name, test in tests:
        try:
            test()
            print(f"PASS {suite}::{name}")
        except Exception as error:  # Keep running so no later test is omitted.
            failures.append(f"{suite}::{name}: {type(error).__name__}: {error}")
            print(f"FAIL {failures[-1]}", file=sys.stderr)
            traceback.print_exc()
    return failures


def run_suites(suites: Sequence[str] = REQUIRED_SUITES) -> list[str]:
    validate_suite_manifest(suites)
    failures: list[str] = []
    for suite in suites:
        path = ROOT / suite
        try:
            namespace = runpy.run_path(str(path), run_name=f"__focusa_gate_{path.stem}__")
            if suite in FUNCTION_SUITES:
                failures.extend(run_test_functions(namespace, suite))
            else:
                print(f"PASS {suite}")
        except Exception as error:  # Aggregate suite failures rather than skipping later suites.
            failures.append(f"{suite}: {type(error).__name__}: {error}")
            print(f"FAIL {failures[-1]}", file=sys.stderr)
            traceback.print_exc()
    return failures


def main() -> int:
    failures = run_suites()
    if failures:
        print(
            f"Spec137/137A/138/138A full-conformance gates: FAIL ({len(failures)} failures)",
            file=sys.stderr,
        )
        return 1
    print("Spec137/137A/138/138A full-conformance gates: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
