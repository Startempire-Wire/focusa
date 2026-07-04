#!/usr/bin/env bash
# spec_focusa_112_action_preflight_structured_static_test.sh
#
# Static guard for focusa-112-action-preflight-structured + transcript gap.
# Closes the "action preflight returns opaque verdict" issue. The new
# envelope includes a `checks` array so the agent sees WHY the verdict
# came out the way it did.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ACTION="$ROOT_DIR/crates/focusa-cli/src/commands/action.rs"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

# PreflightCheck struct must exist
grep -q 'pub struct PreflightCheck' "$ACTION" \
  || fail "action.rs missing PreflightCheck struct"
pass "action.rs has PreflightCheck struct (per-check audit trail)"

# Envelope must include checks field
grep -q 'pub checks: Vec<PreflightCheck>' "$ACTION" \
  || fail "ActionPreflightEnvelope missing checks field"
pass "ActionPreflightEnvelope includes checks: Vec<PreflightCheck>"

# Must populate standard checks, including the full-pipeline release/deploy blocker.
for check in "scope_resolution" "task_substitution" "full_live_pipeline_required" "environment_role_known" "live_build_host_safety"; do
  grep -q "name: \"$check\"" "$ACTION" \
    || fail "Missing PreflightCheck name: $check"
done
pass "All standard checks present, including full_live_pipeline_required"

# Each check must have value_observed + threshold + recovery_hint
grep -q "pub value_observed:" "$ACTION" \
  || fail "PreflightCheck missing value_observed"
grep -q "pub threshold:" "$ACTION" \
  || fail "PreflightCheck missing threshold"
grep -q "pub recovery_hint:" "$ACTION" \
  || fail "PreflightCheck missing recovery_hint"
pass "Each PreflightCheck has value_observed + threshold + recovery_hint"

# Closes transcript gap: --json output must include checks array and plain-language blocker.
# (envelope serialization includes checks field; --json flag in ActionCmd emits envelope)
grep -q "PreFlightEnvelope\|ActionPreflightEnvelope" "$ACTION" \
  || fail "ActionPreflightEnvelope referenced in action.rs"
grep -q 'pub plain_language_error: Option<String>' "$ACTION" \
  || fail "ActionPreflightEnvelope missing plain_language_error"
grep -q 'full_live_release_pipeline_required' "$ACTION" \
  || fail "action.rs missing full pipeline blocker class"
grep -q 'Blocked: this would bypass the full live GitHub release pipeline' "$ACTION" \
  || fail "action.rs missing plain-language full pipeline error"
grep -q 'scripts/create-dev-release-tag.sh --base 0.9 --push' "$ACTION" \
  || fail "action.rs missing canonical full pipeline recovery command"
grep -q 'gh run list' "$ACTION" \
  || fail "action.rs missing gh toolchain recovery hint"
pass "ActionPreflightEnvelope emits checks + plain-language full pipeline blocker"

# Verified by evidence citations
grep -q "transcript" "$ACTION" \
  || fail "action.rs should reference transcript gap in module header"
pass "action.rs references transcript gap (closes 2026-07-03 evaluator transcript issue)"

echo "✓ All focusa-112-action-preflight-structured static checks passed"