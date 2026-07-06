#!/usr/bin/env bash
# Spec 117 §17.3 — /v1/deck/* read-first API routes static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

DECK="$ROOT_DIR/crates/focusa-api/src/routes/deck.rs"
MOD="$ROOT_DIR/crates/focusa-api/src/routes/mod.rs"
SERVER="$ROOT_DIR/crates/focusa-api/src/server.rs"
[[ -f "$DECK" ]] || fail "deck API route module missing"

for needle in \
  'focusa.deck.v1' \
  'focusa.walkthrough.v1' \
  'focusa.recall_deck_card.v1' \
  '/v1/deck/home' \
  '/v1/deck/walkthroughs' \
  '/v1/deck/recall/schema' \
  '/v1/deck/proof-meter' \
  '/v1/deck/next-safe-action' \
  'read_only' \
  'advisory_only' \
  'focusa-117-arch.29'; do
  grep -qF -- "$needle" "$DECK" || fail "deck API route missing: $needle"
done
pass "deck API module exposes required read-first routes and schemas"

grep -qF 'pub mod deck;' "$MOD" || fail "routes mod missing deck export"
grep -qF 'routes::deck::router()' "$SERVER" || fail "server does not merge deck router"
pass "deck API routes wired into daemon router"

echo "focusa-117 deck-api-routes static test: PASS"
