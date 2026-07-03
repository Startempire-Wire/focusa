#!/usr/bin/env bash
# spec_focusa_112_doctor_scope_modes_static_test.sh
#
# Static guard for focusa-112-doctor-scope-modes + transcript gap.
# Closes the "Doctor too rigid for non-focusa hosts" issue. The previous
# single-mode doctor would run project-shape + repo-shape checks on any
# host, which broke for non-focusa hosts like the Cursor transcript's
# Next.js trapnstudio project. New --scope=host|project|repo modes let
# the operator pick the right level.
#
# NO COMPROMISE on scope enforcement: each mode RUNS at least the
# daemon + lifecycle + scope-safety checks. Weaker checks move to
# --scope=project or --scope=repo, but they still run when requested.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOCTOR="$ROOT_DIR/crates/focusa-cli/src/commands/doctor.rs"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

# DoctorArgs must have --scope flag
grep -q 'pub scope: DoctorScope' "$DOCTOR" \
  || fail "DoctorArgs missing --scope field"
pass "DoctorArgs exposes --scope field"

# DoctorScope enum has 3 variants
for v in Host Project Repo; do
  grep -q "$v," "$DOCTOR" \
    || fail "DoctorScope enum missing variant: $v"
done
pass "DoctorScope enum has Host/Project/Repo (3 modes)"

# Default = host (most permissive for non-focusa hosts)
grep -q 'default_value = "host"' "$DOCTOR" \
  || fail "DoctorArgs.scope must default to 'host' (permissive for non-focusa hosts)"
grep -q 'impl Default for DoctorScope' "$DOCTOR" \
  || fail "DoctorScope must implement Default for DoctorArgs::default()"
grep -q 'Self::Host' "$DOCTOR" \
  || fail "DoctorScope::default() must be Host"
pass "DoctorArgs.scope default is 'host'/Host (backward-compatible default)"

# DoctorScope has serde rename_all = lowercase (for --json output)
grep -q 'serde(rename_all = "lowercase")' "$DOCTOR" \
  || fail "DoctorScope missing #[serde(rename_all = lowercase)] for --json output"
pass "DoctorScope serializes lowercase in --json output"

# Conditionally run project-shape checks only on Project/Repo scope
# (Transcript gap: the Next.js trapnstudio project broke because the
# doctor ran Spec90/Spec91/Pi-skills/Mac-app checks which don't exist in
# non-focusa repos.)
grep -q 'matches!(args.scope, DoctorScope::Project | DoctorScope::Repo)' "$DOCTOR" \
  || fail "Project-shape checks not gated on --scope=project|repo"
pass "Project-shape checks gated on --scope=project|repo (default = host skips them)"

# Conditionally run repo-shape checks only on Repo scope
grep -q 'matches!(args.scope, DoctorScope::Repo)' "$DOCTOR" \
  || fail "Repo-shape checks (Guardian) not gated on --scope=repo"
pass "Repo-shape checks (Guardian) gated on --scope=repo (project scope skips them)"

# JSON envelope includes scope field (additive, backward compatible)
grep -q '"scope": args.scope' "$DOCTOR" \
  || fail "Doctor response envelope missing 'scope' field"
pass "Doctor --json envelope includes scope field (additive, backward compat)"

# Transcript gap: doctor does NOT generate dummy files (Cursor evaluator
# transcript 2026-07-03 generated target/release/focusa-daemon and
# scripts/validate-focusa-tool-contracts.mjs to satisfy doctor). Doctor
# remains read-only: it may check path existence, but must not write/create
# placeholder project files.
if grep -qE 'fs::write|std::fs::write|File::create|create_dir_all' "$DOCTOR"; then
  fail "doctor.rs contains file/directory creation; doctor must remain read-only"
fi
pass "doctor is read-only and cannot generate dummy project files"

# Scope enforcement is not weakened: host still runs daemon/workpoint checks;
# project/repo checks are only shifted to explicit modes, not removed.
grep -q '"Workpoint canonicality"' "$DOCTOR" \
  || fail "Host-scope doctor must still run Workpoint canonicality check"
grep -q '"Spec90 contract validator"' "$DOCTOR" \
  || fail "Project-scope doctor must still retain Spec90 contract validator check"
pass "scope enforcement retained: host checks authority, project scope keeps Spec90 guard"

echo "✓ All focusa-112-doctor-scope-modes static checks passed"