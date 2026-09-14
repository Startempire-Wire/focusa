#!/usr/bin/env python3
"""Static privacy gate for public Spec 135 UI evaluation artifacts."""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURE_DIR = ROOT / "docs/contracts/spec135/generated-contract-v1"
PRIVATE_REFERENCE = re.compile(
    r"/(?:home|root|Users)/|[A-Za-z]:\\(?:Users|home)\\|"
    r"uiai-(?:browser|diagnostics):session=|session-screenshots",
    re.IGNORECASE,
)
PRIVATE_FIELDS = {"browser_session_refs", "screenshot_refs", "visual_comparison_refs"}


def walk(value: object, path: str = "$") -> None:
    if isinstance(value, str):
        assert not PRIVATE_REFERENCE.search(value), f"private reference at {path}"
    elif isinstance(value, list):
        for index, item in enumerate(value):
            walk(item, f"{path}[{index}]")
    elif isinstance(value, dict):
        for key, item in value.items():
            if key in PRIVATE_FIELDS:
                assert item == [], f"{path}.{key} must be redacted from public fixture"
            walk(item, f"{path}.{key}")


paths = sorted(FIXTURE_DIR.glob("uiai-eval*.result.json"))
assert paths, "expected Spec 135 UI evaluation fixtures"
for fixture in paths:
    document = json.loads(fixture.read_text(encoding="utf-8"))
    assert document.get("public_evidence_disposition"), f"{fixture} lacks public evidence disposition"
    walk(document, str(fixture))

print(f"Spec135 public artifact privacy: PASS ({len(paths)} fixtures)")
