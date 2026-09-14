#!/usr/bin/env python3
"""Locked-release gate for non-blocking Pi startup project binding."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORE = (ROOT / "crates/focusa-core/src/working_subpath.rs").read_text()
API = (ROOT / "crates/focusa-api/src/routes/project.rs").read_text()
SESSION = (ROOT / "apps/pi-extension/src/session.ts").read_text()
STATE = (ROOT / "apps/pi-extension/src/state.ts").read_text()
TOOLS = (ROOT / "apps/pi-extension/src/tools.ts").read_text()
BINDING = (ROOT / "apps/pi-extension/src/project-binding.ts").read_text()
MAC_E2E = (ROOT / "apps/pi-extension/tests/project-binding-macos-e2e.mjs").read_text()

for token in [
    '"DISCOVER"',
    '"RECONCILE"',
    '"VERIFY"',
    '"BOUND"',
    '"RECOVERING"',
    '"QUARANTINED"',
    "focusa.project_binding_decision.v1",
    "selected_project_root",
    "selected_worktree_root",
    "continuity_id",
    "evidence_revision",
    "scope_safety_policy_version",
    "permitted_capability_tier",
    "supersedes_decision_id",
]:
    assert token in BINDING, f"missing typed binding contract token: {token}"

assert "ProjectBindingDecision" in CORE
assert "repo_fingerprint" in CORE
assert "project_fingerprint" in CORE
assert "macos_volumes_user_home_is_never_promoted_as_project" in CORE
assert "binding_candidates_canonicalize_temp_space_unicode_and_symlink_roots" in CORE
assert "decision = decision.mark_verified()" in API

verify_start = SESSION.index("async function promptForProjectVerifyIfNeeded")
verify_end = SESSION.index("async function promptForWorkpointIfNeeded", verify_start)
verify = SESSION[verify_start:verify_end]
assert "ctx.ui.confirm" not in verify
assert "shouldEmitProjectScopeRecoveryPacket" in verify
assert "Conversation and diagnosis continue" in verify
assert 'decision.state === "BOUND"' in verify

assert "persistedBindingDecision" in SESSION
assert "canReuseFreshVerifiedBindingOffline" in SESSION
assert "persisted_binding_conflicts_with_current_repo" in SESSION
assert "Candidate selection is deferred until a project-aware mutation is requested" in SESSION
assert "projectBindingDecisions" in STATE
assert "projectBindingTelemetry" in STATE
assert "projectBindingAllowsDurableWrites(sessionBindingDecision)" in STATE

fetch_start = TOOLS.index("async function focusaFetchDetailed")
fetch_end = TOOLS.index("function formatWorkLoopBudgetRemaining", fetch_start)
fetch = TOOLS[fetch_start:fetch_end]
assert 'failure_class: "scope_recovery_required"' in fetch
assert "operator_selection_required: firstMutationSelection" in fetch
assert "duplicate_selection_suppressed" in fetch
assert 'path.startsWith("/project/verify")' in fetch
assert "projectBindingAllowsDurableWrites(bindingDecision)" in fetch

assert "operatorConfirmed = false" in SESSION
assert SESSION.index("if (!operatorConfirmed)") < SESSION.index('focusaFetch("/project/genesis/start"')
for token in [
    "direct macOS user home is quarantined and never promoted",
    "verified marked macOS project binds without a verification modal",
    "same-project worktree fingerprint permits bounded fresh offline reuse",
    "stale different-repo evidence cannot reuse authority",
]:
    assert token in MAC_E2E, f"missing macOS E2E case: {token}"
print("Spec143 project binding decision release gate: PASS")
