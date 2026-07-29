#!/usr/bin/env python3
"""Spec 135K-2 UXP/UFI friction capture proof."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
C=json.loads((ROOT/"docs/contracts/spec135-friction-capture-evaluation.v1.json").read_text())
T=(ROOT/"crates/focusa-core/src/types.rs").read_text()
assert C["record"]["raw_input_stored"] is False
assert C["record"]["secret_fields_stored"] is False
assert C["consent"]["default"] is False
assert C["consent"]["explicit_required"] is True
assert C["authority"]["canonical_state_owner"] is False
assert all(C["authority"][k] for k in ("cannot_change_workpoint","cannot_change_permission","cannot_promote_ontology"))
assert set(C["record"]["cohorts"]) == {"canvas-guided","terminal-guided","headless"}
assert C["evaluation"]["separate_cohorts"] is True
assert C["evaluation"]["minimum_window"] >= 30
assert C["evaluation"]["learning_rate_max"] <= 0.1
for token in ("pub struct UxpProfile","pub struct UfiState","learning_rate","window_size","citations"):
    assert token in T, token
assert "Friction observations remain advisory telemetry" in C["laws"]
print("Spec 135 K2 UXP/UFI friction capture and evaluation: PASS")
