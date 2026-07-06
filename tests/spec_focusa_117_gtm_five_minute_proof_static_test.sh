#!/usr/bin/env bash
# Spec 117 Phase 8 — GTM Five-Minute Proof static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

GTM="$ROOT_DIR/docs/GTM_FIVE_MINUTE_PROOF.md"
README="$ROOT_DIR/README.md"
POSTCARD="$ROOT_DIR/docs/RELEASE_INSTALL_POSTCARD.md"
[[ -f "$GTM" ]] || fail "GTM five-minute proof missing"

for needle in \
  'Focusa GTM Five-Minute Proof' \
  'Minute 0' \
  'Minute 1' \
  'Minute 2' \
  'Minute 3' \
  'Minute 4' \
  'Minute 5' \
  'scripts/install-daemon.sh /usr/local' \
  'focusa init --quickstart' \
  'focusa deck' \
  'focusa-tui --headless-self-test' \
  'focusa walkthrough show --walkthrough first-mission' \
  'focusa walkthrough show --walkthrough agent-handoff' \
  'focusa walkthrough show --walkthrough no-proof-no-done' \
  '/v1/deck/home' \
  '/v1/deck/proof-meter' \
  'CI proof link'; do
  grep -qF -- "$needle" "$GTM" || fail "GTM proof missing: $needle"
done
pass "GTM proof covers five-minute install→deck→walkthrough→proof path"

grep -qF 'GTM Five-Minute Proof](docs/GTM_FIVE_MINUTE_PROOF.md)' "$README" || fail "README missing GTM proof link"
grep -qF 'GTM Five-Minute Proof](GTM_FIVE_MINUTE_PROOF.md)' "$POSTCARD" || fail "postcard missing GTM proof link"
pass "README and postcard link GTM proof"

echo "focusa-117 gtm-five-minute-proof static test: PASS"
