#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/FOCUSA_GLOSSARY_LINKED_DOCS_UI.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$DOC" ] || fail "FOCUSA_GLOSSARY_LINKED_DOCS_UI.md missing"
for section in 'Canonical term index' 'UI expectations' 'Suggested navigation groups' 'Proof'; do
  rg -n -F "$section" "$DOC" >/dev/null || fail "glossary UI doc missing section $section"
done
pass "glossary UI sections present"

terms=('Focus State' 'Trajectory' 'Workpoint' 'Evidence' 'Context Authority' 'Context Cognition' 'Project Identity' 'Continuity ID' 'Session ID' 'Call Stack Design' 'Public Card' 'Tool Result Envelope')
for term in "${terms[@]}"; do
  rg -n -F "| $term |" "$DOC" >/dev/null || fail "glossary missing term $term"
done
pass "canonical terms indexed"

refs=(
  AUTHORITY_MODEL.md
  TRAJECTORY_GTM_AND_GAPS.md
  WORKPOINT_LIFECYCLE_GUIDE.md
  GOLDEN_WORKFLOW_PUBLIC_DEMO.md
  CONTEXT_AUTHORITY_CURRENT.md
  GOLDEN_WORKFLOW.md
  MULTI_AGENT_SCOPE_MODEL.md
  CALL_STACK_DESIGN_CURRENT.md
  PUBLIC_STREAM_REDACTION_POLICY.md
  TOOL_RESULT_ENVELOPE_V1.md
  PUBLIC_PROOF_BUNDLE_VIEWER.md
  VALIDATION_AND_RELEASE_PROOF.md
  AGENT_ADAPTER_CONTRACT.md
  NON_PI_AGENT_ADAPTER_EXAMPLES.md
)
for ref in "${refs[@]}"; do
  rg -n -F "$ref" "$DOC" >/dev/null || fail "glossary missing doc link $ref"
  [ -f "$ROOT_DIR/docs/current/$ref" ] || fail "glossary linked doc missing docs/current/$ref"
done
pass "glossary doc links resolve"

for marker in 'Show glossary hover/card' 'Mark advisory vs authority terms visually' 'search aliases' 'never replace canonical names' 'redaction-safe'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "glossary UI expectation missing $marker"
done
pass "glossary UI expectations preserved"

for marker in 'FOCUSA_GLOSSARY_LINKED_DOCS_UI.md' 'glossary_linked_docs_ui_static_test.sh'; do
  rg -n -F "$marker" "$SPEC" >/dev/null || fail "Spec106 missing glossary proof marker $marker"
done
pass "Spec106 references glossary UI proof"

echo "glossary linked docs UI static test: PASS"
