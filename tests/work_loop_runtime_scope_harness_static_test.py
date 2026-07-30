#!/usr/bin/env python3
"""Fail closed when a Workloop HTTP runtime harness omits typed scope headers."""

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
TESTS = ROOT / "tests"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def main() -> None:
    checked = 0
    for path in sorted(TESTS.glob("work_loop_*.sh")):
        text = path.read_text()
        if not any(marker in text for marker in ("FOCUSA_BASE_URL", "127.0.0.1", "curl")):
            continue
        checked += 1
        for marker in ["x-scope-project-root", "x-scope-continuity-id"]:
            if marker not in text:
                fail(f"{path.name} missing scoped runtime marker: {marker}")
        if "FOCUSA_BASE_URL" in text:
            for marker in ["FOCUSA_PROJECT_ROOT", "FOCUSA_CONTINUITY_ID"]:
                if marker not in text:
                    fail(f"{path.name} missing external-daemon scope override: {marker}")
    if checked == 0:
        fail("no Workloop HTTP runtime harnesses discovered")
    print(f"✓ PASS: {checked} Workloop HTTP runtime harnesses carry typed project/workstream scope")


if __name__ == "__main__":
    main()
