#!/usr/bin/env bash
# Spec123 Order 13 — README product-page / visual-proof static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
README="$ROOT_DIR/README.md"
ASSET_DIR="$ROOT_DIR/docs/assets/readme"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[ -f "$README" ] || fail "README.md missing"

grep -q 'Keep AI coding agents on mission' "$README" || fail "README missing public hero promise"
grep -q 'local-first proof and continuity layer' "$README" || fail "README missing product sentence"
grep -q '^## Install' "$README" || fail "README missing Install section"
grep -q '^## Five-minute proof' "$README" || fail "README missing Five-minute proof section"
grep -q 'focusa setup wizard --dry-run' "$README" || fail "README install uses non-current setup command"
grep -q 'focusa first-mission --project-root' "$README" || fail "README missing first-mission proof command"
grep -q 'focusa workpoint checkpoint' "$README" || fail "README missing Workpoint checkpoint proof"
grep -q 'focusa workpoint evidence-link' "$README" || fail "README missing Evidence link proof"
grep -q 'focusa status operator --json' "$README" || fail "README missing status operator command"
grep -q 'tests/spec_cli_cross_phase_smoke_test.sh' "$README" || fail "README missing cross-phase smoke proof"

for n in 01 02 03 04 05 06 07 08; do
  grep -q "### $n ·" "$README" || fail "README missing numbered visual section $n"
done

for asset in \
  focusa-hero.svg \
  01-resume-after-compaction.svg \
  02-evidence-refs.svg \
  03-context-authority.svg \
  04-mission-deck.svg \
  05-pi-extension.svg \
  06-local-api.svg \
  07-menubar-preview.svg \
  08-public-proof.svg; do
  [ -f "$ASSET_DIR/$asset" ] || fail "README visual asset missing: $asset"
  grep -q 'public-safe illustrative visual proof' "$ASSET_DIR/$asset" || fail "asset not labeled public-safe illustrative proof: $asset"
done

# Keep the public README away from private strategy/proof/transcript framing.
if rg -n 'SignalOS|raw transcript|pricing/cap|operator private|agent-kb|/home/|/root/' "$README" >/tmp/spec123-readme-private-hits.txt; then
  cat /tmp/spec123-readme-private-hits.txt >&2
  fail "README contains private/internal wording"
fi

pass "Spec123 README product page static guard passed"
