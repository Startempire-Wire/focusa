#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEMO="$ROOT_DIR/docs/current/GOLDEN_WORKFLOW_PUBLIC_DEMO.md"
GOLDEN="$ROOT_DIR/docs/current/GOLDEN_WORKFLOW.md"
AUTH="$ROOT_DIR/docs/current/AUTHORITY_MODEL.md"
REDACTION="$ROOT_DIR/docs/current/PUBLIC_STREAM_REDACTION_POLICY.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$DEMO" ] || fail "GOLDEN_WORKFLOW_PUBLIC_DEMO.md missing"
for needle in \
  'Project identity' \
  'Trajectory' \
  'Workpoint' \
  'Context Authority' \
  'Context Cognition' \
  'Call Stack' \
  'Evidence' \
  'Prediction + Metacognition' \
  'Public card'; do
  rg -n -F "$needle" "$DEMO" >/dev/null || fail "demo missing step $needle"
done
pass "public demo covers Golden Workflow path"

for needle in \
  'Operator steering wins' \
  'project_root + continuity_id' \
  'Workpoint is immediate continuation authority' \
  'Context Cognition is advisory' \
  'Evidence/proof handles replace raw logs' \
  'publish_allowed=false'; do
  rg -n -F "$needle" "$DEMO" >/dev/null || fail "demo missing public-safe beat $needle"
done
pass "public demo preserves authority/redaction beats"

for ref in "$GOLDEN" "$AUTH" "$REDACTION"; do
  [ -f "$ref" ] || fail "required reference missing $ref"
done
for marker in 'focusa project identity' 'focusa trajectory view' 'focusa workpoint resume' 'focusa action preflight' 'focusa context-cognition render' 'focusa call-stack verify'; do
  rg -n -F "$marker" "$DEMO" >/dev/null || fail "demo missing command $marker"
done
pass "public demo includes bounded command path"

for marker in 'GOLDEN_WORKFLOW_PUBLIC_DEMO.md' 'golden_workflow_public_demo_static_test.sh'; do
  rg -n -F "$marker" "$SPEC" >/dev/null || fail "Spec106 missing demo marker $marker"
done
pass "Spec106 references public demo proof"

echo "golden workflow public demo static test: PASS"
