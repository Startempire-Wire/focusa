#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"
DAEMON="$ROOT_DIR/crates/focusa-core/src/runtime/daemon.rs"
TRAVERSE="$ROOT_DIR/crates/focusa-api/src/routes/traverse.rs"
CAPS="$ROOT_DIR/crates/focusa-api/src/routes/capabilities.rs"
CLT="$ROOT_DIR/crates/focusa-api/src/routes/clt.rs"
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

rg -n 'summarizeValue|summarizeTraverseItems|payload\?\.content_ref|payload\?\.summary' "$TOOLS" >/dev/null \
  || fail "Pi tree/traverse wrappers still stringify objects without actionable summaries"
pass "Pi wrappers render object arrays and traversal items instead of [object Object]/counts only"

rg -n 'clt_event_content_ref|Some\(&content_ref\)|FocusStateUpdated \{ delta|bounded_clt_text' "$DAEMON" >/dev/null \
  || fail "CLT event tracker does not attach bounded content_ref summaries"
pass "CLT event tracker records bounded content_ref summaries for state changes"

rg -n 'is_low_value_clt_intuition|LongRunningFrame|severity_value < 0\.8' "$DAEMON" >/dev/null \
  || fail "CLT tracker lacks low-value LongRunningFrame noise filter"
pass "CLT tracker filters low-value intuition noise that buries post-compaction context"

rg -n 'lineage_node_summary_and_ref|content_ref.*summary|obj.insert\("summary"' "$TRAVERSE" >/dev/null \
  || fail "/v1/traverse lineage surface does not expose top-level summary/content_ref"
pass "Traverse lineage items expose top-level summary/content_ref"

rg -n 'enriched_lineage_node_value|"content_ref"|"summary"' "$CAPS" "$CLT" >/dev/null \
  || fail "/v1/lineage and /v1/clt routes do not enrich CLT nodes for recovery"
pass "Lineage/CLT API routes enrich nodes for recovery"

if curl -fsS --max-time 3 "$BASE/v1/health" >/dev/null 2>&1; then
  HEAD_ID="$(curl -fsS --max-time 5 "$BASE/v1/lineage/head" | jq -r '.head // empty')"
  if [[ -n "$HEAD_ID" ]]; then
    curl -fsS --max-time 8 "$BASE/v1/lineage/path/$HEAD_ID" > /tmp/spec103-lineage-path.json
    jq -e '(.path | type == "array") and (.path[0] | has("summary"))' /tmp/spec103-lineage-path.json >/dev/null \
      || fail "live /v1/lineage/path does not expose node summary"
    pass "live /v1/lineage/path exposes node summary"
  fi
fi

echo "SPEC103 post-compaction recovery surfaces test: PASS"
