#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/FOCUSA_AGENT_INTELLIGENCE_EVALS.md"
CASES="$ROOT_DIR/tests/evals/agent_intelligence_cases.json"
RUNNER="$ROOT_DIR/scripts/run-agent-intelligence-evals.sh"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
fail(){ echo "FAIL: $*" >&2; exit 1; }
pass(){ echo "PASS (definition/refusal check only): $*"; }

for file in "$DOC" "$CASES" "$RUNNER"; do
  [ -f "$file" ] || fail "missing $file"
done
[ -x "$RUNNER" ] || fail "runner not executable"
for category in Continuity Scope Evidence Context Execution Learning Safety; do
  rg -n -F "**$category**" "$DOC" >/dev/null || fail "benchmark doc missing category $category"
done
for marker in 'unmeasured' 'aggregate score' 'score >= threshold' 'advisory' 'never mutates Focus State'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "benchmark doc missing marker $marker"
done
jq -e '.schema == "focusa.agent_intelligence_evals.v1" and (.required_categories | length == 7) and (.cases | length >= 7) and (all(.cases[]; has("score") | not))' "$CASES" >/dev/null || fail "scenario definitions contain scores or invalid schema/count"
for category in continuity scope evidence context execution learning safety; do
  jq -e --arg category "$category" '.cases[] | select(.category == $category and (.threshold | type) == "number")' "$CASES" >/dev/null || fail "missing scenario for $category"
done
pass "definitions cover categories without self-scored observations"

if output=$("$RUNNER" "$CASES" 2>&1); then
  fail "definition-only runner returned successful evaluation"
else
  code=$?
fi
[ "$code" -eq 78 ] || fail "expected unmeasured exit78, got $code: $output"
[[ "$output" == *UNMEASURED:* ]] || fail "runner omitted unmeasured reason"
[[ "$output" != *aggregate=* ]] || fail "runner fabricated aggregate"
pass "actual runner withholds acceptance for absent observations"
for marker in 'FOCUSA_AGENT_INTELLIGENCE_EVALS.md' 'agent_intelligence_cases.json' 'run-agent-intelligence-evals.sh' 'agent_intelligence_benchmark_static_test.sh'; do
  rg -n -F "$marker" "$SPEC" >/dev/null || fail "Spec106 missing marker $marker"
done
pass "Spec106 retains references; agent/model efficacy remains unmeasured"
