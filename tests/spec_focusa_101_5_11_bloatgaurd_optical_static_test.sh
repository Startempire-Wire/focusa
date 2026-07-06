#!/usr/bin/env bash
# Spec 101 §5.11 — Bloatgaurd Optical Context Gateway scaffold + provider policy ledger static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

BO="$ROOT_DIR/crates/focusa-api/src/routes/bloatgaurd_optical.rs"
MOD="$ROOT_DIR/crates/focusa-api/src/routes/mod.rs"
SERVER="$ROOT_DIR/crates/focusa-api/src/server.rs"
[[ -f "$BO" ]] || fail "bloatgaurd_optical.rs missing"

for needle in \
  'focusa.bloatgaurd_optical.v1' \
  'focusa.provider_policy_ledger.v1' \
  'safe_auto' \
  'text_passthrough' \
  'cold_opt_in' \
  'min_net_savings' \
  'max_quality_regression' \
  'POLICY_STATUS_ALLOWED' \
  'POLICY_STATUS_BLOCKED' \
  'POLICY_STATUS_UNKNOWN' \
  'POLICY_STATUS_STALE' \
  'workpoint_action_authority' \
  'evidence_refs_themselves' \
  'exact_diffs' \
  'secrets' \
  'hashes' \
  'uuids' \
  'provider_supports_image_input' \
  'canary_read_passes' \
  'IMAGED_ALLOWED_KINDS' \
  'NEVER_IMAGED' \
  '/v1/bloatgaurd/optical/policy' \
  '/v1/bloatgaurd/optical/ledger' \
  '/v1/bloatgaurd/optical/probe'; do
  grep -qF -- "$needle" "$BO" || fail "bloatgaurd_optical missing: $needle"
done
pass "Spec 101 §5.11 scaffold covers policy ledger, compatibility probe, imaged kinds, never imaged"

grep -qF 'pub mod bloatgaurd_optical;' "$MOD" || fail "routes mod missing bloatgaurd_optical export"
grep -qF 'routes::bloatgaurd_optical::router()' "$SERVER" || fail "server does not merge bloatgaurd_optical router"
pass "Bloatgaurd Optical Context Gateway wired into daemon router"

echo "focusa-101-5-11 bloatgaurd-optical static test: PASS"
