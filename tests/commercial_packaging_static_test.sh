#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/COMMERCIAL_PACKAGING.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
README="$ROOT_DIR/README.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$DOC" ] || fail "COMMERCIAL_PACKAGING.md missing"
for section in 'Editions' 'Package artifacts' 'Commercial readiness gates' 'License and billing placeholders' 'Non-goals' 'Proof'; do
  rg -n -F "$section" "$DOC" >/dev/null || fail "commercial packaging missing section $section"
done
pass "commercial packaging sections present"

for edition in 'Community source' 'Pro local' 'Team self-hosted' 'Enterprise'; do
  rg -n -F "$edition" "$DOC" >/dev/null || fail "commercial packaging missing edition $edition"
done
pass "commercial packaging editions defined"

for artifact in 'Focusa daemon binary' 'Focusa CLI binary' 'Mac menubar app bundle' 'Pi extension package' 'release proof bundle' 'security/trust docs' 'installer/update policy' 'migration/backup policy'; do
  rg -n -F "$artifact" "$DOC" >/dev/null || fail "commercial packaging missing artifact $artifact"
done
pass "commercial packaging artifacts enumerated"

for gate in 'Version consistency passes' 'Tool-surface summary is current' 'Security/trust docs exist' 'Public proof artifacts are redacted' 'License and billing terms are explicit'; do
  rg -n -F "$gate" "$DOC" >/dev/null || fail "commercial packaging missing readiness gate $gate"
done
pass "commercial readiness gates preserved"

rg -n -F 'BSL--1.1' "$README" >/dev/null || fail "README license badge missing BSL--1.1"
rg -n -F 'not a cloud memory service' "$DOC" >/dev/null || fail "commercial doc missing local-first non-cloud positioning"
for marker in 'COMMERCIAL_PACKAGING.md' 'commercial_packaging_static_test.sh'; do
  rg -n -F "$marker" "$SPEC" >/dev/null || fail "Spec106 missing commercial proof marker $marker"
done
pass "Spec106 references commercial packaging proof"

echo "commercial packaging static test: PASS"
