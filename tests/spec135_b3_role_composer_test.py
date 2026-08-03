#!/usr/bin/env python3
"""Spec 135B-3 grounded Role Composer, alternatives, approval, and Pi UI proof."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md").read_text()
TYPES = (ROOT / "crates/focusa-core/src/types.rs").read_text()
ROUTE = (ROOT / "crates/focusa-api/src/routes/role_profiles.rs").read_text()
PI = (ROOT / "apps/pi-extension/src/role-composer.ts").read_text()
INDEX = (ROOT / "apps/pi-extension/src/index.ts").read_text()
TEST = (ROOT / "tests/spec135_role_profile_e2e_test.py").read_text()
OPENAPI = (ROOT / "docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json").read_text()
GENERATOR = (ROOT / "scripts/generate-spec135-role-contracts.py").read_text()

for token in (
    "RoleAlternativeRecord",
    "alternatives: Vec<RoleAlternativeRecord>",
    "RoleProfileGrounding",
    "RoleReviewRecord",
    "grants_permissions",
    "permission_profile_refs",
    "RoleProfileStatus",
):
    assert token in TYPES, token

for token in (
    "RoleAlternativeInput",
    "role alternatives may cite only grounding refs accepted by the draft",
    "responsibility cannot grant",
    '"project-role-profile"',
    '"role-alternative"',
):
    assert token in ROUTE, token

for token in (
    'registerCommand("focusa-role"',
    "Create grounded draft",
    "Inspect latest profile",
    "Review latest profile",
    "Operator decision",
    "Context grounding",
    "Role responsibility implies authority",
    "permission_assertions: []",
):
    assert token in PI, token

assert "registerRoleComposer(pi)" in INDEX
assert 'registerMessageRenderer("focusa-role-profile"' in INDEX
assert 'profile1["alternatives"]' in TEST
assert 'profile3["alternatives"]' in TEST
assert "focusa_role_alternative_v1" in OPENAPI
assert '"alternatives"' in OPENAPI
assert "alternative_schema" in GENERATOR and "augment_profiles" in GENERATOR

for requirement in (
    "Role title",
    "Forbidden assumptions",
    "operator approval",
    "before/after redline",
    "revision history",
    "It does not define permission or authority",
):
    assert requirement in SPEC, requirement

print("Spec 135 B3 Role Composer: PASS")
