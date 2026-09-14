#!/usr/bin/env python3
"""Make public Spec 135 UI evaluation fixtures portable and private by construction."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

PRIVATE_REFERENCE = re.compile(
    r"/(?:home|root|Users)/|[A-Za-z]:\\(?:Users|home)\\|"
    r"uiai-(?:browser|diagnostics):session=|session-screenshots",
    re.IGNORECASE,
)
PRIVATE_REFERENCE_FIELDS = {
    "browser_session_refs",
    "screenshot_refs",
    "visual_comparison_refs",
}
REDACTED = "redacted-private-reference"


def redact(value: Any, key: str | None = None) -> Any:
    if key in PRIVATE_REFERENCE_FIELDS:
        return []
    if isinstance(value, str):
        return REDACTED if PRIVATE_REFERENCE.search(value) else value
    if isinstance(value, list):
        return [redact(item) for item in value]
    if isinstance(value, dict):
        return {item_key: redact(item_value, item_key) for item_key, item_value in value.items()}
    return value


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    fixture_dir = root / "docs/contracts/spec135/generated-contract-v1"
    paths = sorted(fixture_dir.glob("uiai-eval*.result.json"))
    if not paths:
        raise SystemExit(f"no Spec 135 UI evaluation fixtures found under {fixture_dir}")

    for path in paths:
        document = json.loads(path.read_text(encoding="utf-8"))
        sanitized = redact(document)
        sanitized["public_evidence_disposition"] = (
            "private browser/session evidence redacted; no public artifact is claimed"
        )
        path.write_text(json.dumps(sanitized, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    print(f"redacted {len(paths)} public Spec 135 UI evaluation fixtures")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
