#!/usr/bin/env python3
"""Spec 172 verified limited project test — one mutable project with preserved read/export."""

from __future__ import annotations

import json
import pathlib
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]

def run_cargo_test(test_filter: str) -> tuple[int, str]:
    """Run a single cargo test and return (exit_code, stdout)."""
    result = subprocess.run(
        ["cargo", "test", "-p", "focusa-core", test_filter, "--", "--nocapture"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        timeout=300,
    )
    return result.returncode, result.stdout + result.stderr

def main() -> int:
    failures: list[str] = []

    # ── 1. Run the Rust verified_limited_project tests ──
    exit_code, output = run_cargo_test("verified_limited_project")
    if exit_code != 0:
        # Extract just the test summary
        lines = output.splitlines()
        test_line = ""
        for line in lines:
            if "test result:" in line:
                test_line = line.strip()
            if "FAILED" in line and "test " in line:
                failures.append(f"Rust test failed: {line.strip()}")
        if not failures:
            failures.append(f"cargo test exited with code {exit_code}")
        failures.append(f"test summary: {test_line}" if test_line else "no test summary found")
    else:
        # Parse test count
        for line in output.splitlines():
            if "test result:" in line:
                print(f"Rust tests: {line.strip()}")

    # ── 2. Verify the limited_project module exists and exports correctly ──
    limited_project_path = ROOT / "crates" / "focusa-core" / "src" / "limited_project.rs"
    if not limited_project_path.exists():
        failures.append("limited_project.rs not found")

    # ── 3. Verify the ActiveProjectSelection is serializable and round-trips ──
    # This is covered by the Rust test verified_limited_project_serialization_round_trips

    # ── 4. Verify the entitlement execution guard integration ──
    guard_path = ROOT / "crates" / "focusa-core" / "src" / "entitlement_execution_guard.rs"
    if not guard_path.exists():
        failures.append("entitlement_execution_guard.rs not found")
    else:
        guard_content = guard_path.read_text()
        if "evaluate_entitlement_execution_for_project" not in guard_content:
            failures.append("evaluate_entitlement_execution_for_project not found in guard")
        if "verified_limited_project" not in guard_content:
            failures.append("verified_limited_project tests not found in guard")

    # ── 5. Verify the active decision enums carry upgrade/switch actions ──
    # Covered by Rust tests

    # ── 6. Verify the lib.rs export ──
    lib_path = ROOT / "crates" / "focusa-core" / "src" / "lib.rs"
    lib_content = lib_path.read_text()
    if "pub mod limited_project;" not in lib_content:
        failures.append("limited_project module not exported from lib.rs")

    # ── 7. Verify the license.rs re-export ──
    license_path = ROOT / "crates" / "focusa-core" / "src" / "license.rs"
    license_content = license_path.read_text()
    if "evaluate_entitlement_execution_for_project" not in license_content:
        failures.append("evaluate_entitlement_execution_for_project not re-exported from license.rs")

    # ── Report ──
    if failures:
        print("Spec 172 limited project test FAILED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("Spec 172 limited project test passed")
    print("verification=verified_limited_project")
    print("active_project_guard=ActiveProjectGuard")
    print("decisions=Allowed,DeniedSecondProject,DeniedNoSelection")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())