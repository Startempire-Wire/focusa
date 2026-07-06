#!/usr/bin/env bash
# Spec 111 Slices 4+5 — safe write + receipt preview/commit integration static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

PRE="$ROOT_DIR/crates/focusa-api/src/routes/preload.rs"
[[ -f "$PRE" ]] || fail "preload.rs missing"
for needle in \
  'async fn write_packet' \
  'is_safe_target' \
  '/tmp/focusa-preload/' \
  '/var/cache/focusa/preload/' \
  'idempotency_key' \
  'unsafe_target_path' \
  'FOCUSA_PRELOAD_FAIL' \
  'pub fn receipt_preview_for' \
  'BOOTSTRAP_RECEIPT_KIND' \
  'bootstrap_delivery' \
  'rendered'; do
  grep -qF -- "$needle" "$PRE" || fail "preload slice 4/5 missing: $needle"
done
pass "preload slice 4+5 enforces safe write + receipt preview integration"
echo "focusa-111 preload slice4_5 static test: PASS"
