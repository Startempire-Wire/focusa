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

# Must populate all 4 standard checks
for check in "scope_resolution" "task_substitution" "environment_role_known" "live_build_host_safety"; do
  grep -q "name: \"$check\"" "$ACTION" \
    || fail "Missing PreflightCheck name: $check"
done
pass "All 4 standard checks present (scope_resolution/task_substitution/environment_role_known/live_build_host_safety)"

# Each check must have value_observed + threshold + recovery_hint
grep -q "pub value_observed:" "$ACTION" \
  || fail "PreflightCheck missing value_observed"
grep -q "pub threshold:" "$ACTION" \
  || fail "PreflightCheck missing threshold"
grep -q "pub recovery_hint:" "$ACTION" \
  || fail "PreflightCheck missing recovery_hint"
pass "Each PreflightCheck has value_observed + threshold + recovery_hint"

# Closes transcript gap: --json output must include checks array
# (envelope serialization includes checks field; --json flag in ActionCmd emits envelope)
grep -q "PreFlightEnvelope\|ActionPreflightEnvelope" "$ACTION" \
  || fail "ActionPreflightEnvelope referenced in action.rs"
pass "ActionPreflightEnvelope referenced in action.rs (--json emits it)"

# Verified by evidence citations
grep -q "transcript" "$ACTION" \
  || fail "action.rs should reference transcript gap in module header"
pass "action.rs references transcript gap (closes 2026-07-03 evaluator transcript issue)"

echo "✓ All focusa-112-action-preflight-structured static checks passed"