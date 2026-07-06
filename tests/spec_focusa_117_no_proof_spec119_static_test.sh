#!/usr/bin/env bash
# Spec 117 .33 — No Proof, No Done walkthrough aligned with Spec 119 §7.8 proof-before-completion.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

WT="$ROOT_DIR/crates/focusa-cli/src/commands/walkthrough.rs"
[[ -f "$WT" ]] || fail "walkthrough.rs missing"
for needle in \
  'proof_precedes_completion: bool' \
  'default_proof_precedes_completion' \
  'proof_precedes_completion: true' \
  'no_proof_no_done_enforces_proof_precedes_completion' \
  'EVIDENCE_ACTUAL' \
  'EVIDENCE_PARTIAL' \
  'EVIDENCE_SURROGATE' \
  'EVIDENCE_BLOCKED' \
  'EVIDENCE_MISSING'; do
  grep -qF -- "$needle" "$WT" || fail "walkthrough missing: $needle"
done
pass "walkthrough enforces proof_precedes_completion invariant + Spec 119 §7.8 vocabulary"
echo "focusa-117 no-proof-spec119 static test: PASS"
