#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() { printf '✗ FINAL RELEASE GAP: %s\n' "$*" >&2; exit 1; }
pass() { printf '✓ %s\n' "$*"; }
search() {
  if command -v rg >/dev/null 2>&1; then rg -n "$1" "$2"; else grep -En "$1" "$2"; fi
}
require() {
  local pattern="$1" file="$2" message="$3"
  search "$pattern" "$file" >/dev/null || fail "$message"
  pass "$message"
}
forbid() {
  local pattern="$1" file="$2" message="$3"
  if search "$pattern" "$file" >/dev/null; then fail "$message"; fi
  pass "$message"
}

# All-surface OTA: policy, atomic local promotion, delegated signed macOS
# activation, automatic Pi runtime reload, rollback, and release artifacts.
for surface in cli daemon tui pi_extension menubar installer; do
  require "${surface}: enabled" crates/focusa-core/src/update.rs "OTA policy includes ${surface}"
done
require 'execute_verified_apply\(&apply\.plan\)' crates/focusa-cli/src/commands/update.rs 'verified OTA executor is active'
require '"would_update" \| "would_install"' crates/focusa-cli/src/commands/update.rs 'binary and installer assets are atomically promoted'
require '"would_update_package" \| "would_install_package"' crates/focusa-cli/src/commands/update.rs 'Pi extension package is atomically promoted'
require 'registerAutomaticOtaActivation' apps/pi-extension/src/index.ts 'Pi OTA activation is registered'
require 'ctx\.reload\(\)' apps/pi-extension/src/ota-activation.ts 'Pi OTA reload is automatic'
require 'pi-extension-activation-receipt' apps/pi-extension/src/ota-activation.ts 'Pi OTA activation receipt is durable'
require 'downloadAndInstall' apps/menubar/src/lib/updater.ts 'menubar signed update installs automatically'
require 'await relaunch\(\)' apps/menubar/src/lib/updater.ts 'menubar relaunches automatically'
require 'focusa\.menubar\.ota\.activation' apps/menubar/src/lib/updater.ts 'menubar activation receipt is durable'
require 'rollback_promoted_parts' crates/focusa-cli/src/commands/update.rs 'OTA rollback restores promoted local surfaces'
require 'focusa-pi-extension-\$\{VERSION\}\.tar\.gz|focusa-pi-extension-' .github/workflows/release.yml 'release packages Pi extension'
require 'focusa-installer-|install-focusa\.sh' .github/workflows/release.yml 'release packages installer'
require 'latest\.json|updater' .github/workflows/release.yml 'release publishes signed menubar updater metadata'
require 'focusa-daemon' .github/workflows/release.yml 'release packages daemon/API surface'
require 'focusa-tui' .github/workflows/release.yml 'release packages TUI surface'
bash tests/spec132_public_uninstall_preservation_test.sh
pass 'public uninstall preserves user data unless purge is explicit'

# Every Focusa Pi tool, skill, runbook, machine projection, and public agent
# entry point must remain one-to-one and release-current.
python3 scripts/generate-agent-skills.py --check
bun scripts/generate-agent-capability-descriptors.ts --check
bun scripts/generate-agent-tool-docs.ts --check
bun tests/spec141_agent_conformance_test.ts
python3 tests/spec141_agent_first_tool_audit_test.py
bash tests/spec129_agent_docs_surface_static_test.sh
pass 'all-Pi-tool, Agent Card, skill/runbook, and onboarding documentation gates'

# Worktree/authority aggregate proof.
bash tests/authority_scope_static_test.sh
bash tests/spec96_project_identity_quorum_static_test.sh
python3 tests/spec104_mismatch_semantic_static_test.py
bash tests/spec130_rotating_continuity_transfer_static_test.sh
pass 'worktree and authority gates'

# Cache-safe prefix stabilization and automatic Pi activation runtime proof.
(
  cd apps/pi-extension
  npm run test:cache-safe-context
  npm run test:ota-activation
  npm run test:spec104-attachment
)
pass 'cache miss mitigation and Pi OTA activation gates'

# Compaction provider-overflow, native recovery, persistence, crash, and rotating-agent proof.
forbid 'pi\.sendUserMessage\("/focusa-rollover execute"' apps/pi-extension/src/auto-compaction.ts 'transport retry exhaustion must not auto-queue rollover'
bash tests/spec130a_proactive_compaction_runtime_test.sh
bash tests/spec130a_persistence_actor_static_test.sh
python3 tests/spec130a_release_stress_static_test.py
npx --yes tsx tests/spec130a_release_stress_runtime_test.mts
pass 'compaction and session-recovery gates'

printf 'FINAL RELEASE GAP GATE: PASS\n'
