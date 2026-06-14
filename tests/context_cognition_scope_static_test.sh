#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROUTE="$ROOT_DIR/crates/focusa-api/src/routes/context_cognition.rs"
DOC="$ROOT_DIR/docs/focusa-tools/tools/focusa_context_cognition.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for needle in \
  'exact_scope_ready' \
  'missing_continuity_id' \
  'canonical Workpoint/Trajectory selection requires verified project_root + continuity_id' \
  'Do not treat Context Cognition as canonical without exact scope' \
  'r.continuity_id.as_deref() == continuity_id.as_deref()' \
  'record.continuity_id.as_deref() == continuity_id.as_deref()' \
  'active trajectory omitted: scope mismatch or missing continuity_id'; do
  rg -n -F "$needle" "$ROUTE" >/dev/null || fail "Context Cognition route missing exact-scope guard: $needle"
done
pass "Context Cognition route exact-matches project_root + continuity_id before Workpoint/Trajectory selection"

for term in 'advisory context' 'not continuation authority' 'AUTHORITY_MODEL.md'; do
  rg -n "$term" "$DOC" >/dev/null || fail "Context Cognition doc missing advisory/authority boundary: $term"
done
pass "Context Cognition doc declares advisory authority boundary"

for term in 'Exact-match Workpoint by `project_root + continuity_id`' 'Exact-match Trajectory by `project_root + continuity_id`' 'scope_status' 'context_budget' 'tokens_used' 'do_not_drift' 'source_refs' 'rehydrate_refs'; do
  rg -n -F "$term" "$SPEC" >/dev/null || fail "Spec106 missing Context Cognition requirement $term"
done
pass "Spec106 records Context Cognition hardening requirements"

echo "context cognition scope static test: PASS"
