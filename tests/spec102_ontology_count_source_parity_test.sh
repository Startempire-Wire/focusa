#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
TMP="${TMPDIR:-/tmp}/spec102-ontology-count-source-parity"
mkdir -p "$TMP"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

curl -fsS --max-time 15 "$BASE/v1/project/card?project_root=$ROOT_DIR&current_ask=spec102-ontology-parity" > "$TMP/project-card.json"
curl -fsS --max-time 15 -H 'Content-Type: application/json' -d '{"surface":"ontology","selector":"window","limit":20}' "$BASE/v1/traverse" > "$TMP/traverse-ontology.json"

jq -e '
  .ontology.source_index != null
  and .ontology.scope_key != null
  and .ontology.selector != null
  and .ontology.freshness != null
  and .ontology.count_semantics != null
  and .ontology.why_zero_if_empty != null
  and .ontology.next_selector != null
  and .ontology.counts.runtime_objects != null
  and .ontology.counts.effective_project_card_objects != null
' "$TMP/project-card.json" >/dev/null || fail "project card ontology count source metadata missing"
pass "project-card ontology declares source/scope/selector/freshness/zero semantics"

jq -e '
  .traversal.source_index != null
  and .traversal.scope_key != null
  and .traversal.selector == "window"
  and .traversal.freshness != null
  and .traversal.count_semantics != null
  and .traversal.why_zero_if_empty != null
  and .traversal.next_selector != null
' "$TMP/traverse-ontology.json" >/dev/null || fail "traverse ontology count source metadata missing"
pass "traverse ontology declares source/scope/selector/freshness/zero semantics"

python3 - "$TMP/project-card.json" "$TMP/traverse-ontology.json" <<'PY'
import json, sys
card=json.load(open(sys.argv[1]))
trav=json.load(open(sys.argv[2]))
pc=card["ontology"]
t=trav["traversal"]
runtime=int(pc["counts"]["runtime_objects"])
effective=int(pc["counts"]["effective_project_card_objects"])
traverse_total=int(t["total"])
if runtime != traverse_total:
    raise SystemExit(f"runtime/traverse mismatch lacks parity: runtime={runtime} traverse_total={traverse_total}")
if effective != traverse_total:
    for obj, name in [(pc,"project_card"),(t,"traverse")]:
        for key in ["count_semantics","source_index","selector","next_selector"]:
            if not str(obj.get(key,"")):
                raise SystemExit(f"{name} missing mismatch explanation key {key}")
        if "derived" not in str(obj.get("count_semantics","")).lower() and name == "project_card":
            raise SystemExit("project_card mismatch semantics must mention derived/effective count")
print("✓ PASS: runtime traverse count agrees; effective mismatch explains source/selector/next selector")
PY

echo "SPEC102 ontology count source parity test: PASS"
