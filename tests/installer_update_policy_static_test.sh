#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/INSTALLER_UPDATE_POLICY.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
COOKBOOK="$ROOT_DIR/docs/current/AGENT_COMMAND_COOKBOOK.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$DOC" ] || fail "INSTALLER_UPDATE_POLICY.md missing"
for section in 'Install channels' 'Required preflight' 'Live build host policy' 'Update checklist' 'Rollback checklist' 'Forbidden substitutions' 'Proof'; do
  rg -n -F "$section" "$DOC" >/dev/null || fail "installer/update policy missing section $section"
done
pass "installer/update sections present"

for marker in 'Source build' 'Release asset' 'Menubar app bundle' 'Pi extension package'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "install channel missing $marker"
done
pass "installer/update channels present"

for marker in 'focusa action preflight' 'binary_replace' 'github_release_asset' 'live_build_host' 'block' 'ask_operator'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "preflight marker missing $marker"
done
for marker in 'action preflight' 'binary_replace' 'github_release_asset' 'live_build_host'; do
  rg -n -F "$marker" "$COOKBOOK" >/dev/null || fail "cookbook missing preflight marker $marker"
done
pass "installer/update preflight mirrors command cookbook"

for marker in 'Verify checksum/signature' 'Snapshot current binary/config/service state' 'Run daemon health' 'focusa release prove --tag <tag>' 'Restore previous binary' 'Link rollback evidence'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "update/rollback marker missing $marker"
done
pass "installer/update rollback safeguards present"

for marker in 'Pairing troubleshooting must not trigger installer/update work by default' 'Stale menubar UI must first try refresh/reconnect' 'Release asset replacement on a live build host requires Context Authority approval'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "forbidden substitution marker missing $marker"
done
pass "installer/update forbidden substitutions present"

for marker in 'INSTALLER_UPDATE_POLICY.md' 'installer_update_policy_static_test.sh'; do
  rg -n -F "$marker" "$SPEC" >/dev/null || fail "Spec106 missing installer/update proof marker $marker"
done
pass "Spec106 references installer/update proof"

for marker in 'One canonical Focusa Pi package' 'retired-extensions' 'focusa-pi-bridge' 'never satisfies acceptance' 'FOCUSA_UPDATE_FAULT_AFTER_PI_ACTIVATION' 'typed activation receipt'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "Pi package policy missing $marker"
done
pass "Pi package transaction/discovery requirements documented"

for marker in 'One canonical Focusa Pi package' 'retired-extensions' '-ne' 'zero duplicate tool'; do
  rg -n -F -e "$marker" "$ROOT_DIR/AGENTS.md" >/dev/null || fail "AGENTS.md missing Pi package marker $marker"
done
pass "one-canonical-Pi-package rule documented in AGENTS.md"

for marker in 'pi_package' 'FOCUSA_UPDATE_FAULT_AFTER_PI_ACTIVATION' 'rollback_pi_activation'; do
  rg -n -F "$marker" "$ROOT_DIR/crates/focusa-cli/src/commands" >/dev/null || fail "shared Pi activation transaction missing $marker"
done
pass "shared Pi activation transaction implemented"

echo "installer update policy static test: PASS"
