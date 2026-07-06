#!/usr/bin/env bash
# Spec 117 Phase 8 — Public Docs Sync static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

DOC="$ROOT_DIR/docs/PUBLIC_DOCS_SYNC.md"
README="$ROOT_DIR/README.md"
[[ -f "$DOC" ]] || fail "public docs sync doc missing"

for needle in \
  'Focusa Public Docs Sync' \
  'Proven public entry points' \
  'Public claims allowed now' \
  'Public claims to avoid until separately proven' \
  'Current launch-blocking polish beads' \
  'RELEASE_INSTALL_POSTCARD.md' \
  'GTM_FIVE_MINUTE_PROOF.md' \
  'Mission Deck has Deck Home' \
  'Recall is advisory' \
  'PWA work remains roadmap/deferred' \
  'focusa-117-arch.24' \
  'focusa-117-arch.25' \
  'focusa-117-arch.26' \
  'focusa-117-arch.27' \
  'focusa-117-arch.28' \
  'focusa-117-arch.29'; do
  grep -qF -- "$needle" "$DOC" || fail "public docs sync missing: $needle"
done
pass "public docs sync separates proven claims from roadmap/deferred claims"

grep -qF 'docs/PUBLIC_DOCS_SYNC.md' "$README" || fail "README docs map missing PUBLIC_DOCS_SYNC link"
pass "README documentation map links public docs sync"

echo "focusa-117 public-docs-sync static test: PASS"
