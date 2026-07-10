#!/usr/bin/env bash
# Trajectory audit phase 2 flailing-pattern guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/131-focusa-trajectory-audit-flailing-patterns.md"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[ -f "$DOC" ] || fail "trajectory flailing-pattern doc missing"

for section in \
  'Placeholder or non-actionable' \
  'Vague or non-actionable gap descriptions' \
  'Clarity gate blocks too often or too rarely' \
  'define_goal validation weaknesses' \
  'assess output weaknesses' \
  'propose_workpoint wrong-next-step risks' \
  'Field-shuttling without strong opinion' \
  'Strongest existing parts to preserve' \
  'Phase 3 design implications'; do
  grep -q "$section" "$DOC" || fail "missing phase-2 section: $section"
done

for pattern in \
  'learning_refs' \
  'prediction_refs' \
  'stale_refs' \
  'tool_affordances' \
  'Current verified state differs from desired end state' \
  'Trajectory gap unclear until desired end state and current verified state are both present' \
  'stale_or_missing_evidence_refs' \
  'next_workpoint' \
  'basic validation only checks non-empty' \
  'observed_state can be absent' \
  'Proposal mission is active gap'; do
  grep -q "$pattern" "$DOC" || fail "missing identified weak pattern: $pattern"
done

for implication in \
  'Every gap has `gap_description`, `next_tool`, `next_command`, and `proof_needed`' \
  'rejects vague/circular/unverifiable goals' \
  'requires concrete verb + target + proof hook'; do
  grep -q "$implication" "$DOC" || fail "missing phase-3 implication: $implication"
done

pass "Trajectory flailing-pattern static guard passed"
