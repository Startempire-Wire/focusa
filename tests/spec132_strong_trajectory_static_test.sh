#!/usr/bin/env bash
# Trajectory audit phase 3 strong-trajectory spec guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/132-focusa-strong-trajectory-spec.md"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[ -f "$DOC" ] || fail "strong trajectory spec missing"

for section in \
  'Definition' \
  'Good Trajectory shape' \
  'Invariants' \
  'Clarity gate policy' \
  'What Trajectory must never let an agent do' \
  'Clear opinion examples' \
  'Implementation target for phase 4'; do
  grep -q "$section" "$DOC" || fail "missing strong spec section: $section"
done

for invariant in \
  'No gap without a next call' \
  'No done without proof' \
  'No vague goal' \
  'No hidden scope' \
  'No execution leap' \
  'No placeholder authority' \
  'No Workpoint laundering'; do
  grep -q "$invariant" "$DOC" || fail "missing invariant: $invariant"
done

for field in \
  'gap_description' \
  'next_tool' \
  'next_command' \
  'proof_needed' \
  'workpoint_relation' \
  'vague_or_unverifiable_trajectory_goal'; do
  grep -q "$field" "$DOC" || fail "missing target field/failure: $field"
done

pass "Strong Trajectory spec static guard passed"
