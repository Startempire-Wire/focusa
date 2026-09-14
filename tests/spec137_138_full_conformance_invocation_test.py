#!/usr/bin/env python3
"""Regression coverage for strict Spec137/137A/138/138A gate invocation."""
from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUNNER_PATH = ROOT / "tests/run_spec137_138_full_conformance_gates.py"
EXPECTED_SUITES = (
    "tests/spec137_full_conformance_gate.py",
    "tests/spec137a_applicability_decision_gate.py",
    "tests/spec138_full_conformance_gate.py",
    "tests/spec138a_full_conformance_gate.py",
)
FUNCTION_SUITES = EXPECTED_SUITES[2:]
CANONICAL_INVOCATION = "python3 ./tests/run_spec137_138_full_conformance_gates.py"

spec = importlib.util.spec_from_file_location("full_conformance_runner", RUNNER_PATH)
assert spec and spec.loader
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)

assert runner.REQUIRED_SUITES == EXPECTED_SUITES
assert runner.FUNCTION_SUITES == frozenset(FUNCTION_SUITES)
runner.validate_suite_manifest(EXPECTED_SUITES)

for omitted in EXPECTED_SUITES:
    incomplete = tuple(suite for suite in EXPECTED_SUITES if suite != omitted)
    try:
        runner.validate_suite_manifest(incomplete)
    except RuntimeError as error:
        assert omitted in str(error)
    else:
        raise AssertionError(f"omitting required suite {omitted} did not fail")

try:
    runner.run_test_functions({}, "empty_fixture.py")
except RuntimeError as error:
    assert "discovered zero test functions" in str(error)
else:
    raise AssertionError("a direct Python load with zero discovered tests did not fail")

calls: list[str] = []
namespace = {
    "test_second": lambda: calls.append("second"),
    "test_first": lambda: calls.append("first"),
}
assert runner.run_test_functions(namespace, "fixture.py") == []
assert calls == ["first", "second"]

for suite in FUNCTION_SUITES:
    source = (ROOT / suite).read_text()
    assert 'if __name__ == "__main__":' in source
    assert "run_test_functions(globals()," in source, (
        f"{suite}: direct Python invocation would silently run zero tests"
    )

for integration_path in (
    "scripts/ci/run-spec-gates.sh",
    "tests/final_release_gap_gate.sh",
):
    text = (ROOT / integration_path).read_text()
    assert text.count(CANONICAL_INVOCATION) == 1, (
        f"{integration_path}: canonical full-conformance invocation missing or duplicated"
    )
    for suite in FUNCTION_SUITES:
        direct = f"python3 ./{suite}"
        assert direct not in text, (
            f"{integration_path}: {suite} invoked directly and would silently run zero tests"
        )

print("Spec137/137A/138/138A full-conformance invocation regression: PASS")
