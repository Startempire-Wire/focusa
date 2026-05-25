#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"
REGISTRY="$ROOT_DIR/docs/current/focusa-reflex-primitives.json"
SPEC="$ROOT_DIR/docs/97-focusa-reflex-primitives-spec.md"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

rg -n 'reflex_suggestions\?: string\[\]|function reflexSuggestionsForFailure|reflex_suggestions: reflexSuggestions' "$TOOLS" >/dev/null || fail "Pi tool_result_v1 lacks reflex_suggestions metadata plumbing"
pass "Pi tool_result_v1 carries reflex_suggestions"

rg -n 'diagnose_scope_mismatch|resource_mode_fallback|degrade_with_recovery|retry_safe_pending|route_noncanonical_result|preflight_writer_ownership|require_destructive_confirmation' "$TOOLS" >/dev/null || fail "common failure postures do not map to Spec97 primitive ids"
pass "common failure postures map to primitive ids"

python3 - "$TOOLS" "$REGISTRY" <<'PY'
import json, re, sys
source=open(sys.argv[1]).read()
registry=json.load(open(sys.argv[2]))
ids={p['primitive_id'] for p in registry['primitives']}
used=set(re.findall(r'"([a-z][a-z0-9_]+)"', source)) & ids
required={'diagnose_scope_mismatch','confirm_continuity_scope','resource_mode_fallback','degrade_with_recovery','retry_safe_pending','route_noncanonical_result','resume_from_canonical_workpoint','require_destructive_confirmation','preflight_writer_ownership','guard_stale_focus_state','bind_project_root','prefer_summary_hot_path'}
missing=required-used
if missing:
    raise SystemExit(f"missing registry-backed reflex ids: {sorted(missing)}")
PY
pass "all envelope primitive ids exist in registry"

rg -n 'G97-reflex-envelope-metadata|reflex_suggestions' "$SPEC" >/dev/null || fail "Spec97 does not mention reflex envelope metadata gap/field"
pass "Spec97 tracks reflex envelope metadata"

echo "SPEC97 reflex envelope metadata static test: PASS"
