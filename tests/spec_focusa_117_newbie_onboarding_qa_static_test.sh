#!/usr/bin/env bash
# Spec 117 launch blocker — Full newbie onboarding/walkthrough QA static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

DOC="$ROOT_DIR/docs/NEWBIE_ONBOARDING_WALKTHROUGH_QA.md"
README="$ROOT_DIR/README.md"
[[ -f "$DOC" ]] || fail "newbie onboarding QA doc missing"

for needle in \
  'Newbie Onboarding and Walkthrough Experience QA' \
  'First install' \
  'Daemon start' \
  'Project bind' \
  'First mission' \
  'Agent handoff' \
  'No Proof, No Done' \
  'Help overlay' \
  'Mission Deck TUI' \
  'Recovery states' \
  'Evidence education' \
  'Source-backed claim matrix' \
  'Focusa Mission Deck' \
  'Deck Home' \
  'Recall is advisory' \
  'first-mission' \
  'agent-handoff' \
  'no-proof-no-done'; do
  grep -qF -- "$needle" "$DOC" || fail "newbie QA doc missing: $needle"
done
pass "newbie QA doc covers every onboarding stage and walkthrough"

grep -qF 'NEWBIE_ONBOARDING_WALKTHROUGH_QA.md' "$README" || fail "README missing newbie QA link"
pass "README links newbie QA doc"

echo "focusa-117 newbie-onboarding-qa static test: PASS"
