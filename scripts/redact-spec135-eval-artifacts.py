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
COMPACT_SUMMARY_SCHEMA = "focusa.ui_eval_compact_summary.v1"
LEGACY_COMPACT_FIELDS = {"artifact_refs", "diagnostics_ref", "limitations", "completed_at"}


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


def public_projection(document: dict[str, Any], full_schema: dict[str, Any]) -> dict[str, Any]:
    """Publish historical summaries without promoting their reported claims to proof."""
    sanitized = redact(document)
    full_id = full_schema["properties"]["schema"]["const"]
    if (
        sanitized.get("schema") == full_id
        and LEGACY_COMPACT_FIELDS.issubset(sanitized)
        and not set(full_schema["required"]).issubset(sanitized)
    ):
        sanitized["source_schema"] = sanitized["schema"]
        sanitized["schema"] = COMPACT_SUMMARY_SCHEMA
        sanitized["reported_status"] = sanitized["status"]
        sanitized["status"] = "incomplete"
        sanitized["verification_status"] = "incomplete"
        sanitized["closure_eligible"] = False
        for step in sanitized["step_results"]:
            step["reported_status"] = step["status"]
            step["status"] = "unverified"
    elif sanitized.get("schema") == full_id:
        for diagnostic in sanitized.get("diagnostics", []):
            if diagnostic.get("category") == "responsive":
                # Legacy classification, not a new measurement: retain the
                # reported category verbatim while using the v1 visual enum.
                diagnostic["category"] = "visual"
                diagnostic["summary"] = "reported_category=responsive; " + diagnostic["summary"]
    sanitized["public_evidence_disposition"] = (
        "private browser/session evidence redacted; no public artifact is claimed"
    )
    return sanitized


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    fixture_dir = root / "docs/contracts/spec135/generated-contract-v1"
    paths = sorted(fixture_dir.glob("uiai-eval*.result.json"))
    if not paths:
        raise SystemExit(f"no Spec 135 UI evaluation fixtures found under {fixture_dir}")

    full_schema = json.loads((fixture_dir / "uiai.focusa_ui_eval_result.v1.schema.json").read_text(encoding="utf-8"))
    for path in paths:
        document = json.loads(path.read_text(encoding="utf-8"))
        sanitized = public_projection(document, full_schema)
        if sanitized != document:
            path.write_text(json.dumps(sanitized, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    print(f"redacted {len(paths)} public Spec 135 UI evaluation fixtures")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
