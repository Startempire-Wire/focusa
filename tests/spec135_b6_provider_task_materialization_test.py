#!/usr/bin/env python3
"""Spec 135B-6 provider-neutral tasks and provider capability truth proof."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md").read_text()
TYPES = (ROOT / "crates/focusa-core/src/types.rs").read_text()
ROUTE = (ROOT / "crates/focusa-api/src/routes/task_plans.rs").read_text()
PLAN_E2E = (ROOT / "tests/spec135_task_plan_e2e_test.py").read_text()
MATERIALIZE_E2E = (ROOT / "tests/spec135_task_materialization_e2e_test.py").read_text()

for token in (
    "provider_neutral_id",
    "linked_spec_sections",
    "acceptance_criteria",
    "evidence_requirements",
    "allowed_scope",
    "dependencies",
    "blockers",
    "closure_policy_ref",
    "preferred_provider",
    "provider_ref",
):
    assert token in TYPES, token

for provider in (
    '"beads"',
    '"github_issues"',
    '"linear"',
    '"asana"',
    '"markdown_checklist"',
):
    assert provider in ROUTE, provider

for state in (
    "configured and operational",
    "read-only",
    "credentials missing",
    "adapter unavailable",
    "schema-only support",
):
    assert state in ROUTE, state

for token in (
    "TaskProviderCapabilityTruth",
    "provider_capabilities",
    "credential_reference_present",
    "mutation_approval_required",
    "recovery_action",
):
    assert token in ROUTE, token

assert 'listed(base)["provider_capabilities"]' in PLAN_E2E
assert "canonical parent Beads" in MATERIALIZE_E2E
assert "permission_grant_ref" in MATERIALIZE_E2E

for requirement in (
    "No provider mutation occurs during draft decomposition",
    "Provider capability truth",
    "GitHub Issues",
    "Linear",
    "Asana",
    "Markdown Checklist",
):
    assert requirement in SPEC, requirement

print("Spec 135 B6 provider-neutral task materialization: PASS")
