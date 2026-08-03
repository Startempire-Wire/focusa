#!/usr/bin/env python3
"""Spec 135A-3 Work Rail typed preview/commit interaction proof."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md").read_text()
TYPES = (ROOT / "crates/focusa-core/src/types.rs").read_text()
ROUTE = (ROOT / "crates/focusa-api/src/routes/work_rail.rs").read_text()
E2E = (ROOT / "tests/spec135_work_rail_e2e_test.py").read_text()
PI = (ROOT / "apps/pi-extension/src/work-rail-interactions.ts").read_text()
INDEX = (ROOT / "apps/pi-extension/src/index.ts").read_text()
OPENAPI = (ROOT / "docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json").read_text()
GENERATOR = (ROOT / "scripts/generate-spec135-work-rail-contracts.py").read_text()

for token in (
    "WorkRailInteractionRecord",
    "instance_id",
    "session_id",
    "work_surface_ids",
    "priority",
    "rank",
    "change_set_ref",
    "interaction_history",
):
    assert token in TYPES, token

for token in (
    "RailSideEffectPolicy",
    "Preview",
    "Commit",
    "request_preview_token",
    "commit requires the exact typed Work Rail preview_token",
    "Steer",
    "Defer",
    "RequestApproval",
    "Reopen",
    "reopen_bead",
    "WorkRailInteractionRecord",
):
    assert token in ROUTE, token

assert '"side_effect_policy": "preview"' in E2E
assert 'body["side_effect_policy"] = "commit"' in E2E
assert '"wrong-token"' in E2E
assert 'row["interaction_history"]' in E2E
assert "focusa_work_rail_interaction_v1" in OPENAPI
assert '"side_effect_policy"' in OPENAPI
assert "augment_rows" in GENERATOR
assert 'registerCommand("focusa-rail"' in PI
assert "side_effect_policy: \"preview\"" in PI
assert "preview_token: preview.preview_token" in PI
assert "ctx.ui.confirm" in PI
assert "registerWorkRailInteractions(pi)" in INDEX

for action in (
    "open or focus related Work Surface",
    "open Workpoint",
    "open provider item",
    "inspect evidence",
    "inspect change artifact",
    "inspect Receipt",
    "steer an explicit active attachment",
    "defer",
    "request approval",
    "reopen",
    "inspect history",
    "copy stable reference",
):
    assert action in SPEC, action

for label in (
    "Open or focus related Work Surface",
    "Open Workpoint",
    "Open provider item",
    "Inspect evidence",
    "Inspect change artifact",
    "Inspect Receipt",
    "Steer active attachment",
    "Defer",
    "Request approval",
    "Reopen",
    "Inspect history",
    "Copy stable reference",
):
    assert label in PI, label

print("Spec 135 A3 Work Rail preview/commit interactions: PASS")
