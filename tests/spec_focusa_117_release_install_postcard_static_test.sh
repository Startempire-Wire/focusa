#!/usr/bin/env bash
# Spec 117 Phase 8 — Release Install Postcard static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

CARD="$ROOT_DIR/docs/RELEASE_INSTALL_POSTCARD.md"
README="$ROOT_DIR/README.md"
[[ -f "$CARD" ]] || fail "release install postcard missing"

for needle in \
  'Focusa Release Install Postcard' \
  'https://install.focusa.dev/focusa' \
  'focusa start' \
  'http://127.0.0.1:8787/v1/health' \
  'focusa doctor --scope host' \
  'focusa init --quickstart' \
  'focusa walkthrough show --walkthrough first-mission' \
  'focusa deck' \
  'focusa-tui' \
  'Focusa Mission Deck' \
  'Deck Home' \
  'Proof meter' \
  'Scope badge'; do
  grep -qF -- "$needle" "$CARD" || fail "postcard missing: $needle"
done
pass "postcard highlights install, post-install, quickstart, and Mission Deck"
grep -Fq 'Commands::Walkthrough(args) => commands::walkthrough::run(args, cli.json).await' "$ROOT_DIR/crates/focusa-cli/src/main.rs" || fail "walkthrough must dispatch without a dead replacement hint"
grep -Fq 'The quickstart only writes the marker.' "$CARD" || fail "postcard conflates a marker with attachment"


grep -qF 'Release Install Postcard](docs/RELEASE_INSTALL_POSTCARD.md)' "$README" || fail "README Quickstart missing postcard link"
pass "README links launch postcard from Quickstart"

echo "focusa-117 release-install-postcard static test: PASS"
