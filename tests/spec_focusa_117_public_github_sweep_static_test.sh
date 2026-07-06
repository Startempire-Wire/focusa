#!/usr/bin/env bash
# Spec 117 launch blocker — Public GitHub Focusa sweep static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

DOC="$ROOT_DIR/docs/PUBLIC_GITHUB_SWEEP.md"
README="$ROOT_DIR/README.md"
[[ -f "$DOC" ]] || fail "public sweep doc missing"

for needle in \
  'Focusa Public GitHub Sweep' \
  'README install command' \
  'README quickstart' \
  'Mission Deck reference' \
  'Release Install Postcard' \
  'GTM Five-Minute Proof' \
  'Public Docs Sync' \
  'Newbie Onboarding QA' \
  'Spec 117 plan doc' \
  'Inaccuracies to avoid in public copy' \
  'Claiming full PWA is shipped' \
  'Claiming full Recall implementation is shipped' \
  'Recall can directly create canonical Workpoints' \
  'Verification steps for the next sweep' \
  'gh run list --workflow CI'; do
  grep -qF -- "$needle" "$DOC" || fail "public sweep doc missing: $needle"
done
pass "public sweep doc covers findings, inaccuracies, verification"

grep -qF 'PUBLIC_GITHUB_SWEEP.md' "$README" || fail "README missing public sweep link"
pass "README links public sweep doc"

echo "focusa-117 public-github-sweep static test: PASS"
