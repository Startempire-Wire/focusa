#!/usr/bin/env python3
"""Spec 135B-7 Genesis-to-first-Workpoint and no-fork resume proof."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ROUTE = (ROOT / "crates/focusa-api/src/routes/project_genesis.rs").read_text()
RESUME = (ROOT / "crates/focusa-api/src/routes/project_genesis_resume.rs").read_text()
CRIST = (ROOT / "crates/focusa-api/src/routes/project_genesis_crist.rs").read_text()
SUPPORT = (ROOT / "crates/focusa-api/src/routes/project_genesis_support.rs").read_text()
E2E = (ROOT / "tests/spec135_b7_genesis_resume_e2e_test.py").read_text()
OPENAPI = (ROOT / "docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json").read_text()
GENERATOR = (ROOT / "scripts/generate-spec135-genesis-contracts.py").read_text()

for token in (
    "existing_continuity_id",
    "same_owner",
    '"genesis_id"',
    '"created_at"',
    '"transition_receipts"',
    "packet[key] = existing[key].clone()",
    '"resume_context_collection"',
    "record_crist_transition",
    "WorkpointCheckpointPromoted",
):
    assert token in ROUTE + RESUME, token

for token in (
    "focusa.crist.transition_receipt.v1",
    "attempt",
    "transition_receipt_persist_failed",
):
    assert token in CRIST, token

for token in (
    '"first_workpoint"',
    '"evidence_refs"',
    '"task_provider_and_task_graph"',
    "build_staged_packet",
):
    assert token in SUPPORT + ROUTE, token

for scenario in (
    'roots / "new"',
    'roots / "degraded"',
    'roots / "existing"',
    "assert_one_workpoint",
    "len({item[\"receipt_id\"]",
):
    assert scenario in E2E, scenario

assert "focusa_genesis_first_workpoint_v1" in OPENAPI
assert '"evidence_refs"' in OPENAPI
assert "focusa_genesis_first_workpoint_v1" in GENERATOR

print("Spec 135 B7 Genesis-to-first-Workpoint resume: PASS")
