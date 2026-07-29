#!/usr/bin/env python3
"""Spec 135K-3 adaptive UI proposal/promotion/rollback proof."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
C=json.loads((ROOT/"docs/contracts/spec135-adaptive-ui-governance.v1.json").read_text())
assert C["candidate"]["advisory_only"] is True
assert C["promotion"]["self_promotion"] is False
assert C["promotion"]["operator_approval_required"] is True
assert C["promotion"]["rule"] == "eval_score > baseline_score AND eval_score >= score_threshold"
assert C["promotion"]["minimum_evaluation_window"] >= 30
assert C["promotion"]["authority_panels_hideable"] is False
assert C["rollback"]["exact_snapshot_required"] is True
assert C["rollback"]["idempotent"] is True
assert C["rollback"]["mutates_canonical_project_state"] is False
assert C["rollback"]["receipt_required"] is True
assert "any authority/proof/safety regression blocks promotion" == C["evaluation"]["harm_gate"]
assert "No UI adaptation self-promotes" in C["laws"]
print("Spec 135 K3 adaptive UI proposals/promotion/rollback: PASS")
