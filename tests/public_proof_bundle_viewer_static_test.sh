#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/PUBLIC_PROOF_BUNDLE_VIEWER.md"
REDACTION="$ROOT_DIR/docs/current/PUBLIC_STREAM_REDACTION_POLICY.md"
DEMO="$ROOT_DIR/docs/current/GOLDEN_WORKFLOW_PUBLIC_DEMO.md"
PROOF="$ROOT_DIR/docs/current/VALIDATION_AND_RELEASE_PROOF.md"
STATUS="$ROOT_DIR/docs/current/CURRENT_RUNTIME_STATUS.md"
PEEK="$ROOT_DIR/apps/menubar/src/lib/components/ProofPeek.svelte"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for path in "$DOC" "$REDACTION" "$DEMO" "$PROOF" "$STATUS" "$PEEK"; do
  [ -f "$path" ] || fail "required proof viewer ref missing $path"
done
for section in 'Viewer inputs' 'Required viewer fields' 'Viewer states' 'Menubar/operator preview' 'Public safety gates' 'Proof'; do
  rg -n -F "$section" "$DOC" >/dev/null || fail "proof viewer doc missing section $section"
done
pass "public proof viewer sections present"

for field in 'schema' 'project identity display name' 'redacted scope id' 'canonical/advisory/degraded status' 'proof bundle version/tag' 'evidence refs if public-safe' 'redaction status' 'secret scan status' 'publish_allowed'; do
  rg -n -F "$field" "$DOC" >/dev/null || fail "proof viewer missing field $field"
done
pass "public proof viewer fields present"

for state in 'draft_private' 'redaction_pending' 'publish_blocked' 'publish_ready' 'published_snapshot'; do
  rg -n -F "$state" "$DOC" >/dev/null || fail "proof viewer state missing $state"
done
pass "public proof viewer states present"

for marker in 'ProofPeek.svelte' 'not a public publisher' 'PUBLIC_STREAM_REDACTION_POLICY.md' 'Deny by default' 'publish_allowed=true' 'secret_scan_status=passed'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "proof viewer safety marker missing $marker"
done
pass "public proof viewer safety gates present"

for marker in 'Proof peek' 'Workpoint evidence' 'normalizeToolResult' 'evidence_refs' 'side_effects'; do
  rg -n -F "$marker" "$PEEK" >/dev/null || fail "ProofPeek component missing marker $marker"
done
pass "ProofPeek remains operator proof preview"

for marker in 'PUBLIC_PROOF_BUNDLE_VIEWER.md' 'public_proof_bundle_viewer_static_test.sh'; do
  rg -n -F "$marker" "$SPEC" >/dev/null || fail "Spec106 missing proof viewer marker $marker"
done
pass "Spec106 references public proof viewer proof"

echo "public proof bundle viewer static test: PASS"
