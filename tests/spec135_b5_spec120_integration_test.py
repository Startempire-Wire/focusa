#!/usr/bin/env python3
"""Spec 135B-5 C.R.I.S.T. handoff and governed Spec 120 static proof."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md").read_text()
TYPES = (ROOT / "crates/focusa-core/src/types.rs").read_text()
ROUTE = (ROOT / "crates/focusa-api/src/routes/spec_workbench.rs").read_text()
E2E = (ROOT / "tests/spec135_b5_spec120_integration_e2e_test.py").read_text()

for token in (
    "CristSpecHandoff",
    "focusa.crist_spec_handoff.v1",
    "active_domain_pack_refs",
    "context_pack_refs",
    "accepted_project_claim_refs",
    "role_profile_ref",
    "interview_session_refs",
    "known_contradictions",
    "desired_spec_template",
    "reality_classification",
):
    assert token in TYPES + ROUTE, token

for classification in (
    "implemented",
    "partial",
    "docs_only",
    "normative_target",
    "planned",
    "speculative",
    "stale",
    "blocked",
    "unknown",
):
    assert f'"{classification}"' in ROUTE, classification

assert "PROJECT_GENESIS_SECTIONS: [&str; 22]" in ROUTE
assert "project_genesis final approval requires all 22 sections" in ROUTE
assert "source-linked Context, an approved Role Profile, and a closed Interview" in ROUTE
assert "len(final[\"sections\"]) == 22" in E2E
assert 'blocked["failure_class"] == "approval_required"' in E2E

for requirement in (
    "focusa.crist_spec_handoff.v1",
    "Every section uses Spec 120 grounding",
    "Docs-only behavior cannot be presented as runtime behavior",
    "Final approval record",
    "Trajectory and Workpoint promotion remain governed",
):
    assert requirement in SPEC, requirement

print("Spec 135 B5 governed Spec 120 integration: PASS")
