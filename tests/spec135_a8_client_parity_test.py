#!/usr/bin/env python3
"""Spec 135A-8 client parity + permanent evidence proof."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md").read_text()
CONTRACT = ROOT / "docs/contracts/spec135-client-operation-parity.v1.json"
REGISTRY = json.loads(
    (ROOT / "docs/contracts/spec135/generated-contract-v1/operation-registry.json").read_text()
)
assert CONTRACT.exists(), "run scripts/generate-spec135-client-parity-matrix.py"
p = json.loads(CONTRACT.read_text())

assert p["schema"] == "focusa.spec135.client_operation_parity.v1"
assert "OpenAPI 3.0.3" in p["canonical_contracts"]
assert "canonical state of its own" in p["parity_invariant"]

ops = [o["operation_id"] for o in REGISTRY["operations"]]
rows = p["rows"]
assert len(rows) == len(ops) * len(p["clients"]), len(rows)
assert set(p["operations"]) == set(ops)
assert set(p["operations"]) == set(ops), "operation set drift"

traits = {row["capability_trait"] for row in rows}
assert traits == {"full", "preview", "read_only", "unsupported"}, traits

# Canonical-state law: no client owns canonical state; only reducers do.
assert all(row["canonical_state_owner"] is False for row in rows)

# The daemon HTTP route is the sole full-canonical source for every operation.
api_rows = [r for r in rows if r["client_id"] == "api"]
assert len(api_rows) == len(ops)
assert all(r["capability_trait"] == "full" for r in api_rows)
assert all("canonical HTTP source" in r["capability_limit"] for r in api_rows)

# Side effects on presentation clients must be preview-gated (never "full").
PRESENTATION_KINDS = {
    "pi_extension",
    "pi_widget",
    "ratatui_view",
    "operator_peek",
    "mission_deck",
    "ui_action_bridge",
}
for row in rows:
    if row["capability_trait"] == "full" and row["client_kind"] in PRESENTATION_KINDS:
        raise AssertionError(f"presentation client with full trait: {row}")
    if row["operation_mode"] == "write" and row["client_kind"] in PRESENTATION_KINDS:
        assert row["capability_trait"] in {"preview", "unsupported"}, row

# Dogfood receipts are durable and reference real proof fixtures.
receipts = p["dogfood_receipts"]
assert len(receipts) >= 10, len(receipts)
for receipt in receipts:
    assert receipt["status"] == "passed", receipt
    assert (ROOT / receipt["receipt_ref"]).exists(), receipt["receipt_ref"]

for requirement in (
    "All clients consume shared contracts",
    "stock Pi compatibility widgets/drawer and session switcher",
    "enhanced Pi Mission Canvas docks/sidebar",
    "Mission Deck PWA",
    "shared contracts",
):
    assert requirement in SPEC, requirement

print("Spec 135 A8 client parity and permanent evidence: PASS")