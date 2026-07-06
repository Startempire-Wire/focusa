#!/usr/bin/env bash
# Spec 117 §13.3 — No Proof, No Done walkthrough static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

WT="$ROOT_DIR/crates/focusa-cli/src/commands/walkthrough.rs"
[[ -f "$WT" ]] || fail "walkthrough.rs missing"

for needle in \
  "no-proof-no-done" \
  "pub fn no_proof_no_done" \
  "Display the agent completion claim" \
  "Check evidence refs" \
  "Show proof gap if missing" \
  "Attach proof or mark intentionally missing" \
  "Re-render proof meter" \
  "proof gap" \
  "proof meter" \
  "no_proof_no_done_round_trips"; do
  grep -qF -- "$needle" "$WT" || fail "no-proof-no-done walkthrough missing: $needle"
done
pass "no-proof-no-done walkthrough covers Spec 117 §13.3 required steps"

grep -qF '"no-proof-no-done"' "$WT" || fail "catalog/render/start missing no-proof-no-done"
grep -qF '"no-proof-no-done" => no_proof_no_done().steps[0].id.clone()' "$WT" || fail "start action missing no-proof-no-done first step"
grep -qF '"no-proof-no-done" => Ok(serde_json::to_value(no_proof_no_done())?)' "$WT" || fail "render action missing no-proof-no-done"
pass "no-proof-no-done walkthrough wired into catalog/start/show"

echo "focusa-117 no-proof-no-done walkthrough static test: PASS"
