#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in spec_availability spec_only partial implemented deprecated runtime_entrypoint docs_ref first_implementation_slice SpecRuntimeAvailabilityLabel; do
  rg -F "$term" crates/focusa-api/src/routes/traverse.rs >/dev/null || fail "spec availability registry missing $term"
done
pass "traverse declares spec availability registry terms"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d '{"surface":"spec_availability","selector":"registry","limit":20}' \
  "$BASE/v1/traverse" >/tmp/spec102-spec-availability.json
jq -e '
  .traversal.spec_availability.schema == "focusa.spec_availability.v1"
  and (.items | length) >= 3
  and ([.items[].data.spec_id] | index("Spec100"))
  and ([.items[].data.spec_id] | index("Spec101"))
  and ([.items[].data.spec_id] | index("Spec102"))
' /tmp/spec102-spec-availability.json >/dev/null || fail "registry missing expected specs"
pass "registry lists Spec100/101/102"

jq -e '
  [.items[].data | select(.spec_id == "Spec100" or .spec_id == "Spec101")]
  | length == 2
  and all(.availability != "implemented")
  and all(.runtime_entrypoint == null or .runtime_entrypoint == "")
  and all((.trust_badges | index("spec_only")) or (.trust_badges | index("partial")))
' /tmp/spec102-spec-availability.json >/dev/null || fail "Spec100/101 incorrectly look runtime-ready"
pass "Spec100/101 do not appear runtime-ready"

jq -e '
  [.items[].data | select(.spec_id == "Spec102")][0]
  | .availability == "implemented"
  and (.runtime_entrypoint | test("/v1/traverse|tests/spec102"))
  and ((.trust_badges | index("implemented")) and ((.trust_badges | index("spec_only")) | not))
' /tmp/spec102-spec-availability.json >/dev/null || fail "implemented Spec102 shows spec-only caveat or lacks runtime entrypoint"
pass "implemented happy path omits spec-only caveat"

echo "SPEC102 spec availability registry test: PASS"
