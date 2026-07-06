#!/usr/bin/env bash
# Spec 117 §15.2 — RecallDeckCard schema static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

RECALL="$ROOT_DIR/crates/focusa-tui/src/views/recall.rs"
MAIN="$ROOT_DIR/crates/focusa-tui/src/main.rs"
[[ -f "$RECALL" ]] || fail "recall.rs missing"

for needle in \
  'pub struct RecallDeckCard' \
  'pub enum MemoryStatus' \
  'pub enum ScopeStatus' \
  'pub enum ProofStatus' \
  'pub enum AllowedUse' \
  'result_id' \
  'provider' \
  'source_session_id' \
  'project_root' \
  'continuity_id' \
  'timestamp' \
  'span_type' \
  'memory_status' \
  'scope_status' \
  'proof_status' \
  'allowed_use' \
  'safe_excerpt' \
  'evidence_refs' \
  'next_action'; do
  grep -qF -- "$needle" "$RECALL" || fail "RecallDeckCard schema missing: $needle"
done
pass "RecallDeckCard schema fields are present"

for needle in \
  'MEMORY_STATUS_VALUES' \
  'active' \
  'stale' \
  'superseded' \
  'contradicted' \
  'noise' \
  'quarantined' \
  'SCOPE_STATUS_VALUES' \
  'same_project_other_continuity' \
  'global_advisory' \
  'PROOF_STATUS_VALUES' \
  'ALLOWED_USE_VALUES' \
  'verify_first'; do
  grep -qF -- "$needle" "$RECALL" || fail "RecallDeckCard enum/value missing: $needle"
done
pass "RecallDeckCard status/allowed-use values match Spec 117"

grep -qF 'memory_status_values' "$MAIN" || fail "headless proof missing memory_status_values"
grep -qF 'allowed_use_values' "$MAIN" || fail "headless proof missing allowed_use_values"
pass "RecallDeckCard schema exposed in headless proof"

echo "focusa-117 recall-card-schema static test: PASS"
