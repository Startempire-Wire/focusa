#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE="$ROOT/crates/focusa-core/src/silent_sessions/state_machine.rs"
TYPES="$ROOT/crates/focusa-core/src/silent_sessions/types.rs"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

test -f "$STATE" || fail "state machine module missing"

for state in Draft Validating Queued Launching Initializing Running WaitingInput Blocked Pausing Paused Resuming Recovering Orphaned Completing Completed Failed Cancelling Cancelled; do
  rg -n "${state}" "$TYPES" "$STATE" >/dev/null || fail "missing lifecycle state: $state"
done
pass "all Spec133 §10 lifecycle states are present"

for axis in SilentSessionHealth SemanticActivity reduce_health reduce_activity; do
  rg -n "$axis" "$TYPES" "$STATE" >/dev/null || fail "missing orthogonal state axis: $axis"
done
pass "health and semantic activity remain orthogonal"

for marker in WaitingInputUnproven BlockerUnproven CompletionUnproven HarnessExplicit AdapterHeuristic ConfidenceBasisPoints fresh_until; do
  rg -n "$marker" "$STATE" >/dev/null || fail "missing truthful-state guard: $marker"
done
pass "waiting, blocker, completion, provenance, confidence and freshness guards exist"

if rg -n 'std::fs|tokio::|rusqlite|reqwest|std::process::Command|sleep\(|spawn\(' "$STATE" >/dev/null; then
  fail "pure reducer contains I/O, process, persistence, transport, timer or retry behavior"
fi
pass "reducer remains facts-only and side-effect free"
