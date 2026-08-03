#!/usr/bin/env python3
"""Validate P00 governance, writer isolation, and false-closure boundaries."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PLAN = json.loads((ROOT / "docs/contracts/spec135-writer-scope-plan.v1.json").read_text())
GRAPH = json.loads((ROOT / "docs/contracts/spec135-mission-canvas-completion-dag.v2.json").read_text())
BASELINE = json.loads((ROOT / "docs/contracts/spec135-p00-recovery-baseline.v1.json").read_text())
ACCEPTANCE = json.loads((ROOT / "docs/contracts/spec135-master-final-acceptance.v1.json").read_text())

assert PLAN["schema"] == "focusa.spec135.writer_scope_plan.v1"
assert PLAN["status"] == "active"
assert PLAN["writer_protocol"]["single_writer_per_file"] is True
assert PLAN["writer_protocol"]["single_writer_per_canonical_object"] is True
assert PLAN["writer_protocol"]["writer_lease_required_for_mutations"] is True
assert PLAN["writer_protocol"]["explicit_attachment_required"] is True
assert PLAN["writer_protocol"]["phase_gate_required_before_next_phase"] is True

lanes = PLAN["lanes"]
assert len(lanes) == 10
assert len({lane["lane_id"] for lane in lanes}) == len(lanes)
assert len({lane["worktree_slug"] for lane in lanes}) == len(lanes)
assert len({lane["attachment_id"] for lane in lanes}) == len(lanes)
assert all(lane["exclusive_paths"] for lane in lanes)
assert all(lane["shared_paths_requiring_integration_writer"] for lane in lanes)

assert GRAPH["status"] == "operator_approved_p00_execution"
assert GRAPH["operator_confirmations"]["completion_dag_approved_by_continue_steering"] is True
assert BASELINE["status"] == "p00_complete_p01_authority_ready"
assert BASELINE["scope"]["pull_request_draft"] is True
assert BASELINE["test_baseline"]["github_ci_run_30568434414"]["result"] == "passed"
assert ACCEPTANCE["status"] == "verified"
assert ACCEPTANCE["passed_count"] == 14
assert ACCEPTANCE["gate_count"] == 14
assert ACCEPTANCE["merge_ready"] is True

print("Spec 135 P00 governance: PASS (isolated writers, draft PR, verified closure truth)")
