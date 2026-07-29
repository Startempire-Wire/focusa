#!/usr/bin/env python3
"""Spec 135B-4 durable Grill Interview and closure compendium proof."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md").read_text()
TYPES = (ROOT / "crates/focusa-core/src/types.rs").read_text()
ROUTE = (ROOT / "crates/focusa-api/src/routes/interview_sessions.rs").read_text()
STRATEGY = (ROOT / "crates/focusa-api/src/routes/interview_strategy.rs").read_text()
PI = (ROOT / "apps/pi-extension/src/interview-composer.ts").read_text()
INDEX = (ROOT / "apps/pi-extension/src/index.ts").read_text()
E2E = (ROOT / "tests/spec135_interview_session_e2e_test.py").read_text()
OPENAPI = (ROOT / "docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json").read_text()
GENERATOR = (ROOT / "scripts/generate-spec135-interview-contracts.py").read_text()

for token in (
    "ProjectInterviewSessionRecord",
    "ProjectInterviewQuestionRecord",
    "ProjectInterviewAnswerRecord",
    "ProjectInterviewBranchRecord",
    "supersedes",
    "operator_id",
    "confidence",
):
    assert token in TYPES, token

for token in (
    '"focusa.interview_closure_package.v1"',
    '"answer_provenance"',
    '"compendium"',
    '"receipt_ref"',
    "ProjectInterviewSessionStatus::ReadyForSpec",
):
    assert token in ROUTE, token

for token in (
    "triggering_gap",
    "contradiction_refs",
    "linked_context_refs",
    "stop_condition",
):
    assert token in STRATEGY + ROUTE + TYPES, token

for action in (
    "Continue Interview",
    "Add Context",
    "Revisit Answer",
    "Ask About New Context",
    "Resolve Contradiction",
    "Pause Interview",
    "Close and Build Compendium",
):
    assert action in PI, action
assert 'registerCommand("focusa-interview"' in PI
assert "supersedes: prior.answer_id" in PI
assert "registerInterviewComposer(pi)" in INDEX
assert 'registerMessageRenderer("focusa-interview-session"' in INDEX
assert 'package["compendium"]' in E2E
assert 'entry.get("answer_provenance"' in E2E
assert "focusa_interview_answer_provenance_v1" in OPENAPI
assert "focusa_interview_compendium_entry_v1" in OPENAPI
assert "compendium_entry" in GENERATOR

for requirement in (
    "no unresolved blocker information gap remains",
    "Continue Interview",
    "Revisit Answer",
    "Resolve Contradiction",
    "AI-generated summaries never replace the operator answer",
):
    assert requirement in SPEC, requirement

print("Spec 135 B4 Grill Interview/compendium: PASS")
