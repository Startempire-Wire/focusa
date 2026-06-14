#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

docs=(
  SECURITY_MODEL.md
  DEVICE_PAIRING_THREAT_MODEL.md
  TOKEN_AND_SECRET_HANDLING.md
  PUBLIC_STREAM_REDACTION_POLICY.md
  LOCAL_FIRST_DATA_MODEL.md
  MULTI_AGENT_SCOPE_MODEL.md
)
for doc in "${docs[@]}"; do
  path="$ROOT_DIR/docs/current/$doc"
  [ -f "$path" ] || fail "$doc missing"
  rg -n -F "$doc" "$SPEC" >/dev/null || fail "Spec106 missing $doc"
done
pass "Spec106 trust-doc set exists and is referenced"

rg -n -F 'API auth token support' "$ROOT_DIR/docs/current/SECURITY_MODEL.md" >/dev/null || fail "SECURITY_MODEL missing API auth"
rg -n -F 'project_root + continuity_id' "$ROOT_DIR/docs/current/SECURITY_MODEL.md" >/dev/null || fail "SECURITY_MODEL missing scope boundary"
rg -n -F '32-byte CSPRNG' "$ROOT_DIR/docs/current/DEVICE_PAIRING_THREAT_MODEL.md" >/dev/null || fail "DEVICE_PAIRING_THREAT_MODEL missing token entropy"
rg -n -F 'Never paste tokens' "$ROOT_DIR/docs/current/TOKEN_AND_SECRET_HANDLING.md" >/dev/null || fail "TOKEN_AND_SECRET_HANDLING missing token handling rule"
rg -n -F 'publish_allowed=false' "$ROOT_DIR/docs/current/PUBLIC_STREAM_REDACTION_POLICY.md" >/dev/null || fail "PUBLIC_STREAM_REDACTION_POLICY missing deny default"
rg -n -F 'Append-only ledgers' "$ROOT_DIR/docs/current/LOCAL_FIRST_DATA_MODEL.md" >/dev/null || fail "LOCAL_FIRST_DATA_MODEL missing append-only model"
rg -n -F 'Same project root does not imply same Workpoint' "$ROOT_DIR/docs/current/MULTI_AGENT_SCOPE_MODEL.md" >/dev/null || fail "MULTI_AGENT_SCOPE_MODEL missing multi-agent scope rule"
pass "trust docs cover required security boundaries"

for marker in 'local data storage' 'token storage' 'API auth' 'pairing tokens' 'public URL exposure' 'redaction' 'scope isolation' 'agent mutation boundaries' 'audit logs' 'append-only ledgers' 'destructive action policies'; do
  rg -n -F "$marker" "$SPEC" >/dev/null || fail "Spec106 missing trust coverage marker $marker"
done
pass "Spec106 trust coverage markers preserved"

echo "security trust docs static test: PASS"
