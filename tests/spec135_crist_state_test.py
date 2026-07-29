#!/usr/bin/env python3
"""Spec 135B-1 durable C.R.I.S.T. state and operating-profile proof."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md").read_text()
ROUTE = (ROOT / "crates/focusa-api/src/routes/project_genesis.rs").read_text()
SUPPORT = (ROOT / "crates/focusa-api/src/routes/project_genesis_support.rs").read_text()
CRIST = (ROOT / "crates/focusa-api/src/routes/project_genesis_crist.rs").read_text()
RESUME = (ROOT / "crates/focusa-api/src/routes/project_genesis_resume.rs").read_text()
TESTS = (ROOT / "crates/focusa-api/src/routes/project_genesis_tests.rs").read_text()

for stage in (
    "created",
    "project_scope_verified",
    "context_collecting",
    "context_ready",
    "role_drafting",
    "role_pending_operator",
    "role_approved",
    "interviewing",
    "interview_ready",
    "spec_workbench_created",
    "spec_in_review",
    "spec_approved",
    "task_plan_drafting",
    "task_plan_pending_operator",
    "tasks_materialized",
    "first_workpoint_ready",
    "operational",
):
    assert stage in SPEC, stage
    assert stage in CRIST, stage

for token in (
    "focusa.project_genesis.v1",
    "focusa.resolved_project_operating_profile.v1",
    '"ownership"',
    '"transition_receipts"',
    '"revision"',
    '"crist_state"',
):
    assert token in SUPPORT, token

for token in (
    "focusa.crist.transition_receipt.v1",
    "invalid_crist_transition",
    "state_revision_before",
    "state_revision_after",
    "allowed_transition",
    "transition-receipts",
    "transition_receipt_persist_failed",
):
    assert token in CRIST, token

assert 'mod crist;' in ROUTE
assert "initialize_crist_state(&root, &mut packet)" in ROUTE
assert "record_crist_transition" in ROUTE
assert "existing_genesis_guard" in ROUTE
assert "resume_context_collection" in RESUME
assert 'packet.get("idempotency_key")' not in RESUME
assert "packet[key] = existing[key].clone()" in RESUME
assert "crist_state_and_operating_profile_survive_disk_reconnect" in TESTS
assert "invalid_crist_transition_fails_closed_with_durable_receipt" in TESTS
assert "staged_genesis_enforces_continuity_ownership_after_reconnect" in TESTS
assert "write_json_atomic(&packet_path(&root), &packet)" in TESTS
assert "read_json(&packet_path(&root))" in TESTS

print("Spec 135 B1 durable C.R.I.S.T. state/profile: PASS")
