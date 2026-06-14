#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/FOCUSA_AGENT_INTELLIGENCE_EVALS.md"
CASES="$ROOT_DIR/tests/evals/agent_intelligence_cases.json"
RUNNER="$ROOT_DIR/scripts/run-agent-intelligence-evals.sh"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for file in "$DOC" "$CASES" "$RUNNER"; do
  [ -f "$file" ] || fail "missing $file"
done
[ -x "$RUNNER" ] || fail "runner not executable"
pass "benchmark doc, cases, and runner exist"

for category in Continuity Scope Evidence Context Execution Learning Safety; do
  rg -n -F "**$category**" "$DOC" >/dev/null || fail "benchmark doc missing category $category"
done
for marker in 'aggregate score' 'score >= threshold' 'advisory' 'never mutates Focus State'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "benchmark doc missing marker $marker"
done
pass "benchmark doc defines categories and promotion boundary"

jq -e '.schema == "focusa.agent_intelligence_evals.v1" and (.required_categories | length == 7) and (.cases | length >= 7)' "$CASES" >/dev/null || fail "cases schema/count mismatch"
for category in continuity scope evidence context execution learning safety; do
  jq -e --arg category "$category" '.cases[] | select(.category == $category and .score >= .threshold)' "$CASES" >/dev/null || fail "missing passing case for $category"
done
pass "eval fixture covers all required categories"

"$RUNNER" "$CASES" >/tmp/focusa-agent-intelligence-evals.out
rg -n -F 'agent intelligence eval cases pass' /tmp/focusa-agent-intelligence-evals.out >/dev/null || fail "runner did not report pass"
pass "agent intelligence eval runner passes fixtures"

for marker in 'FOCUSA_AGENT_INTELLIGENCE_EVALS.md' 'agent_intelligence_cases.json' 'run-agent-intelligence-evals.sh' 'agent_intelligence_benchmark_static_test.sh'; do
  rg -n -F "$marker" "$SPEC" >/dev/null || fail "Spec106 missing marker $marker"
done
pass "Spec106 references benchmark docs/tests/runner"

echo "agent intelligence benchmark static test: PASS"
