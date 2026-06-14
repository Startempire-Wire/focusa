#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
POLICY="$ROOT_DIR/docs/current/PUBLIC_STREAM_REDACTION_POLICY.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
AWARENESS="$ROOT_DIR/crates/focusa-api/src/routes/awareness.rs"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$POLICY" ] || fail "PUBLIC_STREAM_REDACTION_POLICY.md missing"
for needle in \
  'deny-by-default' \
  'Required public card fields' \
  'raw logs' \
  'secrets' \
  'tokens' \
  'private file contents' \
  'browser diagnostics with sensitive URLs' \
  'publish_allowed=true'; do
  rg -n -F "$needle" "$POLICY" >/dev/null || fail "policy missing $needle"
done
pass "public stream policy declares deny-by-default redaction rules"

for needle in schema project_identity_display_name redacted_scope_id canonical_status tool_family evidence_refs_public_safe redaction_status secret_scan_status publish_allowed; do
  rg -n -F "$needle" "$POLICY" >/dev/null || fail "policy missing required field $needle"
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec106 missing required field $needle"
  rg -n -F "$needle" "$AWARENESS" >/dev/null || fail "awareness route missing required field $needle"
done
pass "required public card fields present in policy/spec/awareness route"

for needle in 'public_stream_policy' 'PUBLIC_CARD:' 'redacted_scope_id' 'publish_allowed": false' 'not_required_no_raw_payload'; do
  rg -n -F "$needle" "$AWARENESS" >/dev/null || fail "awareness route missing hardening marker $needle"
done
pass "awareness card exposes public stream policy block"

echo "public stream redaction policy static test: PASS"
