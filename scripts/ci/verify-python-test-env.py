#!/usr/bin/env python3
"""Reject ambient, incomplete, or version-drifted Python test environments."""
from __future__ import annotations

import re
import sys
from importlib.metadata import PackageNotFoundError, distribution
from pathlib import Path


def main() -> None:
    if sys.prefix == sys.base_prefix:
        raise SystemExit("Use scripts/ci/setup-python-test-env.sh and its isolated interpreter")
    prefix = Path(sys.prefix).resolve()
    lock = Path(__file__).resolve().parents[2] / "requirements-test.txt"
    pins = re.findall(r"^([A-Za-z0-9_.-]+)==([^\s\\]+)", lock.read_text(), re.MULTILINE)
    if not pins:
        raise SystemExit("requirements-test.txt contains no exact dependency pins")
    for name, expected in pins:
        try:
            installed = distribution(name)
        except PackageNotFoundError:
            raise SystemExit(f"Missing locked dependency: {name}=={expected}") from None
        if installed.version != expected:
            raise SystemExit(f"Dependency drift: {name} expected={expected} actual={installed.version}")
        if not Path(installed.locate_file("")).resolve().is_relative_to(prefix):
            raise SystemExit(f"Ambient dependency rejected: {name}")
    from jsonschema import Draft202012Validator
    Draft202012Validator.check_schema({"type": "object"})
    print(f"Verified {len(pins)} locked dependencies in an isolated environment")


if __name__ == "__main__":
    main()
