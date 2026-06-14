#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROUTE="$ROOT_DIR/crates/focusa-api/src/routes/device_pairing.rs"
SPEC="$ROOT_DIR/docs/53-focusa-device-pairing-spec.md"
SPEC106="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
START_DOC="$ROOT_DIR/docs/focusa-tools/tools/focusa_device_pair_start.md"
COMPLETE_DOC="$ROOT_DIR/docs/focusa-tools/tools/focusa_device_pair_complete.md"
CARGO="$ROOT_DIR/crates/focusa-api/Cargo.toml"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for needle in \
  'OsRng.fill_bytes' \
  'URL_SAFE_NO_PAD.encode' \
  'normalize_scopes' \
  'scope_not_allowed' \
  'validate_pairing_url' \
  'pairing_url_invalid' \
  'bounded_label' \
  'pair_code_already_used' \
  '/home/wirebot/.cargo'; do
  rg -n -F "$needle" "$ROUTE" >/dev/null || fail "device_pairing.rs missing hardening marker: $needle"
done
pass "device pairing route has token/scope/url/label/single-use hardening markers"

for needle in 'base64 = "0.22"' 'rand = "0.8"'; do
  rg -n -F "$needle" "$CARGO" >/dev/null || fail "focusa-api Cargo missing dependency: $needle"
done
pass "device pairing CSPRNG/base64 dependencies declared"

for needle in \
  'Weak token entropy' \
  'Over-broad OAuth scopes' \
  'Malicious pairing URL' \
  'Label/log injection' \
  'Agent runtime path confusion' \
  'Endpoint hardening requirements'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "pairing spec missing threat marker: $needle"
done
pass "pairing spec includes threat model and endpoint hardening requirements"

for needle in \
  'CSPRNG tokens' \
  'scope allowlist' \
  'URL validation' \
  'unsafe host rejection' \
  'tests/device_pairing_endpoint_hardening_live_safe_test.sh'; do
  rg -n -F "$needle" "$SPEC106" >/dev/null || fail "Spec106 missing device hardening marker: $needle"
done
pass "Spec106 records device pairing hardening proof boundary"

for needle in 'scope_not_allowed' 'pairing_url_invalid' 'read` and `write`' 'https://' ; do
  rg -n -F "$needle" "$START_DOC" >/dev/null || fail "pair start doc missing: $needle"
done
for needle in '32-byte CSPRNG token' 'pair_code_already_used' 'sanitized to a bounded safe label'; do
  rg -n -F "$needle" "$COMPLETE_DOC" >/dev/null || fail "pair complete doc missing: $needle"
done
pass "device pairing tool docs include hardening failure classes and token model"

echo "device pairing threat model static test: PASS"
