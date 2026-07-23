#!/usr/bin/env python3
"""Spec 100 Phase 4 explicit audit entrypoint.

Delegates to the combined eval/optimizer static audit while preserving the
Phase-4-specific filename referenced by backlog/proof gates.
"""

import runpy
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
runpy.run_path(
    str(ROOT / "tests" / "spec100_eval_optimizer_static_test.py"), run_name="__main__"
)
