#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/EVAL_METRICS_DASHBOARD.md"
AGENT_EVALS="$ROOT_DIR/docs/current/FOCUSA_AGENT_INTELLIGENCE_EVALS.md"
PROOF_VIEWER="$ROOT_DIR/docs/current/PUBLIC_PROOF_BUNDLE_VIEWER.md"
STATUS="$ROOT_DIR/docs/current/CURRENT_RUNTIME_STATUS.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for path in "$DOC" "$AGENT_EVALS" "$PROOF_VIEWER" "$STATUS"; do
  [ -f "$path" ] || fail "required eval dashboard ref missing $path"
done
for section in 'Metric sources' 'Dashboard cards' 'Minimum filters' 'Authority and privacy' 'Suggested proof commands' 'Proof'; do
  rg -n -F "$section" "$DOC" >/dev/null || fail "eval dashboard doc missing section $section"
done
pass "eval dashboard sections present"

for marker in 'precision' 'recall' 'F1' 'baseline_score' 'eval_score' 'Prediction records' 'Metacognition adjustments' 'Tool contract validation counts' 'Release proof / runtime status gates'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "eval dashboard metric source missing $marker"
done
pass "eval dashboard metric sources present"

for card in 'Curator quality' 'Prediction calibration' 'Optimizer promotions' 'Tool contract health' 'Release proof health' 'Agent intelligence'; do
  rg -n -F "$card" "$DOC" >/dev/null || fail "eval dashboard card missing $card"
done
pass "eval dashboard cards present"

for filter in 'project root' 'continuity id' 'date/time window' 'module name' 'status'; do
  rg -n -F "$filter" "$DOC" >/dev/null || fail "eval dashboard filter missing $filter"
done
pass "eval dashboard filters present"

for marker in 'Dashboard metrics are advisory' 'do not promote artifacts' 'PUBLIC_STREAM_REDACTION_POLICY.md' 'focusa predict stats' 'optimizer-artifacts' 'validate-focusa-tool-contracts.mjs'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "eval dashboard authority/proof marker missing $marker"
done
pass "eval dashboard authority/proof markers present"

for marker in 'EVAL_METRICS_DASHBOARD.md' 'eval_metrics_dashboard_static_test.sh'; do
  rg -n -F "$marker" "$SPEC" >/dev/null || fail "Spec106 missing eval dashboard proof marker $marker"
done
pass "Spec106 references eval dashboard proof"

echo "eval metrics dashboard static test: PASS"
