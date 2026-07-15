#!/usr/bin/env bash
# spec_focusa_112_onboard_scoped_static_test.sh
#
# Static guard for focusa-112-onboard-scoped + transcript gap.
# Backward compatibility: default scope remains project, preserving existing
# project onboarding/demo Workpoint behavior.
# Scope enforcement: project scope rejects unsafe broad roots before any
# Workpoint checkpoint write; host scope is opt-in and does not bind project
# identity or create a Workpoint.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ONBOARD="$ROOT_DIR/crates/focusa-cli/src/commands/onboard.rs"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

# New additive scope flag, default project for backward compatibility
grep -q 'pub scope: OnboardScope' "$ONBOARD" \
  || fail "OnboardArgs missing scope field"
grep -q 'default_value = "project"' "$ONBOARD" \
  || fail "onboard --scope default must remain project for backward compatibility"
grep -q '#\[default\]' "$ONBOARD" \
  || fail "OnboardScope must mark Project as #[default]"
pass "onboard scope is additive; default remains project (backward compatible)"

# Scope enum must have only host/project for now
for v in Host Project; do
  grep -q "$v," "$ONBOARD" \
    || fail "OnboardScope missing variant: $v"
done
pass "OnboardScope supports Host and Project"

# Project scope must reject broad/unsafe roots before checkpoint write
grep -q 'fn safe_project_root' "$ONBOARD" \
  || fail "onboard.rs missing safe_project_root guard"
grep -q 'let trimmed = root.trim_end_matches' "$ONBOARD" \
  || fail "safe_project_root should trim trailing slash for broad-root guard"
grep -q 'match trimmed' "$ONBOARD" \
  || fail "safe_project_root should reject broad roots explicitly"
grep -q '"" | "/" | "/root" | "/home" | "/tmp" | "/var" | "/usr" | "/opt" => false' "$ONBOARD" \
  || fail "safe_project_root should reject broad roots explicitly"

grep -q 'project_scope && !safe_project_root' "$ONBOARD" \
  || fail "project-scope onboarding must bail before unsafe project writes"
pass "project-scope onboarding blocks unsafe broad roots before writes"

# Host scope must not bind project identity or create demo Workpoint
grep -q 'host-scope onboarding does not bind project identity' "$ONBOARD" \
  || fail "host scope should skip project identity binding"
grep -q 'if project_scope && !args.no_demo_workpoint && health_ok' "$ONBOARD" \
  || fail "demo Workpoint creation must be gated by project_scope"
pass "host scope skips project identity and demo Workpoint creation"

# JSON output includes additive scope field
grep -q '"scope": args.scope' "$ONBOARD" \
  || fail "onboard response missing additive scope field"
pass "onboard response includes additive scope field"

# Existing flags retained
grep -q 'pub project_root: Option<String>' "$ONBOARD" \
  || fail "existing --project-root flag removed"
grep -q 'pub continuity_id: Option<String>' "$ONBOARD" \
  || fail "existing --continuity-id flag removed"
grep -q 'pub no_demo_workpoint: bool' "$ONBOARD" \
  || fail "existing --no-demo-workpoint flag removed"
pass "existing onboarding flags retained"

# JSON/quiet/noninteractive behavior is now guarded in run() path
grep -q 'let quiet = std::env::args().any(|arg| arg == "--quiet")' "$ONBOARD" \
  || fail "onboard run() must detect --quiet"
grep -q 'FOCUSA_QUIET' "$ONBOARD" \
  || fail "onboard run() must support quiet via FOCUSA_QUIET env fallback"
grep -q 'let tty = std::io::stdin().is_terminal() && std::io::stdout().is_terminal()' "$ONBOARD" \
  || fail "onboard run() must gate scope picker on TTY"
grep -q 'if !json_mode && !quiet && tty' "$ONBOARD" \
  || fail "onboard banner must be suppressed in json/quiet mode"
grep -q 'let _scope_idx = if json_mode || quiet || !tty' "$ONBOARD" \
  || fail "onboard should disable scope picker for json/quiet/non-tty"
grep -q 'else if !quiet' "$ONBOARD" \
  || fail "onboard should suppress human output in --quiet mode"
pass "onboard json/quiet/noninteractive guards are present"

echo "✓ All focusa-112_onboard-scoped static checks passed"