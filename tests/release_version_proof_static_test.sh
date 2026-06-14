#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
STATUS="$ROOT_DIR/docs/current/CURRENT_RUNTIME_STATUS.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for script in \
  scripts/stamp-release-version \
  scripts/generate-current-runtime-status \
  scripts/generate-tool-surface-summary \
  scripts/verify-doc-version-consistency; do
  [ -x "$ROOT_DIR/$script" ] || fail "$script missing or not executable"
  rg -n -F "$script" "$SPEC" >/dev/null || fail "Spec106 missing $script"
done
pass "release/version scripts exist and are referenced by Spec106"

(cd "$ROOT_DIR" && scripts/stamp-release-version && scripts/generate-tool-surface-summary --check && scripts/generate-current-runtime-status && scripts/verify-doc-version-consistency) >/tmp/focusa-release-version-proof.out
rg -n -F 'version consistency ok' /tmp/focusa-release-version-proof.out >/dev/null || fail "version consistency did not pass"
rg -n -F 'tool-surface-summary is current' /tmp/focusa-release-version-proof.out >/dev/null || fail "tool surface summary check did not pass"
pass "release/version proof script chain passes"

[ -f "$ROOT_DIR/docs/current/.release-version-stamp" ] || fail "release version stamp missing"
for marker in \
  'GENERATED: scripts/generate-current-runtime-status' \
  'Version:' \
  'Tool contracts:' \
  'proof command: focusa release prove --tag <tag>'; do
  rg -n -F "$marker" "$STATUS" >/dev/null || fail "runtime status missing $marker"
done
pass "current runtime status generated from release inputs"

for marker in \
  'release stamp is generated' \
  'CLI/daemon/core/menubar versions match' \
  'generated docs updated' \
  'tool contract summary updated' \
  'proof bundle captured' \
  'runtime status updated from generated source'; do
  rg -n -F "$marker" "$SPEC" >/dev/null || fail "Spec106 release invariant missing $marker"
done
pass "Spec106 release invariant preserved"

echo "release version proof static test: PASS"
