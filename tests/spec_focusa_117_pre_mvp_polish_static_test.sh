#!/usr/bin/env bash
# Spec 117 launch blocker — Final pre-MVP polish across every layer static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

DOC="$ROOT_DIR/docs/PRE_MVP_LAUNCH_READINESS_2026-07-06.md"
[[ -f "$DOC" ]] || fail "pre-MVP launch readiness doc missing"

for needle in \
  'Pre-MVP Launch Readiness' \
  'Layer-by-layer status' \
  'Rust workspace build' \
  'focusa-tui unit tests' \
  'focusa-cli walkthrough tests' \
  'Static aggregate' \
  'Mission Deck headless proof' \
  'Daemon health' \
  'GitHub CI' \
  'Spec 101 Bloatgaurd §5.11' \
  'Spec 100 Context Cognition' \
  'focusa-117-arch.17' \
  'focusa-117-arch.18' \
  'focusa-117-arch.19' \
  'focusa-117-arch.20' \
  'focusa-117-arch.29' \
  'focusa-29ew.1' \
  'focusa-29ew.6' \
  'Sign-off requirements'; do
  grep -qF -- "$needle" "$DOC" || fail "pre-MVP doc missing: $needle"
done
pass "pre-MVP launch readiness doc covers every layer and open blockers"
echo "focusa-117 pre-mvp-polish static test: PASS"
