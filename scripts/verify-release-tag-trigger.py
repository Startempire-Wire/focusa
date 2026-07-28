#!/usr/bin/env python3
"""Fail before immutable tagging when Release workflow cannot match the proposed tag."""

import fnmatch
import re
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github/workflows/release.yml"

if len(sys.argv) != 2:
    print("usage: verify-release-tag-trigger.py TAG", file=sys.stderr)
    raise SystemExit(2)

tag = sys.argv[1]
if not re.fullmatch(r"v\d+\.\d+\.\d+(?:-[0-9A-Za-z][0-9A-Za-z.-]*)?", tag):
    print(f"invalid release tag: {tag}", file=sys.stderr)
    raise SystemExit(2)

workflow = yaml.safe_load(WORKFLOW.read_text())
triggers = workflow.get("on", workflow.get(True, {}))
push = triggers.get("push", {}) if isinstance(triggers, dict) else {}
patterns = push.get("tags", []) if isinstance(push, dict) else []
if isinstance(patterns, str):
    patterns = [patterns]
matched = [pattern for pattern in patterns if fnmatch.fnmatchcase(tag, str(pattern))]
if not matched:
    print(
        f"release tag trigger mismatch: {tag} does not match {WORKFLOW.relative_to(ROOT)} on.push.tags={patterns}",
        file=sys.stderr,
    )
    raise SystemExit(1)
print(f"release tag trigger: PASS tag={tag} pattern={matched[0]}")
