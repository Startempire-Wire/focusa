#!/usr/bin/env bash
set -euo pipefail
BASE="${UIAI_ENGINE_URL:-http://127.0.0.1:7456}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

curl -fsS --max-time 10 "$BASE/api/health/browser" >/dev/null || { echo "UIAI unavailable; live checks skipped"; exit 0; }

curl -fsS --max-time 15 "$BASE/api/tools/search?q=visual%20failure%20diagnostics" >/tmp/spec102-uiai-tokenized-search.json
jq -e '
  .count >= 1
  and .tools[0].name == "browser_diagnostics"
  and ([.tools[].name] | index("uiai_tool_search"))
  and ([.tools[].name] | index("browser_diagnostics_clear"))
' /tmp/spec102-uiai-tokenized-search.json >/dev/null || fail "multi-token visual failure diagnostics did not return useful tools"
pass "multi-token UIAI search returns diagnostics-first useful tools"

curl -fsS --max-time 15 "$BASE/api/tools/agent-card" >/tmp/spec102-uiai-agent-card.json
jq -e '(.search_hints | index("visual failure")) and (.search_hints | index("diagnostics"))' /tmp/spec102-uiai-agent-card.json >/dev/null || fail "agent card missing split-query hints"
pass "UIAI agent card advertises split-query fallback hints"

for q in "visual failure" diagnostics visual failure; do
  encoded=$(python3 - <<PY
import urllib.parse
print(urllib.parse.quote('''$q'''))
PY
)
  curl -fsS --max-time 15 "$BASE/api/tools/search?q=$encoded" >/tmp/spec102-uiai-split.json
  jq -e '.count >= 1' /tmp/spec102-uiai-split.json >/dev/null || fail "split query $q returned no tools"
done
pass "split-query fallbacks remain useful"

echo "SPEC102 UIAI tokenized tool search test: PASS"
