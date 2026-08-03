#!/usr/bin/env python3
"""Spec 135K-4 usability/accessibility/headless proof."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
C=json.loads((ROOT/"docs/contracts/spec135-k4-usability-headless-proof.v1.json").read_text())
ids={s["scenario_id"] for s in C["scenarios"]}
assert ids == {"canvas_guided","terminal_guided","headless","accessibility","live_toggle","reconnect","compaction","model_switch","project_reopen"}
for scenario in C["scenarios"]:
    assert scenario["status"] == "passed", scenario
    assert scenario["canonical_state_ref"] == "focusa:canonical-state:unchanged"
    assert (ROOT/scenario["proof_ref"]).exists()
for receipt in C["receipts"]: assert (ROOT/receipt).exists(), receipt
assert "Canvas, terminal, and headless modes share canonical state" in C["invariants"]
assert "Headless mode performs no UI calls and emits no nags" in C["invariants"]
print("Spec 135 K4 usability, accessibility, and headless proof: PASS")
