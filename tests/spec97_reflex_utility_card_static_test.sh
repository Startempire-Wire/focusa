#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
AWARENESS="$ROOT_DIR/crates/focusa-api/src/routes/awareness.rs"
DOC="$ROOT_DIR/docs/current/NON_PI_AGENT_FOCUSA_USAGE.md"
SPEC="$ROOT_DIR/docs/97-focusa-reflex-primitives-spec.md"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

rg -n 'Reflex affordances|reflex_suggestions|surface=reflex_primitives|smallest safe next step' "$AWARENESS" >/dev/null || fail "awareness Utility Card lacks concise reflex affordance language"
pass "awareness card exposes concise reflex affordances"

rg -n 'awareness_card_mentions_required_agent_rules|Reflex affordances|surface=reflex_primitives' "$AWARENESS" >/dev/null || fail "awareness regression test lacks reflex needles"
pass "awareness card regression includes reflex needles"

rg -n 'reflex_suggestions|reflex_primitives|Reflex' "$SPEC" >/dev/null || fail "Spec97 lacks reflex presentation guidance"
pass "Spec97 documents reflex presentation guidance"

echo "SPEC97 reflex Utility Card static test: PASS"
