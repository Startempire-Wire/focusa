#!/usr/bin/env python3
"""Spec98 focusa-877z.7: core cognition/runtime/telemetry plane guard."""

from pathlib import Path
import re
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = (
    ROOT
    / "docs/worksheets/focusa-877z.7-runtime-telemetry-cognition-plane-contract.yaml"
)
TYPES = ROOT / "crates/focusa-core/src/types.rs"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def main() -> None:
    data = yaml.safe_load(CONTRACT.read_text())
    if (
        data.get("schema_version")
        != "focusa.runtime_telemetry_cognition_plane_contract.v1"
    ):
        fail("unexpected .7 contract schema")
    if data.get("status") != "core_authority_planes_defined_in_types":
        fail("unexpected .7 contract status")

    text = TYPES.read_text()
    enum_start = text.find("pub enum AuthorityPlane")
    enum_end = text.find("pub const FOCUSA_STATE_PLANE_CONTRACT", enum_start)
    if enum_start == -1 or enum_end == -1:
        fail("AuthorityPlane enum must exist before FOCUSA_STATE_PLANE_CONTRACT")
    enum_body = text[enum_start:enum_end]
    for variant in [
        "CanonicalCognition",
        "RuntimeCorrelation",
        "TelemetryHistory",
        "AdvisoryProjection",
        "BoundedOrchestration",
    ]:
        if variant not in enum_body:
            fail(f"AuthorityPlane missing {variant}")

    if (
        "Only `CanonicalCognition` fields participate in Focus State authority"
        not in text
    ):
        fail(
            "types.rs must state only canonical cognition participates in Focus State authority"
        )
    if (
        "INVARIANT: Only AuthorityPlane::CanonicalCognition participates in Focus State authority."
        not in text
    ):
        fail("FocusaState invariant missing canonical-only authority rule")

    contract_start = text.find("pub const FOCUSA_STATE_PLANE_CONTRACT")
    contract_end = text.find("/// The complete cognitive state", contract_start)
    plane_map_text = text[contract_start:contract_end]
    pairs = dict(
        re.findall(r'\("([a-z_]+)", AuthorityPlane::([A-Za-z]+)\)', plane_map_text)
    )

    state_start = text.find("pub struct FocusaState")
    state_end = text.find("impl FocusaState", state_start)
    state_body = text[state_start:state_end]
    fields = re.findall(r"pub ([a-z_]+):", state_body)
    for field in fields:
        if field not in pairs:
            fail(f"FOCUSA_STATE_PLANE_CONTRACT missing FocusaState field {field}")

    expected = {
        "focus_stack": "CanonicalCognition",
        "memory": "CanonicalCognition",
        "constitution": "CanonicalCognition",
        "workpoint": "CanonicalCognition",
        "active_turn": "RuntimeCorrelation",
        "session": "RuntimeCorrelation",
        "instances": "RuntimeCorrelation",
        "telemetry": "TelemetryHistory",
        "clt": "TelemetryHistory",
        "contribution": "TelemetryHistory",
        "focus_gate": "AdvisoryProjection",
        "ontology": "AdvisoryProjection",
        "trajectory": "AdvisoryProjection",
        "pre": "AdvisoryProjection",
        "work_loop": "BoundedOrchestration",
    }
    for field, plane in expected.items():
        if pairs.get(field) != plane:
            fail(f"{field} must map to {plane}, got {pairs.get(field)}")

    if pairs.get("active_turn") == "CanonicalCognition":
        fail("active_turn must not be canonical cognition")
    if pairs.get("telemetry") == "CanonicalCognition":
        fail("telemetry must not be canonical cognition")
    if pairs.get("work_loop") == "CanonicalCognition":
        fail("work_loop must not be canonical cognition")

    print(
        "✓ PASS: core state fields declare cognition/runtime/telemetry/advisory/orchestration planes"
    )


if __name__ == "__main__":
    main()
