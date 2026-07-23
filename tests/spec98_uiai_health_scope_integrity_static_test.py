#!/usr/bin/env python3
"""Guard: uiai_health is infrastructure telemetry, never project-scope authority."""

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
SPEC98 = ROOT / "docs/98-project-root-crdt-reconciliation-foundation-spec.md"
IMPACT = ROOT / "docs/worksheets/focusa-877z.14-pi-uiai-impact.yaml"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def require(path: Path, needle: str, label: str) -> None:
    text = path.read_text()
    if needle not in text:
        fail(f"{label} missing {needle!r}")


def main() -> None:
    for needle in [
        "uiai_health is infrastructure-only telemetry",
        "must not seed project_root or continuity_id",
        "project truth comes from ProjectIdentity plus Workpoint/Trajectory scope",
        "focusa_scope is echo/provenance metadata, not authority",
    ]:
        require(SPEC98, needle, "Spec98 UIAI health scope guidance")
    for needle in [
        "uiai_health_scope_integrity",
        "infra_only_advisory_telemetry",
        "no_project_scope_authority",
        "ProjectIdentity plus Workpoint/Trajectory scope",
    ]:
        require(IMPACT, needle, "Pi/UIAI impact worksheet")
    print(
        "✓ PASS: uiai_health remains infra-only and cannot become project-scope authority"
    )


if __name__ == "__main__":
    main()
